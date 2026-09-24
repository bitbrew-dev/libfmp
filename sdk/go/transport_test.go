package fmp

import (
	"context"
	"errors"
	"fmt"
	"net"
	"net/http"
	"net/http/httptest"
	"net/url"
	"slices"
	"strings"
	"testing"
	"time"
)

func TestProviderStatusFamiliesAreStructuredRedactedAndNeverRetried(t *testing.T) {
	t.Parallel()
	const secret = "status-family-secret"
	for _, status := range []int{401, 402, 403, 429, 500, 503} {
		t.Run(fmt.Sprint(status), func(t *testing.T) {
			t.Parallel()
			server, rec := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(status)
				_, _ = fmt.Fprintf(w, `{"Error Message":"denied for %s"}`, secret)
			})
			client := newClient(t, server, WithAuthentication(Bearer(secret)))

			_, err := probe(t, client, queryParam{"symbol", "AAPL"})
			typed := assertError(t, err, CategoryStatus, status)
			if typed.Body == nil || typed.Body.Text != `{"Error Message":"denied for [REDACTED]"}` {
				t.Fatalf("body = %+v", typed.Body)
			}
			for _, text := range []string{err.Error(), fmt.Sprintf("%+v", typed), fmt.Sprintf("%#v", typed)} {
				if strings.Contains(text, secret) {
					t.Fatalf("status %d leaked the secret: %s", status, text)
				}
			}
			if rec.count() != 1 {
				t.Fatalf("status %d was retried: %d requests", status, rec.count())
			}
		})
	}
}

func TestTransportErrorsNeverEmbedTheRequestURL(t *testing.T) {
	t.Parallel()
	const secret = "connection-refused-key"
	server := httptest.NewServer(jsonHandler(`[]`))
	baseURL := server.URL
	server.Close()
	client, err := NewClient(WithBaseURL(baseURL), WithAuthentication(FmpQuery(secret)),
		WithTimeout(2*time.Second))
	if err != nil {
		t.Fatal(err)
	}

	_, err = probe(t, client, queryParam{"symbol", "AAPL"})
	typed := assertError(t, err, CategoryTransport, 0)
	cause := typed.Unwrap()
	if cause == nil {
		t.Fatal("transport error kept no cause")
	}
	for _, text := range []string{err.Error(), cause.Error(), fmt.Sprintf("%+v", typed)} {
		if strings.Contains(text, secret) || strings.Contains(text, "contract-probe") {
			t.Fatalf("transport error leaked request details: %s", text)
		}
	}
	var urlErr interface{ Timeout() bool }
	if errors.As(err, &urlErr) {
		t.Fatalf("a raw transport error escaped: %T", cause)
	}
}

func TestTransportCausesOmitNetworkTopology(t *testing.T) {
	t.Parallel()
	closed := func() *url.URL {
		server := httptest.NewServer(jsonHandler(`[]`))
		parsed, err := url.Parse(server.URL)
		if err != nil {
			t.Fatal(err)
		}
		server.Close()
		return parsed
	}
	direct := closed()
	proxy := closed()
	proxy.User = url.UserPassword("proxy-user", "proxy-password")
	cases := []struct {
		name   string
		opts   []Option
		hidden []string
	}{
		{"direct", []Option{WithBaseURL(direct.String())}, []string{direct.Host}},
		{"proxy", []Option{
			WithBaseURL("http://upstream.invalid:8080"),
			WithHTTPClient(&http.Client{Transport: &http.Transport{Proxy: http.ProxyURL(proxy)}}),
		}, []string{proxy.Host, "proxy-password", "upstream.invalid"}},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			client, err := NewClient(append(tc.opts, WithTimeout(2*time.Second))...)
			if err != nil {
				t.Fatal(err)
			}
			_, err = probe(t, client)
			typed := assertError(t, err, CategoryTransport, 0)
			cause := typed.Unwrap()
			if cause == nil || !strings.Contains(cause.Error(), Redacted) {
				t.Fatalf("transport cause = %v, want a redacted cause", cause)
			}
			for _, text := range []string{cause.Error(), fmt.Sprintf("%+v", typed)} {
				for _, host := range tc.hidden {
					if strings.Contains(text, host) {
						t.Fatalf("transport error leaked %q: %s", host, text)
					}
				}
			}
		})
	}
}

