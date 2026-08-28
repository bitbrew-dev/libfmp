use std::collections::BTreeSet;

use libfmp::responses::statements::AsReportedFinancialStatement;

const CASH: &[u8] = include_bytes!("fixtures/cash_flow_statement_as_reported.json");
const FULL: &[u8] = include_bytes!("fixtures/financial_statement_full_as_reported.json");

#[test]
fn cash_fixture_preserves_exact_envelope_and_all_28_unique_dynamic_members() {
    let source: serde_json::Value = serde_json::from_slice(CASH).unwrap();
    let rows: Vec<AsReportedFinancialStatement> = serde_json::from_value(source.clone()).unwrap();

    assert_eq!(rows.len(), 1);
    assert_envelope(&rows[0]);
    assert_eq!(rows[0].data.len(), 28);
    assert_eq!(
        rows[0].data.keys().collect::<BTreeSet<_>>().len(),
        rows[0].data.len()
    );
    assert!(rows[0].data.values().all(serde_json::Value::is_number));
    assert_eq!(serde_json::to_value(&rows).unwrap(), source);

    assert_eq!(
        rows[0].data["netincomeloss"].as_i64(),
        Some(112_010_000_000)
    );
    assert_eq!(
        rows[0].data["increasedecreaseininventories"].as_i64(),
        Some(-1_400_000_000)
    );
    assert_eq!(
        rows[0].data["netcashprovidedbyusedinfinancingactivities"].as_i64(),
        Some(-120_686_000_000)
    );
}

#[test]
fn full_fixture_preserves_all_300_unique_members_and_exact_value_kinds() {
    let source: serde_json::Value = serde_json::from_slice(FULL).unwrap();
    let rows: Vec<AsReportedFinancialStatement> = serde_json::from_value(source.clone()).unwrap();

    assert_eq!(rows.len(), 1);
    assert_envelope(&rows[0]);
    assert_eq!(rows[0].data.len(), 300);
    assert_eq!(
        rows[0].data.keys().collect::<BTreeSet<_>>().len(),
        rows[0].data.len()
    );
    assert_eq!(
        rows[0]
            .data
            .values()
            .filter(|value| value.is_number())
            .count(),
        253
    );
    assert_eq!(
        rows[0]
            .data
            .values()
            .filter(|value| value.is_string())
            .count(),
        47
    );
    assert_eq!(serde_json::to_value(&rows).unwrap(), source);
}

#[test]
fn full_dynamic_values_keep_large_negative_scientific_and_string_semantics() {
    let rows: Vec<AsReportedFinancialStatement> = serde_json::from_slice(FULL).unwrap();
    let data = &rows[0].data;

    assert_eq!(
        data["revenuefromcontractwithcustomerexcludingassessedtax"].as_i64(),
        Some(416_161_000_000)
    );
    assert_eq!(
        data["nonoperatingincomeexpense"].as_i64(),
        Some(-321_000_000)
    );
    assert_eq!(
        data["commonstockparorstatedvaluepershare"].as_f64(),
        Some(0.00001)
    );
    assert!(data["commonstockparorstatedvaluepershare"].is_number());
    assert_eq!(data["documentannualreport"].as_str(), Some("true"));
    assert_eq!(data["documenttransitionreport"].as_str(), Some("false"));
    assert_eq!(
        data["maximumlengthoftimeforeigncurrencycashflowhedge"].as_str(),
        Some("P17Y")
    );
    assert_eq!(
        data["operatingandfinanceleaseweightedaverageremainingleaseterm"].as_str(),
        Some("P9Y9M18D")
    );
    assert_eq!(
        data["hedgedliabilitystatementoffinancialpositionextensibleenumeration"].as_str(),
        Some("http://fasb.org/us-gaap/2025#LongTermDebtNoncurrent")
    );
}

