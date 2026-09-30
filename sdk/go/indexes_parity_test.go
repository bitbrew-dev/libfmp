package fmp

import (
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

// The 15 shared fixtures the Rust tests decode into a model the indexes
// endpoints return (crates/libfmp/tests/indexes_responses.rs,
// indexes_constituent_responses.rs and indexes_history_responses.rs). Only
// IndexListing, IndexConstituent and HistoricalIndexConstituent are defined
// in this domain; the quote and chart models are reused from quote_models.go
// and chart_models.go, so their fixtures are proven here through the same
// types. indexes_quotes.json is the closed batch-index-quotes route, which the
// registry does not expose as an indexes method, so it is parity-only.
func TestIndexesFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[IndexListing](t, "indexes_list.json")
	assertFixtureParity[Quote](t, "indexes_quote.json")
	for _, name := range []string{"indexes_quote_short.json", "indexes_quotes.json"} {
		assertFixtureParity[QuoteShort](t, name)
	}
	assertFixtureParity[StockChartLightBar](t, "indexes_chart_light.json")
	assertFixtureParity[StockChartFullBar](t, "indexes_chart_full.json")
	for _, name := range []string{"indexes_chart_one_minute.json", "indexes_chart_five_minutes.json",
		"indexes_chart_one_hour.json"} {
		assertFixtureParity[StockChartIntradayBar](t, name)
	}
	for _, name := range []string{"indexes_sp500_constituents.json", "indexes_nasdaq_constituents.json",
		"indexes_dow_jones_constituents.json"} {
		assertFixtureParity[IndexConstituent](t, name)
	}
	for _, name := range []string{"indexes_historical_sp500_constituents.json",
		"indexes_historical_nasdaq_constituents.json", "indexes_historical_dow_jones_constituents.json"} {
		assertFixtureParity[HistoricalIndexConstituent](t, name)
	}
}

// Exact values copied from indexes_responses.rs
// (every_documented_index_directory_and_quote_fixture_matches_the_typed_wire_contract).
func TestDocumentedIndexDirectoryAndQuoteFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	listing := assertFixtureParity[IndexListing](t, "indexes_list.json")
	if want := (IndexListing{Symbol: "^TTIN", Name: "S&P/TSX Capped Industrials Index", Exchange: "TSX",
		Currency: "CAD"}); len(listing) != 1 || listing[0] != want {
		t.Fatalf("indexes_list = %+v, want %+v", listing, want)
	}

	quotes := assertFixtureParity[Quote](t, "indexes_quote.json")
	if len(quotes) != 1 || quotes[0].Symbol != "^VIX" || quotes[0].Name != "CBOE Volatility Index" ||
		quotes[0].Exchange != "INDEX" || quotes[0].Price != 18.1 || quotes[0].Change != -2.56 ||
		quotes[0].Volume != 0 || quotes[0].Timestamp != UnixSeconds(1_785_430_786) {
		t.Fatalf("indexes_quote = %+v", quotes)
	}
	if quotes[0].MarketCap == nil || *quotes[0].MarketCap != 0 {
		t.Fatalf("marketCap = %v, want a non-nil pointer to 0 for JSON 0", quotes[0].MarketCap)
	}

	short := assertFixtureParity[QuoteShort](t, "indexes_quote_short.json")
	if len(short) != 1 || !quoteShortMatches(short[0], "^VIX", 18.1, -2.56, 0) {
		t.Fatalf("indexes_quote_short = %+v", short)
	}
	batch := assertFixtureParity[QuoteShort](t, "indexes_quotes.json")
	if len(batch) != 1 || !quoteShortMatches(batch[0], "^SPROME10", 4520.37, 44.67, 0) {
		t.Fatalf("indexes_quotes = %+v", batch)
	}
}

// Exact values copied from indexes_history_responses.rs
// (all_five_documented_index_history_fixtures_match_the_shared_chart_contracts_exactly).
func TestDocumentedIndexChartFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	date := mustParseDate(t, "2026-07-30")

	light := assertFixtureParity[StockChartLightBar](t, "indexes_chart_light.json")
	if want := (StockChartLightBar{Symbol: "^VIX", Date: date, Price: 17.9, Volume: 0}); len(light) != 1 ||
		light[0] != want {
		t.Fatalf("indexes_chart_light = %+v, want %+v", light, want)
	}
	full := assertFixtureParity[StockChartFullBar](t, "indexes_chart_full.json")
	if want := (StockChartFullBar{Symbol: "^VIX", Date: date, Open: 19.56, High: 20.08, Low: 17.9, Close: 17.9,
		Volume: 0, Change: -1.66, ChangePercent: -8.48671, Vwap: 18.63}); len(full) != 1 || full[0] != want {
		t.Fatalf("indexes_chart_full = %+v, want %+v", full, want)
	}

	intraday := []struct {
		fixture string
		date    string
		close   float64
	}{
		{"indexes_chart_one_minute.json", "2026-07-30 13:17:00", 17.91},
		{"indexes_chart_five_minutes.json", "2026-07-30 13:15:00", 17.96},
		{"indexes_chart_one_hour.json", "2026-07-30 12:30:00", 17.96},
	}
	for _, tc := range intraday {
		rows := assertFixtureParity[StockChartIntradayBar](t, tc.fixture)
		if len(rows) != 1 || rows[0].Date.String() != tc.date || rows[0].Close != tc.close || rows[0].Volume != 0 {
			t.Fatalf("%s = %+v, want date %s close %v", tc.fixture, rows, tc.date, tc.close)
		}
	}
}

