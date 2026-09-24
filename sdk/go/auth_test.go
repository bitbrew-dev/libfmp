package fmp

import (
	"errors"
	"fmt"
	"net/http"
	"os"
	"strings"
	"testing"
)

func TestFmpHeaderFromEnvNormalizesTheVariable(t *testing.T) {
	cases := []struct {
		name  string
		value string
		want  string
		ok    bool
	}{
		{name: "unset", ok: false},
		{name: "empty", value: "", ok: false},
		{name: "blank", value: "   \t\n", ok: false},
		{name: "plain", value: "shell-secret", want: "shell-secret", ok: true},
		{name: "padded", value: "  shell-secret\n", want: "shell-secret", ok: true},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if tc.name == "unset" {
				t.Setenv(EnvAPIKey, "")
				if err := os.Unsetenv(EnvAPIKey); err != nil {
					t.Fatal(err)
				}
			} else {
				t.Setenv(EnvAPIKey, tc.value)
			}
			auth, ok := FmpHeaderFromEnv()
			if ok != tc.ok {
				t.Fatalf("ok = %v, want %v", ok, tc.ok)
			}
			if ok && (auth.mode != authFmpHeader || auth.secret != tc.want) {
				t.Fatalf("auth = mode %d secret %q, want %q", auth.mode, auth.secret, tc.want)
			}
			if !ok && auth != (Authentication{}) {
				t.Fatalf("absent key produced %v", auth)
			}
		})
	}
}

func TestAuthenticationFormattingNeverPrintsSecrets(t *testing.T) {
	t.Parallel()
	const secret = "format-secret"
	auths := []Authentication{
		FmpHeader(secret), FmpQuery(secret), Bearer(secret),
		CustomHeader("X-Proxy-Token", secret), CustomQuery("router_token", secret),
		CustomHeaderWithPrefix("X-Proxy-Token", "Bearer ", secret),
	}
	for _, auth := range auths {
		for _, verb := range []string{"%v", "%+v", "%#v", "%s", "%d", "%q"} {
			text := fmt.Sprintf(verb, auth)
			if strings.Contains(text, secret) {
				t.Fatalf("%s leaked the secret: %s", verb, text)
			}
			if !strings.Contains(text, Redacted) {
				t.Fatalf("%s did not describe the mode: %s", verb, text)
			}
		}
	}
	if got := fmt.Sprint(Authentication{}); got != "None" {
		t.Fatalf("zero value prints %q", got)
	}
}

func TestAuthMaterialMirrorsTheRustModes(t *testing.T) {
	t.Parallel()
	header, err := buildAuthMaterial(FmpHeader("k"))
	if err != nil || header.headerName != fmpHeaderName || header.headerValue != "k" || header.queryName != "" {
		t.Fatalf("FmpHeader material = %+v, err %v", header, err)
	}
	bearer, err := buildAuthMaterial(Bearer("tok"))
	if err != nil || bearer.headerName != bearerHeader || bearer.headerValue != bearerPrefix+"tok" {
		t.Fatalf("bearer material = %+v, err %v", bearer, err)
	}
	custom, err := buildAuthMaterial(CustomHeader("X-Proxy-Token", "tok"))
	if err != nil || custom.headerName != "x-proxy-token" || custom.headerValue != "tok" {
		t.Fatalf("CustomHeader material = %+v, err %v", custom, err)
	}
	spaced, err := buildAuthMaterial(CustomHeaderWithPrefix("X-Proxy-Token", "Bearer ", "tok"))
	if err != nil || spaced.headerName != "x-proxy-token" || spaced.headerValue != "Bearer tok" {
		t.Fatalf("CustomHeaderWithPrefix material = %+v, err %v", spaced, err)
	}
	joined, err := buildAuthMaterial(CustomHeaderWithPrefix("X-Proxy-Token", "Bearer", "tok"))
	if err != nil || joined.headerValue != "Bearertok" {
		t.Fatalf("CustomHeaderWithPrefix inserted a separator: %+v, err %v", joined, err)
	}
	query, err := buildAuthMaterial(CustomQuery("router_token", "tok"))
	if err != nil || query.queryName != "router_token" || query.querySecret != "tok" || query.headerName != "" {
		t.Fatalf("CustomQuery material = %+v, err %v", query, err)
	}
	unreserved, err := buildAuthMaterial(CustomQuery("Router.api-key_1~", "tok"))
	if err != nil || unreserved.queryName != "Router.api-key_1~" {
		t.Fatalf("CustomQuery rejected an unreserved name: %+v, err %v", unreserved, err)
	}
	none, err := buildAuthMaterial(Authentication{})
	if err != nil || none != (authMaterial{}) {
		t.Fatalf("None material = %+v, err %v", none, err)
	}
}

