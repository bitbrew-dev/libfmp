package fmp

import (
	"context"
	"errors"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"
)

const probeEndpoint = "endpoint-contract-test"

// recorder captures every request a test server received.
type recorder struct {
	mu       sync.Mutex
	requests []*http.Request
}

func (r *recorder) record(req *http.Request) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.requests = append(r.requests, req.Clone(context.Background()))
}

func (r *recorder) all() []*http.Request {
	r.mu.Lock()
	defer r.mu.Unlock()
	return append([]*http.Request(nil), r.requests...)
}

func (r *recorder) count() int { return len(r.all()) }

// newServer starts a TLS test server that records requests before handing
// them to handler.
func newServer(t *testing.T, handler http.HandlerFunc) (*httptest.Server, *recorder) {
	t.Helper()
	rec := &recorder{}
	server := httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		rec.record(r)
		handler(w, r)
	}))
	server.Config.ErrorLog = log.New(io.Discard, "", 0)
	server.StartTLS()
	t.Cleanup(server.Close)
	return server, rec
}

func jsonHandler(body string) http.HandlerFunc {
	return func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(body))
	}
}

// newClient builds a client against server through the WithHTTPClient seam.
func newClient(t *testing.T, server *httptest.Server, opts ...Option) *Client {
	t.Helper()
	base := []Option{
		WithBaseURL(server.URL + "/router"),
		WithPathPrefix("stable"),
		WithHTTPClient(server.Client()),
	}
	client, err := NewClient(append(base, opts...)...)
	if err != nil {
		t.Fatalf("NewClient: %v", err)
	}
	return client
}

type probeRow struct {
	Symbol string `json:"symbol"`
}

func probe(t *testing.T, client *Client, query ...queryParam) ([]probeRow, error) {
	t.Helper()
	var rows []probeRow
	err := client.getJSON(context.Background(), probeEndpoint, "contract-probe", query, &rows)
	return rows, err
}

func assertError(t *testing.T, err error, category ErrorCategory, status int) *Error {
	t.Helper()
	var typed *Error
	if !errors.As(err, &typed) {
		t.Fatalf("error %v (%T) is not *Error", err, err)
	}
	if typed.Category != category || typed.Status != status || typed.Endpoint != probeEndpoint {
		t.Fatalf("error = %+v, want category %v status %d endpoint %q", typed, category, status, probeEndpoint)
	}
	return typed
}

func TestNewClientRequiresCredentialsForTheDefaultOrigin(t *testing.T) {
	t.Parallel()
	_, err := NewClient()
	assertConfigurationKind(t, err, ConfigurationKindMissingCredential)
	if _, err := NewClient(WithAuthentication(FMPHeader("k"))); err != nil {
		t.Fatalf("authenticated default client: %v", err)
	}
	if _, err := NewClient(WithBaseURL("https://proxy.example/router")); err != nil {
		t.Fatalf("unauthenticated proxy client: %v", err)
	}
}

