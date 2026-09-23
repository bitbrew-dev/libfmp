package fmp

import (
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

func TestQuoteFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	for _, name := range []string{"quote.json", "quote_empty.json", "quote_commodity.json"} {
		assertFixtureParity[Quote](t, name)
	}
	for _, name := range []string{"quote_short.json", "quote_short_empty.json", "quote_short_multiple.json",
		"quote_etf_short.json"} {
		assertFixtureParity[QuoteShort](t, name)
	}
	assertFixtureParity[QuoteShort](t, "quote_short_unknown.json", "futureProviderField")
	assertFixtureParity[AftermarketQuote](t, "aftermarket_quote.json")
	assertFixtureParity[AftermarketQuote](t, "aftermarket_quote_synthetic.json", "futureField")
	assertFixtureParity[AftermarketTrade](t, "aftermarket_trade.json")
	assertFixtureParity[AftermarketTrade](t, "aftermarket_trade_synthetic.json", "futureField")
	assertFixtureParity[StockPriceChange](t, "stock_price_change.json")
	assertFixtureParity[StockPriceChange](t, "stock_price_change_synthetic.json", "futureField")
	assertFixtureParity[QuoteShort](t, "quote_exchange_short.json")
	// The closed short-only universes (crates/libfmp/tests/quote_universe_endpoints.rs).
	for _, name := range []string{"quote_mutual_fund_short.json", "quote_commodity_short.json",
		"quote_crypto_short.json", "quote_forex_short.json", "quote_index_short.json"} {
		assertFixtureParity[QuoteShort](t, name)
	}
}

// Exact values copied from crates/libfmp/tests/quote_responses.rs for the
// documented aftermarket and price-change fixtures.
func TestDocumentedAftermarketAndPriceChangeFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	trades := assertFixtureParity[AftermarketTrade](t, "aftermarket_trade.json")
	if want := (AftermarketTrade{Symbol: "AAPL", Price: 331.85999, TradeSize: 16,
		Timestamp: UnixMilliseconds(1_785_430_813_000)}); len(trades) != 1 || trades[0] != want {
		t.Fatalf("aftermarket_trade = %+v", trades)
	}
	if got := trades[0].Timestamp.Time().UnixMilli(); got != 1_785_430_813_000 {
		t.Fatalf("timestamp.Time().UnixMilli() = %d", got)
	}
	quotes := assertFixtureParity[AftermarketQuote](t, "aftermarket_quote.json")
	if want := (AftermarketQuote{Symbol: "AAPL", BidSize: 16, BidPrice: 331.85, AskSize: 40, AskPrice: 331.88,
		Volume: 28_718_455, Timestamp: UnixMilliseconds(1_785_430_813_000)}); len(quotes) != 1 || quotes[0] != want {
		t.Fatalf("aftermarket_quote = %+v", quotes)
	}
	changes := assertFixtureParity[StockPriceChange](t, "stock_price_change.json")
	if len(changes) != 1 || changes[0].Symbol != "AAPL" || changes[0].OneDay != -1.8732 ||
		changes[0].TenYears != 1151.81068 || changes[0].YearToDate != 22.06835 || changes[0].Max != 258454.74094 {
		t.Fatalf("stock_price_change = %+v", changes)
	}
}

// Exact values copied from crates/libfmp/tests/quote_responses.rs.
func TestDocumentedFullQuoteDecodesExactWireValues(t *testing.T) {
	t.Parallel()
	quotes := assertFixtureParity[Quote](t, "quote.json")
	if len(quotes) != 1 {
		t.Fatalf("rows = %d, want 1", len(quotes))
	}
	quote := quotes[0]
	if quote.MarketCap == nil {
		t.Fatal("marketCap decoded as nil")
	}
	want := Quote{
		Symbol: "AAPL", Name: "Apple Inc.", Price: 331.85501, ChangePercentage: -1.8732, Change: -6.33498,
		Volume: 28_718_014, DayLow: 329.59, DayHigh: 334.48, YearHigh: 344.57, YearLow: 201.5,
		MarketCap: quote.MarketCap, PriceAvg50: 308.5888, PriceAvg200: 277.21344, Exchange: "NASDAQ",
		Open: 333.13, PreviousClose: 338.18999, Timestamp: UnixSeconds(1_785_430_812),
	}
	if quote != want {
		t.Fatalf("quote = %+v, want %+v", quote, want)
	}
	if *quote.MarketCap != 4_874_072_686_740 {
		t.Fatalf("marketCap = %d", *quote.MarketCap)
	}
	if got := quote.Timestamp.Time().Unix(); got != 1_785_430_812 {
		t.Fatalf("timestamp.Time().Unix() = %d", got)
	}
}

