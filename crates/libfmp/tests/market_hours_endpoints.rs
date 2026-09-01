mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        market_hours::{
            AllExchangeMarketHoursQuery, ExchangeMarketHoursQuery, HolidaysByExchangeQuery,
            all_exchange_market_hours, exchange_market_hours, holidays_by_exchange,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::market_hours::{ExchangeHoliday, ExchangeMarketHours},
    transport::HttpMethod,
    types::{Date, ExchangeCode, MarketHoursTimestamp},
};

use support::{FixtureExecutor, json_fixture};

const EXCHANGE: &[u8] = include_bytes!("fixtures/exchange_market_hours.json");
const HOLIDAYS: &[u8] = include_bytes!("fixtures/holidays_by_exchange.json");
const ALL_EXCHANGES: &[u8] = include_bytes!("fixtures/all_exchange_market_hours.json");

#[test]
fn descriptors_use_exact_get_paths_response_rows_and_worldwide_metadata() {
    let exchange_query = ExchangeMarketHoursQuery::new(ExchangeCode::new("NASDAQ").unwrap())
        .with_timestamp(MarketHoursTimestamp::new("001769527402").unwrap());
    let holiday_query = HolidaysByExchangeQuery::new(ExchangeCode::new("NASDAQ").unwrap())
        .with_from(Date::from_str("2025-04-27").unwrap())
        .with_to(Date::from_str("2026-04-27").unwrap());
    let all_query = AllExchangeMarketHoursQuery::new()
        .with_timestamp(MarketHoursTimestamp::new("001769527402").unwrap());

    assert_eq!(exchange_query.exchange().as_str(), "NASDAQ");
    assert_eq!(exchange_query.timestamp().unwrap().as_str(), "001769527402");
    assert_eq!(holiday_query.exchange().as_str(), "NASDAQ");
    assert_eq!(holiday_query.from().unwrap().to_string(), "2025-04-27");
    assert_eq!(holiday_query.to().unwrap().to_string(), "2026-04-27");
    assert_eq!(all_query.timestamp().unwrap().as_str(), "001769527402");

    assert_exchange_facts(&exchange_market_hours(exchange_query));
    assert_holiday_facts(&holidays_by_exchange(holiday_query));
    assert_all_facts(&all_exchange_market_hours(all_query));
}

fn assert_exchange_facts(
    endpoint: &EndpointSpec<ExchangeMarketHoursQuery, Vec<ExchangeMarketHours>>,
) {
    assert_common_facts(endpoint, "exchange-market-hours");
}

fn assert_holiday_facts(endpoint: &EndpointSpec<HolidaysByExchangeQuery, Vec<ExchangeHoliday>>) {
    assert_common_facts(endpoint, "holidays-by-exchange");
}

fn assert_all_facts(
    endpoint: &EndpointSpec<AllExchangeMarketHoursQuery, Vec<ExchangeMarketHours>>,
) {
    assert_common_facts(endpoint, "all-exchange-market-hours");
}

fn assert_common_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
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
async fn proxy_routing_preserves_paths_query_order_opaque_timestamp_headers_and_fixtures() {
    let executor = Arc::new(FixtureExecutor::new(fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "market-hours")
        .executor(executor.clone())
        .build()
        .unwrap();
    let timestamp = MarketHoursTimestamp::new("001769527402").unwrap();

    let exchange = client
        .exchange_market_hours(
            ExchangeMarketHoursQuery::new(ExchangeCode::new("NASDAQ / Global").unwrap())
                .with_timestamp(timestamp.clone()),
        )
        .await
        .unwrap();
    let holidays = client
        .holidays_by_exchange(
            HolidaysByExchangeQuery::new(ExchangeCode::new("NASDAQ / Global").unwrap())
                .with_from(Date::from_str("2025-04-27").unwrap())
                .with_to(Date::from_str("2026-04-27").unwrap()),
        )
        .await
        .unwrap();
    let all = client
        .all_exchange_market_hours(AllExchangeMarketHoursQuery::new().with_timestamp(timestamp))
        .await
        .unwrap();

    assert_eq!(exchange[0].opening_hour, "09:30 AM -04:00");
    assert_eq!(holidays[0].name, "Independence Day");
    assert_eq!(holidays[0].adj_open_time, None);
    assert_eq!(all[0].exchange.as_str(), "ASX");

    let requests = executor.requests();
    assert_eq!(requests.len(), 3);
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "market-hours"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/exchange-market-hours?exchange=NASDAQ+%2F+Global&timestamp=001769527402",
            "https://proxy.example/router/gateway/stable/holidays-by-exchange?exchange=NASDAQ+%2F+Global&from=2025-04-27&to=2026-04-27",
            "https://proxy.example/router/gateway/stable/all-exchange-market-hours?timestamp=001769527402",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_all_optional_omissions() {
    for (authentication, query_auth, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            false,
            Some("header-secret"),
        ),
        (Authentication::fmp_query("query-secret"), true, None),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(EXCHANGE),
            json_fixture(HOLIDAYS),
            json_fixture(HOLIDAYS),
            json_fixture(ALL_EXCHANGES),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let exchange = ExchangeCode::new("NASDAQ").unwrap();

        client.exchange_market_hours(&exchange).await.unwrap();
        client
            .holidays_by_exchange(
                HolidaysByExchangeQuery::new(exchange.clone())
                    .with_from(Date::from_str("2025-04-27").unwrap()),
            )
            .await
            .unwrap();
        client
            .holidays_by_exchange(
                HolidaysByExchangeQuery::new(exchange)
                    .with_to(Date::from_str("2026-04-27").unwrap()),
            )
            .await
            .unwrap();
        client
            .all_exchange_market_hours(AllExchangeMarketHoursQuery::new())
            .await
            .unwrap();

        let auth_amp = if query_auth {
            "&apikey=query-secret"
        } else {
            ""
        };
        let auth_question = if query_auth {
            "?apikey=query-secret"
        } else {
            ""
        };
        let requests = executor.requests();
        assert_eq!(
            request_urls(&requests),
            [
                format!(
                    "https://financialmodelingprep.com/stable/exchange-market-hours?exchange=NASDAQ{auth_amp}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/holidays-by-exchange?exchange=NASDAQ&from=2025-04-27{auth_amp}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/holidays-by-exchange?exchange=NASDAQ&to=2026-04-27{auth_amp}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/all-exchange-market-hours{auth_question}"
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
async fn malformed_non_array_responses_keep_all_three_endpoint_identities() {
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
    let exchange = ExchangeCode::new("NASDAQ").unwrap();

    let errors = [
        client.exchange_market_hours(&exchange).await.unwrap_err(),
        client.holidays_by_exchange(&exchange).await.unwrap_err(),
        client
            .all_exchange_market_hours(AllExchangeMarketHoursQuery::new())
            .await
            .unwrap_err(),
    ];

    for (error, id) in errors.iter().zip([
        "exchange-market-hours",
        "holidays-by-exchange",
        "all-exchange-market-hours",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
}

fn fixtures() -> [libfmp::transport::TransportResponse; 3] {
    [
        json_fixture(EXCHANGE),
        json_fixture(HOLIDAYS),
        json_fixture(ALL_EXCHANGES),
    ]
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}