func TestNewClientRejectsInvalidConfiguration(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name string
		opts []Option
		kind ConfigurationKind
	}{
		{"authentication twice", []Option{WithAuthentication(FMPHeader("k")), WithAuthentication(Authentication{})},
			ConfigurationKindConflictingAuthentication},
		{"relative base", []Option{WithBaseURL("/relative")}, ConfigurationKindInvalidBaseURL},
		{"ftp base", []Option{WithBaseURL("ftp://example.test")}, ConfigurationKindInvalidBaseURL},
		{"opaque base", []Option{WithBaseURL("mailto:x@example.test")}, ConfigurationKindInvalidBaseURL},
		{"user info", []Option{WithBaseURL("https://user:pw@example.test")}, ConfigurationKindUnsafeBaseURL},
		{"query", []Option{WithBaseURL("https://example.test/?q=1")}, ConfigurationKindUnsafeBaseURL},
		{"fragment", []Option{WithBaseURL("https://example.test/#f")}, ConfigurationKindUnsafeBaseURL},
		{"dot prefix", []Option{WithBaseURL("https://example.test"), WithPathPrefix("../x")}, ConfigurationKindInvalidPath},
		{"query prefix", []Option{WithBaseURL("https://example.test"), WithPathPrefix("a?b")}, ConfigurationKindInvalidPath},
		{"insecure header auth", []Option{WithBaseURL("http://example.test"), WithAuthentication(FMPHeader("k"))},
			ConfigurationKindInsecureAuthentication},
		{"insecure localhost name", []Option{WithBaseURL("http://localhost:8080"), WithAuthentication(Bearer("k"))},
			ConfigurationKindInsecureAuthentication},
		{"bad user agent", []Option{WithBaseURL("https://example.test"), WithUserAgent("x\ny")},
			ConfigurationKindInvalidHeaderValue},
		{"bad header name", []Option{WithBaseURL("https://example.test"), WithDefaultHeader("x y", "v")},
			ConfigurationKindInvalidHeaderName},
		{"bad header value", []Option{WithBaseURL("https://example.test"), WithDefaultHeader("X-Mode", "v\x7f")},
			ConfigurationKindInvalidHeaderValue},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			_, err := NewClient(tc.opts...)
			assertConfigurationKind(t, err, tc.kind)
		})
	}
}

func TestHeaderValuesAcceptObsText(t *testing.T) {
	t.Parallel()
	for _, value := range []string{"Träger", "\x80\xff", "tab\tseparated", "~visible!"} {
		if !validHeaderValue(value) {
			t.Errorf("validHeaderValue(%q) = false, want true", value)
		}
	}
	for _, value := range []string{"v\x7f", "v\x00", "v\r\n", "v\x1f"} {
		if validHeaderValue(value) {
			t.Errorf("validHeaderValue(%q) = true, want false", value)
		}
	}
	server, rec := newServer(t, jsonHandler(`[]`))
	client := newClient(t, server, WithDefaultHeader("X-Region", "Zürich"),
		WithAuthentication(CustomHeaderWithPrefix("X-Proxy-Token", "Träger ", "tok")))
	if _, err := probe(t, client); err != nil {
		t.Fatal(err)
	}
	req := rec.all()[0]
	if req.Header.Get("X-Region") != "Zürich" || req.Header.Get("X-Proxy-Token") != "Träger tok" {
		t.Fatalf("obs-text headers were not sent verbatim: %v", req.Header)
	}
}

func TestInsecureAuthenticationOverrideAndLoopback(t *testing.T) {
	t.Parallel()
	_, err := NewClient(WithBaseURL("http://example.test"), WithAuthentication(FMPHeader("k")),
		WithDangerAllowInsecureAuthentication())
	if err != nil {
		t.Fatalf("override refused: %v", err)
	}
	_, err = NewClient(WithBaseURL("http://example.test"), WithAuthentication(FMPHeader("k")))
	assertConfigurationKind(t, err, ConfigurationKindInsecureAuthentication)
	for _, base := range []string{"http://127.0.0.1:8080", "http://[::1]:8080"} {
		if _, err := NewClient(WithBaseURL(base), WithAuthentication(FMPHeader("k"))); err != nil {
			t.Fatalf("loopback %s refused: %v", base, err)
		}
	}
}

func TestDefaultHeadersCannotReplaceProtectedFields(t *testing.T) {
	t.Parallel()
	for _, name := range []string{"AUTHORIZATION", fmpHeaderName, "X-API-KEY", "Cookie", "Host",
		"Content-Length", "Connection", "Transfer-Encoding", "User-Agent"} {
		_, err := NewClient(WithBaseURL("https://example.test"), WithDefaultHeader(name, "replacement"))
		assertConfigurationKind(t, err, ConfigurationKindProtectedFieldCollision)
	}
	_, err := NewClient(WithBaseURL("https://example.test"),
		WithAuthentication(CustomHeader("X-Proxy-Token", "tok")), WithDefaultHeader("x-proxy-token", "other"))
	assertConfigurationKind(t, err, ConfigurationKindProtectedFieldCollision)
}