func TestDocumentedCommodityQuoteAcceptsNullMarketCap(t *testing.T) {
	t.Parallel()
	quotes := assertFixtureParity[Quote](t, "quote_commodity.json")
	if len(quotes) != 1 || quotes[0].Symbol != "GCUSD" || quotes[0].Exchange != "COMMODITY" {
		t.Fatalf("quotes = %+v", quotes)
	}
	if quotes[0].MarketCap != nil {
		t.Fatalf("marketCap = %v, want nil for JSON null", *quotes[0].MarketCap)
	}
}

func TestShortQuoteFixturesPreserveDocumentedValues(t *testing.T) {
	t.Parallel()
	short := assertFixtureParity[QuoteShort](t, "quote_short.json")
	if want := (QuoteShort{Symbol: "AAPL", Price: 331.85501, Change: -6.33498, Volume: 28_718_014}); len(short) != 1 ||
		short[0] != want {
		t.Fatalf("quote_short = %+v", short)
	}
	if empty := assertFixtureParity[QuoteShort](t, "quote_short_empty.json"); empty == nil || len(empty) != 0 {
		t.Fatalf("quote_short_empty = %#v, want a non-nil empty slice", empty)
	}
	multiple := assertFixtureParity[QuoteShort](t, "quote_short_multiple.json")
	if len(multiple) != 2 || multiple[0].Symbol != "000001.SZ" || multiple[0].Volume != 4_294_967_296 ||
		multiple[1].Symbol != "^VIX" || multiple[1].Volume != 0 {
		t.Fatalf("quote_short_multiple = %+v", multiple)
	}
	unknown := assertFixtureParity[QuoteShort](t, "quote_short_unknown.json", "futureProviderField")
	if len(unknown) != 1 || unknown[0] != short[0] {
		t.Fatalf("quote_short_unknown = %+v, want %+v", unknown, short)
	}
	fractional := assertFixtureParity[QuoteShort](t, "quote_short_fractional_volume.json")
	if want := (QuoteShort{Symbol: "AAPL", Price: 342.395, Change: 3.415, Volume: 20_201_922.82733}); len(fractional) != 1 ||
		fractional[0] != want {
		t.Fatalf("quote_short_fractional_volume = %+v, want %+v", fractional, want)
	}
}

func TestRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing member", `[{"symbol":"AAPL","price":1.5,"volume":1}]`, "change"},
		{"null member", `[{"symbol":"AAPL","price":null,"change":0,"volume":1}]`, "price"},
		{"empty object", `[{}]`, "symbol"},
		{"null element", `[null]`, "symbol"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []QuoteShort
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "QuoteShort") {
				t.Fatalf("message = %q, want it to name member %q of QuoteShort", typed.Message, tc.member)
			}
		})
	}

	var quotes []Quote
	err := json.Unmarshal([]byte(`[{"symbol":"AAPL","name":"Apple Inc."}]`), &quotes)
	var typed *Error
	if !errors.As(err, &typed) || !strings.Contains(typed.Message, `"price"`) {
		t.Fatalf("Quote error = %v, want the first missing member price", err)
	}
	if err := json.Unmarshal([]byte(`[{"symbol":"AAPL","price":"1.5","change":0,"volume":1}]`), &quotes); err == nil {
		t.Fatal("a numeric string decoded into a float64 member")
	}
}
