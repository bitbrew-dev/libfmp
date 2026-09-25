package fmp

import (
	"errors"
	"fmt"
	"os"
	"strings"
	"testing"
)

// Opt-in live tests against the real provider and a production proxy route,
// mirroring crates/libfmp/tests/live_opt_in.rs. Every TestLive* test skips
// unless FMP_LIVE_TESTS is 1 (after trimming) and its variables are set, so the
// default gate never opens a socket. Run them explicitly from sdk/go:
//
//	FMP_LIVE_TESTS=1 FMP_API_KEY=... go test -run 'TestLive' -count=1 ./...
//
// The proxy test also needs FMP_PROXY_BASE_URL and FMP_PROXY_TOKEN: the token
// is sent as "X-Proxy-Token: Bearer <token>", FMP_PROXY_PATH_PREFIX overrides
// the router/stable prefix, and FMP_TENANT adds the X-Tenant header when set.
// Values are read from the environment, handed to the client, and never
// formatted by the tests: on failure the helpers prove the error text does
// not contain the credential (the API key or the proxy token, as in the Rust
// file) before the test fails with that text.
const (
	liveSwitchEnv          = "FMP_LIVE_TESTS"
	liveAPIKeyEnv          = EnvAPIKey
	liveProxyBaseURLEnv    = "FMP_PROXY_BASE_URL"
	liveProxyTokenEnv      = "FMP_PROXY_TOKEN"
	liveProxyPathPrefixEnv = "FMP_PROXY_PATH_PREFIX"
	liveTenantEnv          = "FMP_TENANT"

	liveDefaultProxyPathPrefix = "router/stable"
	liveSymbol                 = "AAPL"
)

// liveEnvNames lists every variable the live tests read, so the offline proof
// can clear all of them regardless of the developer's shell.
var liveEnvNames = []string{
	liveSwitchEnv, liveAPIKeyEnv, liveProxyBaseURLEnv,
	liveProxyTokenEnv, liveProxyPathPrefixEnv, liveTenantEnv,
}

// liveValue reads one variable, trims it, and treats empty as unset.
func liveValue(name string) (string, bool) {
	value := strings.TrimSpace(os.Getenv(name))
	return value, value != ""
}

// liveEnv returns the values of names only when the live switch is on and
// every variable is set; otherwise it skips the test naming the missing
// variable, never a value.
func liveEnv(t *testing.T, names ...string) []string {
	t.Helper()
	if value, _ := liveValue(liveSwitchEnv); value != "1" {
		t.Skipf("%s is not set to 1", liveSwitchEnv)
	}
	values := make([]string, 0, len(names))
	for _, name := range names {
		value, ok := liveValue(name)
		if !ok {
			t.Skipf("%s is not set", name)
		}
		values = append(values, value)
	}
	return values
}

// liveAssertNoLeak fails with a fixed message when text contains any of the
// credentials. Empty secrets are ignored because every string contains the
// empty string.
func liveAssertNoLeak(t *testing.T, text string, secrets ...string) {
	t.Helper()
	for _, secret := range secrets {
		if secret != "" && strings.Contains(text, secret) {
			t.Fatal("diagnostic text leaked a configured secret")
		}
	}
}

// liveErrorText concatenates the error and every unwrapped cause, the nearest
// match to the Rust file's "{error:?} {error}" diagnostic.
func liveErrorText(err error) string {
	var b strings.Builder
	for ; err != nil; err = errors.Unwrap(err) {
		b.WriteString(err.Error())
		b.WriteString(" | ")
	}
	return b.String()
}

// liveQuoteShort requests the compact AAPL quote and returns the rows. The
// client's formatted shape and, on failure, the error text are both proven
// free of the credential before anything is reported.
func liveQuoteShort(t *testing.T, client *Client, secrets ...string) []QuoteShort {
	t.Helper()
	liveAssertNoLeak(t, fmt.Sprintf("%v %+v %#v", client, client, client), secrets...)
	rows, err := client.Quote.Short(t.Context(), NewQuoteShortQuery(liveSymbol))
	if err != nil {
		diagnostic := liveErrorText(err)
		liveAssertNoLeak(t, diagnostic, secrets...)
		t.Fatalf("live quote-short request failed: %s", diagnostic)
	}
	return rows
}

