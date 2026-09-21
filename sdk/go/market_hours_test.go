package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// marketHoursRoutes is the exact request-URI table of the three market_hours
// endpoints, copied from the URL assertions in
// crates/libfmp/tests/market_hours_endpoints.rs: the required exchange code
// with and without the opaque timestamp, every holiday date-filter
// combination, and the timestamp-only or empty query of the all-exchanges feed.
var marketHoursRoutes = map[string]string{
	"/router/stable/exchange-market-hours?exchange=NASDAQ+%2F+Global&timestamp=001769527402":       "exchange_market_hours.json",
	"/router/stable/exchange-market-hours?exchange=NASDAQ":                                         "exchange_market_hours.json",
	"/router/stable/holidays-by-exchange?exchange=NASDAQ+%2F+Global&from=2025-04-27&to=2026-04-27": "holidays_by_exchange.json",
	"/router/stable/holidays-by-exchange?exchange=NASDAQ&from=2025-04-27":                          "holidays_by_exchange.json",
	"/router/stable/holidays-by-exchange?exchange=NASDAQ&to=2026-04-27":                            "holidays_by_exchange.json",
	"/router/stable/holidays-by-exchange?exchange=NASDAQ":                                          "holidays_by_exchange.json",
	"/router/stable/all-exchange-market-hours?timestamp=001769527402":                              "all_exchange_market_hours.json",
	"/router/stable/all-exchange-market-hours":                                                     "all_exchange_market_hours.json",
}

func marketHoursRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := marketHoursRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestMarketHoursMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, marketHoursRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	hours := client.MarketHours
	from := mustParseDate(t, "2025-04-27")
	to := mustParseDate(t, "2026-04-27")

	exchange, err := hours.ExchangeMarketHours(ctx, NewExchangeMarketHoursQuery("NASDAQ / Global").WithTimestamp("001769527402"))
	if err != nil || len(exchange) != 1 || exchange[0].OpeningHour != "09:30 AM -04:00" {
		t.Fatalf("ExchangeMarketHours = %+v, %v", exchange, err)
	}
	if _, err := hours.ExchangeMarketHours(ctx, NewExchangeMarketHoursQuery("NASDAQ")); err != nil {
		t.Fatalf("ExchangeMarketHours without timestamp: %v", err)
	}
	holidays, err := hours.HolidaysByExchange(ctx, NewHolidaysByExchangeQuery("NASDAQ / Global").WithFrom(from).WithTo(to))
	if err != nil || len(holidays) != 1 || holidays[0].Name != "Independence Day" || holidays[0].AdjOpenTime != nil {
		t.Fatalf("HolidaysByExchange = %+v, %v", holidays, err)
	}
	if _, err := hours.HolidaysByExchange(ctx, NewHolidaysByExchangeQuery("NASDAQ").WithFrom(from)); err != nil {
		t.Fatalf("HolidaysByExchange from only: %v", err)
	}
	if _, err := hours.HolidaysByExchange(ctx, NewHolidaysByExchangeQuery("NASDAQ").WithTo(to)); err != nil {
		t.Fatalf("HolidaysByExchange to only: %v", err)
	}
	if _, err := hours.HolidaysByExchange(ctx, NewHolidaysByExchangeQuery("NASDAQ")); err != nil {
		t.Fatalf("HolidaysByExchange exchange only: %v", err)
	}
	all, err := hours.AllExchangeMarketHours(ctx, NewAllExchangeMarketHoursQuery().WithTimestamp("001769527402"))
	if err != nil || len(all) != 1 || all[0].Exchange != "ASX" {
		t.Fatalf("AllExchangeMarketHours = %+v, %v", all, err)
	}
	if _, err := hours.AllExchangeMarketHours(ctx, NewAllExchangeMarketHoursQuery()); err != nil {
		t.Fatalf("AllExchangeMarketHours without timestamp: %v", err)
	}

	want := []string{
		"/router/stable/exchange-market-hours?exchange=NASDAQ+%2F+Global&timestamp=001769527402",
		"/router/stable/exchange-market-hours?exchange=NASDAQ",
		"/router/stable/holidays-by-exchange?exchange=NASDAQ+%2F+Global&from=2025-04-27&to=2026-04-27",
		"/router/stable/holidays-by-exchange?exchange=NASDAQ&from=2025-04-27",
		"/router/stable/holidays-by-exchange?exchange=NASDAQ&to=2026-04-27",
		"/router/stable/holidays-by-exchange?exchange=NASDAQ",
		"/router/stable/all-exchange-market-hours?timestamp=001769527402",
		"/router/stable/all-exchange-market-hours",
	}
	requests := rec.all()
	if len(requests) != len(want) {
		t.Fatalf("requests = %d, want exactly one per call", len(requests))
	}
	for index, req := range requests {
		if got := req.URL.RequestURI(); got != want[index] {
			t.Fatalf("request %d = %s, want %s", index, got, want[index])
		}
		if req.Method != http.MethodGet {
			t.Fatalf("%s used %s, want GET", req.URL.Path, req.Method)
		}
		if strings.Contains(req.URL.RawQuery, "apikey") {
			t.Fatalf("%s carried the credential in the query: %s", req.URL.Path, req.URL.RawQuery)
		}
	}
}

