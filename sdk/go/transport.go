package fmp

import (
	"context"
	"errors"
	"io"
	"net"
	"net/http"
	"net/url"
	"slices"
	"strings"
	"time"
)

// useLastResponse stops net/http from following redirects. The client walks
// every hop itself so all authentication modes share one same-origin rule.
func useLastResponse(*http.Request, []*http.Request) error {
	return http.ErrUseLastResponse
}

// newHTTPClient builds the SDK-owned transport. It mirrors
// http.DefaultTransport except for the dialer, which carries the configured
// connect timeout.
func newHTTPClient(connectTimeout time.Duration) *http.Client {
	dialer := &net.Dialer{Timeout: connectTimeout, KeepAlive: 30 * time.Second}
	transport := &http.Transport{
		Proxy:                 http.ProxyFromEnvironment,
		DialContext:           dialer.DialContext,
		ForceAttemptHTTP2:     true,
		MaxIdleConns:          100,
		IdleConnTimeout:       90 * time.Second,
		TLSHandshakeTimeout:   10 * time.Second,
		ExpectContinueTimeout: 1 * time.Second,
	}
	return &http.Client{Transport: transport, CheckRedirect: useLastResponse}
}

// installRedirectPolicy shallow-copies a caller-owned client so the SDK can
// own redirect handling without mutating the caller's value.
func installRedirectPolicy(client *http.Client) *http.Client {
	copied := *client
	copied.CheckRedirect = useLastResponse
	return &copied
}

// bufferedResponse is one hop's status, headers, and capped body.
type bufferedResponse struct {
	status int
	header http.Header
	body   []byte
}

func (c *Client) executeRedirects(ctx context.Context, endpointID string, target *url.URL,
	maxBodyBytes int64) (*bufferedResponse, error) {
	for hop := 0; hop <= maxRedirects; hop++ {
		c.applyQueryAuth(target)
		resp, err := c.roundTrip(ctx, endpointID, target, maxBodyBytes)
		if err != nil {
			return nil, err
		}
		if isRedirect(resp.status) {
			if c.redirectPolicy == RedirectNone {
				return nil, c.statusError(endpointID, resp)
			}
			if hop == maxRedirects {
				return nil, transportError(endpointID, "same-origin redirect limit exceeded", nil)
			}
			destination, ok := c.redirectDestination(target, resp.header.Get("Location"))
			if !ok {
				return nil, c.statusError(endpointID, resp)
			}
			target = destination
			continue
		}
		if resp.status < 200 || resp.status >= 300 {
			return nil, c.statusError(endpointID, resp)
		}
		return resp, nil
	}
	return nil, transportError(endpointID, "redirect processing ended unexpectedly", nil)
}

// redirectDestination resolves a Location header against the current URL
// and accepts it only when it stays on the same origin without user info.
func (c *Client) redirectDestination(current *url.URL, location string) (*url.URL, bool) {
	if location == "" {
		return nil, false
	}
	destination, err := current.Parse(location)
	if err != nil {
		return nil, false
	}
	destination.Fragment = ""
	destination.RawFragment = ""
	if destination.User != nil || !sameOrigin(current, destination) {
		return nil, false
	}
	return destination, true
}

func (c *Client) roundTrip(ctx context.Context, endpointID string, target *url.URL,
	maxBodyBytes int64) (*bufferedResponse, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, target.String(), nil)
	if err != nil {
		return nil, transportError(endpointID, "request execution failed", c.redactCause(err))
	}
	req.Header = c.defaultHeaders.Clone()
	if c.auth.headerName != "" {
		req.Header.Set(c.auth.headerName, c.auth.headerValue)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		message := "request execution failed"
		if errors.Is(err, context.DeadlineExceeded) {
			message = "request deadline exceeded"
		}
		return nil, transportError(endpointID, message, c.redactCause(err))
	}
	defer func() { _ = resp.Body.Close() }()

	buffered := &bufferedResponse{status: resp.StatusCode, header: resp.Header}
	if isRedirect(resp.StatusCode) {
		return buffered, nil
	}
	if resp.ContentLength > maxBodyBytes {
		return nil, transportError(endpointID, "response body exceeded configured limit", nil)
	}
	body, err := io.ReadAll(io.LimitReader(resp.Body, maxBodyBytes+1))
	if err != nil {
		message := "request execution failed"
		if errors.Is(err, context.DeadlineExceeded) {
			message = "request deadline exceeded"
		}
		return nil, transportError(endpointID, message, c.redactCause(err))
	}
	if int64(len(body)) > maxBodyBytes {
		return nil, transportError(endpointID, "response body exceeded configured limit", nil)
	}
	buffered.body = body
	return buffered, nil
}

// redactCause rebuilds a transport or decoder failure without the request
// URL (*url.Error embeds it, and it carries the API key in query mode),
// passes the remaining text through the client's Redactor, and then removes
// the network topology a dial or proxy failure prints: the base URL and
// proxy hosts registered at build time and the addresses the failure itself
// names. The topology pass applies to causes only, never to provider bodies.
func (c *Client) redactCause(err error) error {
	var urlErr *url.Error
	if errors.As(err, &urlErr) && urlErr.Err != nil {
		err = urlErr.Err
	}
	text := redactSecretValues(c.redactor.Redact(err.Error()),
		append(slices.Clone(c.topology), causeAddresses(err)...))
	var sentinel error
	switch {
	case errors.Is(err, context.DeadlineExceeded):
		sentinel = context.DeadlineExceeded
	case errors.Is(err, context.Canceled):
		sentinel = context.Canceled
	}
	return &redactedCause{text: text, sentinel: sentinel}
}