func TestRequestCarriesDefaultsHeaderAuthAndOrderedQuery(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"symbol":"^VIX"}]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("header-key")),
		WithDefaultHeader("X-Mode", "default"), WithDefaultHeader("x-mode", "final"),
		WithUserAgent("custom-agent/1"))

	rows, err := probe(t, client, queryParam{"symbol", "^VIX"}, queryParam{"limit", "100"})
	if err != nil {
		t.Fatal(err)
	}
	if len(rows) != 1 || rows[0].Symbol != "^VIX" {
		t.Fatalf("rows = %+v", rows)
	}
	reqs := rec.all()
	if len(reqs) != 1 {
		t.Fatalf("%d requests, want 1", len(reqs))
	}
	req := reqs[0]
	if req.URL.Path != "/router/stable/contract-probe" || req.URL.RawQuery != "symbol=%5EVIX&limit=100" {
		t.Fatalf("request line = %s?%s", req.URL.Path, req.URL.RawQuery)
	}
	if req.Header.Get(fmpHeaderName) != "header-key" || req.Header.Get("X-Mode") != "final" ||
		req.Header.Get("User-Agent") != "custom-agent/1" {
		t.Fatalf("headers = %v", req.Header)
	}
	if strings.Contains(req.URL.String(), "header-key") {
		t.Fatal("header credential appeared in the URL")
	}
}

