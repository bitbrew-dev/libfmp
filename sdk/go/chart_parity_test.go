package fmp

import (
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

// The 32 shared fixtures the Rust tests decode into a chart model
// (crates/libfmp/tests/chart_*.rs, asset_history_responses.rs and
// indexes_history_responses.rs). The commodity, crypto, forex and index
// domains reuse these models, so their chart fixtures are proven here.
var (
	chartLightFixtures = []string{"stock_chart_light.json", "commodity_chart_light.json", "crypto_chart_light.json",
		"forex_chart_light.json", "indexes_chart_light.json"}
	chartFullFixtures = []string{"stock_chart_full.json", "commodity_chart_full.json", "crypto_chart_full.json",
		"forex_chart_full.json", "indexes_chart_full.json"}
	chartAdjustedFixtures = []string{"stock_chart_adjusted.json", "stock_chart_non_split_adjusted.json",
		"stock_chart_dividend_adjusted.json"}
	chartIntradayFixtures = []string{"stock_chart_intraday.json", "stock_chart_one_minute.json",
		"stock_chart_five_minutes.json", "stock_chart_fifteen_minutes.json", "stock_chart_thirty_minutes.json",
		"stock_chart_one_hour.json", "stock_chart_four_hours.json",
		"commodity_chart_one_minute.json", "commodity_chart_five_minutes.json", "commodity_chart_one_hour.json",
		"crypto_chart_one_minute.json", "crypto_chart_five_minutes.json", "crypto_chart_one_hour.json",
		"forex_chart_one_minute.json", "forex_chart_five_minutes.json", "forex_chart_one_hour.json",
		"indexes_chart_one_minute.json", "indexes_chart_five_minutes.json", "indexes_chart_one_hour.json"}
)

func TestChartFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	for _, name := range chartLightFixtures {
		assertFixtureParity[StockChartLightBar](t, name)
	}
	for _, name := range chartFullFixtures {
		assertFixtureParity[StockChartFullBar](t, name)
	}
	for _, name := range chartAdjustedFixtures {
		assertFixtureParity[StockChartAdjustedBar](t, name)
	}
	for _, name := range chartIntradayFixtures {
		assertFixtureParity[StockChartIntradayBar](t, name)
	}
}

// Exact values copied from crates/libfmp/tests/chart_responses.rs.
func TestDocumentedStockChartFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	date := mustParseDate(t, "2026-07-30")

	light := assertFixtureParity[StockChartLightBar](t, "stock_chart_light.json")
	if want := (StockChartLightBar{Symbol: "AAPL", Date: date, Price: 332.39, Volume: 29_207_295}); len(light) != 1 ||
		light[0] != want {
		t.Fatalf("stock_chart_light = %+v, want %+v", light, want)
	}

	full := assertFixtureParity[StockChartFullBar](t, "stock_chart_full.json")
	if want := (StockChartFullBar{Symbol: "AAPL", Date: date, Open: 333.13, High: 334.48, Low: 329.59, Close: 332.39,
		Volume: 29_207_295, Change: -0.74, ChangePercent: -0.2221355, Vwap: 332.15}); len(full) != 1 || full[0] != want {
		t.Fatalf("stock_chart_full = %+v, want %+v", full, want)
	}

	wantAdjusted := StockChartAdjustedBar{Symbol: "AAPL", Date: date, AdjOpen: 333.13, AdjHigh: 334.48, AdjLow: 329.59,
		AdjClose: 332.39, Volume: 29_207_295}
	for _, name := range chartAdjustedFixtures {
		adjusted := assertFixtureParity[StockChartAdjustedBar](t, name)
		if len(adjusted) != 1 || adjusted[0] != wantAdjusted {
			t.Fatalf("%s = %+v, want %+v", name, adjusted, wantAdjusted)
		}
		encoded, err := json.Marshal(adjusted[0])
		if err != nil {
			t.Fatalf("%s: re-encode: %v", name, err)
		}
		for _, key := range []string{`"adjOpen"`, `"adjHigh"`, `"adjLow"`, `"adjClose"`} {
			if !strings.Contains(string(encoded), key) {
				t.Fatalf("%s: re-encoded row %s lacks %s", name, encoded, key)
			}
		}
	}

	intraday := assertFixtureParity[StockChartIntradayBar](t, "stock_chart_intraday.json")
	want := StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:16:00"), Open: 332.4, Low: 332.27499,
		High: 332.48, Close: 332.47, Volume: 67_660}
	if len(intraday) != 1 || intraday[0] != want {
		t.Fatalf("stock_chart_intraday = %+v, want %+v", intraday, want)
	}
	if got := intraday[0].Date.String(); got != "2026-07-30 13:16:00" {
		t.Fatalf("date = %q, want the naive wire text", got)
	}
}

