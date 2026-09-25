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

	screener := assertFixtureParity[CompanyScreenerResult](t, "company_screener_fractional_synthetic.json")
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

// Issue #340: share quantities and money amounts documented as JSON integers
// decode a fractional, negative, or integral-float number too. The synthetic
// fixtures mirror crates/libfmp/tests/integral_f64_responses.rs.
func TestQuantityAndAmountFieldsDecodeFractionalForms(t *testing.T) {
	t.Parallel()

	insider := assertFixtureParity[InsiderTrade](t, "latest_insider_trades_fractional_synthetic.json")
	if len(insider) != 1 || insider[0].SecuritiesOwned != 62_959.5 || insider[0].SecuritiesTransacted != 1 {
		t.Fatalf("latest_insider_trades_fractional_synthetic = %+v", insider)
	}

	disclosures := assertFixtureParity[FundDisclosure](t, "fund_disclosures_fractional_synthetic.json")
	if len(disclosures) != 1 || disclosures[0].Balance != -2_438_784.5 {
		t.Fatalf("fund_disclosures_fractional_synthetic = %+v", disclosures)
	}

	shareFloat := assertFixtureParity[CompanyShareFloat](t, "company_shares_float_fractional_synthetic.json")
	if len(shareFloat) != 1 || shareFloat[0].FloatShares != 14_662_387_495.5 || shareFloat[0].OutstandingShares != 14_687_356_000 {
		t.Fatalf("company_shares_float_fractional_synthetic = %+v", shareFloat)
	}

	offerings := assertFixtureParity[RegulationDOffering](t, "fundraising_by_cik_fractional_synthetic.json")
	if len(offerings) != 1 || offerings[0].TotalOfferingAmount != 71_999_990.5 || offerings[0].TotalAmountSold != 1 {
		t.Fatalf("fundraising_by_cik_fractional_synthetic = %+v", offerings)
	}

	earnings := assertFixtureParity[EarningsEvent](t, "earnings_calendar_fractional_synthetic.json")
	if len(earnings) != 1 || earnings[0].RevenueEstimated != 1_086_300_000.5 ||
		earnings[0].RevenueActual == nil || *earnings[0].RevenueActual != 1_101_500_000 {
		t.Fatalf("earnings_calendar_fractional_synthetic = %+v", earnings)
	}

	dcf := assertFixtureParity[CustomDCFValuation](t, "custom_discounted_cash_flow_fractional_synthetic.json")
	if len(dcf) != 1 || dcf[0].DilutedSharesOutstanding != 15_004_697_000.5 {
		t.Fatalf("custom_discounted_cash_flow_fractional_synthetic = %+v", dcf)
	}

	income := assertFixtureParity[IncomeStatement](t, "income_statement_fractional_synthetic.json")
	if len(income) != 1 || income[0].WeightedAverageShsOut != 14_948_500_000.5 ||
		income[0].WeightedAverageShsOutDil != 15_004_697_000 {
		t.Fatalf("income_statement_fractional_synthetic = %+v", income)
	}

	trades := assertFixtureParity[AftermarketTrade](t, "aftermarket_trade_fractional_synthetic.json")
	if len(trades) != 1 || trades[0].TradeSize != 16.5 {
		t.Fatalf("aftermarket_trade_fractional_synthetic = %+v", trades)
	}

	holdings := assertFixtureParity[InstitutionalHolding](t, "institutional_ownership_extract_fractional_synthetic.json")
	if len(holdings) != 1 || holdings[0].Shares != 13_280.5 || holdings[0].Value != 1 {
		t.Fatalf("institutional_ownership_extract_fractional_synthetic = %+v", holdings)
	}
}

func TestQuantityAndAmountFieldsDecodeExponentForms(t *testing.T) {
	t.Parallel()

	var trade AftermarketTrade
	err := json.Unmarshal([]byte(`{"symbol":"AAPL","price":232.53,"tradeSize":1.6e1,"timestamp":1738715334311}`), &trade)
	if err != nil || trade.TradeSize != 16 {
		t.Fatalf("AftermarketTrade = %+v, %v", trade, err)
	}

	var holding InstitutionalHolding
	err = json.Unmarshal([]byte(`{"date":"2023-09-30","filingDate":"2023-11-13","acceptedDate":"2023-11-13",`+
		`"cik":"0001388838","securityCusip":"674215207","symbol":"CHRD","nameOfIssuer":"CHORD ENERGY CORPORATION",`+
		`"shares":1.32805e4,"titleOfClass":"COM NEW","sharesType":"SH","putCallShare":"","value":2.5E6,`+
		`"link":"https://example.invalid","finalLink":"https://example.invalid"}`), &holding)
	if err != nil || holding.Shares != 13_280.5 || holding.Value != 2_500_000 {
		t.Fatalf("InstitutionalHolding = %+v, %v", holding, err)
	}
}

// Issue #341: statement amounts documented as JSON integers decode a
// fractional, integral-float, or exponent-form number too, mirroring
// crates/libfmp/tests/integral_f64_responses.rs.
func TestStatementAmountsDecodeFractionalAndExponentForms(t *testing.T) {
	t.Parallel()

	income := assertFixtureParity[IncomeStatement](t, "income_statement_fractional_synthetic.json")
	if len(income) != 1 || income[0].Revenue != 416_161_000_000.5 || income[0].Ebitda != 144_427_000_000 {
		t.Fatalf("income_statement_fractional_synthetic = %+v", income)
	}

	var rows []IncomeStatement
	source := statementsWithMember(t, "income_statement_fractional_synthetic.json", "revenue", "4.161610000005e11")
	if err := json.Unmarshal(source, &rows); err != nil || rows[0].Revenue != 416_161_000_000.5 {
		t.Fatalf("IncomeStatement = %+v, %v", rows, err)
	}
	source = statementsWithMember(t, "income_statement_fractional_synthetic.json", "ebitda", "-1.44427E11")
	if err := json.Unmarshal(source, &rows); err != nil || rows[0].Ebitda != -144_427_000_000 {
		t.Fatalf("IncomeStatement = %+v, %v", rows, err)
	}
}
