use libfmp::responses::directory::{
    ActivelyTradingSymbol, CikListing, CompanySymbol, EarningsTranscriptAvailability, EtfSymbol,
    FinancialStatementSymbol, SymbolChange,
};
use serde_json::{Value, json};

#[test]
fn documented_directory_rows_decode_exact_field_names_and_wire_types() {
    let companies: Vec<CompanySymbol> =
        serde_json::from_str(include_str!("fixtures/directory_company_symbols.json")).unwrap();
    let financials: Vec<FinancialStatementSymbol> = serde_json::from_str(include_str!(
        "fixtures/directory_financial_statement_symbols.json"
    ))
    .unwrap();
    let ciks: Vec<CikListing> =
        serde_json::from_str(include_str!("fixtures/directory_cik_list.json")).unwrap();
    let changes: Vec<SymbolChange> =
        serde_json::from_str(include_str!("fixtures/directory_symbol_changes.json")).unwrap();
    let etfs: Vec<EtfSymbol> =
        serde_json::from_str(include_str!("fixtures/directory_etf_symbols.json")).unwrap();
    let active: Vec<ActivelyTradingSymbol> =
        serde_json::from_str(include_str!("fixtures/directory_actively_trading.json")).unwrap();
    let transcripts: Vec<EarningsTranscriptAvailability> = serde_json::from_str(include_str!(
        "fixtures/directory_earnings_transcript_list.json"
    ))
    .unwrap();

    assert_eq!(companies[0].symbol.as_str(), "URBANCO.BO");
    assert_eq!(companies[0].company_name, "Urban Company Limited");

    assert_eq!(financials[0].symbol.as_str(), "RMES.CN");
    assert_eq!(financials[0].company_name, "Red Metal Resources Ltd.");
    assert_eq!(financials[0].trading_currency.as_str(), "CAD");
    assert_eq!(
        financials[0].reporting_currency.as_ref().unwrap().as_str(),
        "USD"
    );

    assert_eq!(ciks[0].cik.as_str(), "0002137358");
    assert_eq!(ciks[0].company_name, "Osotspa Public Co Limited/ADR");

    assert_eq!(changes[0].date.to_string(), "2026-07-28");
    assert_eq!(
        changes[0].company_name,
        "Yarrow Bioscience, Inc. Common Stock"
    );
    assert_eq!(changes[0].old_symbol.as_str(), "VYNE");
    assert_eq!(changes[0].new_symbol.as_str(), "YARW");

    assert_eq!(etfs[0].symbol.as_str(), "P60.SI");
    assert_eq!(
        etfs[0].name,
        "MULTI-UNITS LUXEMBOURG - Lyxor MSCI AC Asia Pacific Ex Japan UCITS ETF"
    );
    assert_eq!(active[0].symbol.as_str(), "URBANCO.BO");
    assert_eq!(active[0].name, "Urban Company Limited");

    assert_eq!(transcripts[0].symbol.as_str(), "INBS");
    assert_eq!(
        transcripts[0].company_name,
        "Intelligent Bio Solutions Inc."
    );
    assert_eq!(transcripts[0].no_of_transcripts.as_str(), "6");

    let company_wire = serde_json::to_value(&companies[0]).unwrap();
    let etf_wire = serde_json::to_value(&etfs[0]).unwrap();
    let transcript_wire = serde_json::to_value(&transcripts[0]).unwrap();
    assert_eq!(company_wire["companyName"], "Urban Company Limited");
    assert!(company_wire.get("name").is_none());
    assert_eq!(etf_wire["name"], etfs[0].name);
    assert!(etf_wire.get("companyName").is_none());
    assert_eq!(transcript_wire["noOfTranscripts"], "6");
    assert!(transcript_wire["noOfTranscripts"].is_string());
}

#[test]
fn financial_statement_symbol_reporting_currency_decodes_null_and_empty_as_none() {
    for reporting_currency in [Value::Null, json!("")] {
        let row = json!({
            "symbol": "1609.HK",
            "companyName": "Chong Kin Group Holdings Limited",
            "tradingCurrency": "HKD",
            "reportingCurrency": reporting_currency,
        });
        let decoded: FinancialStatementSymbol = serde_json::from_value(row).unwrap();
        assert_eq!(decoded.reporting_currency, None);
        assert!(serde_json::to_value(&decoded).unwrap()["reportingCurrency"].is_null());
    }

    let missing = json!({
        "symbol": "1609.HK",
        "companyName": "Chong Kin Group Holdings Limited",
        "tradingCurrency": "HKD",
    });
    assert!(serde_json::from_value::<FinancialStatementSymbol>(missing).is_err());
}

#[test]
fn every_directory_response_contract_preserves_empty_arrays() {
    let empty = include_str!("fixtures/directory_empty.json");

    assert!(
        serde_json::from_str::<Vec<CompanySymbol>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<FinancialStatementSymbol>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<CikListing>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<SymbolChange>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<EtfSymbol>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<ActivelyTradingSymbol>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<EarningsTranscriptAvailability>>(empty)
            .unwrap()
            .is_empty()
    );
}