// Exact rows copied from crates/libfmp/tests/chart_intraday_responses.rs:
// every interval fixture carries a distinct documented row.
func TestDocumentedIntradayRoutesDecodeTheirExactRows(t *testing.T) {
	t.Parallel()
	cases := []struct {
		fixture string
		want    StockChartIntradayBar
	}{
		{"stock_chart_one_minute.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:16:00"),
			Open: 332.4, Low: 332.27499, High: 332.48, Close: 332.47, Volume: 67_660}},
		{"stock_chart_five_minutes.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:15:00"),
			Open: 332.655, Low: 332.31989, High: 332.755, Close: 332.31989, Volume: 123_020}},
		{"stock_chart_fifteen_minutes.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:15:00"),
			Open: 332.655, Low: 332.31989, High: 332.755, Close: 332.31989, Volume: 123_020}},
		{"stock_chart_thirty_minutes.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:00:00"),
			Open: 331.71, Low: 331.71, High: 332.82999, Close: 332.31989, Volume: 980_442}},
		{"stock_chart_one_hour.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 12:30:00"),
			Open: 332.14, Low: 331.43, High: 332.82999, Close: 332.31989, Volume: 3_285_503}},
		{"stock_chart_four_hours.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 09:30:00"),
			Open: 333.13, Low: 329.70499, High: 334.26, Close: 332.31989, Volume: 28_439_347}},
	}
	for _, tc := range cases {
		rows := assertFixtureParity[StockChartIntradayBar](t, tc.fixture)
		if len(rows) != 1 || rows[0] != tc.want {
			t.Fatalf("%s = %+v, want %+v", tc.fixture, rows, tc.want)
		}
	}
}

// Exact values copied from crates/libfmp/tests/asset_history_responses.rs and
// indexes_history_responses.rs for the fixtures of the domains that reuse
// the chart models.
func TestAssetAndIndexChartFixturesPreserveDocumentedValues(t *testing.T) {
	t.Parallel()
	commodity := assertFixtureParity[StockChartIntradayBar](t, "commodity_chart_one_minute.json")
	if len(commodity) != 1 || commodity[0].Date.String() != "2026-07-30 13:06:00" {
		t.Fatalf("commodity_chart_one_minute = %+v", commodity)
	}
	forex := assertFixtureParity[StockChartIntradayBar](t, "forex_chart_one_minute.json")
	if len(forex) != 1 || forex[0].Date.String() != "2026-07-30 13:17:00" {
		t.Fatalf("forex_chart_one_minute = %+v", forex)
	}
	cryptoLight := assertFixtureParity[StockChartLightBar](t, "crypto_chart_light.json")
	if len(cryptoLight) != 1 || cryptoLight[0].Symbol != "BTCUSD" || cryptoLight[0].Volume != 32_030_003_200 {
		t.Fatalf("crypto_chart_light = %+v", cryptoLight)
	}
	cryptoFull := assertFixtureParity[StockChartFullBar](t, "crypto_chart_full.json")
	if len(cryptoFull) != 1 || cryptoFull[0].Volume != 32_030_003_200 {
		t.Fatalf("crypto_chart_full = %+v", cryptoFull)
	}
	cryptoIntraday := assertFixtureParity[StockChartIntradayBar](t, "crypto_chart_one_minute.json")
	if len(cryptoIntraday) != 1 || cryptoIntraday[0].Volume != 0 {
		t.Fatalf("crypto_chart_one_minute = %+v", cryptoIntraday)
	}

	indexLight := assertFixtureParity[StockChartLightBar](t, "indexes_chart_light.json")
	if len(indexLight) != 1 || indexLight[0].Symbol != "^VIX" || indexLight[0].Price != 17.9 {
		t.Fatalf("indexes_chart_light = %+v", indexLight)
	}
	indexFull := assertFixtureParity[StockChartFullBar](t, "indexes_chart_full.json")
	if len(indexFull) != 1 || indexFull[0].Symbol != "^VIX" || indexFull[0].ChangePercent != -8.48671 {
		t.Fatalf("indexes_chart_full = %+v", indexFull)
	}
	for fixture, date := range map[string]string{
		"indexes_chart_one_minute.json":   "2026-07-30 13:17:00",
		"indexes_chart_five_minutes.json": "2026-07-30 13:15:00",
		"indexes_chart_one_hour.json":     "2026-07-30 12:30:00",
	} {
		rows := assertFixtureParity[StockChartIntradayBar](t, fixture)
		if len(rows) != 1 || rows[0].Date.String() != date {
			t.Fatalf("%s = %+v, want date %s", fixture, rows, date)
		}
	}
}

