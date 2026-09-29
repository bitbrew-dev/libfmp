package fmp

import (
	"context"
	"encoding/json/v2"
	"fmt"
	"net/http"
	"net/netip"
	"net/url"
	"sort"
	"strings"
	"time"
	"unicode"
)

const (
	// DefaultBaseURL is the Financial Modeling Prep API origin.
	DefaultBaseURL = "https://financialmodelingprep.com"
	// DefaultPathPrefix is the path prefix of the stable FMP API.
	DefaultPathPrefix = "stable"
	// DefaultTimeout is the logical deadline for one call, redirects included.
	DefaultTimeout = 30 * time.Second
	// DefaultConnectTimeout bounds connection establishment.
	DefaultConnectTimeout = 10 * time.Second
	// DefaultMaxResponseBodyBytes is the largest body buffered per response (64 MiB).
	DefaultMaxResponseBodyBytes int64 = 64 * 1024 * 1024

	maxRedirects = 10
)

// RedirectPolicy selects how redirect responses are handled.
type RedirectPolicy int

const (
	// RedirectSameOrigin follows at most ten redirects when every hop keeps
	// the same scheme, host, and port. It is the default.
	RedirectSameOrigin RedirectPolicy = iota
	// RedirectNone returns redirect responses as status errors.
	RedirectNone
)

// String names the policy.
func (p RedirectPolicy) String() string {
	switch p {
	case RedirectSameOrigin:
		return "SameOrigin"
	case RedirectNone:
		return "None"
	default:
		return fmt.Sprintf("RedirectPolicy(%d)", int(p))
	}
}

type headerPair struct{ name, value string }

type clientConfig struct {
	baseURL                  string
	pathPrefix               string
	authentication           Authentication
	authenticationConfigured bool
	authenticationConflict   bool
	defaultHeaders           []headerPair
	userAgent                string
	timeout                  time.Duration
	connectTimeout           time.Duration
	maxResponseBodyBytes     int64
	dangerAllowInsecureAuth  bool
	redirectPolicy           RedirectPolicy
	httpClient               *http.Client
}

// Option configures NewClient. Each option mirrors one ClientBuilder method
// of the Rust crate.
type Option func(*clientConfig)

// WithBaseURL sets an absolute HTTP(S) base URL. Existing path segments are kept.
func WithBaseURL(baseURL string) Option {
	return func(c *clientConfig) { c.baseURL = baseURL }
}

// WithPathPrefix sets the segments inserted between the base URL and the
// endpoint path. An empty prefix is valid for proxies that expose endpoints
// at their root; leading and trailing slashes are ignored.
func WithPathPrefix(prefix string) Option {
	return func(c *clientConfig) { c.pathPrefix = prefix }
}

// WithAuthentication selects the transport-owned authentication mode. It may
// be given only once.
func WithAuthentication(auth Authentication) Option {
	return func(c *clientConfig) {
		if c.authenticationConfigured {
			c.authenticationConflict = true
		}
		c.authenticationConfigured = true
		c.authentication = auth
	}
}

// WithDefaultHeader adds or replaces a default header. Later calls win
// case-insensitively. Transport-owned headers cannot be set this way.
func WithDefaultHeader(name, value string) Option {
	return func(c *clientConfig) {
		c.defaultHeaders = append(c.defaultHeaders, headerPair{name, value})
	}
}

// WithUserAgent sets the transport-owned User-Agent header.
func WithUserAgent(userAgent string) Option {
	return func(c *clientConfig) { c.userAgent = userAgent }
}

// WithTimeout sets one logical deadline across redirects and the body read.
func WithTimeout(timeout time.Duration) Option {
	return func(c *clientConfig) { c.timeout = timeout }
}

// WithConnectTimeout bounds connection establishment. It applies only to the
// transport the SDK builds; a client given through WithHTTPClient keeps its
// own dialer.
func WithConnectTimeout(timeout time.Duration) Option {
	return func(c *clientConfig) { c.connectTimeout = timeout }
}

// WithMaxResponseBodyBytes sets the largest body buffered for one response.
func WithMaxResponseBodyBytes(maxBytes int64) Option {
	return func(c *clientConfig) { c.maxResponseBodyBytes = maxBytes }
}

