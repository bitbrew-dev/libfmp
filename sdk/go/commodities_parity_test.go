package fmp

import (
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

// The 9 shared fixtures the Rust tests decode into a model the commodities
// endpoints return (crates/libfmp/tests/asset_catalog_quote_responses.rs and
// asset_history_responses.rs). Only CommodityListing is defined in this
// domain; the quote and chart models are reused from quote_models.go and
// chart_models.go, so their fixtures are proven here through the same types.
func TestCommoditiesFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[CommodityListing](t, "commodities_list.json")
	assertFixtureParity[Quote](t, "commodities_quote.json")
	for _, name := range []string{"commodities_quote_short.json", "commodities_quotes.json"} {
		assertFixtureParity[QuoteShort](t, name)
	}
	assertFixtureParity[StockChartLightBar](t, "commodity_chart_light.json")
	assertFixtureParity[StockChartFullBar](t, "commodity_chart_full.json")
	for _, name := range []string{"commodity_chart_one_minute.json", "commodity_chart_five_minutes.json",
		"commodity_chart_one_hour.json"} {
		assertFixtureParity[StockChartIntradayBar](t, name)
	}
}

// Exact values copied from asset_catalog_quote_responses.rs
// (catalog_rows_preserve_exact_identifiers_dates_and_large_unsigned_supplies
// and crypto_quotes_preserve_large_volume_market_cap_and_nullable_market_cap)
// and asset_catalog_quote_endpoints.rs.
func TestDocumentedCommodityCatalogAndQuoteFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	listing := assertFixtureParity[CommodityListing](t, "commodities_list.json")
	if len(listing) != 1 || listing[0].Symbol != "ZMUSD" || listing[0].Name != "Soybean Meal Futures" ||
		listing[0].TradeMonth != "Dec" || listing[0].Currency != "USD" {
		t.Fatalf("commodities_list = %+v", listing)
	}
	if listing[0].Exchange != nil {
		t.Fatalf("exchange = %q, want nil for JSON null", *listing[0].Exchange)
	}

	quotes := assertFixtureParity[Quote](t, "commodities_quote.json")
	if len(quotes) != 1 || quotes[0].Symbol != "GCUSD" || quotes[0].Name != "Gold Futures" ||
		quotes[0].Exchange != "COMMODITY" || quotes[0].Price != 4168.3 || quotes[0].Change != 132 ||
		quotes[0].Volume != 125_925 || quotes[0].Timestamp != UnixSeconds(1_785_430_211) {
		t.Fatalf("commodities_quote = %+v", quotes)
	}
	if quotes[0].MarketCap != nil {
		t.Fatalf("marketCap = %v, want nil for JSON null", *quotes[0].MarketCap)
	}

	short := assertFixtureParity[QuoteShort](t, "commodities_quote_short.json")
	if want := (QuoteShort{Symbol: "GCUSD", Price: 4168.3, Change: 132, Volume: 125_925}); len(short) != 1 ||
		short[0] != want {
		t.Fatalf("commodities_quote_short = %+v, want %+v", short, want)
	}
	batch := assertFixtureParity[QuoteShort](t, "commodities_quotes.json")
	if want := (QuoteShort{Symbol: "DCUSD", Price: 16.91, Change: 0.02, Volume: 615}); len(batch) != 1 ||
		batch[0] != want {
		t.Fatalf("commodities_quotes = %+v, want %+v", batch, want)
	}
}

