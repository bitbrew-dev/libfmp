package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// technicalIndicatorsRoutes is the exact path and query table of the nine
// technical-indicator endpoints, keyed on path plus RawQuery so the
// independent omission of each optional date can share the table with the
// full query. Wire text copied from the TechnicalIndicatorQuery unit tests in
// crates/libfmp/src/endpoints/technical_indicators.rs.
var technicalIndicatorsRoutes = map[string]string{
	"/router/stable/technical-indicators/sma?symbol=BRK.B+%2F+Class+A&periodLength=4294967295&timeframe=1min&from=2026-06-01&to=2026-03-01": "technical_indicator_sma.json",
	"/router/stable/technical-indicators/sma?symbol=AAPL&periodLength=10&timeframe=1day":                                                    "technical_indicator_sma.json",
	"/router/stable/technical-indicators/ema?symbol=AAPL&periodLength=10&timeframe=5min&from=2026-06-01":                                    "technical_indicator_ema.json",
	"/router/stable/technical-indicators/wma?symbol=AAPL&periodLength=10&timeframe=15min&to=2026-03-01":                                     "technical_indicator_wma.json",
	"/router/stable/technical-indicators/dema?symbol=AAPL&periodLength=10&timeframe=30min":                                                  "technical_indicator_dema.json",
	"/router/stable/technical-indicators/tema?symbol=AAPL&periodLength=10&timeframe=1hour":                                                  "technical_indicator_tema.json",
	"/router/stable/technical-indicators/rsi?symbol=AAPL&periodLength=10&timeframe=4hour":                                                   "technical_indicator_rsi.json",
	"/router/stable/technical-indicators/standarddeviation?symbol=AAPL&periodLength=10&timeframe=1day":                                      "technical_indicator_standard_deviation.json",
	"/router/stable/technical-indicators/williams?symbol=AAPL&periodLength=10&timeframe=1day":                                               "technical_indicator_williams.json",
	"/router/stable/technical-indicators/adx?symbol=AAPL&periodLength=10&timeframe=1day":                                                    "technical_indicator_adx.json",
}

func technicalIndicatorsRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		key := r.URL.Path
		if r.URL.RawQuery != "" {
			key += "?" + r.URL.RawQuery
		}
		fixture, ok := technicalIndicatorsRoutes[key]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", key)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestTechnicalIndicatorsMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, technicalIndicatorsRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	from, to := mustParseDate(t, "2026-06-01"), mustParseDate(t, "2026-03-01")
	daily := NewTechnicalIndicatorQuery("AAPL", 10, ChartTimeframeOneDay)

	sma, err := client.TechnicalIndicators.SimpleMovingAverage(ctx,
		NewTechnicalIndicatorQuery("BRK.B / Class A", 4_294_967_295, ChartTimeframeOneMinute).WithFrom(from).WithTo(to))
	if err != nil || len(sma) != 1 || sma[0].Sma != 331.621 {
		t.Fatalf("SimpleMovingAverage = %+v, %v", sma, err)
	}
	if sma, err = client.TechnicalIndicators.SimpleMovingAverage(ctx, daily); err != nil || len(sma) != 1 {
		t.Fatalf("SimpleMovingAverage without dates = %+v, %v", sma, err)
	}
	ema, err := client.TechnicalIndicators.ExponentialMovingAverage(ctx,
		NewTechnicalIndicatorQuery("AAPL", 10, ChartTimeframeFiveMinutes).WithFrom(from))
	if err != nil || len(ema) != 1 || ema[0].Ema != 331.1209325826155 {
		t.Fatalf("ExponentialMovingAverage with only from = %+v, %v", ema, err)
	}
	wma, err := client.TechnicalIndicators.WeightedMovingAverage(ctx,
		NewTechnicalIndicatorQuery("AAPL", 10, ChartTimeframeFifteenMinutes).WithTo(to))
	if err != nil || len(wma) != 1 || wma[0].Wma != 333.21345454545457 {
		t.Fatalf("WeightedMovingAverage with only to = %+v, %v", wma, err)
	}
	dema, err := client.TechnicalIndicators.DoubleExponentialMovingAverage(ctx,
		NewTechnicalIndicatorQuery("AAPL", 10, ChartTimeframeThirtyMinutes))
	if err != nil || len(dema) != 1 || dema[0].Dema != 337.7659642917977 {
		t.Fatalf("DoubleExponentialMovingAverage = %+v, %v", dema, err)
	}
	tema, err := client.TechnicalIndicators.TripleExponentialMovingAverage(ctx,
		NewTechnicalIndicatorQuery("AAPL", 10, ChartTimeframeOneHour))
	if err != nil || len(tema) != 1 || tema[0].Tema != 337.09671042323214 {
		t.Fatalf("TripleExponentialMovingAverage = %+v, %v", tema, err)
	}
	rsi, err := client.TechnicalIndicators.RelativeStrengthIndex(ctx,
		NewTechnicalIndicatorQuery("AAPL", 10, ChartTimeframeFourHours))
	if err != nil || len(rsi) != 1 || rsi[0].Rsi != 59.55175118203601 {
		t.Fatalf("RelativeStrengthIndex = %+v, %v", rsi, err)
	}
	deviation, err := client.TechnicalIndicators.StandardDeviation(ctx, daily)
	if err != nil || len(deviation) != 1 || deviation[0].StandardDeviation != 5.675893674127448 {
		t.Fatalf("StandardDeviation = %+v, %v", deviation, err)
	}
	williams, err := client.TechnicalIndicators.Williams(ctx, daily)
	if err != nil || len(williams) != 1 || williams[0].Williams != -48.29500396510714 {
		t.Fatalf("Williams = %+v, %v", williams, err)
	}
	adx, err := client.TechnicalIndicators.AverageDirectionalIndex(ctx, daily)
	if err != nil || len(adx) != 1 || adx[0].Adx != 34.69458756515438 {
		t.Fatalf("AverageDirectionalIndex = %+v, %v", adx, err)
	}

	requests := rec.all()
	if len(requests) != len(technicalIndicatorsRoutes) {
		t.Fatalf("requests = %d, want exactly one per route", len(requests))
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

func TestTechnicalIndicatorsQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, technicalIndicatorsRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()

	_, err := client.TechnicalIndicators.SimpleMovingAverage(ctx, NewTechnicalIndicatorQuery("AAPL,MSFT", 10, ChartTimeframeOneDay))
	typed := assertQuoteError(t, err, CategoryValidation, 0, "")
	if !errors.Is(err, ErrCommaInTicker) || typed.Message != "symbol: "+ErrCommaInTicker.Error() {
		t.Fatalf("comma in symbol: error = %v", err)
	}
	_, err = client.TechnicalIndicators.RelativeStrengthIndex(ctx, NewTechnicalIndicatorQuery("AAPL", 0, ChartTimeframeOneDay))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroPeriodLength) ||
		typed.Message != "periodLength: "+ErrZeroPeriodLength.Error() {
		t.Fatalf("zero period length: error = %v", err)
	}
	_, err = client.TechnicalIndicators.Williams(ctx, NewTechnicalIndicatorQuery("AAPL", 10, "daily"))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrUnknownWireValue) ||
		!strings.HasPrefix(typed.Message, "timeframe: "+ErrUnknownWireValue.Error()) {
		t.Fatalf("undocumented timeframe: error = %v", err)
	}
	_, err = client.TechnicalIndicators.AverageDirectionalIndex(ctx,
		NewTechnicalIndicatorQuery("AAPL", 10, ChartTimeframeOneDay).WithFrom(Date{}))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "from: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero from date: error = %v", err)
	}
	_, err = client.TechnicalIndicators.StandardDeviation(ctx,
		NewTechnicalIndicatorQuery("AAPL", 10, ChartTimeframeOneDay).WithTo(Date{}))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "to: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero to date: error = %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests, want none", rec.count())
	}

	q := NewTechnicalIndicatorQuery(" AAPL ", 3, ChartTimeframeFourHours)
	if q.Symbol() != " AAPL " || q.PeriodLength() != 3 || q.Timeframe() != ChartTimeframeFourHours || q.From() != nil || q.To() != nil {
		t.Fatalf("accessors changed the arguments: %+v", q)
	}
	from := mustParseDate(t, "2026-06-01")
	if withFrom := q.WithFrom(from); withFrom.From() == nil || *withFrom.From() != from || q.From() != nil {
		t.Fatalf("WithFrom did not copy the query: %+v, %+v", withFrom, q)
	}
}