// WithDangerAllowInsecureAuthentication allows credentials over plaintext
// HTTP to a non-loopback host. Leaving it out keeps the refusal, so there is
// no way to pass it in its disabled form. Literal IPv4 and IPv6 loopback URLs
// used by local test servers never need it.
func WithDangerAllowInsecureAuthentication() Option {
	return func(c *clientConfig) { c.dangerAllowInsecureAuth = true }
}

// WithRedirectPolicy selects RedirectNone or RedirectSameOrigin.
func WithRedirectPolicy(policy RedirectPolicy) Option {
	return func(c *clientConfig) { c.redirectPolicy = policy }
}

// WithHTTPClient injects a caller-owned *http.Client for proxies and tests.
// The SDK shallow-copies it to install its redirect policy and never mutates
// the caller's value.
func WithHTTPClient(client *http.Client) Option {
	return func(c *clientConfig) { c.httpClient = client }
}

// Client is the shared FMP client. It is safe for concurrent use. Formatting
// it with any fmt verb never reveals a credential or a configured URL.
// Endpoints are grouped by registry domain into the namespace fields of the
// embedded Namespaces (client.Quote.Full and so on); see the "Endpoint
// surface" section of the package documentation.
type Client struct {
	Namespaces

	baseURL              *url.URL
	pathPrefix           string
	auth                 authMaterial
	defaultHeaders       http.Header
	redactor             *Redactor
	topology             []string
	httpClient           *http.Client
	timeout              time.Duration
	maxResponseBodyBytes int64
	redirectPolicy       RedirectPolicy
}

// NewClient validates the options and builds a client with the direct FMP
// stable API defaults. Every rejection is an *Error of category
// CategoryConfiguration with a ConfigurationKind.
func NewClient(opts ...Option) (*Client, error) {
	cfg := clientConfig{
		baseURL:              DefaultBaseURL,
		pathPrefix:           DefaultPathPrefix,
		userAgent:            defaultUserAgent,
		timeout:              DefaultTimeout,
		connectTimeout:       DefaultConnectTimeout,
		maxResponseBodyBytes: DefaultMaxResponseBodyBytes,
		redirectPolicy:       RedirectSameOrigin,
	}
	for _, opt := range opts {
		opt(&cfg)
	}
	if cfg.authenticationConflict {
		return nil, configurationError(ConfigurationKindConflictingAuthentication,
			"authentication can be configured only once")
	}
	baseURL, err := parseBaseURL(cfg.baseURL)
	if err != nil {
		return nil, err
	}
	if err := validateRelativePath(cfg.pathPrefix); err != nil {
		return nil, err
	}
	auth, err := buildAuthMaterial(cfg.authentication)
	if err != nil {
		return nil, err
	}
	if cfg.authentication.mode != authNone && baseURL.Scheme == "http" &&
		!isLoopbackOrigin(baseURL) && !cfg.dangerAllowInsecureAuth {
		return nil, configurationError(ConfigurationKindInsecureAuthentication,
			"authenticated plaintext HTTP requires an explicit dangerous opt-in")
	}

	protected := reservedHeaderNames()
	if auth.headerName != "" {
		protected[auth.headerName] = struct{}{}
	}
	defaultHeaders, err := parseHeaders(cfg.defaultHeaders, protected)
	if err != nil {
		return nil, err
	}
	if !validHeaderValue(cfg.userAgent) {
		return nil, configurationError(ConfigurationKindInvalidHeaderValue,
			"header value is not a valid HTTP field value")
	}
	defaultHeaders.Set("User-Agent", cfg.userAgent)
	if cfg.authentication.mode == authNone && isDefaultFMPOrigin(baseURL) {
		return nil, configurationError(ConfigurationKindMissingCredential,
			"direct FMP access requires explicit authentication")
	}

	redactor := NewRedactor()
	if err := registerAuthRedaction(redactor, cfg.authentication); err != nil {
		return nil, err
	}
	redactor.AddSecret(cfg.baseURL)
	redactor.AddSecret(baseURL.String())
	redactor.AddSecret(cfg.pathPrefix)
	redactor.AddSecret(cfg.userAgent)
	for _, header := range cfg.defaultHeaders {
		redactor.AddSecret(header.value)
	}

	httpClient := cfg.httpClient
	if httpClient == nil {
		httpClient = newHTTPClient(cfg.connectTimeout)
	} else {
		httpClient = installRedirectPolicy(httpClient)
	}

	client := &Client{
		baseURL:              baseURL,
		pathPrefix:           cfg.pathPrefix,
		auth:                 auth,
		defaultHeaders:       defaultHeaders,
		redactor:             redactor,
		topology:             networkTopology(baseURL, transportProxy(httpClient)),
		httpClient:           httpClient,
		timeout:              cfg.timeout,
		maxResponseBodyBytes: cfg.maxResponseBodyBytes,
		redirectPolicy:       cfg.redirectPolicy,
	}
	client.bindNamespaces()
	return client, nil
}