#[test]
fn dynamic_object_preserves_arbitrary_precision_and_open_nested_future_values() {
    let mut value: serde_json::Value = serde_json::from_slice(FULL).unwrap();
    let huge = serde_json::from_str::<serde_json::Value>(
        "12345678901234567890123456789012345678901234567890.123456789",
    )
    .unwrap();
    let tiny = serde_json::from_str::<serde_json::Value>("-9.87654321e-1234").unwrap();
    value[0]["data"]["issuerHugePrecision"] = huge.clone();
    value[0]["data"]["issuerTinyScientific"] = tiny.clone();
    value[0]["data"]["issuerFutureNested"] =
        serde_json::json!({ "values": [null, false, "raw", { "ratio": 1.25 }] });

    let rows: Vec<AsReportedFinancialStatement> = serde_json::from_value(value).unwrap();
    assert_eq!(rows[0].data["issuerHugePrecision"], huge);
    assert_eq!(rows[0].data["issuerTinyScientific"], tiny);
    assert_eq!(
        rows[0].data["issuerHugePrecision"].to_string(),
        "12345678901234567890123456789012345678901234567890.123456789"
    );
    assert_eq!(
        rows[0].data["issuerTinyScientific"].to_string(),
        "-9.87654321e-1234"
    );
    assert_eq!(
        rows[0].data["issuerFutureNested"]["values"][3]["ratio"].as_f64(),
        Some(1.25)
    );
}

#[test]
fn every_envelope_field_is_required_and_uses_its_strict_wire_kind() {
    for fixture in [CASH, FULL] {
        let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        assert_eq!(source[0].as_object().unwrap().len(), 6);
        for key in [
            "symbol",
            "fiscalYear",
            "period",
            "reportedCurrency",
            "date",
            "data",
        ] {
            let mut missing = source.clone();
            missing[0].as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<Vec<AsReportedFinancialStatement>>(missing).is_err(),
                "{key} unexpectedly became optional"
            );
        }
    }

    let mut string_year: serde_json::Value = serde_json::from_slice(FULL).unwrap();
    string_year[0]["fiscalYear"] = serde_json::json!("2025");
    assert!(serde_json::from_value::<Vec<AsReportedFinancialStatement>>(string_year).is_err());

    let mut retrieval_period: serde_json::Value = serde_json::from_slice(FULL).unwrap();
    retrieval_period[0]["period"] = serde_json::json!("annual");
    assert!(serde_json::from_value::<Vec<AsReportedFinancialStatement>>(retrieval_period).is_err());

    for invalid_data in [serde_json::json!(null), serde_json::json!([])] {
        let mut value: serde_json::Value = serde_json::from_slice(CASH).unwrap();
        value[0]["data"] = invalid_data;
        assert!(serde_json::from_value::<Vec<AsReportedFinancialStatement>>(value).is_err());
    }
}

#[test]
fn shared_bare_array_contract_preserves_empty_and_multiple_rows() {
    assert!(
        serde_json::from_slice::<Vec<AsReportedFinancialStatement>>(b"[]")
            .unwrap()
            .is_empty()
    );

    let cash: serde_json::Value = serde_json::from_slice(CASH).unwrap();
    let full: serde_json::Value = serde_json::from_slice(FULL).unwrap();
    let multiple = serde_json::Value::Array(vec![cash[0].clone(), full[0].clone()]);
    assert_eq!(
        serde_json::from_value::<Vec<AsReportedFinancialStatement>>(multiple)
            .unwrap()
            .len(),
        2
    );

    let wrapped = serde_json::json!({ "financialStatementFullAsReported": full });
    assert!(serde_json::from_value::<Vec<AsReportedFinancialStatement>>(wrapped).is_err());
}

fn assert_envelope(row: &AsReportedFinancialStatement) {
    assert_eq!(row.symbol.as_str(), "AAPL");
    assert_eq!(row.fiscal_year.get(), 2025);
    assert_eq!(row.period.to_string(), "FY");
    assert_eq!(row.reported_currency.as_str(), "USD");
    assert_eq!(row.date.to_string(), "2025-09-26");
}
