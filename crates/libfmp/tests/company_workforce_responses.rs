use libfmp::responses::company::{DelistedCompany, EmployeeCount};

#[test]
fn documented_delisted_company_decodes_every_exact_field_and_wire_type() {
    let rows: Vec<DelistedCompany> =
        serde_json::from_str(include_str!("fixtures/company_delisted.json")).unwrap();

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.symbol.as_str(), "CCIX");
    assert_eq!(
        row.company_name,
        "Churchill Capital Corp IX Ordinary Shares"
    );
    assert_eq!(row.exchange.as_str(), "NASDAQ");
    assert_eq!(row.ipo_date.to_string(), "2007-03-01");
    assert_eq!(row.delisted_date.to_string(), "2026-07-28");

    let wire = serde_json::to_value(row).unwrap();
    assert_eq!(
        wire["companyName"],
        "Churchill Capital Corp IX Ordinary Shares"
    );
    assert_eq!(wire["ipoDate"], "2007-03-01");
    assert_eq!(wire["delistedDate"], "2026-07-28");
    assert!(wire.get("company_name").is_none());
    assert!(wire.get("ipo_date").is_none());
    assert!(wire.get("delisted_date").is_none());
}

#[test]
fn documented_employee_count_decodes_every_exact_field_and_wire_type() {
    let rows: Vec<EmployeeCount> =
        serde_json::from_str(include_str!("fixtures/company_employee_count.json")).unwrap();

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.symbol.as_str(), "AAPL");
    assert_eq!(row.cik.as_str(), "0000320193");
    assert_eq!(row.acceptance_time.to_string(), "2025-10-31 06:01:26");
    assert_eq!(row.period_of_report.to_string(), "2025-09-27");
    assert_eq!(row.company_name, "Apple Inc.");
    assert_eq!(row.form_type, "10-K");
    assert_eq!(row.filing_date.to_string(), "2025-10-31");
    assert_eq!(row.employee_count, 166_000);
    assert_eq!(
        row.source,
        "https://www.sec.gov/Archives/edgar/data/320193/000032019325000079/0000320193-25-000079-index.htm"
    );

    let wire = serde_json::to_value(row).unwrap();
    assert_eq!(wire["cik"], "0000320193");
    assert!(wire["cik"].is_string());
    assert_eq!(wire["acceptanceTime"], "2025-10-31 06:01:26");
    assert_eq!(wire["periodOfReport"], "2025-09-27");
    assert_eq!(wire["filingDate"], "2025-10-31");
    assert_eq!(wire["employeeCount"], 166_000_u64);
    assert!(wire["employeeCount"].is_u64());
    assert!(wire.get("acceptance_time").is_none());
    assert!(wire.get("period_of_report").is_none());
    assert!(wire.get("employee_count").is_none());
}

#[test]
fn workforce_arrays_preserve_empty_multiple_unknown_fields_and_u64_counts() {
    let empty = "[]";
    assert!(
        serde_json::from_str::<Vec<DelistedCompany>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<EmployeeCount>>(empty)
            .unwrap()
            .is_empty()
    );

    let delisted = include_str!("fixtures/company_delisted.json");
    let delisted_row = delisted
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix("]\n"))
        .unwrap();
    let multiple_delisted = format!("[{delisted_row},{delisted_row}]");
    assert_eq!(
        serde_json::from_str::<Vec<DelistedCompany>>(&multiple_delisted)
            .unwrap()
            .len(),
        2
    );

    let employee_with_unknown_and_large_count = r#"[
      {
        "symbol": "AAPL",
        "cik": "0000320193",
        "acceptanceTime": "2025-10-31 06:01:26",
        "periodOfReport": "2025-09-27",
        "companyName": "Apple Inc.",
        "formType": "10-K",
        "filingDate": "2025-10-31",
        "employeeCount": 18446744073709551615,
        "source": "https://www.sec.gov/example",
        "futureField": { "nested": true }
      },
      {
        "symbol": "AAPL",
        "cik": "0000320193",
        "acceptanceTime": "2025-10-31 06:01:26",
        "periodOfReport": "2025-09-27",
        "companyName": "Apple Inc.",
        "formType": "10-K",
        "filingDate": "2025-10-31",
        "employeeCount": 166000,
        "source": "https://www.sec.gov/example"
      }
    ]"#;
    let employees: Vec<EmployeeCount> =
        serde_json::from_str(employee_with_unknown_and_large_count).unwrap();
    assert_eq!(employees.len(), 2);
    assert_eq!(employees[0].employee_count, u64::MAX);
    assert_eq!(employees[0].cik.as_str(), "0000320193");
}