// Format prints the configuration shape without URLs, header values, or secrets.
func (c Client) Format(f fmt.State, _ rune) {
	names := make([]string, 0, len(c.defaultHeaders))
	for name := range c.defaultHeaders {
		names = append(names, name)
	}
	sort.Strings(names)
	_, _ = fmt.Fprintf(f, "Client{base_url: [CONFIGURED URL], path_prefix: [CONFIGURED PATH], "+
		"authentication_header: %q, authentication_query: %q, redirect_policy: %v, default_header_names: %v}",
		c.auth.headerName, c.auth.queryName, c.redirectPolicy, names)
}

// queryParam is one endpoint query pair. Pairs are encoded in slice order so
// the wire order documented by the provider is preserved.
type queryParam struct {
	Name  string
	Value string
}

// getJSON issues one GET for a logical endpoint and decodes the JSON body
// into out. endpointID is the registry id carried by errors, never a URL.
func (c *Client) getJSON(ctx context.Context, endpointID, relativePath string, query []queryParam, out any) error {
	if err := validateRelativePath(relativePath); err != nil {
		return err
	}
	target := c.buildEndpointURL(relativePath)
	rawQuery, err := encodeQuery(query, c.auth.queryName)
	if err != nil {
		return err
	}
	target.RawQuery = rawQuery

	ctx, cancel := context.WithTimeout(ctx, c.timeout)
	defer cancel()
	resp, err := c.executeRedirects(ctx, endpointID, target)
	if err != nil {
		return err
	}

	contentType := resp.header.Get("Content-Type")
	if contentType == "" {
		return decodeError(endpointID, resp.status, c.safeBody(resp.body),
			"successful response omitted its content type", nil)
	}
	if !validJSONMediaType(contentType) {
		return decodeError(endpointID, resp.status, c.safeBody(resp.body),
			"successful response used an unexpected content type", nil)
	}
	if err := json.Unmarshal(resp.body, out); err != nil {
		decoded := decodeError(endpointID, resp.status, c.safeBody(resp.body),
			"successful response could not be decoded", c.redactCause(err))
		path, kind := decodeLocation(resp.body, err)
		decoded.Path, decoded.DecodeKind = c.redactor.Redact(path), kind
		return decoded
	}
	return nil
}

func (c *Client) safeBody(body []byte) *SafeBody {
	safe := NewSafeBody(strings.ToValidUTF8(string(body), "�"), c.redactor)
	return &safe
}

func parseBaseURL(value string) (*url.URL, error) {
	parsed, err := url.Parse(value)
	if err != nil || parsed.Opaque != "" || parsed.Host == "" || parsed.Hostname() == "" ||
		(parsed.Scheme != "http" && parsed.Scheme != "https") {
		return nil, configurationError(ConfigurationKindInvalidBaseURL,
			"base URL must be an absolute HTTP(S) URL")
	}
	if parsed.User != nil || parsed.RawQuery != "" || parsed.ForceQuery || parsed.Fragment != "" ||
		parsed.RawFragment != "" {
		return nil, configurationError(ConfigurationKindUnsafeBaseURL,
			"base URL must not contain credentials, a query, or a fragment")
	}
	return parsed, nil
}

func isDefaultFMPOrigin(u *url.URL) bool {
	return u.Scheme == "https" && strings.EqualFold(u.Hostname(), "financialmodelingprep.com") &&
		(u.Port() == "" || u.Port() == "443")
}

func isLoopbackOrigin(u *url.URL) bool {
	addr, err := netip.ParseAddr(u.Hostname())
	return err == nil && addr.IsLoopback()
}

func hasControlCharacter(value string) bool {
	return strings.ContainsFunc(value, unicode.IsControl)
}

