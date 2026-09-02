mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{
            AccessRequirement, ConditionalPlanRequirement, EndpointBounds, GeographicAvailability,
            PlanCondition,
        },
        tipranks::{
            PointInTimeRatingsByAnalystQuery, PointInTimeRatingsBySymbolQuery,
            tipranks_point_in_time_ratings_by_analyst, tipranks_point_in_time_ratings_by_symbol,
        },
    },
    error::ErrorCategory,
    responses::tipranks::TipRanksPointInTimeRating,
    transport::HttpMethod,
    types::{Date, Limit, Page, SearchTerm, Ticker, TipRanksExpertUid},
};

use support::{FixtureExecutor, json_fixture};

const SYMBOL: &[u8] = include_bytes!("fixtures/tipranks_point_in_time_symbol.json");
const ANALYST: &[u8] = include_bytes!("fixtures/tipranks_point_in_time_analyst.json");

#[test]
fn descriptors_have_exact_paths_response_types_and_shared_metadata_boundaries() {
    let symbol = tipranks_point_in_time_ratings_by_symbol(PointInTimeRatingsBySymbolQuery::new(
        Ticker::new("AAPL").unwrap(),
    ));
    let analyst =
        tipranks_point_in_time_ratings_by_analyst(PointInTimeRatingsByAnalystQuery::new());
    assert_symbol_response_type(&symbol);
    assert_analyst_response_type(&analyst);

    assert_eq!(symbol.method(), HttpMethod::Get);
    assert_eq!(symbol.id(), "tipranks-pit-symbol");
    assert_eq!(symbol.relative_path(), "tipranks-pit-symbol");
    assert_eq!(analyst.method(), HttpMethod::Get);
    assert_eq!(analyst.id(), "tipranks-pit-analyst");
    assert_eq!(analyst.relative_path(), "tipranks-pit-analyst");

    for metadata in [symbol.metadata(), analyst.metadata()] {
        assert_eq!(metadata.geography(), GeographicAvailability::Unspecified);
        assert_eq!(metadata.access(), AccessRequirement::NamedAddOn("TipRanks"));
        assert_eq!(
            metadata.conditional_plan(),
            Some(ConditionalPlanRequirement::new(
                "Enterprise",
                PlanCondition::HistoryOlderThanYears(3)
            ))
        );
        assert_eq!(
            metadata.bounds(),
            EndpointBounds::new()
                .with_limit(5_000)
                .with_response_rows(5_000)
        );
        assert!(metadata.bounds().accepts_limit(Limit(5_000)));
        assert!(!metadata.bounds().accepts_limit(Limit(5_001)));
        assert!(metadata.bounds().accepts_response_rows(5_000));
        assert!(!metadata.bounds().accepts_response_rows(5_001));
        assert_eq!(metadata.realtime(), None);
    }
}

fn assert_symbol_response_type(
    _: &EndpointSpec<PointInTimeRatingsBySymbolQuery, Vec<TipRanksPointInTimeRating>>,
) {
}

fn assert_analyst_response_type(
    _: &EndpointSpec<PointInTimeRatingsByAnalystQuery, Vec<TipRanksPointInTimeRating>>,
) {
}

#[tokio::test]
async fn custom_proxy_symbol_call_sends_one_exact_request_with_auth_and_default_header() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(SYMBOL)]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Bearer ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "tipranks point in time")
        .executor(executor.clone())
        .build()
        .unwrap();
    let query = PointInTimeRatingsBySymbolQuery::new(Ticker::new("BRK.B").unwrap())
        .with_date(Date::parse("2026-06-10").unwrap())
        .with_limit(Limit(5_000))
        .with_page(Page(0))
        .with_nonadjusted(false);

    let rows = client
        .tipranks_point_in_time_ratings_by_symbol(query)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].symbol.as_str(), "AAPL");
    assert_eq!(rows[0].beat_target, Some(false));

    let requests = executor.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method(), HttpMethod::Get);
    assert_eq!(
        requests[0].expose_headers()["x-router-token"],
        "Bearer proxy-secret"
    );
    assert_eq!(
        requests[0].expose_headers()["x-data-scope"],
        "tipranks point in time"
    );
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://proxy.example/router/gateway/stable/tipranks-pit-symbol?symbol=BRK.B&date=2026-06-10&limit=5000&page=0&nonadjusted=false"
    );
}

