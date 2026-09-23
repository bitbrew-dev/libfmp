mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        asset_chart::AssetChartQuery,
        commodities::{
            commodity_chart_five_minutes, commodity_chart_full, commodity_chart_light,
            commodity_chart_one_hour, commodity_chart_one_minute,
        },
        crypto::{
            cryptocurrency_chart_five_minutes, cryptocurrency_chart_full,
            cryptocurrency_chart_light, cryptocurrency_chart_one_hour,
            cryptocurrency_chart_one_minute,
        },
        forex::{
            forex_chart_five_minutes, forex_chart_full, forex_chart_light, forex_chart_one_hour,
            forex_chart_one_minute,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::chart::StockChartIntradayBar,
    transport::{HttpMethod, TransportResponse},
    types::{Date, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const COMMODITY_LIGHT: &[u8] = include_bytes!("fixtures/commodity_chart_light.json");
const COMMODITY_FULL: &[u8] = include_bytes!("fixtures/commodity_chart_full.json");
const COMMODITY_ONE_MINUTE: &[u8] = include_bytes!("fixtures/commodity_chart_one_minute.json");
const COMMODITY_FIVE_MINUTES: &[u8] = include_bytes!("fixtures/commodity_chart_five_minutes.json");
const COMMODITY_ONE_HOUR: &[u8] = include_bytes!("fixtures/commodity_chart_one_hour.json");
const FOREX_LIGHT: &[u8] = include_bytes!("fixtures/forex_chart_light.json");
const FOREX_FULL: &[u8] = include_bytes!("fixtures/forex_chart_full.json");
const FOREX_ONE_MINUTE: &[u8] = include_bytes!("fixtures/forex_chart_one_minute.json");
const FOREX_FIVE_MINUTES: &[u8] = include_bytes!("fixtures/forex_chart_five_minutes.json");
const FOREX_ONE_HOUR: &[u8] = include_bytes!("fixtures/forex_chart_one_hour.json");
const CRYPTO_LIGHT: &[u8] = include_bytes!("fixtures/crypto_chart_light.json");
const CRYPTO_FULL: &[u8] = include_bytes!("fixtures/crypto_chart_full.json");
const CRYPTO_ONE_MINUTE: &[u8] = include_bytes!("fixtures/crypto_chart_one_minute.json");
const CRYPTO_FIVE_MINUTES: &[u8] = include_bytes!("fixtures/crypto_chart_five_minutes.json");
const CRYPTO_ONE_HOUR: &[u8] = include_bytes!("fixtures/crypto_chart_one_hour.json");

#[test]
fn all_descriptors_share_the_narrow_query_exact_paths_and_documented_bounds() {
    let query = AssetChartQuery::new(Ticker::new("GCUSD").unwrap())
        .with_from(Date::from_str("2024-01-01").unwrap())
        .with_to(Date::from_str("2024-03-01").unwrap());

    assert_eq!(query.symbol().as_str(), "GCUSD");
    assert_eq!(query.from().unwrap().to_string(), "2024-01-01");
    assert_eq!(query.to().unwrap().to_string(), "2024-03-01");

    assert_eod(
        &commodity_chart_light(query.clone()),
        "historical-price-eod/light",
    );
    assert_eod(
        &commodity_chart_full(query.clone()),
        "historical-price-eod/full",
    );
    assert_intraday(
        &commodity_chart_one_minute(query.clone()),
        "historical-chart/1min",
    );
    assert_intraday(
        &commodity_chart_five_minutes(query.clone()),
        "historical-chart/5min",
    );
    assert_intraday(
        &commodity_chart_one_hour(query.clone()),
        "historical-chart/1hour",
    );
    assert_eod(
        &forex_chart_light(query.clone()),
        "historical-price-eod/light",
    );
    assert_eod(
        &forex_chart_full(query.clone()),
        "historical-price-eod/full",
    );
    assert_intraday(
        &forex_chart_one_minute(query.clone()),
        "historical-chart/1min",
    );
    assert_intraday(
        &forex_chart_five_minutes(query.clone()),
        "historical-chart/5min",
    );
    assert_intraday(
        &forex_chart_one_hour(query.clone()),
        "historical-chart/1hour",
    );
    assert_eod(
        &cryptocurrency_chart_light(query.clone()),
        "historical-price-eod/light",
    );
    assert_eod(
        &cryptocurrency_chart_full(query.clone()),
        "historical-price-eod/full",
    );
    assert_intraday(
        &cryptocurrency_chart_one_minute(query.clone()),
        "historical-chart/1min",
    );
    assert_intraday(
        &cryptocurrency_chart_five_minutes(query.clone()),
        "historical-chart/5min",
    );
    assert_intraday(
        &cryptocurrency_chart_one_hour(query),
        "historical-chart/1hour",
    );
}

fn assert_eod<R>(endpoint: &EndpointSpec<AssetChartQuery, Vec<R>>, path: &'static str) {
    assert_common(endpoint, path);
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new().with_response_rows(5_000)
    );
}

fn assert_intraday(
    endpoint: &EndpointSpec<AssetChartQuery, Vec<StockChartIntradayBar>>,
    path: &'static str,
) {
    assert_common(endpoint, path);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
}

fn assert_common<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Unspecified
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_routes_all_fifteen_methods_and_decodes_exact_outer_rows() {
    let executor = Arc::new(FixtureExecutor::new(all_fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "asset-history")
        .executor(executor.clone())
        .build()
        .unwrap();

    let commodity_eod = dated("GCUSD", "2026-01-27", "2026-04-27");
    let commodity_intraday = dated("GCUSD", "2024-01-01", "2024-03-01");
    let forex_eod = dated("EURUSD", "2026-01-27", "2026-04-27");
    let forex_intraday = dated("EURUSD", "2024-01-01", "2024-03-01");
    let crypto_eod = dated("BTCUSD", "2026-01-27", "2026-04-27");
    let crypto_intraday = dated("BTCUSD", "2024-01-01", "2024-03-01");

    assert_eq!(
        client
            .commodity_chart_light(commodity_eod.clone())
            .await
            .unwrap()[0]
            .volume,
        126_573.0
    );
    assert_eq!(
        client.commodity_chart_full(commodity_eod).await.unwrap()[0].change,
        43.3
    );
    assert_eq!(
        client
            .commodity_chart_one_minute(commodity_intraday.clone())
            .await
            .unwrap()[0]
            .volume,
        59.0
    );
    assert_eq!(
        client
            .commodity_chart_five_minutes(commodity_intraday.clone())
            .await
            .unwrap()[0]
            .volume,
        103.0
    );
    assert_eq!(
        client
            .commodity_chart_one_hour(commodity_intraday)
            .await
            .unwrap()[0]
            .volume,
        690.0
    );
    assert_eq!(
        client.forex_chart_light(forex_eod.clone()).await.unwrap()[0].price,
        1.15258
    );
    assert_eq!(
        client.forex_chart_full(forex_eod).await.unwrap()[0].change_percent,
        0.51628207
    );
    assert_eq!(
        client
            .forex_chart_one_minute(forex_intraday.clone())
            .await
            .unwrap()[0]
            .volume,
        76.0
    );
    assert_eq!(
        client
            .forex_chart_five_minutes(forex_intraday.clone())
            .await
            .unwrap()[0]
            .volume,
        91.0
    );
    assert_eq!(
        client.forex_chart_one_hour(forex_intraday).await.unwrap()[0].volume,
        1_420.0
    );
    assert_eq!(
        client
            .cryptocurrency_chart_light(crypto_eod.clone())
            .await
            .unwrap()[0]
            .volume,
        32_030_003_200.0
    );
    assert_eq!(
        client.cryptocurrency_chart_full(crypto_eod).await.unwrap()[0].volume,
        32_030_003_200.0
    );
    assert_eq!(
        client
            .cryptocurrency_chart_one_minute(crypto_intraday.clone())
            .await
            .unwrap()[0]
            .volume,
        0.0
    );
    assert_eq!(
        client
            .cryptocurrency_chart_five_minutes(crypto_intraday.clone())
            .await
            .unwrap()[0]
            .volume,
        0.0
    );
    let hourly = client
        .cryptocurrency_chart_one_hour(crypto_intraday)
        .await
        .unwrap();
    assert_eq!(hourly[0].volume, 0.0);
    assert_eq!(hourly[0].date.to_string(), "2026-07-30 13:00:00");

    let requests = executor.requests();
    assert_eq!(requests.len(), 15);
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "asset-history"
    }));
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect::<Vec<_>>();
    for (chunk, symbol) in urls.chunks_exact(5).zip(["GCUSD", "EURUSD", "BTCUSD"]) {
        assert_eq!(
            chunk,
            [
                proxy_url(
                    "historical-price-eod/light",
                    symbol,
                    "2026-01-27",
                    "2026-04-27"
                ),
                proxy_url(
                    "historical-price-eod/full",
                    symbol,
                    "2026-01-27",
                    "2026-04-27"
                ),
                proxy_url("historical-chart/1min", symbol, "2024-01-01", "2024-03-01"),
                proxy_url("historical-chart/5min", symbol, "2024-01-01", "2024-03-01"),
                proxy_url("historical-chart/1hour", symbol, "2024-01-01", "2024-03-01"),
            ]
        );
    }
}

