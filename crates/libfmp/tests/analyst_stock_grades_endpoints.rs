mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        analyst::{
            HistoricalStockGradesQuery, StockGradesQuery, StockGradesSummaryQuery,
            historical_stock_grades, stock_grades, stock_grades_summary,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::{DecodeErrorKind, ErrorCategory},
    responses::analyst::{HistoricalStockGrade, StockGrade, StockGradesSummary},
    transport::HttpMethod,
    types::{Limit, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const GRADES: &[u8] = include_bytes!("fixtures/stock_grades.json");
const HISTORICAL: &[u8] = include_bytes!("fixtures/historical_stock_grades.json");
const SUMMARY: &[u8] = include_bytes!("fixtures/stock_grades_summary.json");

#[test]
fn descriptors_use_exact_paths_bare_rows_and_only_documented_metadata() {
    let grades = stock_grades(StockGradesQuery::new(Ticker::new("AAPL").unwrap()));
    assert_facts(&grades, "grades", EndpointBounds::new());

    let historical = historical_stock_grades(HistoricalStockGradesQuery::new(
        Ticker::new("AAPL").unwrap(),
    ));
    assert_facts(
        &historical,
        "grades-historical",
        EndpointBounds::new().with_response_rows(1_000),
    );

    let summary = stock_grades_summary(StockGradesSummaryQuery::new(Ticker::new("AAPL").unwrap()));
    assert_facts(&summary, "grades-consensus", EndpointBounds::new());

    for bounds in [
        grades.metadata().bounds(),
        historical.metadata().bounds(),
        summary.metadata().bounds(),
    ] {
        assert_eq!(bounds.limit(), None);
        assert_eq!(bounds.page(), None);
        assert!(bounds.accepts_limit(Limit(u32::MAX)));
    }
    assert_response_types(&grades, &historical, &summary);
}

fn assert_facts<Q, R>(
    endpoint: &EndpointSpec<Q, Vec<R>>,
    path: &'static str,
    bounds: EndpointBounds,
) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), bounds);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_response_types(
    _: &EndpointSpec<StockGradesQuery, Vec<StockGrade>>,
    _: &EndpointSpec<HistoricalStockGradesQuery, Vec<HistoricalStockGrade>>,
    _: &EndpointSpec<StockGradesSummaryQuery, Vec<StockGradesSummary>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_queries_headers_bucket_keys_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(GRADES),
        json_fixture(HISTORICAL),
        json_fixture(SUMMARY),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "stock-grades")
        .executor(executor.clone())
        .build()
        .unwrap();

    let grades = client
        .stock_grades(StockGradesQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
        ))
        .await
        .unwrap();
    let historical = client
        .historical_stock_grades(
            HistoricalStockGradesQuery::new(Ticker::new("AAPL").unwrap()).with_limit(Limit(0)),
        )
        .await
        .unwrap();
    let summary = client
        .stock_grades_summary(StockGradesSummaryQuery::new(Ticker::new("AAPL").unwrap()))
        .await
        .unwrap();

    assert_eq!(grades.len(), 1);
    assert_eq!(grades[0].grading_company, "Morgan Stanley");
    assert_eq!(grades[0].previous_grade, "Overweight");
    assert_eq!(grades[0].new_grade, "Overweight");
    assert_eq!(grades[0].action, "maintain");
    assert_eq!(historical.len(), 1);
    assert_eq!(historical[0].analyst_ratings_strong_buy, 6);
    assert_eq!(historical[0].analyst_ratings_strong_sell, 2);
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].strong_buy, 1);
    assert_eq!(summary[0].strong_sell, 0);
    assert_eq!(summary[0].consensus, "Buy");

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "stock-grades"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/grades?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/grades-historical?symbol=AAPL&limit=0",
            "https://proxy.example/router/gateway/stable/grades-consensus?symbol=AAPL",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_omission_and_full_limit_domain() {
    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(GRADES),
            json_fixture(HISTORICAL),
            json_fixture(HISTORICAL),
            json_fixture(SUMMARY),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .stock_grades(StockGradesQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap();
        client
            .historical_stock_grades(HistoricalStockGradesQuery::new(
                Ticker::new("AAPL").unwrap(),
            ))
            .await
            .unwrap();
        client
            .historical_stock_grades(
                HistoricalStockGradesQuery::new(Ticker::new("AAPL").unwrap())
                    .with_limit(Limit(u32::MAX)),
            )
            .await
            .unwrap();
        client
            .stock_grades_summary(StockGradesSummaryQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!("https://financialmodelingprep.com/stable/grades?symbol=AAPL{suffix}"),
                format!(
                    "https://financialmodelingprep.com/stable/grades-historical?symbol=AAPL{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/grades-historical?symbol=AAPL&limit=4294967295{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/grades-consensus?symbol=AAPL{suffix}"
                ),
            ]
        );
        for request in requests.iter() {
            match expected_header {
                Some(value) => assert_eq!(request.expose_headers()["apikey"], value),
                None => assert!(!request.expose_headers().contains_key("apikey")),
            }
        }
    }
}

#[tokio::test]
async fn malformed_non_array_responses_keep_each_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let grades_error = client
        .stock_grades(StockGradesQuery::new(Ticker::new("AAPL").unwrap()))
        .await
        .unwrap_err();
    assert_decode_error(&grades_error, "grades");

    let historical_error = client
        .historical_stock_grades(HistoricalStockGradesQuery::new(
            Ticker::new("AAPL").unwrap(),
        ))
        .await
        .unwrap_err();
    assert_decode_error(&historical_error, "grades-historical");

    let summary_error = client
        .stock_grades_summary(StockGradesSummaryQuery::new(Ticker::new("AAPL").unwrap()))
        .await
        .unwrap_err();
    assert_decode_error(&summary_error, "grades-consensus");
}

#[tokio::test]
async fn count_members_accept_integral_floats_and_reject_fractions() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(
            br#"[{"symbol":"AAPL","strongBuy":1,"buy":3.0,"hold":32,"sell":8,"strongSell":0,"consensus":"Buy"}]"#,
        ),
        json_fixture(
            br#"[{"symbol":"AAPL","strongBuy":1,"buy":2.9,"hold":32,"sell":8,"strongSell":0,"consensus":"Buy"}]"#,
        ),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();
    let query = || StockGradesSummaryQuery::new(Ticker::new("AAPL").unwrap());

    let rows = client.stock_grades_summary(query()).await.unwrap();
    assert_eq!(rows[0].buy, 3);
    assert_eq!(
        serde_json::to_value(&rows).unwrap()[0]["buy"],
        serde_json::json!(3)
    );

    let error = client.stock_grades_summary(query()).await.unwrap_err();
    assert_decode_error(&error, "grades-consensus");
    assert_eq!(error.decode_path(), Some("[0].buy"));
    assert_eq!(error.decode_kind(), Some(DecodeErrorKind::InvalidValue));
    assert!(!error.message().contains("2.9"));
}

fn assert_decode_error(error: &libfmp::Error, endpoint: &'static str) {
    assert_eq!(error.category(), ErrorCategory::Decode);
    assert_eq!(error.endpoint(), Some(endpoint));
    assert_eq!(error.status_code(), Some(200));
}
