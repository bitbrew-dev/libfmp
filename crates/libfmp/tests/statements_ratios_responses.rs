use libfmp::responses::statements::{FinancialRatios, FinancialRatiosTtm};

const RATIOS: &[u8] = include_bytes!("fixtures/financial_ratios.json");
const RATIOS_TTM: &[u8] = include_bytes!("fixtures/financial_ratios_ttm.json");

#[test]
fn regular_fixture_has_exactly_66_required_fields_and_wire_names() {
    let source: serde_json::Value = serde_json::from_slice(RATIOS).unwrap();
    let rows: Vec<FinancialRatios> = serde_json::from_slice(RATIOS).unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(source[0].as_object().unwrap().len(), 66);
    assert_exact_wire_keys(&rows, &source);
    assert_eq!(rows[0].symbol.as_str(), "AAPL");
    assert_eq!(rows[0].fiscal_year.as_str(), "2025");
    assert_eq!(rows[0].working_capital_turnover_ratio, -20.261496141580857);
    assert_eq!(rows[0].interest_coverage_ratio, 0.0);
    assert_eq!(rows[0].dividend_yield_percentage, 0.4038238951672435);

    assert_every_field_is_required::<FinancialRatios>(&source);
}

#[test]
fn ttm_fixture_has_exactly_62_required_fields_and_preserves_acronym_keys() {
    let source: serde_json::Value = serde_json::from_slice(RATIOS_TTM).unwrap();
    let rows: Vec<FinancialRatiosTtm> = serde_json::from_slice(RATIOS_TTM).unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(source[0].as_object().unwrap().len(), 62);
    assert_exact_wire_keys(&rows, &source);
    assert_eq!(rows[0].enterprise_value_ttm, 4_922_455_686_740);
    assert_eq!(rows[0].interest_coverage_ratio_ttm, 0.0);
    assert_eq!(rows[0].dividend_yield_ttm, 0.0);
    assert_eq!(rows[0].dividend_per_share_ttm, 0.0);
    assert_eq!(rows[0].net_income_per_ebt_ttm, 0.8300602695198754);

    let object = source[0].as_object().unwrap();
    assert!(object.contains_key("netIncomePerEBTTTM"));
    assert!(!object.contains_key("netIncomePerEbtTtm"));
    assert_every_field_is_required::<FinancialRatiosTtm>(&source);
}

#[test]
fn regular_and_ttm_shapes_are_deliberately_distinct_bare_arrays() {
    assert!(serde_json::from_slice::<Vec<FinancialRatiosTtm>>(RATIOS).is_err());
    assert!(serde_json::from_slice::<Vec<FinancialRatios>>(RATIOS_TTM).is_err());

    let regular: serde_json::Value = serde_json::from_slice(RATIOS).unwrap();
    let ttm: serde_json::Value = serde_json::from_slice(RATIOS_TTM).unwrap();
    let regular = regular[0].as_object().unwrap();
    let ttm = ttm[0].as_object().unwrap();

    assert!(regular.contains_key("dividendYieldPercentage"));
    assert!(!regular.contains_key("enterpriseValueTTM"));
    assert!(ttm.contains_key("enterpriseValueTTM"));
    assert!(!ttm.contains_key("dividendYieldPercentage"));

    let wrapped = serde_json::json!({ "ratios": [regular] });
    assert!(serde_json::from_value::<Vec<FinancialRatios>>(wrapped).is_err());
}

fn assert_every_field_is_required<T>(source: &serde_json::Value)
where
    T: serde::de::DeserializeOwned,
{
    let keys = source[0]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    for key in keys {
        let mut missing = source.clone();
        missing[0].as_object_mut().unwrap().remove(&key);
        assert!(
            serde_json::from_value::<Vec<T>>(missing).is_err(),
            "field {key} unexpectedly became optional"
        );
    }
}

fn assert_exact_wire_keys<T>(rows: &[T], source: &serde_json::Value)
where
    T: serde::Serialize,
{
    let encoded = serde_json::to_value(rows).unwrap();
    let encoded_keys = encoded[0]
        .as_object()
        .unwrap()
        .keys()
        .collect::<std::collections::BTreeSet<_>>();
    let source_keys = source[0]
        .as_object()
        .unwrap()
        .keys()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(encoded_keys, source_keys);
}
