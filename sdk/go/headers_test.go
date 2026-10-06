package fmp

import (
	"fmt"
	"net/http"
	"reflect"
	"strings"
	"testing"
	"time"
)

var valetHeaders = [][2]string{
	{"Retry-After", "12"},
	{"X-Proxy-Key-Id", "vk_123"},
	{"X-Proxy-Error", "rate_limited"},
	{"Set-Cookie", "session=cookie-secret"},
	{"Authorization", "Bearer header-secret"},
	{"apikey", "query-secret"},
	{"Cookie", "cookie=jar-secret"},
	{"X-Unrelated", "unrelated-value"},
	{"X-Proxy-Burst-Remaining", "0"},
	{"X-RateLimit-Remaining", "0"},
}

func valetServer(t *testing.T, status int, contentType, body string) *Client {
	t.Helper()
	server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		for _, pair := range valetHeaders {
			w.Header().Add(pair[0], pair[1])
		}
		if contentType != "" {
			w.Header().Set("Content-Type", contentType)
		}
		w.WriteHeader(status)
		_, _ = w.Write([]byte(body))
	})
	return newClient(t, server)
}

func assertOnlyAllowlisted(t *testing.T, typed *Error) {
	t.Helper()
	want := http.Header{
		"Retry-After":             {"12"},
		"X-Proxy-Key-Id":          {"vk_123"},
		"X-Proxy-Error":           {"rate_limited"},
		"X-Proxy-Burst-Remaining": {"0"},
		"X-Ratelimit-Remaining":   {"0"},
	}
	if !reflect.DeepEqual(typed.Headers, want) {
		t.Fatalf("Headers = %v, want %v", typed.Headers, want)
	}
	if delay, ok := typed.RetryAfter(); !ok || delay != 12*time.Second {
		t.Fatalf("RetryAfter() = %v, %v, want 12s", delay, ok)
	}
	if got := typed.ProxyError(); got != "rate_limited" {
		t.Fatalf("ProxyError() = %q", got)
	}
	dump := fmt.Sprintf("%+v %s", typed, typed.Error())
	for _, secret := range []string{"cookie-secret", "header-secret", "query-secret", "jar-secret", "unrelated-value"} {
		if strings.Contains(dump, secret) {
			t.Fatalf("error retained %q: %s", secret, dump)
		}
	}
}

func TestStatusErrorKeepsRetryAfterAndProxyHeadersOnly(t *testing.T) {
	t.Parallel()
	_, err := probe(t, valetServer(t, http.StatusTooManyRequests, "", "slow down"))
	typed := assertError(t, err, CategoryStatus, http.StatusTooManyRequests)
	assertOnlyAllowlisted(t, typed)
	if want := "provider returned HTTP status 429 (endpoint: endpoint-contract-test): slow down"; typed.Error() != want {
		t.Fatalf("Error() = %q, want %q", typed.Error(), want)
	}
}

func TestProviderMessageKeepsAllowlistedHeaders(t *testing.T) {
	t.Parallel()
	_, err := probe(t, valetServer(t, http.StatusOK, "application/json", "Invalid name"))
	assertOnlyAllowlisted(t, assertError(t, err, CategoryStatus, http.StatusOK))
}

func TestStatusErrorWithoutAllowlistedHeadersHasNilHeaders(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	})
	_, err := probe(t, newClient(t, server))
	typed := assertError(t, err, CategoryStatus, http.StatusInternalServerError)
	if _, ok := typed.RetryAfter(); typed.Headers != nil || ok || typed.ProxyError() != "" {
		t.Fatalf("Headers = %v, want nil", typed.Headers)
	}
}

func TestIsRetainedHeaderNameMatchesTheRustAllowlist(t *testing.T) {
	t.Parallel()
	for _, name := range []string{"Retry-After", "X-Proxy-Error", "x-proxy-anything", "X-RateLimit-Reset"} {
		if !IsRetainedHeaderName(name) {
			t.Errorf("%s was not retained", name)
		}
	}
	for _, name := range []string{"set-cookie", "authorization", "apikey", "cookie", "x-proxy", "retry-after-ms"} {
		if IsRetainedHeaderName(name) {
			t.Errorf("%s was retained", name)
		}
	}
}

func TestRetryAfterAcceptsSecondsAndEveryHTTPDateForm(t *testing.T) {
	t.Parallel()
	now := time.Unix(784_111_717, 0)
	cases := []struct {
		value string
		want  time.Duration
		ok    bool
	}{
		{"Sun, 06 Nov 1994 08:49:37 GMT", time.Minute, true},
		{"Sunday, 06-Nov-94 08:49:37 GMT", time.Minute, true},
		{"Sun Nov  6 08:49:37 1994", time.Minute, true},
		{"Sun, 06 Nov 1994 08:00:00 GMT", 0, true},
		{" 7 ", 7 * time.Second, true},
		{"soon", 0, false},
		{"-5", 0, false},
		{"1.5", 0, false},
		{"", 0, false},
	}
	for _, tc := range cases {
		if got, ok := retryAfterAt(tc.value, now); got != tc.want || ok != tc.ok {
			t.Errorf("retryAfterAt(%q) = %v, %v, want %v, %v", tc.value, got, ok, tc.want, tc.ok)
		}
	}
}

func TestRetainedHeadersAreRedactedAndBounded(t *testing.T) {
	t.Parallel()
	redactor := NewRedactor()
	redactor.AddSecret("vk_live_secret")
	header := http.Header{
		"X-Proxy-Key-Id": {"vk_live_secret"},
		"X-Proxy-Long":   {strings.Repeat("x", MaxSafeHeaderValueBytes+1)},
	}
	for i := range MaxSafeHeaders + 4 {
		header.Add("X-Proxy-Zz", fmt.Sprint(i))
	}
	kept := retainedHeaders(header, redactor)
	total := 0
	for _, values := range kept {
		total += len(values)
	}
	if total != MaxSafeHeaders || kept.Get("X-Proxy-Key-Id") != Redacted || kept.Get("X-Proxy-Long") != "" {
		t.Fatalf("retainedHeaders = %v", kept)
	}
}
