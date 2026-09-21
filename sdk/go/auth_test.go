package fmp

import (
	"errors"
	"fmt"
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
	query, err := buildAuthMaterial(CustomQuery("router_token", "tok"))
	if err != nil || query.queryName != "router_token" || query.querySecret != "tok" || query.headerName != "" {
		t.Fatalf("CustomQuery material = %+v, err %v", query, err)
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
		{"custom query bad name", CustomQuery("router\ttoken", "tok"), ConfigurationKindInvalidQueryName},
		{"custom query empty name", CustomQuery("", "tok"), ConfigurationKindInvalidQueryName},
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