func TestCauseAddressesNameEveryNetworkEndpoint(t *testing.T) {
	t.Parallel()
	opErr := &net.OpError{Op: "dial", Net: "tcp",
		Source: &net.TCPAddr{IP: net.IPv4(10, 0, 0, 2), Port: 50000},
		Addr:   &net.TCPAddr{IP: net.IPv4(203, 0, 113, 7), Port: 443},
		Err:    errors.New("connect: connection refused")}
	dnsErr := &net.DNSError{Err: "no such host", Name: "gateway.internal", Server: "10.0.0.53:53"}
	got := causeAddresses(fmt.Errorf("wrapped: %w", errors.Join(opErr, dnsErr)))
	want := []string{"203.0.113.7:443", "10.0.0.2:50000", "gateway.internal", "10.0.0.53:53"}
	if !slices.Equal(got, want) {
		t.Fatalf("causeAddresses = %v, want %v", got, want)
	}
	if got := causeAddresses(errors.New("plain")); len(got) != 0 {
		t.Fatalf("causeAddresses of a plain error = %v, want none", got)
	}
}

func TestTopologyNeverRedactsProviderBodies(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusBadGateway)
		_, _ = w.Write([]byte("upstream 127.0.0.1 unavailable"))
	})
	client := newClient(t, server)
	_, err := probe(t, client)
	typed := assertError(t, err, CategoryStatus, http.StatusBadGateway)
	if typed.Body == nil || typed.Body.Text != "upstream 127.0.0.1 unavailable" {
		t.Fatalf("provider body = %+v, want it unredacted", typed.Body)
	}
}

func TestSameOriginRedirectIsFollowedWithCredentials(t *testing.T) {
	t.Parallel()
	const secret = "redirect-key"
	for _, auth := range []Authentication{Bearer(secret), FmpQuery(secret)} {
		server, rec := newServer(t, func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path == "/router/stable/contract-probe" {
				http.Redirect(w, r, "/router/stable/moved?from=probe#frag", http.StatusFound)
				return
			}
			jsonHandler(`[{"symbol":"AAPL"}]`)(w, r)
		})
		client := newClient(t, server, WithAuthentication(auth))

		rows, err := probe(t, client, queryParam{"symbol", "AAPL"})
		if err != nil || len(rows) != 1 {
			t.Fatalf("%v: rows %v err %v", auth, rows, err)
		}
		reqs := rec.all()
		if len(reqs) != 2 || reqs[1].URL.Path != "/router/stable/moved" {
			t.Fatalf("%v: requests %d, second path %q", auth, len(reqs), reqs[len(reqs)-1].URL.Path)
		}
		for _, req := range reqs {
			carried := req.Header.Get(bearerHeader) == bearerPrefix+secret ||
				req.URL.Query().Get(fmpQueryName) == secret
			if !carried {
				t.Fatalf("%v: hop %s lost the credential", auth, req.URL.Path)
			}
		}
		if got := reqs[1].URL.RawQuery; !strings.HasPrefix(got, "from=probe") ||
			(auth.mode == authFmpQuery && !strings.HasSuffix(got, fmpQueryName+"="+secret)) {
			t.Fatalf("%v: second hop query = %q", auth, got)
		}
	}
}

func TestCrossOriginRedirectIsRefusedWithoutForwardingCredentials(t *testing.T) {
	t.Parallel()
	const secret = "cross-origin-key"
	other, otherRec := newServer(t, jsonHandler(`[]`))
	server, rec := newServer(t, func(w http.ResponseWriter, r *http.Request) {
		http.Redirect(w, r, other.URL+"/router/stable/contract-probe", http.StatusTemporaryRedirect)
	})
	client := newClient(t, server, WithAuthentication(Bearer(secret)))

	_, err := probe(t, client)
	typed := assertError(t, err, CategoryStatus, http.StatusTemporaryRedirect)
	if typed.Body == nil || typed.Body.Text != "" {
		t.Fatalf("redirect status error retained a body: %+v", typed.Body)
	}
	if rec.count() != 1 || otherRec.count() != 0 {
		t.Fatalf("requests: origin %d, other origin %d", rec.count(), otherRec.count())
	}
	if rec.all()[0].Header.Get(bearerHeader) != bearerPrefix+secret {
		t.Fatal("first hop was sent without its credential")
	}
	if strings.Contains(err.Error(), other.URL) {
		t.Fatal("error text contains the redirect target")
	}
}

