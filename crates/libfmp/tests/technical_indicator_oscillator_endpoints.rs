mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        technical_indicators::{
            TechnicalIndicatorQuery, average_directional_index, relative_strength_index,
            standard_deviation, williams,
        },
    },
    error::ErrorCategory,
    query::{ChartTimeframe, PeriodLength},
    responses::technical_indicators::{
        AverageDirectionalIndexBar, RelativeStrengthIndexBar, StandardDeviationBar, WilliamsBar,
    },
    transport::HttpMethod,
    types::{Date, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const RSI: &[u8] = include_bytes!("fixtures/technical_indicator_rsi.json");
const STANDARD_DEVIATION: &[u8] =
    include_bytes!("fixtures/technical_indicator_standard_deviation.json");
const WILLIAMS: &[u8] = include_bytes!("fixtures/technical_indicator_williams.json");
const ADX: &[u8] = include_bytes!("fixtures/technical_indicator_adx.json");

fn query(symbol: &str, timeframe: ChartTimeframe) -> TechnicalIndicatorQuery {
    TechnicalIndicatorQuery::new(
        Ticker::new(symbol).unwrap(),
        PeriodLength::new(10).unwrap(),
        timeframe,
    )
}

#[test]
fn descriptors_use_exact_paths_bare_types_and_only_documented_metadata() {
    let rsi = relative_strength_index(query("AAPL", ChartTimeframe::OneDay));
    let standard = standard_deviation(query("AAPL", ChartTimeframe::OneDay));
    let williams = williams(query("AAPL", ChartTimeframe::OneDay));
    let adx = average_directional_index(query("AAPL", ChartTimeframe::OneDay));
    assert_facts(&rsi, "technical-indicators/rsi");
    assert_facts(&standard, "technical-indicators/standarddeviation");
    assert_facts(&williams, "technical-indicators/williams");
    assert_facts(&adx, "technical-indicators/adx");
    assert_response_types(&rsi, &standard, &williams, &adx);
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
    _: &EndpointSpec<TechnicalIndicatorQuery, Vec<RelativeStrengthIndexBar>>,
    _: &EndpointSpec<TechnicalIndicatorQuery, Vec<StandardDeviationBar>>,
    _: &EndpointSpec<TechnicalIndicatorQuery, Vec<WilliamsBar>>,
    _: &EndpointSpec<TechnicalIndicatorQuery, Vec<AverageDirectionalIndexBar>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_order_headers_fixtures_and_independent_dates() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(RSI),
        json_fixture(STANDARD_DEVIATION),
        json_fixture(WILLIAMS),
        json_fixture(ADX),
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

    let rsi = client
        .relative_strength_index(
            query("BRK.B / Class A", ChartTimeframe::OneMinute).with_from(from),
        )
        .await
        .unwrap();
    let standard = client
        .standard_deviation(query("AAPL", ChartTimeframe::FiveMinutes).with_to(to))
        .await
        .unwrap();
    let williams = client
        .williams(
            query("AAPL", ChartTimeframe::FifteenMinutes)
                .with_from(from)
                .with_to(to),
        )
        .await
        .unwrap();
    let adx = client
        .average_directional_index(query("AAPL", ChartTimeframe::ThirtyMinutes))
        .await
        .unwrap();
    assert_eq!(rsi[0].rsi, 59.55175118203601);
    assert_eq!(standard[0].standard_deviation, 5.675893674127448);
    assert_eq!(williams[0].williams, -48.29500396510714);
    assert_eq!(adx[0].adx, 34.69458756515438);
    assert_eq!(
        [
            rsi[0].volume,
            standard[0].volume,
            williams[0].volume,
            adx[0].volume
        ],
        [29_207_295.0; 4]
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
            "https://proxy.example/router/gateway/stable/technical-indicators/rsi?symbol=BRK.B+%2F+Class+A&periodLength=10&timeframe=1min&from=2026-06-01",
            "https://proxy.example/router/gateway/stable/technical-indicators/standarddeviation?symbol=AAPL&periodLength=10&timeframe=5min&to=2026-03-01",
            "https://proxy.example/router/gateway/stable/technical-indicators/williams?symbol=AAPL&periodLength=10&timeframe=15min&from=2026-06-01&to=2026-03-01",
            "https://proxy.example/router/gateway/stable/technical-indicators/adx?symbol=AAPL&periodLength=10&timeframe=30min",
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
        std::iter::repeat_with(|| json_fixture(RSI)).take(timeframes.len()),
    ));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();
    for (timeframe, _) in timeframes {
        client
            .relative_strength_index(query("AAPL", timeframe))
            .await
            .unwrap();
    }
    for (request, (_, wire)) in executor.requests().iter().zip(timeframes) {
        assert_eq!(
            request.expose_url().as_str(),
            format!(
                "https://financialmodelingprep.com/stable/technical-indicators/rsi?symbol=AAPL&periodLength=10&timeframe={wire}"
            )
        );
    }
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_all_four_urls() {
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
            json_fixture(RSI),
            json_fixture(STANDARD_DEVIATION),
            json_fixture(WILLIAMS),
            json_fixture(ADX),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        client
            .relative_strength_index(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap();
        client
            .standard_deviation(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap();
        client
            .williams(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap();
        client
            .average_directional_index(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap();
        let requests = executor.requests();
        for (request, indicator) in
            requests
                .iter()
                .zip(["rsi", "standarddeviation", "williams", "adx"])
        {
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
            .relative_strength_index(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap_err(),
        client
            .standard_deviation(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap_err(),
        client
            .williams(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap_err(),
        client
            .average_directional_index(query("AAPL", ChartTimeframe::OneDay))
            .await
            .unwrap_err(),
    ];
    for (error, endpoint) in errors.iter().zip([
        "technical-indicators/rsi",
        "technical-indicators/standarddeviation",
        "technical-indicators/williams",
        "technical-indicators/adx",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
