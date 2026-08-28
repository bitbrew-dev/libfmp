mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        chart::{StockChartEodQuery, stock_chart_full, stock_chart_light},
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    transport::HttpMethod,
    types::{Date, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const LIGHT: &[u8] = include_bytes!("fixtures/stock_chart_light.json");
const FULL: &[u8] = include_bytes!("fixtures/stock_chart_full.json");

#[test]
fn descriptors_use_exact_paths_vec_rows_and_only_documented_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let query = StockChartEodQuery::new(symbol.clone())
        .with_from(Date::from_str("2026-04-30").unwrap())
        .with_to(Date::from_str("2026-07-30").unwrap());

    assert_eq!(query.symbol(), &symbol);
    assert_eq!(query.from().unwrap().to_string(), "2026-04-30");
    assert_eq!(query.to().unwrap().to_string(), "2026-07-30");
    assert_facts(
        &stock_chart_light(query.clone()),
        "historical-price-eod/light",
    );
    assert_facts(&stock_chart_full(query), "historical-price-eod/full");
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new().with_response_rows(5_000)
    );
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn custom_proxy_preserves_exact_query_order_auth_headers_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(LIGHT),
        json_fixture(FULL),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "chart-eod")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();
    let from = Date::from_str("2026-04-30").unwrap();
    let to = Date::from_str("2026-07-30").unwrap();

    let light = client
        .stock_chart_light(
            StockChartEodQuery::new(symbol.clone())
                .with_from(from)
                .with_to(to),
        )
        .await
        .unwrap();
    let full = client
        .stock_chart_full(StockChartEodQuery::new(symbol).with_from(from).with_to(to))
        .await
        .unwrap();

    assert_eq!(light.len(), 1);
    assert_eq!(light[0].price, 332.39);
    assert_eq!(full.len(), 1);
    assert_eq!(full[0].change_percent, -0.2221355);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "chart-eod"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/historical-price-eod/light?symbol=BRK.B+%2F+Class+A&from=2026-04-30&to=2026-07-30",
            "https://proxy.example/router/stable/historical-price-eod/full?symbol=BRK.B+%2F+Class+A&from=2026-04-30&to=2026-07-30",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_apply_to_both_eod_contracts() {
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
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(LIGHT),
            json_fixture(FULL),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbol = Ticker::new("AAPL").unwrap();

        client
            .stock_chart_light(
                StockChartEodQuery::new(symbol.clone())
                    .with_from(Date::from_str("2026-04-30").unwrap()),
            )
            .await
            .unwrap();
        client
            .stock_chart_full(
                StockChartEodQuery::new(symbol).with_to(Date::from_str("2026-07-30").unwrap()),
            )
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!(
                    "https://financialmodelingprep.com/stable/historical-price-eod/light?symbol=AAPL&from=2026-04-30{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/historical-price-eod/full?symbol=AAPL&to=2026-07-30{query_suffix}"
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