func TestAuthMaterialRejectsUnsafeInput(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name string
		auth Authentication
		kind ConfigurationKind
	}{
		{"empty header key", FmpHeader(""), ConfigurationKindEmptyCredential},
		{"empty query key", FmpQuery(""), ConfigurationKindEmptyCredential},
		{"empty bearer", Bearer(""), ConfigurationKindEmptyCredential},
		{"custom header bad name", CustomHeader("x proxy", "tok"), ConfigurationKindInvalidHeaderName},
		{"custom header transport owned", CustomHeader("Host", "tok"), ConfigurationKindProtectedFieldCollision},
		{"custom header bad value", CustomHeader("X-Proxy-Token", "tok\n"), ConfigurationKindInvalidHeaderValue},
		{"custom header prefix crlf", CustomHeaderWithPrefix("X-Proxy-Token", "Bearer\r\n", "tok"),
			ConfigurationKindInvalidHeaderValue},
		{"custom header prefix control", CustomHeaderWithPrefix("X-Proxy-Token", "Bearer\x00", "tok"),
			ConfigurationKindInvalidHeaderValue},
		{"custom header prefix non ascii", CustomHeaderWithPrefix("X-Proxy-Token", "Träger ", "tok"),
			ConfigurationKindInvalidHeaderValue},
		{"custom header prefix bad name", CustomHeaderWithPrefix("x proxy", "Bearer ", "tok"),
			ConfigurationKindInvalidHeaderName},
		{"custom header prefix empty secret", CustomHeaderWithPrefix("X-Proxy-Token", "Bearer ", ""),
			ConfigurationKindEmptyCredential},
		{"custom query bad name", CustomQuery("router\ttoken", "tok"), ConfigurationKindInvalidQueryName},
		{"custom query empty name", CustomQuery("", "tok"), ConfigurationKindInvalidQueryName},
		{"custom query space in name", CustomQuery("router token", "tok"), ConfigurationKindInvalidQueryName},
		{"custom query percent in name", CustomQuery("router%74oken", "tok"), ConfigurationKindInvalidQueryName},
		{"custom query plus in name", CustomQuery("router+token", "tok"), ConfigurationKindInvalidQueryName},
		{"custom query non ascii name", CustomQuery("routér_token", "tok"), ConfigurationKindInvalidQueryName},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			_, err := buildAuthMaterial(tc.auth)
			assertConfigurationKind(t, err, tc.kind)
		})
	}
}

func TestRegisterAuthRedactionProtectsNamesAndValues(t *testing.T) {
	t.Parallel()
	redactor := NewRedactor()
	if err := registerAuthRedaction(redactor, CustomQuery("router_token", "auth-secret")); err != nil {
		t.Fatal(err)
	}
	got := redactor.Redact("auth-secret ?router_token=other&x=1")
	if got != "[REDACTED] ?router_token=[REDACTED]&x=1" {
		t.Fatalf("redacted text = %q", got)
	}
	redactor = NewRedactor()
	if err := registerAuthRedaction(redactor, CustomHeader("X-Proxy-Token", "auth-secret")); err != nil {
		t.Fatal(err)
	}
	if redactor.RedactHeader("X-PROXY-TOKEN", "anything") != Redacted {
		t.Fatal("custom header name was not registered as secret")
	}
}

func assertConfigurationKind(t *testing.T, err error, kind ConfigurationKind) {
	t.Helper()
	var typed *Error
	if !errors.As(err, &typed) {
		t.Fatalf("error %v (%T) is not *Error", err, err)
	}
	if typed.Category != CategoryConfiguration || typed.ConfigurationKind != kind {
		t.Fatalf("error = category %v kind %d, want configuration kind %d: %v",
			typed.Category, int(typed.ConfigurationKind), int(kind), err)
	}
}

func TestCustomHeaderWithPrefixEmptyPrefixIsCustomHeader(t *testing.T) {
	t.Parallel()
	plain := CustomHeader("X-Proxy-Token", "example-key")
	unprefixed := CustomHeaderWithPrefix("X-Proxy-Token", "", "example-key")
	if unprefixed != plain {
		t.Fatalf("empty prefix produced a different value: %v vs %v", unprefixed, plain)
	}
	plainMaterial, err := buildAuthMaterial(plain)
	if err != nil {
		t.Fatal(err)
	}
	unprefixedMaterial, err := buildAuthMaterial(unprefixed)
	if err != nil {
		t.Fatal(err)
	}
	if unprefixedMaterial != plainMaterial {
		t.Fatal("empty prefix produced different transport material")
	}
	if fmt.Sprint(unprefixed) != fmt.Sprint(plain) {
		t.Fatalf("empty prefix prints differently: %v vs %v", unprefixed, plain)
	}
}

// TestCustomHeaderFormattingReportsPrefixPresence mirrors the Rust Debug
// output, which prints has_prefix so a "Bearer" prefix missing its trailing
// space is visible in logs while the prefix text itself is not.
func TestCustomHeaderFormattingReportsPrefixPresence(t *testing.T) {
	t.Parallel()
	const prefix = "Bearer "
	prefixed := fmt.Sprint(CustomHeaderWithPrefix("X-Proxy-Token", prefix, "example-key"))
	if !strings.Contains(prefixed, "has_prefix: true") || strings.Contains(prefixed, prefix) {
		t.Fatalf("prefixed value printed as %s", prefixed)
	}
	if plain := fmt.Sprint(CustomHeader("X-Proxy-Token", "example-key")); !strings.Contains(plain, "has_prefix: false") {
		t.Fatalf("plain value printed as %s", plain)
	}
}

