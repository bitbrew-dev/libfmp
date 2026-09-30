package fmp

import (
	"context"
	"net/http"
	"strings"
	"testing"
)

const errorMessageBody = "{\n  \"Error Message\": \"No Data for this symbol or invalid API call. " +
	"Please retry or visit our documentation at https://financialmodelingprep.com/developer/docs.\"\n}"

// economicIndicatorsBody serves body with HTTP 200 as the economic-indicators
// response and returns the rows and error of the call.
func economicIndicatorsBody(t *testing.T, body string) ([]EconomicIndicatorObservation, error) {
	t.Helper()
	server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		_, _ = w.Write([]byte(body))
	})
	client := newClient(t, server)
	return client.Economics.Indicators(context.Background(), NewEconomicIndicatorsQuery(EconomicIndicatorGdp))
}

func TestSuccessStatusProviderMessagesAreStatusErrors(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name string
		body string
	}{
		{"plain text", "Invalid name"},
		{"error message object", errorMessageBody},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			_, err := economicIndicatorsBody(t, tc.body)
			typed := assertQuoteError(t, err, CategoryStatus, http.StatusOK, "economic-indicators")
			if typed.Message != "provider returned an error message with HTTP status 200" {
				t.Fatalf("Message = %q", typed.Message)
			}
			if typed.DecodeKind != DecodeKindNone || typed.Path != "" {
				t.Fatalf("DecodeKind = %v, Path = %q, want none", typed.DecodeKind, typed.Path)
			}
			if typed.Body == nil || typed.Body.Text != tc.body {
				t.Fatalf("Body = %+v, want %q", typed.Body, tc.body)
			}
		})
	}
}

func TestPlainTextProviderMessageFormatsLikeRust(t *testing.T) {
	t.Parallel()
	_, err := economicIndicatorsBody(t, "Invalid name")
	want := "provider returned an error message with HTTP status 200 (endpoint: economic-indicators): Invalid name"
	if err == nil || err.Error() != want {
		t.Fatalf("Error() = %v, want %q", err, want)
	}
}

func TestDocumentedIndicatorRowsStillDecode(t *testing.T) {
	t.Parallel()
	rows, err := economicIndicatorsBody(t, string(readFixture(t, "economic_indicators.json")))
	if err != nil || len(rows) == 0 {
		t.Fatalf("Indicators = %+v, %v", rows, err)
	}
}

func TestOtherMalformedBodiesStayDecodeErrors(t *testing.T) {
	t.Parallel()
	for _, body := range []string{
		"",
		"42",
		`{"Error Message": "x", "date": "2024-01-01"}`,
		`["Invalid name"]`,
		"Invalid\x07name",
		strings.Repeat("x ", 200),
	} {
		_, err := economicIndicatorsBody(t, body)
		typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "economic-indicators")
		if typed.DecodeKind == DecodeKindNone {
			t.Fatalf("body %q: DecodeKind is none", body)
		}
	}
}
