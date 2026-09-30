package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"reflect"
	"slices"
	"strings"
	"testing"
)

// The nine shared fixtures the Rust tests decode into a model the crypto
// domain returns (crates/libfmp/tests/asset_catalog_quote_responses.rs and
// asset_history_responses.rs). The chart fixtures are also proven by
// chart_parity_test.go; they are listed here so every crypto fixture is
// named by the domain that serves it. cryptocurrency_quotes.json is the
// batch-crypto-quotes response, which has no registry method yet, so it is
// proven as a QuoteShort row only.
func TestCryptoFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[CryptocurrencyListing](t, "cryptocurrency_list.json")
	assertFixtureParity[Quote](t, "cryptocurrency_quote.json")
	assertFixtureParity[QuoteShort](t, "cryptocurrency_quote_short.json")
	assertFixtureParity[QuoteShort](t, "cryptocurrency_quotes.json")
	assertFixtureParity[StockChartLightBar](t, "crypto_chart_light.json")
	assertFixtureParity[StockChartFullBar](t, "crypto_chart_full.json")
	for _, name := range []string{"crypto_chart_one_minute.json", "crypto_chart_five_minutes.json",
		"crypto_chart_one_hour.json"} {
		assertFixtureParity[StockChartIntradayBar](t, name)
	}
}

// Exact values copied from crates/libfmp/tests/asset_catalog_quote_responses.rs
// (catalog_rows_preserve_exact_identifiers_dates_and_large_unsigned_supplies
// and crypto_quotes_preserve_large_volume_market_cap_and_nullable_market_cap)
// and asset_catalog_quote_endpoints.rs.
func TestCryptoCatalogAndQuoteFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	listing := assertFixtureParity[CryptocurrencyListing](t, "cryptocurrency_list.json")
	want := CryptocurrencyListing{Symbol: "MIOTAUSD", Name: "IOTA USD", Exchange: "CCC",
		IcoDate: new(mustParseDate(t, "2017-11-09")), CirculatingSupply: new(4_232_705_124.0), TotalSupply: new(4_788_606_639.0)}
	if len(listing) != 1 || !reflect.DeepEqual(listing[0], want) {
		t.Fatalf("cryptocurrency_list = %+v, want %+v", listing, want)
	}

	quotes := assertFixtureParity[Quote](t, "cryptocurrency_quote.json")
	if len(quotes) != 1 || quotes[0].Symbol != "BTCUSD" || quotes[0].Name != "Bitcoin USD" ||
		quotes[0].Exchange != "CRYPTO" || quotes[0].Price != 64756.84 || quotes[0].Volume != 32_030_003_200 ||
		quotes[0].Timestamp != UnixSeconds(1_785_430_805) {
		t.Fatalf("cryptocurrency_quote = %+v", quotes)
	}
	if quotes[0].MarketCap == nil || *quotes[0].MarketCap != 1_293_361_815_015 {
		t.Fatalf("marketCap = %v, want 1293361815015", quotes[0].MarketCap)
	}

	short := assertFixtureParity[QuoteShort](t, "cryptocurrency_quote_short.json")
	if len(short) != 1 || !quoteShortMatches(short[0], "BTCUSD", 64756.84, 853.94, 32_030_003_200) {
		t.Fatalf("cryptocurrency_quote_short = %+v", short)
	}
	batch := assertFixtureParity[QuoteShort](t, "cryptocurrency_quotes.json")
	if len(batch) != 1 || !quoteShortMatches(batch[0], "00USD", 0.0102, 0.000441090368, 329_571) {
		t.Fatalf("cryptocurrency_quotes = %+v", batch)
	}
}

// Exact values copied from crates/libfmp/tests/asset_history_responses.rs
// (asset_history_preserves_timezone_less_times_large_crypto_volume_and_zero_volume)
// and asset_history_endpoints.rs.
func TestCryptoChartFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	date := mustParseDate(t, "2026-07-30")
	light := assertFixtureParity[StockChartLightBar](t, "crypto_chart_light.json")
	if want := (StockChartLightBar{Symbol: "BTCUSD", Date: date, Price: 64766.98828, Volume: 32_030_003_200}); len(light) != 1 ||
		light[0] != want {
		t.Fatalf("crypto_chart_light = %+v, want %+v", light, want)
	}
	full := assertFixtureParity[StockChartFullBar](t, "crypto_chart_full.json")
	if want := (StockChartFullBar{Symbol: "BTCUSD", Date: date, Open: 63902.9, High: 65038.38, Low: 63547.793,
		Close: 64766.98828, Volume: 32_030_003_200, Change: 864.09, ChangePercent: 1.35219,
		Vwap: 64451.05}); len(full) != 1 || full[0] != want {
		t.Fatalf("crypto_chart_full = %+v, want %+v", full, want)
	}
	cases := []struct {
		fixture string
		want    StockChartIntradayBar
	}{
		{"crypto_chart_one_minute.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:16:00"),
			Open: 64727.04, Low: 64718.9, High: 64734.67188, Close: 64734.67, Volume: 0}},
		{"crypto_chart_five_minutes.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:15:00"),
			Open: 64723.74, Low: 64723.73828, High: 64734.67, Close: 64727.05, Volume: 0}},
		{"crypto_chart_one_hour.json", StockChartIntradayBar{Date: mustParseDateTime(t, "2026-07-30 13:00:00"),
			Open: 64760.34, Low: 64703.88, High: 64808.03, Close: 64727.05, Volume: 0}},
	}
	for _, tc := range cases {
		rows := assertFixtureParity[StockChartIntradayBar](t, tc.fixture)
		if len(rows) != 1 || rows[0] != tc.want {
			t.Fatalf("%s = %+v, want %+v", tc.fixture, rows, tc.want)
		}
	}
	if got := cases[2].want.Date.String(); got != "2026-07-30 13:00:00" {
		t.Fatalf("one-hour date = %q, want the naive wire text", got)
	}
}

