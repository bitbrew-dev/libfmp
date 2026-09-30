package fmp

import (
	"context"
	"encoding/json/jsontext"
	"errors"
	"net/http"
	"strings"
	"testing"
)

const decodePathSentinel = "SENTINEL-beta-text"

// decodeScreenerBody serves body as the company screener response and returns
// the Decode error the call must fail with.
func decodeScreenerBody(t *testing.T, body []byte) *Error {
	t.Helper()
	server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(body)
	})
	client := newClient(t, server)
	_, err := client.Screener.Companies(context.Background(), NewCompanyScreenerQuery())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "company-screener")
	if typed.Message != "successful response could not be decoded" {
		t.Fatalf("Message = %q", typed.Message)
	}
	return typed
}

func TestDecodeErrorNamesTheNullMemberRowAndMember(t *testing.T) {
	t.Parallel()
	body := strings.Replace(string(readFixture(t, "company_screener_null_beta_synthetic.json")),
		`"companyName": "Synthetic Row 37"`, `"companyName": null`, 1)
	typed := decodeScreenerBody(t, []byte(body))

	if typed.Path != "/37/companyName" || typed.DecodeKind != DecodeKindNull {
		t.Fatalf("Path = %q, DecodeKind = %v, want /37/companyName null", typed.Path, typed.DecodeKind)
	}
	want := "successful response could not be decoded: null value at /37/companyName (endpoint: company-screener): "
	if !strings.HasPrefix(typed.Error(), want) {
		t.Fatalf("Error() = %.120q, want prefix %q", typed.Error(), want)
	}
}

func TestDecodeErrorNeverCarriesAStringMemberValue(t *testing.T) {
	t.Parallel()
	body := strings.Replace(string(readFixture(t, "company_screener_null_beta_synthetic.json")),
		"\"symbol\": \"R37\",\n    \"volume\": 29909012", "\"symbol\": \"R37\",\n    \"volume\": \""+decodePathSentinel+"\"", 1)
	typed := decodeScreenerBody(t, []byte(body))

	if typed.Path != "/37/volume" || typed.DecodeKind != DecodeKindWrongType {
		t.Fatalf("Path = %q, DecodeKind = %v, want /37/volume wrong_type", typed.Path, typed.DecodeKind)
	}
	if typed.Body == nil || !typed.Body.Truncated {
		t.Fatalf("Body = %+v, want a truncated excerpt that ends before row 37", typed.Body)
	}
	if strings.Contains(typed.Error(), decodePathSentinel) || strings.Contains(typed.Unwrap().Error(), decodePathSentinel) {
		t.Fatalf("decode error leaked the member value: %q / %q", typed.Error(), typed.Unwrap())
	}
}

func TestDecodeErrorLocatesMissingWrongAndMalformedMembers(t *testing.T) {
	t.Parallel()
	row := string(readFixture(t, "company_screener.json"))
	cases := []struct {
		name string
		body string
		path string
		kind DecodeKind
	}{
		{"missing", strings.Replace(row, `"beta": 1.097,`, "", 1), "/0/beta", DecodeKindMissingMember},
		{"wrong type", strings.Replace(row, `"AAPL"`, "5", 1), "/0/symbol", DecodeKindWrongType},
		{"root shape", "{}", "", DecodeKindWrongType},
		{"malformed", `[{"beta": nope}]`, "/0/beta", DecodeKindSyntax},
		{"not json", strings.Repeat("not-json ", 40), "", DecodeKindSyntax},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			typed := decodeScreenerBody(t, []byte(tc.body))
			if typed.Path != tc.path || typed.DecodeKind != tc.kind {
				t.Fatalf("Path = %q, DecodeKind = %v, want %q %v", typed.Path, typed.DecodeKind, tc.path, tc.kind)
			}
		})
	}
}

func TestDecodeKindStringsMatchTheRustBindingValues(t *testing.T) {
	t.Parallel()
	want := map[DecodeKind]string{
		DecodeKindNone:          "",
		DecodeKindSyntax:        "syntax",
		DecodeKindNull:          "null",
		DecodeKindMissingMember: "missing_member",
		DecodeKindWrongType:     "wrong_type",
		DecodeKindInvalidValue:  "invalid_value",
	}
	for kind, value := range want {
		if kind.String() != value {
			t.Fatalf("%d.String() = %q, want %q", int(kind), kind.String(), value)
		}
	}
}

func TestRequireObjectRowsLocatesTheNonObjectRow(t *testing.T) {
	t.Parallel()
	err := requireObjectRows("dynamic-id", []jsontext.Value{jsontext.Value(`{}`), jsontext.Value(`[1]`)})
	var typed *Error
	if !errors.As(err, &typed) || typed.Path != "/1" || typed.DecodeKind != DecodeKindWrongType {
		t.Fatalf("error = %+v, want Path /1 wrong_type", typed)
	}
}