// Exact values copied from indexes_constituent_responses.rs
// (all_six_documented_constituent_fixtures_match_the_typed_wire_contract_exactly).
func TestDocumentedIndexConstituentFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	sp500 := assertFixtureParity[IndexConstituent](t, "indexes_sp500_constituents.json")
	if len(sp500) != 1 || sp500[0].Symbol != "HONA" || sp500[0].CIK != "0002089271" ||
		sp500[0].SubSector != "Aerospace & Defense" || sp500[0].Founded != "1902/1985" {
		t.Fatalf("indexes_sp500_constituents = %+v", sp500)
	}
	if got := sp500[0].DateFirstAdded; got == nil || got.String() != "2026-06-29" {
		t.Fatalf("dateFirstAdded = %v, want 2026-06-29", got)
	}

	nasdaq := assertFixtureParity[IndexConstituent](t, "indexes_nasdaq_constituents.json")
	if len(nasdaq) != 1 || nasdaq[0].Symbol != "ADBE" || nasdaq[0].CIK != "0000796343" ||
		nasdaq[0].HeadQuarter != "San Jose, CA" || nasdaq[0].Founded != "1982-12-01" {
		t.Fatalf("indexes_nasdaq_constituents = %+v", nasdaq)
	}
	if nasdaq[0].DateFirstAdded != nil {
		t.Fatalf("dateFirstAdded = %v, want nil for JSON null", nasdaq[0].DateFirstAdded)
	}
	dowJones := assertFixtureParity[IndexConstituent](t, "indexes_dow_jones_constituents.json")
	if len(dowJones) != 1 || dowJones[0].Symbol != "GOOGL" || dowJones[0].Sector != "Communication Services" {
		t.Fatalf("indexes_dow_jones_constituents = %+v", dowJones)
	}

	history := assertFixtureParity[HistoricalIndexConstituent](t, "indexes_historical_nasdaq_constituents.json")
	if len(history) != 1 || history[0].DateAdded != "July 7, 2026" || history[0].Symbol != "SPCX" ||
		history[0].AddedSecurity == nil || *history[0].AddedSecurity != "Space Exploration Technologies Corp." ||
		history[0].Date != mustParseDate(t, "2026-07-06") {
		t.Fatalf("indexes_historical_nasdaq_constituents = %+v", history)
	}
	if history[0].RemovedTicker != nil || history[0].RemovedSecurity != nil {
		t.Fatalf("removed members = %v %v, want nil for JSON null", history[0].RemovedTicker,
			history[0].RemovedSecurity)
	}
	removal := assertFixtureParity[HistoricalIndexConstituent](t, "indexes_historical_sp500_constituents.json")
	if len(removal) != 1 || removal[0].RemovedTicker == nil || *removal[0].RemovedTicker != "CAG" ||
		removal[0].RemovedSecurity == nil || *removal[0].RemovedSecurity != "Conagra Brands" ||
		removal[0].Reason == nil || *removal[0].Reason != "Market Capitalization Changes" {
		t.Fatalf("indexes_historical_sp500_constituents = %+v", removal)
	}
	dowHistory := assertFixtureParity[HistoricalIndexConstituent](t, "indexes_historical_dow_jones_constituents.json")
	if len(dowHistory) != 1 || dowHistory[0].DateAdded != "June 29, 2026" || dowHistory[0].RemovedTicker == nil ||
		*dowHistory[0].RemovedTicker != "VZ" || dowHistory[0].Reason != nil {
		t.Fatalf("indexes_historical_dow_jones_constituents = %+v", dowHistory)
	}
}

