package fmp

import (
	"bytes"
	"context"
	"encoding/csv"
	"encoding/json/v2"
	"errors"
	"net/http"
	"slices"
	"strconv"
	"testing"
)

const csvEndpoint = "csv-contract-test"

type csvProbeRow struct {
	Symbol     string   `json:"symbol"`
	Date       Date     `json:"date"`
	Price      float64  `json:"price"`
	Listed     bool     `json:"listed"`
	Volume     *string  `json:"volume"`
	StockPrice *float64 `json:"Stock Price"`
	Peers      string   `json:"peers"`
}

const csvProbeHeader = `"symbol","date","price","listed","volume","Stock Price","peers"` + "\n"

func csvServer(t *testing.T, contentType, body string) *Client {
	t.Helper()
	server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", contentType)
		_, _ = w.Write([]byte(body))
	})
	return newClient(t, server)
}

func getCSVProbe[T any](t *testing.T, body string) ([]T, error) {
	t.Helper()
	return getCSV[T](context.Background(), csvServer(t, "text/csv", body), csvEndpoint, "bulk-probe", nil)
}

func assertCSVDecodeError(t *testing.T, err error, path string, kind DecodeKind) {
	t.Helper()
	var typed *Error
	if !errors.As(err, &typed) {
		t.Fatalf("error %v (%T) is not *Error", err, err)
	}
	if typed.Category != CategoryDecode || typed.Endpoint != csvEndpoint || typed.Path != path || typed.DecodeKind != kind {
		t.Fatalf("error = %+v, want decode at %q kind %v", typed, path, kind)
	}
}

// TestGetCSVDecodesEmptyAndHeaderOnlyBodiesToNoRows mirrors
// empty_and_header_only_bodies_decode_to_no_rows.
func TestGetCSVDecodesEmptyAndHeaderOnlyBodiesToNoRows(t *testing.T) {
	t.Parallel()
	for _, body := range []string{"", csvProbeHeader, "symbol,price\n", "Invalid name\n"} {
		rows, err := getCSVProbe[csvProbeRow](t, body)
		if err != nil || rows == nil || len(rows) != 0 {
			t.Fatalf("body %q: rows = %v, err = %v", body, rows, err)
		}
	}
}

// TestGetCSVDecodesCellsByHeaderName mirrors
// cells_decode_by_header_name_with_quoted_commas_and_verbatim_numbers.
func TestGetCSVDecodesCellsByHeaderName(t *testing.T) {
	t.Parallel()
	rows, err := getCSVProbe[csvProbeRow](t, csvProbeHeader+
		`"AAPL","2025-06-02",201.7,true,6.9148336e-9,12.5,"MSFT,GOOG"`+"\n"+
		`"","2025-06-02",-0.5,false,,,"say ""hi"""`+"\n")
	if err != nil {
		t.Fatal(err)
	}
	first, second := rows[0], rows[1]
	if first.Symbol != "AAPL" || first.Date != mustParseDate(t, "2025-06-02") || first.Price != 201.7 || !first.Listed ||
		first.Volume == nil || *first.Volume != "6.9148336e-9" || first.StockPrice == nil || *first.StockPrice != 12.5 ||
		first.Peers != "MSFT,GOOG" {
		t.Fatalf("first row = %+v", first)
	}
	if second.Symbol != "" || second.Volume != nil || second.StockPrice != nil || second.Peers != `say "hi"` {
		t.Fatalf("second row = %+v", second)
	}
}

// TestGetCSVSkipsALeadingByteOrderMark mirrors
// a_leading_byte_order_mark_is_not_part_of_the_first_header.
func TestGetCSVSkipsALeadingByteOrderMark(t *testing.T) {
	t.Parallel()
	rows, err := getCSVProbe[csvProbeRow](t, "\ufeff"+csvProbeHeader+"A,2025-06-02,1,true,1,1,x\r\n")
	if err != nil || len(rows) != 1 || rows[0].Symbol != "A" || rows[0].Peers != "x" {
		t.Fatalf("rows = %+v, err = %v", rows, err)
	}
}

// TestGetCSVReportsRowAndMemberOfAFailure mirrors the Rust decoder's
// null, missing-member, and syntax tests on a generated model.
func TestGetCSVReportsRowAndMemberOfAFailure(t *testing.T) {
	t.Parallel()
	const header = `"symbol","date","dcf","Stock Price"` + "\n"
	cases := []struct {
		body string
		path string
		kind DecodeKind
	}{
		{header + "A,2025-06-02,1,2\nB,,1,2\n", "/1/date", DecodeKindNull},
		{"symbol,date,Stock Price\nA,2025-06-02,2\n", "/0/dcf", DecodeKindMissingMember},
		{header + "A,2025-06-02,1,2\nB,2025-06-02\n", "/1", DecodeKindSyntax},
	}
	for _, tc := range cases {
		_, err := getCSVProbe[BulkDCFValuation](t, tc.body)
		assertCSVDecodeError(t, err, tc.path, tc.kind)
	}
	_, err := getCSVProbe[csvProbeRow](t, `"symbol","date","price","listed","Stock Price","peers"`+"\nA,2025-06-02,1,true,1,x\n")
	assertCSVDecodeError(t, err, "/0/volume", DecodeKindMissingMember)
	rows, err := getCSVProbe[csvProbeRow](t, "symbol,date\n")
	if err != nil || len(rows) != 0 {
		t.Fatalf("header-only body with missing columns: rows = %v, err = %v", rows, err)
	}
}