// Exact values copied from asset_history_responses.rs
// (asset_history_preserves_timezone_less_times_large_crypto_volume_and_zero_volume)
// and the fixture-backed client test in asset_history_endpoints.rs.
func TestDocumentedCommodityChartFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	date := mustParseDate(t, "2026-07-30")

	light := assertFixtureParity[StockChartLightBar](t, "commodity_chart_light.json")
	if want := (StockChartLightBar{Symbol: "GCUSD", Date: date, Price: 4170, Volume: 126_573}); len(light) != 1 ||
		light[0] != want {
		t.Fatalf("commodity_chart_light = %+v, want %+v", light, want)
	}
	full := assertFixtureParity[StockChartFullBar](t, "commodity_chart_full.json")
	if want := (StockChartFullBar{Symbol: "GCUSD", Date: date, Open: 4126.7, High: 4180.2, Low: 4085, Close: 4170,
		Volume: 126_573, Change: 43.3, ChangePercent: 1.04926, Vwap: 4145.07}); len(full) != 1 || full[0] != want {
		t.Fatalf("commodity_chart_full = %+v, want %+v", full, want)
	}

	intraday := []struct {
		fixture string
		date    string
		volume  float64
	}{
		{"commodity_chart_one_minute.json", "2026-07-30 13:06:00", 59},
		{"commodity_chart_five_minutes.json", "2026-07-30 13:05:00", 103},
		{"commodity_chart_one_hour.json", "2026-07-30 13:00:00", 690},
	}
	for _, tc := range intraday {
		rows := assertFixtureParity[StockChartIntradayBar](t, tc.fixture)
		if len(rows) != 1 || rows[0].Date.String() != tc.date || rows[0].Volume != tc.volume ||
			rows[0].Close != 4166.8 {
			t.Fatalf("%s = %+v, want date %s volume %.0f", tc.fixture, rows, tc.date, tc.volume)
		}
	}
}

// Mirrors commodity_exchange_is_required_nullable_and_future_non_null_values_remain_open
// and catalog_non_nullable_fields_are_required_and_unknown_fields_are_accepted
// in asset_catalog_quote_responses.rs: exchange must be present but may be
// null, every other member must be present and non-null, and an unknown
// member is ignored.
func TestCommoditiesRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	const row = `{"currency":"USD","exchange":null,"name":"Soybean Meal Futures","symbol":"ZMUSD","tradeMonth":"Dec"}`

	accepted := []struct {
		name     string
		wire     string
		null     bool
		exchange string
	}{
		{"null exchange", `[` + row + `]`, true, ""},
		{"non-null exchange and unknown member",
			`[{"currency":"USD","exchange":"FUTURES","name":"Soybean Meal Futures","symbol":"ZMUSD",` +
				`"tradeMonth":"Dec","futureField":{"nested":true}}]`, false, "FUTURES"},
	}
	for _, tc := range accepted {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []CommodityListing
			if err := json.Unmarshal([]byte(tc.wire), &rows); err != nil || len(rows) != 1 {
				t.Fatalf("decode = %+v, %v", rows, err)
			}
			got := rows[0].Exchange
			if tc.null && got != nil {
				t.Fatalf("exchange = %q, want nil", *got)
			}
			if !tc.null && (got == nil || *got != tc.exchange) {
				t.Fatalf("exchange = %v, want %q", got, tc.exchange)
			}
		})
	}

	rejected := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing exchange", `[{"currency":"USD","name":"Soybean Meal Futures","symbol":"ZMUSD","tradeMonth":"Dec"}]`,
			"exchange"},
		{"missing tradeMonth", `[{"currency":"USD","exchange":null,"name":"Soybean Meal Futures","symbol":"ZMUSD"}]`,
			"tradeMonth"},
		{"null currency", `[{"currency":null,"exchange":null,"name":"Soybean Meal Futures","symbol":"ZMUSD",` +
			`"tradeMonth":"Dec"}]`, "currency"},
		{"empty object", `[{}]`, "symbol"},
		{"null element", `[null]`, "symbol"},
	}
	for _, tc := range rejected {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []CommodityListing
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "CommodityListing") {
				t.Fatalf("message = %q, want it to name member %q of CommodityListing", typed.Message, tc.member)
			}
		})
	}
	if err := json.Unmarshal([]byte(`[{"currency":"USD","exchange":7,"name":"n","symbol":"ZMUSD","tradeMonth":"Dec"}]`),
		new([]CommodityListing)); err == nil {
		t.Fatal("a numeric exchange decoded into a string member")
	}
}
