use std::collections::BTreeSet;

use libfmp::responses::statements::AsReportedFinancialStatement;

const INCOME: &[u8] = include_bytes!("fixtures/income_statement_as_reported.json");
const BALANCE: &[u8] = include_bytes!("fixtures/balance_sheet_statement_as_reported.json");

const INCOME_KEYS: [&str; 24] = [
    "revenuefromcontractwithcustomerexcludingassessedtax",
    "costofgoodsandservicessold",
    "grossprofit",
    "researchanddevelopmentexpense",
    "sellinggeneralandadministrativeexpense",
    "operatingexpenses",
    "operatingincomeloss",
    "nonoperatingincomeexpense",
    "incomelossfromcontinuingoperationsbeforeincometaxesextraordinaryitemsnoncontrollinginterest",
    "incometaxexpensebenefit",
    "netincomeloss",
    "earningspersharebasic",
    "earningspersharediluted",
    "weightedaveragenumberofsharesoutstandingbasic",
    "weightedaveragenumberofdilutedsharesoutstanding",
    "othercomprehensiveincomelossforeigncurrencytransactionandtranslationadjustmentnetoftax",
    "othercomprehensiveincomelosscashflowhedgegainlossbeforereclassificationaftertax",
    "othercomprehensiveincomelosscashflowhedgegainlossreclassificationaftertax",
    "othercomprehensiveincomelosscashflowhedgegainlossafterreclassificationandtax",
    "othercomprehensiveincomeunrealizedholdinggainlossonsecuritiesarisingduringperiodnetoftax",
    "othercomprehensiveincomelossreclassificationadjustmentfromaociforsaleofsecuritiesnetoftax",
    "othercomprehensiveincomelossavailableforsalesecuritiesadjustmentnetoftax",
    "othercomprehensiveincomelossnetoftaxportionattributabletoparent",
    "comprehensiveincomenetoftax",
];

const BALANCE_KEYS: [&str; 31] = [
    "cashandcashequivalentsatcarryingvalue",
    "marketablesecuritiescurrent",
    "accountsreceivablenetcurrent",
    "nontradereceivablescurrent",
    "inventorynet",
    "otherassetscurrent",
    "assetscurrent",
    "marketablesecuritiesnoncurrent",
    "propertyplantandequipmentnet",
    "otherassetsnoncurrent",
    "assetsnoncurrent",
    "assets",
    "accountspayablecurrent",
    "otherliabilitiescurrent",
    "contractwithcustomerliabilitycurrent",
    "commercialpaper",
    "longtermdebtcurrent",
    "liabilitiescurrent",
    "longtermdebtnoncurrent",
    "otherliabilitiesnoncurrent",
    "liabilitiesnoncurrent",
    "liabilities",
    "commonstocksharesoutstanding",
    "commonstocksharesissued",
    "commonstocksincludingadditionalpaidincapital",
    "retainedearningsaccumulateddeficit",
    "accumulatedothercomprehensiveincomelossnetoftax",
    "stockholdersequity",
    "liabilitiesandstockholdersequity",
    "commonstockparorstatedvaluepershare",
    "commonstocksharesauthorized",
];

#[test]
fn income_fixture_preserves_exact_envelope_all_24_dynamic_keys_and_number_types() {
    let source: serde_json::Value = serde_json::from_slice(INCOME).unwrap();
    let rows: Vec<AsReportedFinancialStatement> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_envelope(&rows[0]);
    assert_exact_dynamic_contract(&rows[0], &INCOME_KEYS);
    assert_eq!(serde_json::to_value(&rows).unwrap(), source);

    assert_eq!(
        rows[0].data["nonoperatingincomeexpense"].as_i64(),
        Some(-321_000_000)
    );
    assert_eq!(rows[0].data["earningspersharebasic"].as_f64(), Some(7.49));
    assert_eq!(
        rows[0].data["revenuefromcontractwithcustomerexcludingassessedtax"].as_i64(),
        Some(416_161_000_000)
    );
}