func TestQueryAuthenticationIsAppendedLastAndProtected(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[]`))
	client := newClient(t, server, WithAuthentication(FMPQuery("query-key")))

	if _, err := probe(t, client, queryParam{"symbol", "AAPL"}); err != nil {
		t.Fatal(err)
	}
	req := rec.all()[0]
	if !strings.HasSuffix(req.URL.RawQuery, "&"+fmpQueryName+"=query-key") ||
		!strings.HasPrefix(req.URL.RawQuery, "symbol=AAPL") {
		t.Fatalf("raw query = %q", req.URL.RawQuery)
	}
	if req.Header.Get(fmpHeaderName) != "" {
		t.Fatal("query mode also sent a header")
	}
	for _, name := range []string{fmpQueryName, strings.ToUpper(fmpQueryName)} {
		_, err := probe(t, client, queryParam{name, "caller"})
		assertConfigurationKind(t, err, ConfigurationKindProtectedFieldCollision)
	}
	_, err := probe(t, client, queryParam{"", "x"})
	assertConfigurationKind(t, err, ConfigurationKindInvalidQueryName)
	if rec.count() != 1 {
		t.Fatalf("%d requests, want 1: local validation must not hit the wire", rec.count())
	}
}

func TestSuccessfulJSONRequiresAMatchingContentType(t *testing.T) {
	t.Parallel()
	rejected := []string{"text/plain", "", "text/html/foo+json", "application/+json",
		"application/problem++json", "/problem+json", "application/problem+json/extra",
		"application/pro blem+json", "application/problem+jsonx"}
	accepted := []string{"application/json", "Application/JSON; charset=utf-8", "application/vnd.fmp.snapshot+json"}
	var contentType string
	var mu sync.Mutex
	server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		mu.Lock()
		ct := contentType
		mu.Unlock()
		if ct == "" {
			w.Header()["Content-Type"] = nil
		} else {
			w.Header().Set("Content-Type", ct)
		}
		_, _ = w.Write([]byte(`[{"symbol":"AAPL"}]`))
	})
	client := newClient(t, server)
	setContentType := func(ct string) {
		mu.Lock()
		defer mu.Unlock()
		contentType = ct
	}
	for _, ct := range rejected {
		setContentType(ct)
		_, err := probe(t, client)
		typed := assertError(t, err, CategoryDecode, http.StatusOK)
		if typed.Body == nil || !strings.Contains(typed.Body.Text, "AAPL") {
			t.Fatalf("content type %q: decode error kept no body: %+v", ct, typed)
		}
	}
	for _, ct := range accepted {
		setContentType(ct)
		if rows, err := probe(t, client); err != nil || len(rows) != 1 {
			t.Fatalf("content type %q: rows %v err %v", ct, rows, err)
		}
	}
}

func TestMalformedJSONIsADecodeErrorWithRedactedCause(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(`[{"symbol": 12`))
	client := newClient(t, server)
	_, err := probe(t, client)
	typed := assertError(t, err, CategoryDecode, http.StatusOK)
	if typed.Unwrap() == nil || typed.Body == nil || typed.Body.Text != `[{"symbol": 12` {
		t.Fatalf("decode error = %+v cause %v", typed, typed.Unwrap())
	}
}

func TestClientFormattingNeverPrintsConfiguration(t *testing.T) {
	t.Parallel()
	client, err := NewClient(WithBaseURL("https://proxy.example/private-router"),
		WithAuthentication(CustomHeader("X-Proxy-Token", "format-secret")),
		WithDefaultHeader("X-Tenant", "tenant-value"))
	if err != nil {
		t.Fatal(err)
	}
	for _, verb := range []string{"%v", "%+v", "%#v", "%s"} {
		for _, text := range []string{fmt.Sprintf(verb, client), fmt.Sprintf(verb, *client)} {
			for _, leak := range []string{"format-secret", "private-router", "tenant-value"} {
				if strings.Contains(text, leak) {
					t.Fatalf("%s leaked %q: %s", verb, leak, text)
				}
			}
			if !strings.Contains(text, "X-Tenant") {
				t.Fatalf("%s did not describe the client: %s", verb, text)
			}
		}
	}
}

func TestCloseIdleConnectionsClosesTheOwnedPool(t *testing.T) {
	t.Parallel()
	var mu sync.Mutex
	dials := 0
	closed := make(chan struct{}, 1)
	server := httptest.NewUnstartedServer(jsonHandler(`[{"symbol":"AAPL"}]`))
	server.Config.ConnState = func(_ net.Conn, state http.ConnState) {
		switch state {
		case http.StateNew:
			mu.Lock()
			dials++
			mu.Unlock()
		case http.StateClosed:
			select {
			case closed <- struct{}{}:
			default:
			}
		}
	}
	server.Start()
	t.Cleanup(server.Close)
	client, err := NewClient(WithBaseURL(server.URL+"/router"), WithPathPrefix("stable"))
	if err != nil {
		t.Fatalf("NewClient: %v", err)
	}
	dialCount := func() int {
		mu.Lock()
		defer mu.Unlock()
		return dials
	}

	for range 2 {
		if _, err := probe(t, client); err != nil {
			t.Fatal(err)
		}
	}
	if got := dialCount(); got != 1 {
		t.Fatalf("dials before CloseIdleConnections = %d, want 1 reused connection", got)
	}
	client.CloseIdleConnections()
	select {
	case <-closed:
	case <-time.After(5 * time.Second):
		t.Fatal("the idle connection was not closed")
	}
	if _, err := probe(t, client); err != nil {
		t.Fatalf("client unusable after CloseIdleConnections: %v", err)
	}
	if got := dialCount(); got != 2 {
		t.Fatalf("dials after CloseIdleConnections = %d, want a fresh connection", got)
	}
}

type idleCountingTransport struct {
	http.RoundTripper
	mu    sync.Mutex
	calls int
}

func (t *idleCountingTransport) CloseIdleConnections() {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.calls++
}

func TestCloseIdleConnectionsLeavesACallerTransportAlone(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(`[{"symbol":"AAPL"}]`))
	transport := &idleCountingTransport{RoundTripper: server.Client().Transport}
	client := newClient(t, server, WithHTTPClient(&http.Client{Transport: transport}))

	if _, err := probe(t, client); err != nil {
		t.Fatal(err)
	}
	client.CloseIdleConnections()
	transport.mu.Lock()
	defer transport.mu.Unlock()
	if transport.calls != 0 {
		t.Fatalf("caller transport CloseIdleConnections called %d times", transport.calls)
	}
}