func TestRedirectLimitAndRedirectNone(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, func(w http.ResponseWriter, r *http.Request) {
		http.Redirect(w, r, "/router/stable/contract-probe", http.StatusMovedPermanently)
	})

	_, err := probe(t, newClient(t, server))
	typed := assertError(t, err, CategoryTransport, 0)
	if typed.Message != "same-origin redirect limit exceeded" || rec.count() != maxRedirects+1 {
		t.Fatalf("limit error = %v after %d requests", err, rec.count())
	}

	none, noneRec := newServer(t, func(w http.ResponseWriter, r *http.Request) {
		http.Redirect(w, r, "/router/stable/moved", http.StatusFound)
	})
	_, err = probe(t, newClient(t, none, WithRedirectPolicy(RedirectNone)))
	_ = assertError(t, err, CategoryStatus, http.StatusFound)
	if noneRec.count() != 1 {
		t.Fatalf("RedirectNone issued %d requests", noneRec.count())
	}
}

func TestResponseBodyCapIsEnforced(t *testing.T) {
	t.Parallel()
	body := strings.Repeat("x", 64)
	for _, chunked := range []bool{false, true} {
		server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			if chunked {
				w.(http.Flusher).Flush()
			}
			_, _ = w.Write([]byte(body))
		})
		client := newClient(t, server, WithMaxResponseBodyBytes(int64(len(body)-1)))
		_, err := probe(t, client)
		typed := assertError(t, err, CategoryTransport, 0)
		if typed.Message != "response body exceeded configured limit" {
			t.Fatalf("chunked=%v: %v", chunked, err)
		}

		exact := newClient(t, server, WithMaxResponseBodyBytes(int64(len(body))))
		_, err = probe(t, exact)
		if typed := assertError(t, err, CategoryDecode, http.StatusOK); typed.Body == nil {
			t.Fatalf("chunked=%v: body at the limit was not buffered", chunked)
		}
	}
}

func TestTimeoutSpansTheWholeCall(t *testing.T) {
	t.Parallel()
	release := make(chan struct{})
	t.Cleanup(func() { close(release) })
	server, _ := newServer(t, func(w http.ResponseWriter, r *http.Request) {
		select {
		case <-release:
		case <-r.Context().Done():
		}
	})
	client := newClient(t, server, WithTimeout(50*time.Millisecond))

	_, err := probe(t, client)
	typed := assertError(t, err, CategoryTransport, 0)
	if typed.Message != "request deadline exceeded" || !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("timeout error = %v (cause %v)", err, typed.Unwrap())
	}
	if strings.Contains(typed.Unwrap().Error(), "contract-probe") {
		t.Fatalf("timeout cause leaked the URL: %v", typed.Unwrap())
	}
}

func TestWithHTTPClientNeverMutatesTheCallerValue(t *testing.T) {
	t.Parallel()
	calls := 0
	caller := &http.Client{CheckRedirect: func(*http.Request, []*http.Request) error {
		calls++
		return nil
	}}
	client, err := NewClient(WithBaseURL("https://proxy.example"), WithHTTPClient(caller))
	if err != nil {
		t.Fatal(err)
	}
	if client.httpClient == caller || client.httpClient.CheckRedirect == nil {
		t.Fatal("client reused the caller's *http.Client")
	}
	if err := client.httpClient.CheckRedirect(nil, nil); !errors.Is(err, http.ErrUseLastResponse) {
		t.Fatalf("installed policy returned %v", err)
	}
	if err := caller.CheckRedirect(nil, nil); err != nil || calls != 1 {
		t.Fatal("caller's CheckRedirect was replaced")
	}
}