#[tokio::test]
async fn direct_analyst_query_auth_preserves_both_selectors_and_exact_wire_order() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(ANALYST)]));
    let client = Client::builder()
        .authentication(Authentication::fmp_query("query-secret"))
        .executor(executor.clone())
        .build()
        .unwrap();
    let query = PointInTimeRatingsByAnalystQuery::new()
        .with_expert_uid(TipRanksExpertUid::new("expert / one").unwrap())
        .with_analyst_name(SearchTerm::new("Keegan Cox").unwrap())
        .with_date(Date::parse("2026-06-10").unwrap())
        .with_limit(Limit(100))
        .with_page(Page(0))
        .with_nonadjusted(false);

    let rows = client
        .tipranks_point_in_time_ratings_by_analyst(query)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].symbol.as_str(), "0J3K.L");
    assert_eq!(rows[0].last_recommendation, "Hold");
    assert_eq!(rows[0].price_target, None);

    let requests = executor.requests();
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].expose_headers().contains_key("apikey"));
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://financialmodelingprep.com/stable/tipranks-pit-analyst?expertUID=expert+%2F+one&analystName=Keegan+Cox&date=2026-06-10&limit=100&page=0&nonadjusted=false&apikey=query-secret"
    );
}

#[tokio::test]
async fn omitted_options_inject_no_defaults_and_direct_header_auth_stays_out_of_url() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(ANALYST)]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("header-secret"))
        .executor(executor.clone())
        .build()
        .unwrap();

    let rows = client
        .tipranks_point_in_time_ratings_by_analyst(PointInTimeRatingsByAnalystQuery::new())
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);

    let requests = executor.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].expose_headers()["apikey"], "header-secret");
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://financialmodelingprep.com/stable/tipranks-pit-analyst"
    );
}

#[tokio::test]
async fn malformed_roots_keep_each_endpoint_identity_and_each_call_sends_one_request() {
    let symbol_executor = Arc::new(FixtureExecutor::new([json_fixture(b"{}")]));
    let symbol_client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(symbol_executor.clone())
        .build()
        .unwrap();
    let symbol_error = symbol_client
        .tipranks_point_in_time_ratings_by_symbol(PointInTimeRatingsBySymbolQuery::new(
            Ticker::new("AAPL").unwrap(),
        ))
        .await
        .unwrap_err();
    assert_eq!(symbol_error.category(), ErrorCategory::Decode);
    assert_eq!(symbol_error.endpoint(), Some("tipranks-pit-symbol"));
    assert_eq!(symbol_error.status_code(), Some(200));
    assert_eq!(symbol_executor.requests().len(), 1);

    let analyst_executor = Arc::new(FixtureExecutor::new([json_fixture(b"{}")]));
    let analyst_client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(analyst_executor.clone())
        .build()
        .unwrap();
    let analyst_error = analyst_client
        .tipranks_point_in_time_ratings_by_analyst(PointInTimeRatingsByAnalystQuery::new())
        .await
        .unwrap_err();
    assert_eq!(analyst_error.category(), ErrorCategory::Decode);
    assert_eq!(analyst_error.endpoint(), Some("tipranks-pit-analyst"));
    assert_eq!(analyst_error.status_code(), Some(200));
    assert_eq!(analyst_executor.requests().len(), 1);
}

#[tokio::test]
async fn bare_empty_arrays_decode_for_both_routes() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"[]"),
        json_fixture(b"[]"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();

    assert!(
        client
            .tipranks_point_in_time_ratings_by_symbol(PointInTimeRatingsBySymbolQuery::new(
                Ticker::new("AAPL").unwrap(),
            ))
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .tipranks_point_in_time_ratings_by_analyst(PointInTimeRatingsByAnalystQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(executor.requests().len(), 2);
}
