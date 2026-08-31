mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        technical_indicators::{
            TechnicalIndicatorQuery, double_exponential_moving_average,
            triple_exponential_moving_average,
        },
    },
    error::ErrorCategory,
    query::{ChartTimeframe, PeriodLength},
    responses::technical_indicators::{
        DoubleExponentialMovingAverageBar, TripleExponentialMovingAverageBar,
    },
    transport::HttpMethod,
    types::{Date, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const DEMA: &[u8] = include_bytes!("fixtures/technical_indicator_dema.json");
const TEMA: &[u8] = include_bytes!("fixtures/technical_indicator_tema.json");

fn query(symbol: &str, timeframe: ChartTimeframe) -> TechnicalIndicatorQuery {
    TechnicalIndicatorQuery::new(
        Ticker::new(symbol).unwrap(),
        PeriodLength::new(10).unwrap(),
        timeframe,
    )
}

#[test]
fn descriptors_use_exact_paths_bare_types_and_only_documented_metadata() {
    let dema = double_exponential_moving_average(query("AAPL", ChartTimeframe::OneDay));
    let tema = triple_exponential_moving_average(query("AAPL", ChartTimeframe::OneDay));
    assert_facts(&dema, "technical-indicators/dema");
    assert_facts(&tema, "technical-indicators/tema");
    assert_response_types(&dema, &tema);
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, Vec<R>>, path: &'static str) {
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

fn assert_response_types(
    _: &EndpointSpec<TechnicalIndicatorQuery, Vec<DoubleExponentialMovingAverageBar>>,
    _: &EndpointSpec<TechnicalIndicatorQuery, Vec<TripleExponentialMovingAverageBar>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_order_headers_fixtures_and_independent_dates() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(DEMA),
        json_fixture(TEMA),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "technical indicators")
        .executor(executor.clone())
        .build()
        .unwrap();
    let from = Date::from_str("2026-06-01").unwrap();
    let to = Date::from_str("2026-03-01").unwrap();

    let dema = client
        .double_exponential_moving_average(
            query("BRK.B / Class A", ChartTimeframe::OneDay).with_from(from),
        )
        .await
        .unwrap();
    let tema = client
        .triple_exponential_moving_average(query("AAPL", ChartTimeframe::FourHours).with_to(to))
        .await
        .unwrap();
    assert_eq!(
        (dema[0].dema, dema[0].volume),
        (337.7659642917977, 29_207_295)
    );
    assert_eq!(
        (tema[0].tema, tema[0].volume),
        (337.09671042323214, 29_207_295)
    );

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "technical indicators"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/technical-indicators/dema?symbol=BRK.B+%2F+Class+A&periodLength=10&timeframe=1day&from=2026-06-01",
            "https://proxy.example/router/gateway/stable/technical-indicators/tema?symbol=AAPL&periodLength=10&timeframe=4hour&to=2026-03-01",
        ]
    );
}

#[tokio::test]
async fn all_seven_timeframes_have_exact_wire_urls_and_omit_dates() {
    let timeframes = [
        (ChartTimeframe::OneMinute, "1min"),
        (ChartTimeframe::FiveMinutes, "5min"),
        (ChartTimeframe::FifteenMinutes, "15min"),
        (ChartTimeframe::ThirtyMinutes, "30min"),
        (ChartTimeframe::OneHour, "1hour"),
        (ChartTimeframe::FourHours, "4hour"),
        (ChartTimeframe::OneDay, "1day"),
    ];
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| json_fixture(DEMA)).take(timeframes.len()),
    ));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();
    for (timeframe, _) in timeframes {
        client
            .double_exponential_moving_average(query("AAPL", timeframe))
            .await
            .unwrap();
    }
    for (request, (_, wire)) in executor.requests().iter().zip(timeframes) {
        assert_eq!(
            request.expose_url().as_str(),
            format!(
                "https://financialmodelingprep.com/stable/technical-indicators/dema?symbol=AAPL&periodLength=10&timeframe={wire}"
            )
        );
    }
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_both_urls() {
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
            json_fixture(DEMA),
            json_fixture(TEMA),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        client
            .double_exponential_moving_average(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap();
        client
            .triple_exponential_moving_average(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap();
        let requests = executor.requests();
        for (request, indicator) in requests.iter().zip(["dema", "tema"]) {
            assert_eq!(
                request.expose_url().as_str(),
                format!(
                    "https://financialmodelingprep.com/stable/technical-indicators/{indicator}?symbol=AAPL&periodLength=10&timeframe=1day{suffix}"
                )
            );
            match expected_header {
                Some(value) => assert_eq!(request.expose_headers()["apikey"], value),
                None => assert!(!request.expose_headers().contains_key("apikey")),
            }
        }
    }
}

#[tokio::test]
async fn malformed_non_arrays_keep_each_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();
    let errors = [
        client
            .double_exponential_moving_average(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap_err(),
        client
            .triple_exponential_moving_average(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap_err(),
    ];
    for (error, endpoint) in errors
        .iter()
        .zip(["technical-indicators/dema", "technical-indicators/tema"])
    {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
