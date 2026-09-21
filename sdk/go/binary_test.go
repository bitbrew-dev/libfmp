package fmp

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

const (
	binaryEndpoint  = "binary-contract-test"
	binaryPath      = "future-download"
	binaryMediaType = "application/octet-stream"
)

func getBinaryProbe(t *testing.T, client *Client) (BinaryPayload, error) {
	t.Helper()
	return client.getBinary(context.Background(), binaryEndpoint, binaryPath, nil, []string{binaryMediaType})
}

func assertBinaryError(t *testing.T, err error, category ErrorCategory, status int) *Error {
	t.Helper()
	var typed *Error
	if !errors.As(err, &typed) {
		t.Fatalf("error %v (%T) is not *Error", err, err)
	}
	if typed.Category != category || typed.Status != status || typed.Endpoint != binaryEndpoint {
		t.Fatalf("error = %+v, want category %v status %d endpoint %q", typed, category, status, binaryEndpoint)
	}
	return typed
}

// TestGetBinaryRetainsBytesAndSafeContentMetadata mirrors
// binary_contract_retains_bytes_and_safe_content_metadata.
func TestGetBinaryRetainsBytesAndSafeContentMetadata(t *testing.T) {
	t.Parallel()
	const (
		contentType = "Application/Octet-Stream; token=private-media-token"
		disposition = "attachment; filename=private-report-name.xlsx"
		payload     = "private-binary-payload"
	)
	server, rec := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", contentType)
		w.Header().Set("Content-Disposition", disposition)
		_, _ = w.Write([]byte(payload))
	})
	body, err := getBinaryProbe(t, newClient(t, server))
	if err != nil {
		t.Fatal(err)
	}
	if string(body.Data) != payload || body.ContentType != contentType || body.ContentDisposition != disposition {
		t.Fatalf("payload = %+v", body)
	}
	if rec.count() != 1 {
		t.Fatalf("issued %d requests, want exactly one", rec.count())
	}
	if body.MediaType() != "Application/Octet-Stream" {
		t.Fatalf("MediaType() = %q", body.MediaType())
	}

	for _, formatted := range []string{
		fmt.Sprintf("%v %+v %#v %s", body, body, body, body),
		fmt.Sprintf("%v %+v %#v %s", &body, &body, &body, &body),
		fmt.Sprint(struct{ Payload BinaryPayload }{body}),
	} {
		for _, want := range []string{"body_bytes: 22", `media_type: "Application/Octet-Stream"`, "has_content_disposition: true"} {
			if !strings.Contains(formatted, want) {
				t.Fatalf("formatted payload %q lacks %q", formatted, want)
			}
		}
		for _, leak := range []string{"private-binary-payload", "private-report-name", "private-media-token"} {
			if strings.Contains(formatted, leak) {
				t.Fatalf("formatted payload leaked %q: %s", leak, formatted)
			}
		}
	}

	plain, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", binaryMediaType)
		_, _ = w.Write([]byte(payload))
	})
	body, err = getBinaryProbe(t, newClient(t, plain))
	if err != nil || body.ContentDisposition != "" || !bytes.Equal(body.Data, []byte(payload)) {
		t.Fatalf("payload without disposition = %+v (%v)", body, err)
	}
	if !strings.Contains(fmt.Sprint(body), "has_content_disposition: false") {
		t.Fatalf("formatted payload = %v", body)
	}
}

// TestGetBinaryRejectsMissingAndWrongContentTypes mirrors
// binary_contract_rejects_missing_and_wrong_content_types.
func TestGetBinaryRejectsMissingAndWrongContentTypes(t *testing.T) {
	t.Parallel()
	const payload = "future-binary-response"
	cases := []struct {
		name    string
		handler http.HandlerFunc
		message string
	}{
		{"json", jsonHandler(payload), "successful response used an unexpected content type"},
		{"missing", func(w http.ResponseWriter, _ *http.Request) {
			w.Header()["Content-Type"] = nil
			_, _ = w.Write([]byte(payload))
		}, "successful response omitted its content type"},
		{"invalid text", func(w http.ResponseWriter, _ *http.Request) {
			w.Header().Set("Content-Type", binaryMediaType+"; charset=\x80")
			_, _ = w.Write([]byte(payload))
		}, "successful response used an invalid content type"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			server, rec := newServer(t, tc.handler)
			_, err := getBinaryProbe(t, newClient(t, server))
			typed := assertBinaryError(t, err, CategoryDecode, http.StatusOK)
			if typed.Message != tc.message || typed.Body == nil || typed.Body.Text != payload || rec.count() != 1 {
				t.Fatalf("error = %+v after %d requests", typed, rec.count())
			}
		})
	}
}

// TestGetBinaryDiagnosticsAreBoundedAndTokenSafe mirrors
// unexpected_binary_content_has_bounded_token_safe_diagnostics.
func TestGetBinaryDiagnosticsAreBoundedAndTokenSafe(t *testing.T) {
	t.Parallel()
	const (
		authSecret = "transport-secret-token"
		bodySecret = "body-secret-token"
	)
	body := fmt.Sprintf(`{"link":"https://example.test/report?apikey=%s","detail":"%s%s"}`,
		bodySecret, authSecret, strings.Repeat("x", MaxSafeBodyBytes))
	server, _ := newServer(t, jsonHandler(body))
	client := newClient(t, server, WithAuthentication(CustomQuery("router_token", authSecret)))

	_, err := getBinaryProbe(t, client)
	typed := assertBinaryError(t, err, CategoryDecode, http.StatusOK)
	if typed.Body == nil || !typed.Body.Truncated || len(typed.Body.Text) > MaxSafeBodyBytes {
		t.Fatalf("safe body = %+v", typed.Body)
	}
	diagnostic := fmt.Sprintf("%v %+v %s", typed, typed, err.Error())
	for _, secret := range []string{authSecret, bodySecret} {
		if strings.Contains(diagnostic, secret) {
			t.Fatalf("diagnostic leaked %q: %s", secret, diagnostic)
		}
	}
}

func TestGetBinaryAppliesTheBodyCapAndPathRules(t *testing.T) {
	t.Parallel()
	const payload = "future-binary-response"
	server, rec := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", binaryMediaType)
		_, _ = w.Write([]byte(payload))
	})
	capped := newClient(t, server, WithMaxResponseBodyBytes(int64(len(payload)-1)))
	_, err := getBinaryProbe(t, capped)
	typed := assertBinaryError(t, err, CategoryTransport, 0)
	if typed.Message != "response body exceeded configured limit" || rec.count() != 1 {
		t.Fatalf("capped error = %v after %d requests", err, rec.count())
	}

	exact := newClient(t, server, WithMaxResponseBodyBytes(int64(len(payload))))
	if body, err := getBinaryProbe(t, exact); err != nil || string(body.Data) != payload {
		t.Fatalf("body at the limit = %+v (%v)", body, err)
	}

	_, err = exact.getBinary(context.Background(), binaryEndpoint, "../escape", nil, []string{binaryMediaType})
	assertConfigurationKind(t, err, ConfigurationKindInvalidPath)
	if rec.count() != 2 {
		t.Fatalf("invalid path issued a request (%d total)", rec.count())
	}
}
