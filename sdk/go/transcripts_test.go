package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// transcriptsRoutes is the exact request-URI table of the three transcripts
// endpoints, one per query shape: no query at all (latest with both setters
// unset), both optional setters, required symbol plus year and quarter with
// the limit setter, and the required symbol alone. The wire text is copied
// from the URL assertions in crates/libfmp/tests/transcript_*_endpoints.rs.
// Keying on the full request URI lets one path carry its no-query and
// with-setter variants.
var transcriptsRoutes = map[string]string{
	"/router/stable/earning-call-transcript-latest":                                  "latest_earnings_transcripts.json",
	"/router/stable/earning-call-transcript-latest?limit=100&page=0":                 "latest_earnings_transcripts.json",
	"/router/stable/earning-call-transcript?symbol=AAPL&year=2020&quarter=3&limit=1": "earnings_transcript.json",
	"/router/stable/earning-call-transcript-dates?symbol=AAPL":                       "earnings_transcript_dates.json",
}

func transcriptsRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := transcriptsRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestTranscriptsMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, transcriptsRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	ns := client.Transcripts

	bare, err := ns.Latest(ctx, NewLatestEarningsTranscriptsQuery())
	if err != nil || len(bare) != 1 || bare[0].Symbol != "VLO" || bare[0].FiscalYear != 2026 {
		t.Fatalf("Latest (no query) = %+v, %v", bare, err)
	}
	paged, err := ns.Latest(ctx, NewLatestEarningsTranscriptsQuery().WithLimit(100).WithPage(0))
	if err != nil || len(paged) != 1 || paged[0].Period != "Q2" {
		t.Fatalf("Latest (limit and page) = %+v, %v", paged, err)
	}
	transcripts, err := ns.ByQuarter(ctx, NewEarningsTranscriptQuery("AAPL", 2020, QuarterQ3).WithLimit(1))
	if err != nil || len(transcripts) != 1 || transcripts[0].Year != 2020 ||
		!strings.HasSuffix(transcripts[0].Content, "Aft...") {
		t.Fatalf("ByQuarter = %+v, %v", transcripts, err)
	}
	dates, err := ns.Dates(ctx, NewEarningsTranscriptDatesQuery("AAPL"))
	if err != nil || len(dates) != 1 || dates[0].Quarter != 2 || dates[0].FiscalYear != 2026 {
		t.Fatalf("Dates = %+v, %v", dates, err)
	}

	requests := rec.all()
	if len(requests) != len(transcriptsRoutes) {
		t.Fatalf("requests = %d, want exactly one per call", len(requests))
	}
	for _, req := range requests {
		if req.Method != http.MethodGet {
			t.Fatalf("%s used %s, want GET", req.URL.Path, req.Method)
		}
		if strings.Contains(req.URL.RawQuery, "apikey") {
			t.Fatalf("%s carried the credential in the query: %s", req.URL.Path, req.URL.RawQuery)
		}
	}
}

func TestTranscriptsQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, transcriptsRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	ns := client.Transcripts

	for _, quarter := range []Quarter{"5", "Q3", "", "3 "} {
		_, err := ns.ByQuarter(ctx, NewEarningsTranscriptQuery("AAPL", 2020, quarter))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, ErrUnknownWireValue) ||
			typed.Message != "quarter: "+ErrUnknownWireValue.Error()+", expected one of 1, 2, 3, 4" {
			t.Fatalf("quarter %q: error = %v", quarter, err)
		}
	}
	cases := []struct {
		name   string
		symbol string
		reason error
	}{
		{"empty", "", ErrEmptyValue},
		{"whitespace", " \t", ErrEmptyValue},
		{"control", "AA\nPL", ErrControlCharacterValue},
		{"comma", "AAPL,MSFT", ErrCommaInTicker},
	}
	for _, tc := range cases {
		_, err := ns.Dates(ctx, NewEarningsTranscriptDatesQuery(tc.symbol))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != "symbol: "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want symbol: %v", tc.name, err, tc.reason)
		}
		_, err = ns.ByQuarter(ctx, NewEarningsTranscriptQuery(tc.symbol, 2020, QuarterQ3))
		if transcript := assertQuoteError(t, err, CategoryValidation, 0, ""); transcript.Message != typed.Message {
			t.Fatalf("%s: ByQuarter message %q differs from Dates message %q",
				tc.name, transcript.Message, typed.Message)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	q := NewEarningsTranscriptQuery(" AAPL ", 2020, QuarterQ3)
	if q.Symbol() != " AAPL " || q.Year() != 2020 || q.Quarter() != QuarterQ3 || q.Limit() != nil {
		t.Fatalf("getters = %q %d %q %v", q.Symbol(), q.Year(), q.Quarter(), q.Limit())
	}
	if limit := q.WithLimit(1).Limit(); limit == nil || *limit != 1 || q.Limit() != nil {
		t.Fatalf("WithLimit mutated the receiver or lost the value: %v %v", limit, q.Limit())
	}
	latest := NewLatestEarningsTranscriptsQuery()
	if latest.Limit() != nil || latest.Page() != nil {
		t.Fatalf("latest getters = %v %v, want both unset", latest.Limit(), latest.Page())
	}
	if page := latest.WithPage(0).Page(); page == nil || *page != 0 || latest.Page() != nil {
		t.Fatalf("WithPage mutated the receiver or lost page zero: %v %v", page, latest.Page())
	}
	if dates := NewEarningsTranscriptDatesQuery("AAPL"); dates.Symbol() != "AAPL" {
		t.Fatalf("dates Symbol() = %q", dates.Symbol())
	}
}

func TestTranscriptsMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"symbol":"AAPL","period":"Q3","year":2020,"date":"2020-07-30"}]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Transcripts.ByQuarter(context.Background(), NewEarningsTranscriptQuery("AAPL", 2020, QuarterQ3))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "earning-call-transcript")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"content"`) {
		t.Fatalf("cause = %v, want it to name the missing member content", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