#[tokio::test]
async fn direct_auth_preserves_independent_dates_and_exact_query_order() {
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
            json_fixture(COMMODITY_LIGHT),
            json_fixture(CRYPTO_ONE_HOUR),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        client
            .commodity_chart_light(
                AssetChartQuery::new(Ticker::new("GCUSD").unwrap())
                    .with_from(Date::from_str("2026-01-27").unwrap()),
            )
            .await
            .unwrap();
        client
            .cryptocurrency_chart_one_hour(
                AssetChartQuery::new(Ticker::new("BTCUSD").unwrap())
                    .with_to(Date::from_str("2024-03-01").unwrap()),
            )
            .await
            .unwrap();
        let requests = executor.requests();
        assert_eq!(
            requests[0].expose_url().as_str(),
            format!(
                "https://financialmodelingprep.com/stable/historical-price-eod/light?symbol=GCUSD&from=2026-01-27{suffix}"
            )
        );
        assert_eq!(
            requests[1].expose_url().as_str(),
            format!(
                "https://financialmodelingprep.com/stable/historical-chart/1hour?symbol=BTCUSD&to=2024-03-01{suffix}"
            )
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
async fn malformed_non_arrays_keep_shared_path_endpoint_identity() {
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
    let symbol = Ticker::new("GCUSD").unwrap();
    let errors = [
        client
            .commodity_chart_light(symbol.clone())
            .await
            .unwrap_err(),
        client.forex_chart_full(symbol.clone()).await.unwrap_err(),
        client
            .cryptocurrency_chart_one_minute(symbol)
            .await
            .unwrap_err(),
    ];
    for (error, endpoint) in errors.iter().zip([
        "historical-price-eod/light",
        "historical-price-eod/full",
        "historical-chart/1min",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}

fn dated(symbol: &str, from: &str, to: &str) -> AssetChartQuery {
    AssetChartQuery::new(Ticker::new(symbol).unwrap())
        .with_from(Date::from_str(from).unwrap())
        .with_to(Date::from_str(to).unwrap())
}

fn proxy_url(path: &str, symbol: &str, from: &str, to: &str) -> String {
    format!(
        "https://proxy.example/router/gateway/stable/{path}?symbol={symbol}&from={from}&to={to}"
    )
}

fn all_fixtures() -> [TransportResponse; 15] {
    [
        json_fixture(COMMODITY_LIGHT),
        json_fixture(COMMODITY_FULL),
        json_fixture(COMMODITY_ONE_MINUTE),
        json_fixture(COMMODITY_FIVE_MINUTES),
        json_fixture(COMMODITY_ONE_HOUR),
        json_fixture(FOREX_LIGHT),
        json_fixture(FOREX_FULL),
        json_fixture(FOREX_ONE_MINUTE),
        json_fixture(FOREX_FIVE_MINUTES),
        json_fixture(FOREX_ONE_HOUR),
        json_fixture(CRYPTO_LIGHT),
        json_fixture(CRYPTO_FULL),
        json_fixture(CRYPTO_ONE_MINUTE),
        json_fixture(CRYPTO_FIVE_MINUTES),
        json_fixture(CRYPTO_ONE_HOUR),
    ]
}