// Mirrors current_constituent_fields_are_required_and_nullable_date_stays_required
// and historical_fields_are_required_while_documented_removals_accept_null in
// indexes_constituent_responses.rs, plus
// index_listing_fields_are_required_non_null_and_open_to_unknown_fields in
// indexes_responses.rs: a required_option member must be present but may be
// null, every other member must be present and non-null, and an unknown
// member is ignored.
func TestIndexesRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	const constituent = `"cik":"0000796343","founded":"1982-12-01","headQuarter":"San Jose, CA","name":"Adobe Inc.",` +
		`"sector":"Technology","subSector":"Software - Infrastructure","symbol":"ADBE"`
	const history = `"addedSecurity":"Space Exploration Technologies Corp.","date":"2026-07-06",` +
		`"dateAdded":"July 7, 2026","reason":"accelerated","symbol":"SPCX"`

	t.Run("accepted", func(t *testing.T) {
		t.Parallel()
		var rows []IndexConstituent
		wire := `[{"dateFirstAdded":null,` + constituent + `,"futureField":{"nested":true}}]`
		if err := json.Unmarshal([]byte(wire), &rows); err != nil || len(rows) != 1 || rows[0].DateFirstAdded != nil {
			t.Fatalf("null dateFirstAdded with unknown member: %+v, %v", rows, err)
		}
		for _, founded := range []string{"1994", "1902/1985", "1902/1985/2001"} {
			wire = `[{"dateFirstAdded":null,` + strings.Replace(constituent, "1982-12-01", founded, 1) + `}]`
			if err := json.Unmarshal([]byte(wire), &rows); err != nil || len(rows) != 1 || rows[0].Founded != founded {
				t.Fatalf("founded %q: %+v, %v", founded, rows, err)
			}
		}
		var changes []HistoricalIndexConstituent
		wire = `[{"removedSecurity":null,"removedTicker":null,` + history + `,"futureField":[1,2,3]}]`
		if err := json.Unmarshal([]byte(wire), &changes); err != nil || len(changes) != 1 ||
			changes[0].RemovedTicker != nil || changes[0].RemovedSecurity != nil {
			t.Fatalf("null removals with unknown member: %+v, %v", changes, err)
		}
		wire = `[{"addedSecurity":null,"date":"2026-07-06","dateAdded":"July 7, 2026","reason":null,` +
			`"removedSecurity":"","removedTicker":"","symbol":"SPCX"}]`
		if err := json.Unmarshal([]byte(wire), &changes); err != nil || len(changes) != 1 ||
			changes[0].AddedSecurity != nil || changes[0].Reason != nil || changes[0].RemovedTicker != nil ||
			changes[0].RemovedSecurity == nil || *changes[0].RemovedSecurity != "" {
			t.Fatalf("null added/reason and empty removedTicker: %+v, %v", changes, err)
		}
	})

	rejected := []struct {
		name   string
		wire   string
		model  string
		member string
	}{
		{"missing dateFirstAdded", `[{` + constituent + `}]`, "IndexConstituent", "dateFirstAdded"},
		{"null cik", `[{"dateFirstAdded":null,"cik":null,"founded":"1982-12-01","headQuarter":"San Jose, CA",` +
			`"name":"Adobe Inc.","sector":"Technology","subSector":"Software - Infrastructure","symbol":"ADBE"}]`,
			"IndexConstituent", "cik"},
		{"null founded", `[{"dateFirstAdded":null,"cik":"0000796343","founded":null,"headQuarter":"San Jose, CA",` +
			`"name":"Adobe Inc.","sector":"Technology","subSector":"Software - Infrastructure","symbol":"ADBE"}]`,
			"IndexConstituent", "founded"},
		{"empty constituent", `[{}]`, "IndexConstituent", "symbol"},
		{"missing removedTicker", `[{"removedSecurity":null,` + history + `}]`, "HistoricalIndexConstituent",
			"removedTicker"},
		{"null dateAdded", `[{"removedSecurity":null,"removedTicker":null,"addedSecurity":"x","date":"2026-07-06",` +
			`"dateAdded":null,"reason":"r","symbol":"SPCX"}]`, "HistoricalIndexConstituent", "dateAdded"},
		{"null listing exchange", `[{"currency":"CAD","exchange":null,"name":"n","symbol":"^TTIN"}]`,
			"IndexListing", "exchange"},
		{"null element", `[null]`, "IndexListing", "symbol"},
	}
	for _, tc := range rejected {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var err error
			switch tc.model {
			case "IndexConstituent":
				err = json.Unmarshal([]byte(tc.wire), new([]IndexConstituent))
			case "HistoricalIndexConstituent":
				err = json.Unmarshal([]byte(tc.wire), new([]HistoricalIndexConstituent))
			default:
				err = json.Unmarshal([]byte(tc.wire), new([]IndexListing))
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
	if err := json.Unmarshal([]byte(`[{"dateFirstAdded":7,`+constituent+`}]`), new([]IndexConstituent)); err == nil {
		t.Fatal("a numeric dateFirstAdded decoded into a Date member")
	}
}
