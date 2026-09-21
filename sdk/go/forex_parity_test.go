package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"maps"
	"strings"
	"testing"
)

// The nine shared fixtures the Rust tests decode into a forex model
// (crates/libfmp/tests/asset_catalog_quote_responses.rs and
// asset_history_responses.rs). ForexPair is the only model the domain owns;
// the quote and chart rows are the shared models its methods return, and
// forex_quotes.json is the batch row the Quote.Forex method decodes.
func TestForexFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[ForexPair](t, "forex_list.json")
	assertFixtureParity[Quote](t, "forex_quote.json")
	for _, name := range []string{"forex_quote_short.json", "forex_quotes.json"} {
		assertFixtureParity[QuoteShort](t, name)
	}
	assertFixtureParity[StockChartLightBar](t, "forex_chart_light.json")
	assertFixtureParity[StockChartFullBar](t, "forex_chart_full.json")
	for _, name := range []string{"forex_chart_one_minute.json", "forex_chart_five_minutes.json",
		"forex_chart_one_hour.json"} {
		assertFixtureParity[StockChartIntradayBar](t, name)
	}
}

// Exact values copied from crates/libfmp/tests/asset_catalog_quote_responses.rs
// and asset_catalog_quote_endpoints.rs.
func TestDocumentedForexCatalogAndQuoteFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	pairs := assertFixtureParity[ForexPair](t, "forex_list.json")
	wantPair := ForexPair{Symbol: "ARSMXN", FromCurrency: "ARS", ToCurrency: "MXN", FromName: "Argentine Peso",
		ToName: "Mexican Peso"}
	if len(pairs) != 1 || pairs[0] != wantPair {
		t.Fatalf("forex_list = %+v, want %+v", pairs, wantPair)
	}

	quotes := assertFixtureParity[Quote](t, "forex_quote.json")
	if len(quotes) != 1 {
		t.Fatalf("forex_quote rows = %d, want 1", len(quotes))
	}
	quote := quotes[0]
	if quote.MarketCap != nil {
		t.Fatalf("marketCap = %v, want nil for JSON null", *quote.MarketCap)
	}
	wantQuote := Quote{
		Symbol: "EURUSD", Name: "EUR/USD", Price: 1.15284, ChangePercentage: 0.55071, Change: 0.006314,
		Volume: 146_872, DayLow: 1.1439, DayHigh: 1.1538, YearHigh: 1.20236, YearLow: 1.13254,
		PriceAvg50: 1.14891, PriceAvg200: 1.163, Exchange: "FOREX", Open: 1.14666, PreviousClose: 1.14653,
		Timestamp: UnixSeconds(1_785_430_798),
	}
	if quote != wantQuote {
		t.Fatalf("forex_quote = %+v, want %+v", quote, wantQuote)
	}

	short := assertFixtureParity[QuoteShort](t, "forex_quote_short.json")
	if want := (QuoteShort{Symbol: "EURUSD", Price: 1.15284, Change: 0.006314, Volume: 146_872}); len(short) != 1 ||
		short[0] != want {
		t.Fatalf("forex_quote_short = %+v, want %+v", short, want)
	}
	batch := assertFixtureParity[QuoteShort](t, "forex_quotes.json")
	if want := (QuoteShort{Symbol: "AEDAUD", Price: 0.38716, Change: -0.00513532, Volume: 0}); len(batch) != 1 ||
		batch[0] != want {
		t.Fatalf("forex_quotes = %+v, want %+v", batch, want)
	}
}

// Exact values copied from crates/libfmp/tests/asset_history_endpoints.rs and
// asset_history_responses.rs for the forex chart fixtures.
func TestDocumentedForexChartFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	date := mustParseDate(t, "2026-07-30")

	light := assertFixtureParity[StockChartLightBar](t, "forex_chart_light.json")
	if want := (StockChartLightBar{Symbol: "EURUSD", Date: date, Price: 1.15258, Volume: 147_799}); len(light) != 1 ||
		light[0] != want {
		t.Fatalf("forex_chart_light = %+v, want %+v", light, want)
	}

	full := assertFixtureParity[StockChartFullBar](t, "forex_chart_full.json")
	if want := (StockChartFullBar{Symbol: "EURUSD", Date: date, Open: 1.14666, High: 1.1538, Low: 1.1439,
		Close: 1.15258, Volume: 147_799, Change: 0.00592, ChangePercent: 0.51628207, Vwap: 1.15}); len(full) != 1 ||
		full[0] != want {
		t.Fatalf("forex_chart_full = %+v, want %+v", full, want)
	}

	cases := []struct {
		fixture string
		want    StockChartIntradayBar
	}{
		{"forex_chart_one_minute.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:17:00"),
			Open: 1.15203, Low: 1.15203, High: 1.15205, Close: 1.15204, Volume: 76}},
		{"forex_chart_five_minutes.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:15:00"),
			Open: 1.1521, Low: 1.15194, High: 1.1521, Close: 1.15194, Volume: 91}},
		{"forex_chart_one_hour.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:00:00"),
			Open: 1.1529, Low: 1.15194, High: 1.15327, Close: 1.15194, Volume: 1_420}},
	}
	for _, tc := range cases {
		rows := assertFixtureParity[StockChartIntradayBar](t, tc.fixture)
		if len(rows) != 1 || rows[0] != tc.want {
			t.Fatalf("%s = %+v, want %+v", tc.fixture, rows, tc.want)
		}
	}
}

// Mirrors assert_required_non_null::<ForexPair> and the bare-array root test
// in crates/libfmp/tests/asset_catalog_quote_responses.rs: every member of
// the documented row is required and non-null, and the decode error names
// the member exactly as the Rust decoder does.
func TestForexPairRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	var wire []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, "forex_list.json"), &wire); err != nil || len(wire) != 1 {
		t.Fatalf("forex_list.json is not a one-row bare array: %v", err)
	}
	for _, member := range []string{"symbol", "fromCurrency", "toCurrency", "fromName", "toName"} {
		missing := maps.Clone(wire[0])
		delete(missing, member)
		null := maps.Clone(wire[0])
		null[member] = jsontext.Value("null")
		for variant, row := range map[string]map[string]jsontext.Value{"missing": missing, "null": null} {
			encoded, err := json.Marshal(row)
			if err != nil {
				t.Fatalf("%s %s: re-encode: %v", variant, member, err)
			}
			var pair ForexPair
			err = json.Unmarshal(encoded, &pair)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("%s %s: error = %v (%T), want a CategoryDecode *Error", variant, member, err, err)
			}
			if !strings.Contains(typed.Message, `"`+member+`"`) || !strings.Contains(typed.Message, "ForexPair") {
				t.Fatalf("%s %s: message = %q, want it to name the member of ForexPair", variant, member, typed.Message)
			}
		}
	}

	var pairs []ForexPair
	if err := json.Unmarshal([]byte(`[]`), &pairs); err != nil || pairs == nil || len(pairs) != 0 {
		t.Fatalf("empty array: rows = %#v, err = %v, want a non-nil empty slice", pairs, err)
	}
	if err := json.Unmarshal([]byte(`{}`), &pairs); err == nil {
		t.Fatal("an object root decoded into the bare-array forex catalog contract")
	}
}