// Mirrors assert_required and the zoned-timestamp rejection in
// crates/libfmp/tests/chart_responses.rs: every member is required and
// non-null, and the intraday date is a strict naive datetime.
func TestChartRequiredMembersAndNaiveDatetimeAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name   string
		wire   string
		model  string
		member string
	}{
		{"light missing volume", `[{"symbol":"AAPL","date":"2026-07-30","price":332.39}]`, "StockChartLightBar", "volume"},
		{"full null vwap", `[{"symbol":"AAPL","date":"2026-07-30","open":1,"high":1,"low":1,"close":1,"volume":1,` +
			`"change":0,"changePercent":0,"vwap":null}]`, "StockChartFullBar", "vwap"},
		{"adjusted missing adjClose", `[{"symbol":"AAPL","date":"2026-07-30","adjOpen":1,"adjHigh":1,"adjLow":1,` +
			`"volume":1}]`, "StockChartAdjustedBar", "adjClose"},
		{"intraday null date", `[{"date":null,"open":1,"low":1,"high":1,"close":1,"volume":1}]`,
			"StockChartIntradayBar", "date"},
		{"intraday empty object", `[{}]`, "StockChartIntradayBar", "date"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var err error
			switch tc.model {
			case "StockChartLightBar":
				err = json.Unmarshal([]byte(tc.wire), new([]StockChartLightBar))
			case "StockChartFullBar":
				err = json.Unmarshal([]byte(tc.wire), new([]StockChartFullBar))
			case "StockChartAdjustedBar":
				err = json.Unmarshal([]byte(tc.wire), new([]StockChartAdjustedBar))
			default:
				err = json.Unmarshal([]byte(tc.wire), new([]StockChartIntradayBar))
			}
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, tc.model) {
				t.Fatalf("message = %q, want it to name member %q of %s", typed.Message, tc.member, tc.model)
			}
		})
	}

	for _, zoned := range []string{"2026-07-30T13:16:00Z", "2026-07-30 13:16:00-04:00"} {
		wire := `[{"date":"` + zoned + `","open":1,"low":1,"high":1,"close":1,"volume":1}]`
		var rows []StockChartIntradayBar
		if err := json.Unmarshal([]byte(wire), &rows); err == nil {
			t.Fatalf("zoned timestamp %q decoded into StockChartIntradayBar", zoned)
		}
	}
	for _, wire := range []string{`[]`, `{"historical":[]}`} {
		var rows []StockChartFullBar
		err := json.Unmarshal([]byte(wire), &rows)
		if wire == `[]` && (err != nil || rows == nil || len(rows) != 0) {
			t.Fatalf("empty array: rows = %#v, err = %v, want a non-nil empty slice", rows, err)
		}
		if wire != `[]` && err == nil {
			t.Fatal("an object root decoded into a bare-array chart contract")
		}
	}
}