// Exchange codes and the opaque timestamp are validated as the Rust
// ExchangeCode and MarketHoursTimestamp newtypes validate them: non-empty
// and free of control characters, with commas allowed.
func TestMarketHoursQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, marketHoursRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	hours := client.MarketHours
	cases := []struct {
		name   string
		call   func() error
		member string
		reason error
	}{
		{"empty exchange", func() error {
			_, err := hours.ExchangeMarketHours(ctx, NewExchangeMarketHoursQuery(""))
			return err
		}, "exchange", ErrEmptyValue},
		{"whitespace exchange", func() error {
			_, err := hours.HolidaysByExchange(ctx, NewHolidaysByExchangeQuery(" \t "))
			return err
		}, "exchange", ErrEmptyValue},
		{"control character in exchange", func() error {
			_, err := hours.HolidaysByExchange(ctx, NewHolidaysByExchangeQuery("NAS\nDAQ"))
			return err
		}, "exchange", ErrControlCharacterValue},
		{"blank optional timestamp", func() error {
			_, err := hours.ExchangeMarketHours(ctx, NewExchangeMarketHoursQuery("NASDAQ").WithTimestamp(" "))
			return err
		}, "timestamp", ErrEmptyValue},
		{"tab in all-exchanges timestamp", func() error {
			_, err := hours.AllExchangeMarketHours(ctx, NewAllExchangeMarketHoursQuery().WithTimestamp("0017\t69527402"))
			return err
		}, "timestamp", ErrControlCharacterValue},
		{"zero from date", func() error {
			_, err := hours.HolidaysByExchange(ctx, NewHolidaysByExchangeQuery("NASDAQ").WithFrom(Date{}))
			return err
		}, "from", ErrZeroTemporalValue},
		{"zero to date", func() error {
			_, err := hours.HolidaysByExchange(ctx, NewHolidaysByExchangeQuery("NASDAQ").WithTo(Date{}))
			return err
		}, "to", ErrZeroTemporalValue},
	}
	for _, tc := range cases {
		err := tc.call()
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != tc.member+": "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want %s: %v", tc.name, err, tc.member, tc.reason)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
	if _, err := NewExchangeMarketHoursQuery("NYSE,NASDAQ").params(); err != nil {
		t.Fatalf("a comma in an exchange code was rejected: %v", err)
	}
	empty := NewAllExchangeMarketHoursQuery()
	if empty.Timestamp() != nil {
		t.Fatalf("NewAllExchangeMarketHoursQuery() set a timestamp: %+v", empty)
	}
	holidays := NewHolidaysByExchangeQuery("NASDAQ / Global")
	if holidays.Exchange() != "NASDAQ / Global" || holidays.From() != nil || holidays.To() != nil {
		t.Fatalf("NewHolidaysByExchangeQuery normalized the exchange or set a date: %+v", holidays)
	}
	if q := NewExchangeMarketHoursQuery("NASDAQ").WithTimestamp("001769527402"); q.Timestamp() == nil || *q.Timestamp() != "001769527402" {
		t.Fatalf("Timestamp() dropped the leading zeros: %v", q.Timestamp())
	}
}

// Mirrors malformed_non_array_responses_keep_all_three_endpoint_identities in
// crates/libfmp/tests/market_hours_endpoints.rs.
func TestMarketHoursMethodsKeepEndpointIdentityOnNonArrayBodies(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(`{}`))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	hours := client.MarketHours

	_, err := hours.ExchangeMarketHours(ctx, NewExchangeMarketHoursQuery("NASDAQ"))
	_ = assertQuoteError(t, err, CategoryDecode, http.StatusOK, "exchange-market-hours")
	_, err = hours.HolidaysByExchange(ctx, NewHolidaysByExchangeQuery("NASDAQ"))
	_ = assertQuoteError(t, err, CategoryDecode, http.StatusOK, "holidays-by-exchange")
	_, err = hours.AllExchangeMarketHours(ctx, NewAllExchangeMarketHoursQuery())
	_ = assertQuoteError(t, err, CategoryDecode, http.StatusOK, "all-exchange-market-hours")
}