func validateRelativePath(value string) error {
	unsafe := hasControlCharacter(value) || strings.ContainsAny(value, "\\?#")
	if !unsafe {
		for segment := range strings.SplitSeq(strings.Trim(value, "/"), "/") {
			if segment == "." || segment == ".." {
				unsafe = true
				break
			}
		}
	}
	if unsafe {
		return configurationError(ConfigurationKindInvalidPath,
			"path must contain only safe relative segments")
	}
	return nil
}

func (c *Client) buildEndpointURL(relativePath string) *url.URL {
	target := *c.baseURL
	path := strings.TrimSuffix(target.Path, "/")
	for _, part := range []string{c.pathPrefix, relativePath} {
		for segment := range strings.SplitSeq(strings.Trim(part, "/"), "/") {
			if segment != "" {
				path += "/" + segment
			}
		}
	}
	if path == "" {
		path = "/"
	}
	target.Path = path
	target.RawPath = ""
	return &target
}

func encodeQuery(params []queryParam, protectedName string) (string, error) {
	var b strings.Builder
	for i, param := range params {
		if param.Name == "" || hasControlCharacter(param.Name) {
			return "", configurationError(ConfigurationKindInvalidQueryName,
				"query parameter name must not be empty or contain controls")
		}
		if protectedName != "" && strings.EqualFold(param.Name, protectedName) {
			return "", configurationError(ConfigurationKindProtectedFieldCollision,
				"endpoint query collides with transport authentication")
		}
		if i > 0 {
			b.WriteByte('&')
		}
		b.WriteString(url.QueryEscape(param.Name))
		b.WriteByte('=')
		b.WriteString(url.QueryEscape(param.Value))
	}
	return b.String(), nil
}

func parseHeaders(pairs []headerPair, protected map[string]struct{}) (http.Header, error) {
	headers := make(http.Header, len(pairs)+1)
	for _, pair := range pairs {
		if !validHeaderName(pair.name) {
			return nil, configurationError(ConfigurationKindInvalidHeaderName,
				"header name is not a valid HTTP field name")
		}
		if _, reserved := protected[asciiLower(pair.name)]; reserved {
			return nil, configurationError(ConfigurationKindProtectedFieldCollision,
				"default header collides with a transport-owned header")
		}
		if !validHeaderValue(pair.value) {
			return nil, configurationError(ConfigurationKindInvalidHeaderValue,
				"header value is not a valid HTTP field value")
		}
		headers.Set(pair.name, pair.value)
	}
	return headers, nil
}

func validHeaderName(name string) bool {
	if name == "" {
		return false
	}
	for i := 0; i < len(name); i++ {
		if !isHeaderNameByte(name[i]) {
			return false
		}
	}
	return true
}

// validHeaderValue mirrors the Rust http::HeaderValue check and the RFC 9110
// field-value grammar: horizontal tab, visible ASCII, and obs-text bytes
// 0x80 through 0xFF are accepted; the other control bytes and DEL (0x7f)
// are rejected.
func validHeaderValue(value string) bool {
	for i := 0; i < len(value); i++ {
		if b := value[i]; b != '\t' && (b < 0x20 || b == 0x7f) {
			return false
		}
	}
	return true
}

// visibleHeaderText mirrors the Rust HeaderValue::to_str check applied to
// response headers: horizontal tab and visible ASCII only, so an obs-text
// response header is treated as unreadable.
func visibleHeaderText(value string) bool {
	for i := 0; i < len(value); i++ {
		if b := value[i]; b != '\t' && (b < 0x20 || b >= 0x7f) {
			return false
		}
	}
	return true
}

func reservedHeaderNames() map[string]struct{} {
	return nameSet(
		"apikey", "authorization", "connection", "content-length", "cookie", "host",
		"http2-settings", "keep-alive", "proxy-authorization", "proxy-connection", "te",
		"trailer", "transfer-encoding", "upgrade", "user-agent", "x-api-key",
	)
}

func isUnsafeTransportHeader(name string) bool {
	switch asciiLower(name) {
	case "connection", "content-length", "host", "http2-settings", "keep-alive",
		"proxy-authorization", "proxy-connection", "te", "trailer", "transfer-encoding",
		"upgrade", "user-agent":
		return true
	default:
		return false
	}
}