// networkTopology lists the host spellings a network failure can print for
// baseURL and for the proxy the transport resolves for it: host, host:port
// with the scheme's default port filled in, and, for a proxy, its full URL
// and any password it carries. It calls proxy once with a GET for baseURL,
// as the first request would; a nil proxy, an error, or a nil URL leaves the
// base URL spellings only.
func networkTopology(baseURL *url.URL, proxy func(*http.Request) (*url.URL, error)) []string {
	topology := hostSpellings(baseURL)
	if proxy == nil {
		return topology
	}
	req, err := http.NewRequestWithContext(context.Background(), http.MethodGet, baseURL.String(), nil)
	if err != nil {
		return topology
	}
	proxyURL, err := proxy(req)
	if err != nil || proxyURL == nil {
		return topology
	}
	topology = append(topology, proxyURL.String())
	if password, ok := proxyURL.User.Password(); ok {
		topology = append(topology, password)
	}
	return append(topology, hostSpellings(proxyURL)...)
}

func hostSpellings(u *url.URL) []string {
	hostname := u.Hostname()
	if hostname == "" {
		return nil
	}
	port := u.Port()
	if port == "" {
		port = "443"
		if u.Scheme == "http" {
			port = "80"
		}
	}
	return []string{u.Host, net.JoinHostPort(hostname, port), hostname}
}

// transportProxy returns the proxy function of the transport the client
// sends through, or nil when it is not an *http.Transport.
func transportProxy(client *http.Client) func(*http.Request) (*url.URL, error) {
	roundTripper := client.Transport
	if roundTripper == nil {
		roundTripper = http.DefaultTransport
	}
	if transport, ok := roundTripper.(*http.Transport); ok {
		return transport.Proxy
	}
	return nil
}

// causeAddresses collects the addresses a network failure names: the local
// and remote endpoints of a *net.OpError and the name and server of a
// *net.DNSError, anywhere in the chain.
func causeAddresses(err error) []string {
	var addresses []string
	var opErr *net.OpError
	if errors.As(err, &opErr) {
		for _, addr := range []net.Addr{opErr.Addr, opErr.Source} {
			if addr != nil {
				addresses = append(addresses, addr.String())
			}
		}
	}
	var dnsErr *net.DNSError
	if errors.As(err, &dnsErr) {
		addresses = append(addresses, dnsErr.Name, dnsErr.Server)
	}
	return slices.DeleteFunc(addresses, func(address string) bool { return address == "" })
}

func (c *Client) statusError(endpointID string, resp *bufferedResponse) *Error {
	err := statusError(endpointID, resp.status, c.safeBody(resp.body))
	err.Headers = retainedHeaders(resp.header, c.redactor)
	return err
}

func (c *Client) providerMessageError(endpointID string, resp *bufferedResponse) *Error {
	err := providerMessageError(endpointID, resp.status, c.safeBody(resp.body))
	err.Headers = retainedHeaders(resp.header, c.redactor)
	return err
}

// applyQueryAuth strips any pair carrying the protected query name and
// appends the secret as the last pair, on every hop.
func (c *Client) applyQueryAuth(target *url.URL) {
	if c.auth.queryName == "" {
		return
	}
	pair := url.QueryEscape(c.auth.queryName) + "=" + url.QueryEscape(c.auth.querySecret)
	retained := stripQueryName(target.RawQuery, c.auth.queryName)
	if retained == "" {
		target.RawQuery = pair
		return
	}
	target.RawQuery = retained + "&" + pair
}

func stripQueryName(rawQuery, name string) string {
	if rawQuery == "" {
		return ""
	}
	var kept []string
	for segment := range strings.SplitSeq(rawQuery, "&") {
		if segment == "" {
			continue
		}
		key, _, _ := strings.Cut(segment, "=")
		if decoded, err := url.QueryUnescape(key); err == nil {
			key = decoded
		}
		if strings.EqualFold(key, name) {
			continue
		}
		kept = append(kept, segment)
	}
	return strings.Join(kept, "&")
}

func isRedirect(status int) bool {
	switch status {
	case http.StatusMovedPermanently, http.StatusFound, http.StatusSeeOther,
		http.StatusTemporaryRedirect, http.StatusPermanentRedirect:
		return true
	default:
		return false
	}
}

func sameOrigin(left, right *url.URL) bool {
	return strings.EqualFold(left.Scheme, right.Scheme) &&
		strings.EqualFold(left.Hostname(), right.Hostname()) &&
		effectivePort(left) == effectivePort(right)
}

func effectivePort(u *url.URL) string {
	if port := u.Port(); port != "" {
		return port
	}
	switch strings.ToLower(u.Scheme) {
	case "https":
		return "443"
	case "http":
		return "80"
	default:
		return ""
	}
}

func validJSONMediaType(contentType string) bool {
	mediaType := contentType
	if i := strings.IndexByte(mediaType, ';'); i >= 0 {
		mediaType = mediaType[:i]
	}
	mediaType = strings.TrimSpace(mediaType)
	typeName, subtype, found := strings.Cut(mediaType, "/")
	if !found || !validMediaTypeToken(typeName) || !validMediaTypeToken(subtype) ||
		strings.Contains(subtype, "/") {
		return false
	}
	if strings.EqualFold(typeName, "application") && strings.EqualFold(subtype, "json") {
		return true
	}
	plus := strings.LastIndexByte(subtype, '+')
	if plus <= 0 {
		return false
	}
	name, suffix := subtype[:plus], subtype[plus+1:]
	return !strings.Contains(name, "+") && strings.EqualFold(suffix, "json")
}

func validMediaTypeToken(value string) bool {
	if value == "" {
		return false
	}
	for i := 0; i < len(value); i++ {
		if !isHeaderNameByte(value[i]) {
			return false
		}
	}
	return true
}
