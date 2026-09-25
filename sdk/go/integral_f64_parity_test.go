package fmp

import (
	"encoding/json/v2"
	"testing"
)

// Issue #339: the provider documents market caps, crypto supplies, and split
// terms as JSON integers, but a fractional (1.5), integral-float (1.0), or
// exponent-form (4.788606639e9) number must decode too. The synthetic fixtures
// and the inline exponent rows mirror
// crates/libfmp/tests/integral_f64_responses.rs.
func TestIntegralFloatFieldsDecodeFractionalAndExponentForms(t *testing.T) {
	t.Parallel()

	screener := assertFixtureParity[CompanyScreenerEntry](t, "company_screener_fractional_synthetic.json")
	if len(screener) != 2 || screener[0].MarketCap != 4_885_602_246_714 ||
		screener[1].MarketCap != 1_234_567.5 || screener[1].Volume != 1 {
		t.Fatalf("company_screener_fractional_synthetic = %+v", screener)
	}

	crypto := assertFixtureParity[CryptocurrencyListing](t, "cryptocurrency_list_fractional_synthetic.json")
	if len(crypto) != 1 || crypto[0].CirculatingSupply != 4_232_705_124.5 ||
		crypto[0].TotalSupply != 4_788_606_639 {
		t.Fatalf("cryptocurrency_list_fractional_synthetic = %+v", crypto)
	}

	splits := assertFixtureParity[StockSplitEvent](t, "stock_splits_fractional_synthetic.json")
	if len(splits) != 1 || splits[0].Numerator != 1.5 || splits[0].Denominator != 1 {
		t.Fatalf("stock_splits_fractional_synthetic = %+v", splits)
	}
}

func TestIntegralFloatFieldsDecodeExponentForms(t *testing.T) {
	t.Parallel()

	var crypto CryptocurrencyListing
	err := json.Unmarshal([]byte(`{"symbol":"MIOTAUSD","name":"IOTA USD","exchange":"CCC","icoDate":"2017-11-09",`+
		`"circulatingSupply":4.2327051245e9,"totalSupply":4.788606639e9}`), &crypto)
	if err != nil || crypto.CirculatingSupply != 4_232_705_124.5 || crypto.TotalSupply != 4_788_606_639 {
		t.Fatalf("CryptocurrencyListing = %+v, %v", crypto, err)
	}

	var split StockSplitEvent
	err = json.Unmarshal([]byte(`{"symbol":"AAPL","date":"2020-08-31","numerator":4e0,"denominator":1E0,"splitType":"stock-split"}`), &split)
	if err != nil || split.Numerator != 4 || split.Denominator != 1 {
		t.Fatalf("StockSplitEvent = %+v, %v", split, err)
	}
}
