mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        chart::{
            StockChartIntradayQuery, stock_chart_fifteen_minutes, stock_chart_five_minutes,
            stock_chart_four_hours, stock_chart_one_hour, stock_chart_one_minute,
            stock_chart_thirty_minutes,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    responses::chart::StockChartIntradayBar,
    transport::HttpMethod,
    types::{Date, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const ONE_MINUTE: &[u8] = include_bytes!("fixtures/stock_chart_one_minute.json");
const FIVE_MINUTES: &[u8] = include_bytes!("fixtures/stock_chart_five_minutes.json");
const FIFTEEN_MINUTES: &[u8] = include_bytes!("fixtures/stock_chart_fifteen_minutes.json");
const THIRTY_MINUTES: &[u8] = include_bytes!("fixtures/stock_chart_thirty_minutes.json");
const ONE_HOUR: &[u8] = include_bytes!("fixtures/stock_chart_one_hour.json");
const FOUR_HOURS: &[u8] = include_bytes!("fixtures/stock_chart_four_hours.json");

const PATHS: [&str; 6] = [
    "historical-chart/1min",
    "historical-chart/5min",
    "historical-chart/15min",
    "historical-chart/30min",
    "historical-chart/1hour",
    "historical-chart/4hour",
];

fn fixtures() -> [libfmp::transport::TransportResponse; 6] {
    [
        json_fixture(ONE_MINUTE),
        json_fixture(FIVE_MINUTES),
        json_fixture(FIFTEEN_MINUTES),
        json_fixture(THIRTY_MINUTES),
        json_fixture(ONE_HOUR),
        json_fixture(FOUR_HOURS),
    ]
}

#[test]
fn descriptors_use_exact_paths_shared_rows_and_only_worldwide_metadata() {
    let query = StockChartIntradayQuery::new(Ticker::new("AAPL").unwrap())
        .with_from(Date::from_str("2024-01-01").unwrap())
        .with_to(Date::from_str("2024-03-01").unwrap())
        .with_nonadjusted(false)
        .with_extended(true);
    let endpoints = [
        stock_chart_one_minute(query.clone()),
        stock_chart_five_minutes(query.clone()),
        stock_chart_fifteen_minutes(query.clone()),
        stock_chart_thirty_minutes(query.clone()),
        stock_chart_one_hour(query.clone()),
        stock_chart_four_hours(query),
    ];

    for (endpoint, path) in endpoints.iter().zip(PATHS) {
        assert_facts(endpoint, path);
        assert_eq!(endpoint.query().symbol().as_str(), "AAPL");
        assert_eq!(endpoint.query().nonadjusted(), Some(false));
        assert_eq!(endpoint.query().extended(), Some(true));
    }
}

fn assert_facts(
    endpoint: &EndpointSpec<StockChartIntradayQuery, Vec<StockChartIntradayBar>>,
    path: &'static str,
) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_routing_preserves_every_path_query_state_auth_and_default_header() {
    let executor = Arc::new(FixtureExecutor::new(fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "chart-intraday")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();
    let from = Date::from_str("2024-01-01").unwrap();
    let to = Date::from_str("2024-03-01").unwrap();

    let one_minute = client.stock_chart_one_minute(&symbol).await.unwrap();
    let five_minutes = client
        .stock_chart_five_minutes(
            StockChartIntradayQuery::new(symbol.clone())
                .with_from(from)
                .with_nonadjusted(false),
        )
        .await
        .unwrap();
    let fifteen_minutes = client
        .stock_chart_fifteen_minutes(
            StockChartIntradayQuery::new(symbol.clone())
                .with_to(to)
                .with_extended(false),
        )
        .await
        .unwrap();
    let thirty_minutes = client
        .stock_chart_thirty_minutes(
            StockChartIntradayQuery::new(symbol.clone())
                .with_from(from)
                .with_to(to)
                .with_nonadjusted(false)
                .with_extended(false),
        )
        .await
        .unwrap();
    let one_hour = client
        .stock_chart_one_hour(
            StockChartIntradayQuery::new(symbol.clone())
                .with_from(from)
                .with_to(to)
                .with_nonadjusted(true)
                .with_extended(true),
        )
        .await
        .unwrap();
    let four_hours = client
        .stock_chart_four_hours(
            StockChartIntradayQuery::new(symbol)
                .with_from(from)
                .with_to(to)
                .with_nonadjusted(false)
                .with_extended(true),
        )
        .await
        .unwrap();

    assert_eq!(one_minute[0].date.to_string(), "2026-07-30 13:16:00");
    assert_eq!(five_minutes[0].volume, 123_020.0);
    assert_eq!(fifteen_minutes[0].open, 332.655);
    assert_eq!(thirty_minutes[0].volume, 980_442.0);
    assert_eq!(one_hour[0].volume, 3_285_503.0);
    assert_eq!(four_hours[0].volume, 28_439_347.0);

    let requests = executor.requests();
    assert_eq!(requests.len(), 6);
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "chart-intraday"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/historical-chart/1min?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/historical-chart/5min?symbol=BRK.B+%2F+Class+A&from=2024-01-01&nonadjusted=false",
            "https://proxy.example/router/gateway/stable/historical-chart/15min?symbol=BRK.B+%2F+Class+A&to=2024-03-01&extended=false",
            "https://proxy.example/router/gateway/stable/historical-chart/30min?symbol=BRK.B+%2F+Class+A&from=2024-01-01&to=2024-03-01&nonadjusted=false&extended=false",
            "https://proxy.example/router/gateway/stable/historical-chart/1hour?symbol=BRK.B+%2F+Class+A&from=2024-01-01&to=2024-03-01&nonadjusted=true&extended=true",
            "https://proxy.example/router/gateway/stable/historical-chart/4hour?symbol=BRK.B+%2F+Class+A&from=2024-01-01&to=2024-03-01&nonadjusted=false&extended=true",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_apply_to_all_six_intraday_contracts() {
    for (authentication, query_suffix, expected_header) in [
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
        let executor = Arc::new(FixtureExecutor::new(fixtures()));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbol = Ticker::new("AAPL").unwrap();

        client.stock_chart_one_minute(symbol.clone()).await.unwrap();
        client
            .stock_chart_five_minutes(symbol.clone())
            .await
            .unwrap();
        client
            .stock_chart_fifteen_minutes(symbol.clone())
            .await
            .unwrap();
        client
            .stock_chart_thirty_minutes(symbol.clone())
            .await
            .unwrap();
        client.stock_chart_one_hour(symbol.clone()).await.unwrap();
        client.stock_chart_four_hours(symbol).await.unwrap();

        let requests = executor.requests();
        assert_eq!(requests.len(), 6);
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            PATHS
                .iter()
                .map(|path| format!(
                    "https://financialmodelingprep.com/stable/{path}?symbol=AAPL{query_suffix}"
                ))
                .collect::<Vec<_>>()
        );
        for request in requests.iter() {
            match expected_header {
                Some(value) => assert_eq!(request.expose_headers()["apikey"], value),
                None => assert!(!request.expose_headers().contains_key("apikey")),
            }
        }
    }
}
