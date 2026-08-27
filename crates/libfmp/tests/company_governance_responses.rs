use libfmp::responses::company::{
    CompanyExecutive, ExecutiveCompensation, ExecutiveCompensationBenchmark,
};
use serde_json::json;

const EXECUTIVES: &[u8] = include_bytes!("fixtures/company_key_executives.json");
const EXECUTIVES_DYNAMIC: &[u8] = include_bytes!("fixtures/company_key_executives_dynamic.json");
const COMPENSATION: &[u8] = include_bytes!("fixtures/company_executive_compensation.json");
const COMPENSATION_LARGE: &[u8] =
    include_bytes!("fixtures/company_executive_compensation_large.json");
const BENCHMARK: &[u8] = include_bytes!("fixtures/company_executive_compensation_benchmark.json");
const EMPTY: &[u8] = include_bytes!("fixtures/company_empty.json");

#[test]
fn documented_company_executive_decodes_exact_fields_and_nulls() {
    let rows: Vec<CompanyExecutive> = serde_json::from_slice(EXECUTIVES).unwrap();
    let row = &rows[0];

    assert_eq!(row.title, "Vice President of Worldwide Communications");
    assert_eq!(row.name, "Kristin Huguet Quayle");
    assert_eq!(row.pay, None);
    assert_eq!(row.currency_pay.as_str(), "USD");
    assert_eq!(row.gender, "female");
    assert_eq!(row.year_born, None);
    assert_eq!(row.title_since, None);
    assert!(row.active);

    let encoded = serde_json::to_value(row).unwrap();
    assert_eq!(
        encoded,
        json!({
            "title": "Vice President of Worldwide Communications",
            "name": "Kristin Huguet Quayle",
            "pay": null,
            "currencyPay": "USD",
            "gender": "female",
            "yearBorn": null,
            "titleSince": null,
            "active": true
        })
    );
}

#[test]
fn company_executive_dynamic_fields_preserve_synthetic_non_null_json() {
    let rows: Vec<CompanyExecutive> = serde_json::from_slice(EXECUTIVES_DYNAMIC).unwrap();
    assert_eq!(rows.len(), 2);

    let row = &rows[1];
    assert_eq!(
        row.pay,
        Some(json!({
            "amount": "00123.450",
            "components": [1, true, null]
        }))
    );
    assert_eq!(row.year_born, Some(json!("01980")));
    assert_eq!(row.title_since, Some(json!(1704067200)));
    assert!(!row.active);
}

#[test]
fn documented_executive_compensation_decodes_dates_integer_year_and_amounts() {
    let rows: Vec<ExecutiveCompensation> = serde_json::from_slice(COMPENSATION).unwrap();
    let row = &rows[0];

    assert_eq!(row.cik.as_str(), "0000320193");
    assert_eq!(row.symbol.as_str(), "AAPL");
    assert_eq!(row.company_name, "Apple Inc.");
    assert_eq!(row.filing_date.to_string(), "2026-01-08");
    assert_eq!(row.accepted_date.to_string(), "2026-01-08 16:31:36");
    assert_eq!(
        row.name_and_position,
        "Luca Maestri Former Senior Vice President, Chief Financial Officer"
    );
    assert_eq!(row.year, 2025);
    assert_eq!(row.salary, 819_231);
    assert_eq!(row.bonus, 0);
    assert_eq!(row.stock_award, 13_003_031);
    assert_eq!(row.option_award, 0);
    assert_eq!(row.incentive_plan_compensation, 1_638_462);
    assert_eq!(row.all_other_compensation, 22_204);
    assert_eq!(row.total, 15_482_928);
    assert_eq!(
        row.link,
        "https://www.sec.gov/Archives/edgar/data/320193/000130817926000008/0001308179-26-000008-index.htm"
    );
}

#[test]
fn executive_compensation_arrays_preserve_large_u64_amounts_and_exact_casing() {
    let rows: Vec<ExecutiveCompensation> = serde_json::from_slice(COMPENSATION_LARGE).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].salary, 5_000_000_000);
    assert_eq!(rows[1].all_other_compensation, 10_000_000_000);
    assert_eq!(rows[1].total, u64::MAX);

    let encoded = serde_json::to_value(&rows[0]).unwrap();
    assert_eq!(encoded["companyName"], "Apple Inc.");
    assert_eq!(encoded["filingDate"], "2026-01-08");
    assert_eq!(encoded["acceptedDate"], "2026-01-08 16:31:36");
    assert_eq!(encoded["nameAndPosition"], rows[0].name_and_position);
    assert_eq!(encoded["stockAward"], 13_003_031);
    assert_eq!(encoded["optionAward"], 0);
    assert_eq!(encoded["incentivePlanCompensation"], 1_638_462);
    assert_eq!(encoded["allOtherCompensation"], 22_204);
}

#[test]
fn documented_benchmark_decodes_exact_fields_with_integer_response_year() {
    let rows: Vec<ExecutiveCompensationBenchmark> = serde_json::from_slice(BENCHMARK).unwrap();
    let row = &rows[0];

    assert_eq!(
        row.industry_title.as_str(),
        "ABRASIVE, ASBESTOS & MISC NONMETALLIC MINERAL PRODS"
    );
    assert_eq!(row.year, 2024);
    assert_eq!(row.average_compensation, 784_407.555_555_555_5);

    let encoded = serde_json::to_value(row).unwrap();
    assert_eq!(encoded["industryTitle"], row.industry_title.as_str());
    assert_eq!(encoded["year"], 2024);
    assert_eq!(encoded["averageCompensation"], 784_407.555_555_555_5);
}

#[test]
fn every_governance_response_contract_accepts_an_empty_array() {
    assert!(
        serde_json::from_slice::<Vec<CompanyExecutive>>(EMPTY)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<ExecutiveCompensation>>(EMPTY)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<ExecutiveCompensationBenchmark>>(EMPTY)
            .unwrap()
            .is_empty()
    );
}
