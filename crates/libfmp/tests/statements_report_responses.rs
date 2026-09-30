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
fn captured_report_is_one_object_that_preserves_every_top_level_key() {
    // Trimmed from a live AAPL 2023 Q1 response captured on 2026-09-30.
    let source: serde_json::Value = serde_json::from_slice(REPORT).unwrap();
    assert_eq!(source.as_object().unwrap().len(), 6);

    let report: FinancialReportJson = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(report.symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(report.period, FiscalPeriod::Q1);
    assert_eq!(report.year, FiscalYearString::new("2023").unwrap());
    assert_eq!(report.sections.len(), 3);

    for name in [
        "CONDENSED CONSOLIDATED BALANC_2",
        "Shareholders' Equity - Addition",
        "Revenue - Additional Informatio",
    ] {
        assert!(report.sections.contains_key(name), "missing section {name}");
    }

    let round_trip = serde_json::to_value(&report).unwrap();
    assert_eq!(round_trip, source);
}

#[test]
fn captured_dynamic_sections_preserve_nbsp_and_scientific_notation() {
    let report: FinancialReportJson = serde_json::from_slice(REPORT).unwrap();
    let sections = &report.sections;

    assert_eq!(
        sections["Shareholders' Equity - Addition"][2]["Share Repurchase Program [Line Items]"][0],
        serde_json::json!("\u{a0}")
    );
    assert_eq!(
        sections["Revenue - Additional Informatio"][2]["Total deferred revenue"],
        serde_json::json!([12.6, 12.4])
    );
    assert_eq!(
        sections["CONDENSED CONSOLIDATED BALANC_2"][2]["Common stock, par value (in dollars per share)"]
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
    let captured: FinancialReportJson = serde_json::from_slice(REPORT).unwrap();

    for reserved in ["symbol", "period", "year"] {
        let mut report = captured.clone();
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
    numeric_report_year["year"] = serde_json::json!(2023);
    assert!(serde_json::from_value::<FinancialReportJson>(numeric_report_year).is_err());

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
        missing.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<FinancialReportJson>(missing).is_err(),
            "accepted missing required report header {key}"
        );
    }
}

#[test]
fn dates_keep_bare_arrays_while_the_report_is_one_object() {
    assert!(
        serde_json::from_slice::<Vec<FinancialReportDate>>(b"[]")
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

    let captured: serde_json::Value = serde_json::from_slice(REPORT).unwrap();
    assert!(serde_json::from_value::<FinancialReportJson>(captured.clone()).is_ok());
    assert!(serde_json::from_slice::<FinancialReportJson>(b"[]").is_err());
    let listed = serde_json::json!([captured]);
    assert!(serde_json::from_value::<FinancialReportJson>(listed).is_err());
    let wrapped = serde_json::json!({ "financialReports": [] });
    assert!(serde_json::from_value::<FinancialReportJson>(wrapped).is_err());
}
