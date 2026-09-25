#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::str::FromStr;

use libfmp::{
    responses::analyst::{
        FinancialEstimate, HistoricalRating, HistoricalStockGrade, PriceTargetConsensus,
        PriceTargetSummary, RatingSnapshot, StockGrade, StockGradesSummary,
    },
    types::{Date, Ticker},
};
use serde::{Serialize, de::DeserializeOwned};

const ESTIMATES: &[u8] = include_bytes!("fixtures/financial_estimates.json");
const SNAPSHOT: &[u8] = include_bytes!("fixtures/ratings_snapshot.json");
const HISTORICAL_RATINGS: &[u8] = include_bytes!("fixtures/historical_ratings.json");
const TARGET_SUMMARY: &[u8] = include_bytes!("fixtures/price_target_summary.json");
const TARGET_CONSENSUS: &[u8] = include_bytes!("fixtures/price_target_consensus.json");
const GRADES: &[u8] = include_bytes!("fixtures/stock_grades.json");
const HISTORICAL_GRADES: &[u8] = include_bytes!("fixtures/historical_stock_grades.json");
const GRADES_SUMMARY: &[u8] = include_bytes!("fixtures/stock_grades_summary.json");

const PUBLISHERS: &str = "[\"StreetInsider\",\"TheFly\",\"Benzinga\",\"Pulse 2.0\",\"TipRanks Contributor\",\"MarketWatch\",\"Investing\",\"Barrons\",\"Investor's Business Daily\"]";