#[test]
fn balance_fixture_preserves_exact_envelope_all_31_dynamic_keys_and_number_types() {
    let source: serde_json::Value = serde_json::from_slice(BALANCE).unwrap();
    let rows: Vec<AsReportedFinancialStatement> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_envelope(&rows[0]);
    assert_exact_dynamic_contract(&rows[0], &BALANCE_KEYS);
    assert_eq!(serde_json::to_value(&rows).unwrap(), source);

    assert_eq!(
        rows[0].data["retainedearningsaccumulateddeficit"].as_i64(),
        Some(-14_264_000_000)
    );
    assert_eq!(
        rows[0].data["commonstockparorstatedvaluepershare"].as_f64(),
        Some(0.00001)
    );
    assert_eq!(
        rows[0].data["liabilitiesandstockholdersequity"].as_i64(),
        Some(359_241_000_000)
    );
}

fn assert_envelope(row: &AsReportedFinancialStatement) {
    assert_eq!(row.symbol.as_str(), "AAPL");
    assert_eq!(row.fiscal_year.get(), 2025);
    assert_eq!(row.period.to_string(), "FY");
    assert_eq!(row.reported_currency.as_str(), "USD");
    assert_eq!(row.date.to_string(), "2025-09-26");
}

fn assert_exact_dynamic_contract(row: &AsReportedFinancialStatement, expected: &[&str]) {
    let actual = row.data.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
    assert!(row.data.values().all(serde_json::Value::is_number));
}

#[test]
fn every_envelope_field_is_required_and_uses_its_strict_wire_kind() {
    for fixture in [INCOME, BALANCE] {
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

    let mut string_year: serde_json::Value = serde_json::from_slice(INCOME).unwrap();
    string_year[0]["fiscalYear"] = serde_json::json!("2025");
    assert!(serde_json::from_value::<Vec<AsReportedFinancialStatement>>(string_year).is_err());

    let mut retrieval_period: serde_json::Value = serde_json::from_slice(INCOME).unwrap();
    retrieval_period[0]["period"] = serde_json::json!("annual");
    assert!(serde_json::from_value::<Vec<AsReportedFinancialStatement>>(retrieval_period).is_err());

    for invalid_data in [serde_json::json!(null), serde_json::json!([])] {
        let mut value: serde_json::Value = serde_json::from_slice(INCOME).unwrap();
        value[0]["data"] = invalid_data;
        assert!(serde_json::from_value::<Vec<AsReportedFinancialStatement>>(value).is_err());
    }
}

#[test]
fn dynamic_data_is_open_while_the_shared_bare_array_shape_preserves_empty_and_multiple_rows() {
    let mut future: serde_json::Value = serde_json::from_slice(INCOME).unwrap();
    future[0]["data"]["issuerFutureMetric"] =
        serde_json::json!({ "nested": [null, false, "raw", 0.125] });
    let rows: Vec<AsReportedFinancialStatement> = serde_json::from_value(future).unwrap();
    assert_eq!(rows[0].data["issuerFutureMetric"]["nested"][2], "raw");

    assert!(
        serde_json::from_slice::<Vec<AsReportedFinancialStatement>>(b"[]")
            .unwrap()
            .is_empty()
    );
    let income: serde_json::Value = serde_json::from_slice(INCOME).unwrap();
    let balance: serde_json::Value = serde_json::from_slice(BALANCE).unwrap();
    let multiple = serde_json::Value::Array(vec![income[0].clone(), balance[0].clone()]);
    assert_eq!(
        serde_json::from_value::<Vec<AsReportedFinancialStatement>>(multiple)
            .unwrap()
            .len(),
        2
    );

    let wrapped = serde_json::json!({ "incomeStatementAsReported": income });
    assert!(serde_json::from_value::<Vec<AsReportedFinancialStatement>>(wrapped).is_err());
}
