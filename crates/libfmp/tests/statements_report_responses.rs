use libfmp::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    responses::statements::{FinancialReportDate, FinancialReportJson},
    types::{CalendarYear, Ticker},
};

const DATES: &[u8] = include_bytes!("fixtures/financial_reports_dates.json");
const REPORT: &[u8] = include_bytes!("fixtures/financial_reports_json.json");
const JSON_LINK: &str = "https://financialmodelingprep.com/stable/financial-reports-json?symbol=AAPL&year=2026&period=Q2&apikey=[REDACTED]";
const XLSX_LINK: &str = "https://financialmodelingprep.com/stable/financial-reports-xlsx?symbol=AAPL&year=2026&period=Q2&apikey=[REDACTED]";

#[test]
fn report_dates_decode_exact_five_fields_and_keep_links_secret() {
    let value: serde_json::Value = serde_json::from_slice(DATES).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 5);

    let rows: Vec<FinancialReportDate> = serde_json::from_value(value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(rows[0].fiscal_year, CalendarYear(2026));
    assert_eq!(rows[0].period, FiscalPeriod::Q2);
    assert_eq!(rows[0].link_json.expose_secret(), JSON_LINK);
    assert_eq!(rows[0].link_xlsx.expose_secret(), XLSX_LINK);
    assert_eq!(rows[0].link_json.to_string(), "[REDACTED URL]");
    assert_eq!(rows[0].link_xlsx.to_string(), "[REDACTED URL]");

    let diagnostic = format!("{:?}", rows[0]);
    assert!(!diagnostic.contains("financialmodelingprep.com"));
    assert!(!diagnostic.contains("financial-reports-json"));
    assert!(!diagnostic.contains("financial-reports-xlsx"));
}

#[test]
fn full_documented_report_preserves_all_70_top_level_keys() {
    let source: serde_json::Value = serde_json::from_slice(REPORT).unwrap();
    assert_eq!(source.as_array().unwrap().len(), 1);
    assert_eq!(source[0].as_object().unwrap().len(), 70);

    let rows: Vec<FinancialReportJson> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(rows[0].period, FiscalPeriod::FullYear);
    assert_eq!(rows[0].year, FiscalYearString::new("2022").unwrap());
    assert_eq!(rows[0].sections.len(), 67);

    assert!(rows[0].sections.contains_key("Cover Page"));
    assert!(rows[0].sections.contains_key("Shareholders' Equity"));
    assert!(
        rows[0]
            .sections
            .contains_key("CONSOLIDATED BALANCE SHEETS (Pa")
    );
    assert!(
        rows[0]
            .sections
            .contains_key("Segment Information and Geogr_6")
    );

    let round_trip = serde_json::to_value(&rows).unwrap();
    assert_eq!(round_trip, source);
}

#[test]
fn documented_dynamic_sections_preserve_null_nbsp_and_heterogeneous_nested_values() {
    let rows: Vec<FinancialReportJson> = serde_json::from_slice(REPORT).unwrap();
    let sections = &rows[0].sections;

    assert_eq!(
        sections["Income Taxes - Additional Infor"][0]["Income Taxes - Additional Information (Details) $ in Millions, € in Billions"]
            [0],
        serde_json::Value::Null
    );
    assert_eq!(
        sections["Cover Page"][2]["Entity Information [Line Items]"][0],
        serde_json::json!("\u{a0}")
    );
    assert_eq!(
        sections["Leases - Lease Liability Maturi"][2]["2023"],
        serde_json::json!([1758, "\u{a0}"])
    );
    assert_eq!(
        sections["CONSOLIDATED BALANCE SHEETS (Pa"][2]["Common stock, par value (in dollars per share)"]
            [0]
            .as_number()
            .unwrap()
            .to_string(),
        "1e-05"
    );
}

#[test]
fn flattened_sections_round_trip_arbitrary_names_large_numbers_and_native_json_kinds() {
    let wire = r#"[
      {
        "symbol":"TEST",
        "period":"Q1",
        "year":"0007",
        "Arbitrary § / Section": {
          "nested": [null, true, " ", -17, 123456789012345678901234567890, 1.25],
          "object": {"unbounded key [Axis]": false}
        }
      }
    ]"#;

    let source: serde_json::Value = serde_json::from_str(wire).unwrap();
    let rows: Vec<FinancialReportJson> = serde_json::from_str(wire).unwrap();

    assert_eq!(rows[0].year.as_str(), "0007");
    assert_eq!(
        rows[0].sections["Arbitrary § / Section"]["nested"][4]
            .as_number()
            .unwrap()
            .to_string(),
        "123456789012345678901234567890"
    );
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn serialization_rejects_every_reserved_header_in_dynamic_sections() {
    let rows: Vec<FinancialReportJson> = serde_json::from_slice(REPORT).unwrap();

    for reserved in ["symbol", "period", "year"] {
        let mut report = rows[0].clone();
        report.sections.insert(
            reserved.to_owned(),
            serde_json::json!("attacker-controlled replacement"),
        );

        let error = serde_json::to_value(report).unwrap_err();
        assert!(
            error.to_string().contains(reserved),
            "collision error did not identify reserved key {reserved}: {error}"
        );
    }
}

#[test]
fn strict_header_wire_kinds_and_requiredness_are_enforced() {
    let mut string_calendar_year: serde_json::Value = serde_json::from_slice(DATES).unwrap();
    string_calendar_year[0]["fiscalYear"] = serde_json::json!("2026");
    assert!(serde_json::from_value::<Vec<FinancialReportDate>>(string_calendar_year).is_err());

    let mut numeric_report_year: serde_json::Value = serde_json::from_slice(REPORT).unwrap();
    numeric_report_year[0]["year"] = serde_json::json!(2022);
    assert!(serde_json::from_value::<Vec<FinancialReportJson>>(numeric_report_year).is_err());

    for key in ["symbol", "fiscalYear", "period", "linkJson", "linkXlsx"] {
        let mut missing: serde_json::Value = serde_json::from_slice(DATES).unwrap();
        missing[0].as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<Vec<FinancialReportDate>>(missing).is_err(),
            "accepted missing required dates field {key}"
        );
    }
    for key in ["symbol", "period", "year"] {
        let mut missing: serde_json::Value = serde_json::from_slice(REPORT).unwrap();
        missing[0].as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<Vec<FinancialReportJson>>(missing).is_err(),
            "accepted missing required report header {key}"
        );
    }
}

#[test]
fn both_json_endpoints_preserve_bare_empty_and_multiple_arrays() {
    assert!(
        serde_json::from_slice::<Vec<FinancialReportDate>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<FinancialReportJson>>(b"[]")
            .unwrap()
            .is_empty()
    );

    let mut multiple_dates: serde_json::Value = serde_json::from_slice(DATES).unwrap();
    let duplicate_date = multiple_dates[0].clone();
    multiple_dates.as_array_mut().unwrap().push(duplicate_date);
    assert_eq!(
        serde_json::from_value::<Vec<FinancialReportDate>>(multiple_dates)
            .unwrap()
            .len(),
        2
    );

    let mut multiple_reports: serde_json::Value = serde_json::from_slice(REPORT).unwrap();
    let duplicate_report = multiple_reports[0].clone();
    multiple_reports
        .as_array_mut()
        .unwrap()
        .push(duplicate_report);
    assert_eq!(
        serde_json::from_value::<Vec<FinancialReportJson>>(multiple_reports)
            .unwrap()
            .len(),
        2
    );

    let wrapped = serde_json::json!({ "financialReports": [] });
    assert!(serde_json::from_value::<Vec<FinancialReportJson>>(wrapped).is_err());
}