#[test]
fn exact_financial_estimate_decodes_all_22_fields() {
    assert_field_count(ESTIMATES, 22);
    let source: serde_json::Value = serde_json::from_slice(ESTIMATES).unwrap();
    let rows: Vec<FinancialEstimate> = serde_json::from_value(source.clone()).unwrap();
    assert_rows!(
        rows,
        [FinancialEstimate {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2030-09-27").unwrap(),
            revenue_low: 648_228_509_004.0,
            revenue_high: 735_022_980_353.0,
            revenue_avg: 679_000_000_000.0,
            ebitda_low: 233_968_328_102.0,
            ebitda_high: 265_295_486_763.0,
            ebitda_avg: 245_074_834_838.0,
            ebit_low: 217_109_092_822.0,
            ebit_high: 246_178_886_382.0,
            ebit_avg: 227_415_289_483.0,
            net_income_low: 191_547_261_069.0,
            net_income_high: 225_370_398_908.0,
            net_income_avg: 203_538_714_818.0,
            sga_expense_low: 41_721_580_524.0,
            sga_expense_high: 47_307_886_087.0,
            sga_expense_avg: 43_702_109_337.0,
            eps_avg: 13.565,
            eps_high: 15.01999,
            eps_low: 12.76582,
            num_analysts_revenue: 16,
            num_analysts_eps: 7,
        }]
    );
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn exact_rating_fixtures_keep_snapshot_and_historical_rows_distinct() {
    assert_field_count(SNAPSHOT, 9);
    assert_field_count(HISTORICAL_RATINGS, 10);
    let snapshot_source: serde_json::Value = serde_json::from_slice(SNAPSHOT).unwrap();
    let snapshots: Vec<RatingSnapshot> = serde_json::from_value(snapshot_source.clone()).unwrap();
    assert_eq!(snapshots[0].rating, "B");
    assert_eq!(snapshots[0].overall_score, 3);
    assert_eq!(snapshots[0].return_on_equity_score, 5);
    assert_eq!(serde_json::to_value(snapshots).unwrap(), snapshot_source);

    let historical_source: serde_json::Value = serde_json::from_slice(HISTORICAL_RATINGS).unwrap();
    let historical: Vec<HistoricalRating> =
        serde_json::from_value(historical_source.clone()).unwrap();
    assert_eq!(historical[0].date, Date::from_str("2026-07-30").unwrap());
    assert_eq!(historical[0].price_to_book_score, 1);
    assert_eq!(serde_json::to_value(historical).unwrap(), historical_source);
}

#[test]
fn exact_price_target_fixtures_preserve_raw_publishers_and_numeric_prices() {
    assert_field_count(TARGET_SUMMARY, 10);
    let summary_source: serde_json::Value = serde_json::from_slice(TARGET_SUMMARY).unwrap();
    let summaries: Vec<PriceTargetSummary> =
        serde_json::from_value(summary_source.clone()).unwrap();
    assert_eq!(summaries[0].publishers, PUBLISHERS);
    assert_eq!(summaries[0].last_month_avg_price_target, 333.75);
    assert_eq!(summaries[0].all_time_count, 254);
    assert_eq!(serde_json::to_value(summaries).unwrap(), summary_source);

    assert_field_count(TARGET_CONSENSUS, 5);
    let consensus_source: serde_json::Value = serde_json::from_slice(TARGET_CONSENSUS).unwrap();
    let consensus: Vec<PriceTargetConsensus> =
        serde_json::from_value(consensus_source.clone()).unwrap();
    assert_eq!(consensus[0].target_high, 400.0);
    assert_eq!(consensus[0].target_consensus, 337.67);
    assert_eq!(consensus[0].target_median, 340.0);
    let encoded = serde_json::to_value(consensus).unwrap();
    assert_eq!(encoded[0]["targetHigh"].as_f64(), Some(400.0));
    assert_eq!(encoded[0]["targetConsensus"].as_f64(), Some(337.67));
}

#[test]
fn exact_grade_fixtures_preserve_both_incompatible_bucket_key_families() {
    assert_field_count(GRADES, 6);
    let grade_source: serde_json::Value = serde_json::from_slice(GRADES).unwrap();
    let grades: Vec<StockGrade> = serde_json::from_value(grade_source.clone()).unwrap();
    assert_eq!(grades[0].grading_company, "Morgan Stanley");
    assert_eq!(grades[0].previous_grade, "Overweight");
    assert_eq!(grades[0].action, "maintain");
    assert_eq!(serde_json::to_value(grades).unwrap(), grade_source);

    assert_field_count(HISTORICAL_GRADES, 7);
    let historical_source: serde_json::Value = serde_json::from_slice(HISTORICAL_GRADES).unwrap();
    let historical: Vec<HistoricalStockGrade> =
        serde_json::from_value(historical_source.clone()).unwrap();
    assert_eq!(historical[0].analyst_ratings_strong_buy, 6);
    assert_eq!(historical[0].analyst_ratings_strong_sell, 2);
    assert_eq!(serde_json::to_value(historical).unwrap(), historical_source);

    assert_field_count(GRADES_SUMMARY, 7);
    let summary_source: serde_json::Value = serde_json::from_slice(GRADES_SUMMARY).unwrap();
    let summary: Vec<StockGradesSummary> = serde_json::from_value(summary_source.clone()).unwrap();
    assert_eq!(summary[0].strong_buy, 1);
    assert_eq!(summary[0].strong_sell, 0);
    assert_eq!(summary[0].consensus, "Buy");
    assert_eq!(serde_json::to_value(summary).unwrap(), summary_source);
}

#[test]
fn estimate_amounts_are_signed_and_counts_preserve_the_full_u64_domain() {
    let amount_fields = [
        "revenueLow",
        "revenueHigh",
        "revenueAvg",
        "ebitdaLow",
        "ebitdaHigh",
        "ebitdaAvg",
        "ebitLow",
        "ebitHigh",
        "ebitAvg",
        "netIncomeLow",
        "netIncomeHigh",
        "netIncomeAvg",
        "sgaExpenseLow",
        "sgaExpenseHigh",
        "sgaExpenseAvg",
    ];
    for (index, field) in amount_fields.into_iter().enumerate() {
        let mut source: serde_json::Value = serde_json::from_slice(ESTIMATES).unwrap();
        source[0][field] = if index % 2 == 0 {
            serde_json::json!(i64::MIN)
        } else {
            serde_json::json!(9_000_000_000_000_000_000_i64)
        };
        let rows: Vec<FinancialEstimate> = serde_json::from_value(source.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(rows).unwrap()[0][field],
            source[0][field]
        );
    }

    let mut source: serde_json::Value = serde_json::from_slice(ESTIMATES).unwrap();
    source[0]["revenueHigh"] = serde_json::json!(9_007_199_254_740_993_i64);
    let rows: Vec<FinancialEstimate> = serde_json::from_value(source).unwrap();
    assert_eq!(
        serde_json::to_value(rows).unwrap()[0]["revenueHigh"],
        9_007_199_254_740_992_u64
    );

    for field in ["numAnalystsRevenue", "numAnalystsEps"] {
        let mut source: serde_json::Value = serde_json::from_slice(ESTIMATES).unwrap();
        source[0][field] = serde_json::json!(u64::MAX);
        let rows: Vec<FinancialEstimate> = serde_json::from_value(source).unwrap();
        assert_eq!(serde_json::to_value(rows).unwrap()[0][field], u64::MAX);
    }

    let mut snapshot: serde_json::Value = serde_json::from_slice(SNAPSHOT).unwrap();
    snapshot[0]["overallScore"] = serde_json::json!(u64::MAX);
    let rows: Vec<RatingSnapshot> = serde_json::from_value(snapshot).unwrap();
    assert_eq!(rows[0].overall_score, u64::MAX);

    let mut historical: serde_json::Value = serde_json::from_slice(HISTORICAL_GRADES).unwrap();
    historical[0]["analystRatingsStrongBuy"] = serde_json::json!(u64::MAX);
    let rows: Vec<HistoricalStockGrade> = serde_json::from_value(historical).unwrap();
    assert_eq!(rows[0].analyst_ratings_strong_buy, u64::MAX);

    let mut summary: serde_json::Value = serde_json::from_slice(GRADES_SUMMARY).unwrap();
    summary[0]["strongBuy"] = serde_json::json!(u64::MAX);
    let rows: Vec<StockGradesSummary> = serde_json::from_value(summary).unwrap();
    assert_eq!(rows[0].strong_buy, u64::MAX);
}

#[test]
fn all_fields_are_required_non_null_and_unknowns_are_accepted() {
    assert_contract::<FinancialEstimate>(ESTIMATES);
    assert_contract::<RatingSnapshot>(SNAPSHOT);
    assert_contract::<HistoricalRating>(HISTORICAL_RATINGS);
    assert_contract::<PriceTargetSummary>(TARGET_SUMMARY);
    assert_contract::<PriceTargetConsensus>(TARGET_CONSENSUS);
    assert_contract::<StockGrade>(GRADES);
    assert_contract::<HistoricalStockGrade>(HISTORICAL_GRADES);
    assert_contract::<StockGradesSummary>(GRADES_SUMMARY);
}

#[test]
fn all_eight_contracts_are_bare_arrays_preserving_empty_and_multiple_rows() {
    assert_bare_array::<FinancialEstimate>(ESTIMATES);
    assert_bare_array::<RatingSnapshot>(SNAPSHOT);
    assert_bare_array::<HistoricalRating>(HISTORICAL_RATINGS);
    assert_bare_array::<PriceTargetSummary>(TARGET_SUMMARY);
    assert_bare_array::<PriceTargetConsensus>(TARGET_CONSENSUS);
    assert_bare_array::<StockGrade>(GRADES);
    assert_bare_array::<HistoricalStockGrade>(HISTORICAL_GRADES);
    assert_bare_array::<StockGradesSummary>(GRADES_SUMMARY);
    assert!(
        serde_json::from_value::<Vec<FinancialEstimate>>(serde_json::json!({ "estimates": [] }))
            .is_err()
    );
}

fn assert_contract<T: DeserializeOwned>(fixture: &[u8]) {
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
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
            "accepted missing {key}"
        );
        let mut null = source.clone();
        null[0][&key] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<Vec<T>>(null).is_err(),
            "accepted null {key}"
        );
    }
    let mut forward = source;
    forward[0]["futureProviderField"] = serde_json::json!({ "nested": [1, true, null] });
    assert_eq!(serde_json::from_value::<Vec<T>>(forward).unwrap().len(), 1);
}

fn assert_bare_array<T: DeserializeOwned + Serialize>(fixture: &[u8]) {
    assert!(serde_json::from_slice::<Vec<T>>(b"[]").unwrap().is_empty());
    let mut source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let duplicate = source[0].clone();
    source.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(source).unwrap().len(), 2);
}

fn assert_field_count(fixture: &[u8], expected: usize) {
    let value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), expected);
}