// Mirrors assert_required_non_null in
// crates/libfmp/tests/asset_catalog_quote_responses.rs for the one model
// the crypto domain owns: every member is required, every member but the
// nullable ones is non-null, and an unknown member is accepted.
func TestCryptocurrencyListingRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	var rows []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, "cryptocurrency_list.json"), &rows); err != nil || len(rows) != 1 {
		t.Fatalf("cryptocurrency_list.json: %v (%d rows)", err, len(rows))
	}
	row := rows[0]
	members := []string{"symbol", "name", "exchange", "icoDate", "circulatingSupply", "totalSupply"}
	nullable := []string{"icoDate", "circulatingSupply", "totalSupply"}
	for _, member := range members {
		for variant, mutate := range map[string]func(map[string]jsontext.Value){
			"missing": func(m map[string]jsontext.Value) { delete(m, member) },
			"null":    func(m map[string]jsontext.Value) { m[member] = jsontext.Value("null") },
		} {
			if variant == "null" && slices.Contains(nullable, member) {
				continue
			}
			t.Run(member+" "+variant, func(t *testing.T) {
				t.Parallel()
				mutated := make(map[string]jsontext.Value, len(row))
				for key, value := range row {
					mutated[key] = value
				}
				mutate(mutated)
				wire, err := json.Marshal([]map[string]jsontext.Value{mutated})
				if err != nil {
					t.Fatalf("encode: %v", err)
				}
				err = json.Unmarshal(wire, new([]CryptocurrencyListing))
				var typed *Error
				if !errors.As(err, &typed) || typed.Category != CategoryDecode {
					t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
				}
				if !strings.Contains(typed.Message, `"`+member+`"`) ||
					!strings.Contains(typed.Message, "CryptocurrencyListing") {
					t.Fatalf("message = %q, want it to name member %q of CryptocurrencyListing", typed.Message, member)
				}
			})
		}
	}

	future := make(map[string]jsontext.Value, len(row)+1)
	for key, value := range row {
		future[key] = value
	}
	future["futureField"] = jsontext.Value(`[1, {"open": true}]`)
	wire, err := json.Marshal([]map[string]jsontext.Value{future})
	if err != nil {
		t.Fatalf("encode: %v", err)
	}
	var decoded []CryptocurrencyListing
	if err := json.Unmarshal(wire, &decoded); err != nil || len(decoded) != 1 || decoded[0].Symbol != "MIOTAUSD" {
		t.Fatalf("unknown member was not ignored: %v, %+v", err, decoded)
	}
	var empty []CryptocurrencyListing
	if err := json.Unmarshal([]byte(`[]`), &empty); err != nil || empty == nil || len(empty) != 0 {
		t.Fatalf("empty array: rows = %#v, err = %v, want a non-nil empty slice", empty, err)
	}
}

// Mirrors crypto_listing_supplies_and_ico_date_are_required_but_nullable in
// crates/libfmp/tests/asset_catalog_quote_responses.rs: a null supply or
// ICO date decodes to nil, and an empty ICO date is absent too.
func TestCryptocurrencyListingNullableMembersDecodeToNil(t *testing.T) {
	t.Parallel()
	const fixture = "cryptocurrency_list.json"
	cases := []struct {
		member string
		value  jsontext.Value
		isNil  func(CryptocurrencyListing) bool
	}{
		{"icoDate", jsontext.Value(`null`), func(row CryptocurrencyListing) bool { return row.IcoDate == nil }},
		{"icoDate", jsontext.Value(`""`), func(row CryptocurrencyListing) bool { return row.IcoDate == nil }},
		{"circulatingSupply", jsontext.Value(`null`), func(row CryptocurrencyListing) bool { return row.CirculatingSupply == nil }},
		{"totalSupply", jsontext.Value(`null`), func(row CryptocurrencyListing) bool { return row.TotalSupply == nil }},
	}
	for _, tc := range cases {
		var rows []CryptocurrencyListing
		if err := json.Unmarshal(mutateFixtureMember(t, fixture, tc.member, tc.value), &rows); err != nil ||
			len(rows) != 1 || !tc.isNil(rows[0]) {
			t.Fatalf("%s = %s: %+v, %v, want nil", tc.member, tc.value, rows, err)
		}
	}
}