func liveAssertAppleRows(t *testing.T, rows []QuoteShort) {
	t.Helper()
	if len(rows) == 0 {
		t.Fatal("quote-short returned an empty array")
	}
	if rows[0].Symbol != liveSymbol {
		t.Fatalf("quote-short symbol = %q, want %q", rows[0].Symbol, liveSymbol)
	}
	if rows[0].Price <= 0 {
		t.Fatalf("quote-short price = %v, want a positive price", rows[0].Price)
	}
}

func TestLiveDirectHeaderAuthenticationReturnsQuoteShortRows(t *testing.T) {
	values := liveEnv(t, liveAPIKeyEnv)
	apiKey := values[0]
	client, err := NewClient(WithAuthentication(FMPHeader(apiKey)))
	if err != nil {
		liveAssertNoLeak(t, liveErrorText(err), apiKey)
		t.Fatalf("NewClient: %v", err)
	}

	rows := liveQuoteShort(t, client, apiKey)

	liveAssertAppleRows(t, rows)
}

func TestLiveProductionProxyRouteReturnsQuoteShortRows(t *testing.T) {
	values := liveEnv(t, liveProxyBaseURLEnv, liveProxyTokenEnv)
	baseURL, token := values[0], values[1]
	pathPrefix, ok := liveValue(liveProxyPathPrefixEnv)
	if !ok {
		pathPrefix = liveDefaultProxyPathPrefix
	}
	opts := []Option{
		WithBaseURL(baseURL),
		WithPathPrefix(pathPrefix),
		WithAuthentication(CustomHeaderWithPrefix("X-Proxy-Token", "Bearer ", token)),
	}
	tenant, hasTenant := liveValue(liveTenantEnv)
	if hasTenant {
		opts = append(opts, WithDefaultHeader("X-Tenant", tenant))
	}
	client, err := NewClient(opts...)
	if err != nil {
		liveAssertNoLeak(t, liveErrorText(err), token)
		t.Fatalf("NewClient: %v", err)
	}

	rows := liveQuoteShort(t, client, token)

	liveAssertAppleRows(t, rows)
}

// liveClearEnv unsets every live variable for the duration of the test, so
// the offline proof holds even when the developer's shell exports the switch.
func liveClearEnv(t *testing.T) {
	t.Helper()
	for _, name := range liveEnvNames {
		t.Setenv(name, "")
		if err := os.Unsetenv(name); err != nil {
			t.Fatal(err)
		}
	}
}

// TestLiveEnvGateSkipsOffline proves the default gate takes the skip path:
// the gate helper skips without the switch or a variable, and both live tests
// return without failing once the environment is cleared. A non-skipping path
// would reach NewClient with an empty credential and fail, so a true result
// from t.Run is only reachable through the skip.
func TestLiveEnvGateSkipsOffline(t *testing.T) {
	liveClearEnv(t)

	reached := false
	ok := t.Run("switch unset", func(t *testing.T) {
		liveEnv(t, liveAPIKeyEnv)
		reached = true
	})
	if !ok || reached {
		t.Fatalf("switch unset: ok = %v, reached = %v, want a skip", ok, reached)
	}

	t.Setenv(liveSwitchEnv, " 1 ")
	ok = t.Run("variable unset", func(t *testing.T) {
		liveEnv(t, liveAPIKeyEnv)
		reached = true
	})
	if !ok || reached {
		t.Fatalf("variable unset: ok = %v, reached = %v, want a skip", ok, reached)
	}

	t.Setenv(liveAPIKeyEnv, " unit-only-value\n")
	var got []string
	ok = t.Run("switch and variable set", func(t *testing.T) {
		got = liveEnv(t, liveAPIKeyEnv)
		reached = true
	})
	if !ok || !reached || len(got) != 1 || got[0] != "unit-only-value" {
		t.Fatalf("switch and variable set: ok = %v, reached = %v, want the trimmed value", ok, reached)
	}

	liveClearEnv(t)
	if !t.Run("direct", TestLiveDirectHeaderAuthenticationReturnsQuoteShortRows) {
		t.Fatal("the direct live test did not skip with a cleared environment")
	}
	if !t.Run("proxy", TestLiveProductionProxyRouteReturnsQuoteShortRows) {
		t.Fatal("the proxy live test did not skip with a cleared environment")
	}
}