func TestCustomHeaderWithPrefixRejectsInvalidPrefixAtClientBuild(t *testing.T) {
	t.Parallel()
	_, err := NewClient(WithBaseURL("https://proxy.example/router"),
		WithAuthentication(CustomHeaderWithPrefix("X-Proxy-Token", "Bearer\r\n", "example-key")))
	assertConfigurationKind(t, err, ConfigurationKindInvalidHeaderValue)
	if strings.Contains(err.Error(), "example-key") {
		t.Fatalf("configuration error leaked the credential: %v", err)
	}
	_, err = NewClient(WithBaseURL("https://proxy.example/router"),
		WithAuthentication(CustomHeaderWithPrefix("X-Proxy-Token", "Bearer ", "example-key")))
	if err != nil {
		t.Fatalf("valid prefix was rejected: %v", err)
	}
}

func TestCustomHeaderWithPrefixSendsPrefixThenSecretByteForByte(t *testing.T) {
	t.Parallel()
	const secret = "example-key"
	cases := []struct {
		name   string
		prefix string
		want   string
	}{
		{name: "trailing space", prefix: "Bearer ", want: "Bearer " + secret},
		{name: "no separator", prefix: "Bearer", want: "Bearer" + secret},
		{name: "tab", prefix: "Token\t", want: "Token\t" + secret},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			server, rec := newServer(t, jsonHandler(`[]`))
			client := newClient(t, server,
				WithAuthentication(CustomHeaderWithPrefix("X-Proxy-Token", tc.prefix, secret)))
			if _, err := probe(t, client, queryParam{"symbol", "AAPL"}); err != nil {
				t.Fatal(err)
			}
			reqs := rec.all()
			if len(reqs) != 1 {
				t.Fatalf("%d requests, want 1", len(reqs))
			}
			if got := reqs[0].Header.Get("X-Proxy-Token"); got != tc.want {
				t.Fatalf("header value = %q, want %q", got, tc.want)
			}
			if strings.Contains(reqs[0].URL.String(), secret) {
				t.Fatal("header credential appeared in the URL")
			}
		})
	}
}

func TestCustomHeaderWithPrefixSecretIsRedactedFromStatusErrors(t *testing.T) {
	t.Parallel()
	const secret = "example-key"
	server, _ := newServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusBadGateway)
		_, _ = fmt.Fprintf(w, "proxy rejected %s", r.Header.Get("X-Proxy-Token"))
	})
	client := newClient(t, server,
		WithAuthentication(CustomHeaderWithPrefix("X-Proxy-Token", "Bearer ", secret)))

	_, err := probe(t, client)
	typed := assertError(t, err, CategoryStatus, http.StatusBadGateway)
	if typed.Body == nil || typed.Body.Text != "proxy rejected Bearer "+Redacted {
		t.Fatalf("status body was not redacted: %v", typed.Body)
	}
	assertNoSecret(t, secret, err.Error(), typed.Body.Text, fmt.Sprintf("%+v", typed))
}

// echoingTransport fails every request with an error that quotes the secret
// header, the way a misbehaving proxy or debugging transport might. The
// http.Client wraps that error in a *url.Error before the SDK sees it.
type echoingTransport struct{}

func (echoingTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	return nil, fmt.Errorf("proxy refused header value %q", req.Header.Get("X-Proxy-Token"))
}

func TestCustomHeaderWithPrefixSecretIsRedactedFromTransportErrors(t *testing.T) {
	t.Parallel()
	const secret = "example-key"
	client, err := NewClient(WithBaseURL("https://proxy.example/router"),
		WithHTTPClient(&http.Client{Transport: echoingTransport{}}),
		WithAuthentication(CustomHeaderWithPrefix("X-Proxy-Token", "Bearer ", secret)))
	if err != nil {
		t.Fatal(err)
	}

	_, err = probe(t, client)
	typed := assertError(t, err, CategoryTransport, 0)
	cause := typed.Unwrap()
	if cause == nil {
		t.Fatal("transport error kept no cause")
	}
	if !strings.Contains(cause.Error(), "Bearer "+Redacted) {
		t.Fatalf("cause did not keep the redacted header value: %v", cause)
	}
	if strings.Contains(cause.Error(), "contract-probe") {
		t.Fatalf("cause kept the *url.Error request URL: %v", cause)
	}
	var urlErr interface{ Timeout() bool }
	if errors.As(err, &urlErr) {
		t.Fatalf("a raw transport error escaped: %T", cause)
	}
	assertNoSecret(t, secret, err.Error(), cause.Error(), fmt.Sprintf("%+v", typed))
}

func assertNoSecret(t *testing.T, secret string, texts ...string) {
	t.Helper()
	for _, text := range texts {
		if strings.Contains(text, secret) {
			t.Fatalf("diagnostic leaked the credential: %s", text)
		}
	}
}
