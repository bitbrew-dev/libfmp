package fmp

import (
	"errors"
	"fmt"
	"strings"
	"testing"
)

func TestRedactorRemovesRegisteredSecretsEverywhere(t *testing.T) {
	t.Parallel()
	redactor := NewRedactor()
	redactor.AddSecret("s3cr3t")
	redactor.AddSecret("")
	redactor.AddSecret("s3cr3t")

	got := redactor.Redact("token s3cr3t and again s3cr3ts3cr3t end")
	want := "token [REDACTED] and again [REDACTED] end"
	if got != want {
		t.Fatalf("Redact() = %q, want %q", got, want)
	}
	if len(redactor.secrets) != 1 {
		t.Fatalf("registered %d secrets, want 1", len(redactor.secrets))
	}
}

func TestRedactorRemovesSecretQueryValues(t *testing.T) {
	t.Parallel()
	redactor := NewRedactor()
	cases := map[string]string{
		"https://x.test/q?apikey=abc&symbol=AAPL":  "https://x.test/q?apikey=[REDACTED]&symbol=AAPL",
		"symbol=AAPL&APIKEY=abc":                   "symbol=AAPL&APIKEY=[REDACTED]",
		"notapikey=keep":                           "notapikey=keep",
		"token=abc;sig=def other 'key=ghi' \"x\"":  "token=[REDACTED];sig=[REDACTED] other 'key=[REDACTED]' \"x\"",
		"apikey=abc#fragment":                      "apikey=[REDACTED]#fragment",
		"{\"link\":\"https://e.test?apikey=abc\"}": "{\"link\":\"https://e.test?apikey=[REDACTED]\"}",
	}
	for input, want := range cases {
		if got := redactor.Redact(input); got != want {
			t.Errorf("Redact(%q) = %q, want %q", input, got, want)
		}
	}
}

func TestRedactorCustomNamesAndHeaders(t *testing.T) {
	t.Parallel()
	redactor := NewRedactor()
	if err := redactor.AddSecretQueryName("Router_Token"); err != nil {
		t.Fatal(err)
	}
	if err := redactor.AddSecretHeaderName("X-Proxy-Token"); err != nil {
		t.Fatal(err)
	}
	if got := redactor.Redact("?router_token=abc&x=1"); got != "?router_token=[REDACTED]&x=1" {
		t.Fatalf("custom query redaction = %q", got)
	}
	if got := redactor.RedactHeader("x-proxy-token", "abc"); got != Redacted {
		t.Fatalf("RedactHeader(secret) = %q", got)
	}
	if got := redactor.RedactHeader("Authorization", "Bearer abc"); got != Redacted {
		t.Fatalf("RedactHeader(authorization) = %q", got)
	}
	if got := redactor.RedactHeader("X-Mode", "plain"); got != "plain" {
		t.Fatalf("RedactHeader(plain) = %q", got)
	}
	if err := redactor.AddSecretQueryName(""); !errors.Is(err, ErrEmptySecretName) {
		t.Fatalf("empty name error = %v", err)
	}
	if err := redactor.AddSecretQueryName("a b"); !errors.Is(err, ErrInvalidSecretName) {
		t.Fatalf("invalid query name error = %v", err)
	}
	if err := redactor.AddSecretHeaderName("a:b"); !errors.Is(err, ErrInvalidSecretName) {
		t.Fatalf("invalid header name error = %v", err)
	}
}

func TestRedactorFormattingNeverPrintsSecrets(t *testing.T) {
	t.Parallel()
	redactor := NewRedactor()
	redactor.AddSecret("format-secret")
	for _, verb := range []string{"%v", "%+v", "%#v", "%s", "%d"} {
		text := fmt.Sprintf(verb, redactor)
		if strings.Contains(text, "format-secret") {
			t.Fatalf("%s leaked the secret: %s", verb, text)
		}
		if !strings.Contains(text, "registered_secret_count: 1") {
			t.Fatalf("%s did not describe the redactor: %s", verb, text)
		}
	}
}

func TestSafeBodyIsRedactedAndBounded(t *testing.T) {
	t.Parallel()
	redactor := NewRedactor()
	redactor.AddSecret("body-secret")

	short := NewSafeBody("denied for body-secret", redactor)
	if short.Truncated || short.Text != "denied for [REDACTED]" {
		t.Fatalf("short body = %+v", short)
	}

	long := NewSafeBody("body-secret"+strings.Repeat("é", MaxSafeBodyBytes), redactor)
	if !long.Truncated {
		t.Fatal("long body was not truncated")
	}
	if len(long.Text) > MaxSafeBodyBytes {
		t.Fatalf("truncated body has %d bytes", len(long.Text))
	}
	if !strings.HasSuffix(long.Text, ellipsis) || !strings.HasPrefix(long.Text, Redacted) {
		t.Fatalf("truncated body shape = %q...", long.Text[:32])
	}
	if strings.Contains(long.Text, "body-secret") {
		t.Fatal("truncated body leaked the secret")
	}
}

func TestErrorFormatsMessageEndpointAndBody(t *testing.T) {
	t.Parallel()
	body := NewSafeBody("{\"Error Message\":\"denied\"}", NewRedactor())
	err := statusError("quote-short", 403, &body)
	want := "provider returned HTTP status 403 (endpoint: quote-short): {\"Error Message\":\"denied\"}"
	if err.Error() != want {
		t.Fatalf("Error() = %q, want %q", err.Error(), want)
	}
	if err.Category != CategoryStatus || err.Status != 403 || err.Unwrap() != nil {
		t.Fatalf("status error fields = %+v", err)
	}

	config := configurationError(ConfigurationKindInvalidPath, "path must contain only safe relative segments")
	if config.Error() != config.Message || config.ConfigurationKind != ConfigurationKindInvalidPath {
		t.Fatalf("configuration error = %+v", config)
	}
	var typed *Error
	if !errors.As(error(config), &typed) {
		t.Fatal("errors.As did not match *Error")
	}
}

func TestCategoryStringsMatchTheRustBinding(t *testing.T) {
	t.Parallel()
	cases := map[ErrorCategory]string{
		CategoryValidation:    "validation",
		CategoryConfiguration: "configuration",
		CategoryTransport:     "transport",
		CategoryStatus:        "status",
		CategoryDecode:        "decode",
	}
	for category, want := range cases {
		if got := category.String(); got != want {
			t.Errorf("%d.String() = %q, want %q", int(category), got, want)
		}
	}
}