func TestGetCSVAcceptsOnlyTextCSV(t *testing.T) {
	t.Parallel()
	rows, err := getCSV[csvProbeRow](context.Background(), csvServer(t, "Text/CSV; charset=utf-8", csvProbeHeader),
		csvEndpoint, "bulk-probe", nil)
	if err != nil || len(rows) != 0 {
		t.Fatalf("rows = %v, err = %v", rows, err)
	}
	_, err = getCSV[csvProbeRow](context.Background(), csvServer(t, "application/json", `{"Error Message":"x"}`),
		csvEndpoint, "bulk-probe", nil)
	var typed *Error
	if !errors.As(err, &typed) || typed.Category != CategoryDecode ||
		typed.Message != "successful response used an unexpected content type" {
		t.Fatalf("err = %v", err)
	}
}

// TestBulkBodyLimitDefaultsUntilTheClientSetsOne mirrors
// csv_bulk_endpoints_default_to_the_bulk_body_limit_until_the_client_sets_one.
func TestBulkBodyLimitDefaultsUntilTheClientSetsOne(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "text/csv")
		_, _ = w.Write([]byte(csvProbeHeader))
	})
	unset := newClient(t, server)
	if unset.bulkMaxResponseBodyBytes != DefaultBulkMaxResponseBodyBytes ||
		unset.maxResponseBodyBytes != DefaultMaxResponseBodyBytes {
		t.Fatalf("limits = %d, %d", unset.bulkMaxResponseBodyBytes, unset.maxResponseBodyBytes)
	}
	configured := newClient(t, server, WithMaxResponseBodyBytes(8))
	if configured.bulkMaxResponseBodyBytes != 8 {
		t.Fatalf("bulk limit = %d, want the configured 8", configured.bulkMaxResponseBodyBytes)
	}
	_, err := getCSV[csvProbeRow](context.Background(), configured, csvEndpoint, "bulk-probe", nil)
	var typed *Error
	if !errors.As(err, &typed) || typed.Category != CategoryTransport ||
		typed.Message != "response body exceeded configured limit" {
		t.Fatalf("err = %v", err)
	}
}

// assertCSVFixtureParity mirrors assert_round_trip in
// crates/libfmp/tests/support/bulk_csv.rs: every row of the shared CSV
// fixture decodes, and the first row re-encodes fields members, each a header
// column equal to its cell (verbatim text, the parsed number or boolean, and
// null or "" for an empty cell).
func assertCSVFixtureParity[T any](t *testing.T, name string, fields int) []T {
	t.Helper()
	raw := readFixture(t, name)
	rows, path, kind, err := decodeCSVRows[T](raw)
	if kind != DecodeKindNone {
		t.Fatalf("%s: decode at %q (%v): %v", name, path, kind, err)
	}
	records, err := csv.NewReader(bytes.NewReader(raw)).ReadAll()
	if err != nil || len(records) != len(rows)+1 {
		t.Fatalf("%s: %d records for %d rows: %v", name, len(records), len(rows), err)
	}
	members := memberValues(t, rows[0])
	if len(members) != fields {
		t.Fatalf("%s: %d members re-encoded, want %d", name, len(members), fields)
	}
	for member, value := range members {
		column := slices.Index(records[0], member)
		if column < 0 {
			t.Fatalf("%s: member %q is not a header column", name, member)
		}
		cell := records[1][column]
		var text string
		switch value.Kind() {
		case 'n':
			text = ""
		case '"':
			if err := json.Unmarshal(value, &text); err != nil {
				t.Fatal(err)
			}
		case '0':
			want, _ := strconv.ParseFloat(cell, 64)
			got, _ := strconv.ParseFloat(string(value), 64)
			if got == want {
				continue
			}
			text = string(value)
		default:
			text = string(value)
		}
		if text != cell {
			t.Fatalf("%s: member %q re-encoded %s for cell %q", name, member, value, cell)
		}
	}
	return rows
}

// cellText returns the text of an optional numeric-string member, or "" when
// its CSV cell was empty.
func cellText(value *string) string {
	if value == nil {
		return ""
	}
	return *value
}
