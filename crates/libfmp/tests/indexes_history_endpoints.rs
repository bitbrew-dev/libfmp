mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        indexes::{
            IndexChartQuery, index_chart_five_minutes, index_chart_full, index_chart_light,
            index_chart_one_hour, index_chart_one_minute,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    responses::chart::StockChartIntradayBar,
    transport::HttpMethod,
    types::{Date, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const LIGHT: &[u8] = include_bytes!("fixtures/indexes_chart_light.json");
const FULL: &[u8] = include_bytes!("fixtures/indexes_chart_full.json");
const ONE_MINUTE: &[u8] = include_bytes!("fixtures/indexes_chart_one_minute.json");
const FIVE_MINUTES: &[u8] = include_bytes!("fixtures/indexes_chart_five_minutes.json");
const ONE_HOUR: &[u8] = include_bytes!("fixtures/indexes_chart_one_hour.json");

fn fixtures() -> [libfmp::transport::TransportResponse; 5] {
    [
        json_fixture(LIGHT),
        json_fixture(FULL),
        json_fixture(ONE_MINUTE),
        json_fixture(FIVE_MINUTES),
        json_fixture(ONE_HOUR),
    ]
}

#[test]
fn descriptors_use_exact_paths_shared_rows_and_only_documented_bounds() {
    let query = IndexChartQuery::new(Ticker::new("^VIX").unwrap())
        .with_from(Date::from_str("2024-01-01").unwrap())
        .with_to(Date::from_str("2024-03-01").unwrap());

    assert_eq!(query.symbol().as_str(), "^VIX");
    assert_eq!(query.from().unwrap().to_string(), "2024-01-01");
    assert_eq!(query.to().unwrap().to_string(), "2024-03-01");
    assert_eod_facts(
        &index_chart_light(query.clone()),
        "historical-price-eod/light",
    );
    assert_eod_facts(
        &index_chart_full(query.clone()),
        "historical-price-eod/full",
    );
    assert_intraday_facts(
        &index_chart_one_minute(query.clone()),
        "historical-chart/1min",
    );
    assert_intraday_facts(
        &index_chart_five_minutes(query.clone()),
        "historical-chart/5min",
    );
    assert_intraday_facts(&index_chart_one_hour(query), "historical-chart/1hour");
}

fn assert_eod_facts<R>(endpoint: &EndpointSpec<IndexChartQuery, Vec<R>>, path: &'static str) {
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_common_facts(endpoint);
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new().with_response_rows(5_000)
    );
}

fn assert_intraday_facts(
    endpoint: &EndpointSpec<IndexChartQuery, Vec<StockChartIntradayBar>>,
    path: &'static str,
) {
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_common_facts(endpoint);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
}

fn assert_common_facts<Q, R>(endpoint: &EndpointSpec<Q, R>) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), endpoint.relative_path());
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_routing_preserves_paths_caret_query_order_auth_headers_and_source_rows() {
    let executor = Arc::new(FixtureExecutor::new(fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "indexes-history")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("^VIX").unwrap();
    let eod = IndexChartQuery::new(symbol.clone())
        .with_from(Date::from_str("2026-01-27").unwrap())
        .with_to(Date::from_str("2026-04-27").unwrap());
    let intraday = IndexChartQuery::new(symbol)
        .with_from(Date::from_str("2024-01-01").unwrap())
        .with_to(Date::from_str("2024-03-01").unwrap());

    let light = client.index_chart_light(eod.clone()).await.unwrap();
    let full = client.index_chart_full(eod).await.unwrap();
    let one_minute = client
        .index_chart_one_minute(intraday.clone())
        .await
        .unwrap();
    let five_minutes = client
        .index_chart_five_minutes(intraday.clone())
        .await
        .unwrap();
    let one_hour = client.index_chart_one_hour(intraday).await.unwrap();

    assert_eq!(light[0].symbol.as_str(), "^VIX");
    assert_eq!(full[0].change, -1.66);
    assert_eq!(one_minute[0].open, 17.92);
    assert_eq!(five_minutes[0].open, 17.99);
    assert_eq!(one_hour[0].open, 18.23);

    let requests = executor.requests();
    assert_eq!(requests.len(), 5);
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "indexes-history"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/historical-price-eod/light?symbol=%5EVIX&from=2026-01-27&to=2026-04-27",
            "https://proxy.example/router/gateway/stable/historical-price-eod/full?symbol=%5EVIX&from=2026-01-27&to=2026-04-27",
            "https://proxy.example/router/gateway/stable/historical-chart/1min?symbol=%5EVIX&from=2024-01-01&to=2024-03-01",
            "https://proxy.example/router/gateway/stable/historical-chart/5min?symbol=%5EVIX&from=2024-01-01&to=2024-03-01",
            "https://proxy.example/router/gateway/stable/historical-chart/1hour?symbol=%5EVIX&from=2024-01-01&to=2024-03-01",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_independent_date_omission() {
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
            json_fixture(LIGHT),
            json_fixture(ONE_HOUR),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbol = Ticker::new("^VIX").unwrap();

        client
            .index_chart_light(
                IndexChartQuery::new(symbol.clone())
                    .with_from(Date::from_str("2026-01-27").unwrap()),
            )
            .await
            .unwrap();
        client
            .index_chart_one_hour(
                IndexChartQuery::new(symbol).with_to(Date::from_str("2024-03-01").unwrap()),
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
                    "https://financialmodelingprep.com/stable/historical-price-eod/light?symbol=%5EVIX&from=2026-01-27{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/historical-chart/1hour?symbol=%5EVIX&to=2024-03-01{suffix}"
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
