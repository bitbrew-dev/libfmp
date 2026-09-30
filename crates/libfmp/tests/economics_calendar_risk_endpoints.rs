mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        economics::{EconomicCalendarQuery, economic_calendar, market_risk_premium},
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    responses::economics::{EconomicCalendarEvent, MarketRiskPremium},
    transport::HttpMethod,
    types::{CountryCode, Date, DateRange},
};

use support::{FixtureExecutor, json_fixture};

const CALENDAR: &[u8] = include_bytes!("fixtures/economic_calendar.json");
const RISK_PREMIUM: &[u8] = include_bytes!("fixtures/market_risk_premium.json");

#[test]
fn descriptors_use_exact_paths_response_rows_and_only_documented_bounds() {
    let calendar = economic_calendar(EconomicCalendarQuery::new());
    assert_facts(
        &calendar,
        "economic-calendar",
        EndpointBounds::new().with_date_range_days(90),
    );
    assert_eq!(calendar.query(), &EconomicCalendarQuery::new());

    let risk = market_risk_premium();
    assert_facts(&risk, "market-risk-premium", EndpointBounds::new());
    assert_eq!(risk.query(), &());
    assert_response_types(&calendar, &risk);

    let bounds = calendar.metadata().bounds();
    let range_90_days = DateRange::new(
        Date::from_str("2026-01-27").unwrap(),
        Date::from_str("2026-04-27").unwrap(),
    )
    .unwrap();
    let range_91_days = DateRange::new(
        Date::from_str("2026-01-27").unwrap(),
        Date::from_str("2026-04-28").unwrap(),
    )
    .unwrap();
    assert!(bounds.accepts_date_range(&range_90_days));
    assert!(!bounds.accepts_date_range(&range_91_days));
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
        GeographicAvailability::Unspecified
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), bounds);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_routing_preserves_calendar_query_order_custom_auth_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(CALENDAR),
        json_fixture(RISK_PREMIUM),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_query("router_token", "proxy-secret"))
        .default_header("x-data-scope", "calendar-risk")
        .executor(executor.clone())
        .build()
        .unwrap();
    let from = Date::from_str("2026-01-27").unwrap();
    let to = Date::from_str("2026-04-27").unwrap();

    let events = client
        .economic_calendar(
            EconomicCalendarQuery::new()
                .with_country(CountryCode::new("US").unwrap())
                .with_from(from)
                .with_to(to),
        )
        .await
        .unwrap();
    let premiums = client.market_risk_premium().await.unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].actual, Some(13.6));
    assert_eq!(premiums.len(), 1);
    assert_eq!(premiums[0].country, "Zimbabwe");

    let requests = executor.requests();
    assert_eq!(requests.len(), 2);
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(
        requests
            .iter()
            .all(|request| request.expose_headers()["x-data-scope"] == "calendar-risk")
    );
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/economic-calendar?country=US&from=2026-01-27&to=2026-04-27&router_token=proxy-secret",
            "https://proxy.example/router/gateway/stable/market-risk-premium?router_token=proxy-secret",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_every_independent_omission() {
    let from = Date::from_str("2026-01-27").unwrap();
    let to = Date::from_str("2026-04-27").unwrap();
    let country = CountryCode::new("US").unwrap();

    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "?apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new(
            std::iter::repeat_with(|| json_fixture(CALENDAR))
                .take(8)
                .chain([json_fixture(RISK_PREMIUM)]),
        ));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let queries = [
            EconomicCalendarQuery::new(),
            EconomicCalendarQuery::new().with_country(country.clone()),
            EconomicCalendarQuery::new().with_from(from),
            EconomicCalendarQuery::new().with_to(to),
            EconomicCalendarQuery::new()
                .with_country(country.clone())
                .with_from(from),
            EconomicCalendarQuery::new()
                .with_country(country.clone())
                .with_to(to),
            EconomicCalendarQuery::new().with_from(from).with_to(to),
            EconomicCalendarQuery::new()
                .with_country(country.clone())
                .with_from(from)
                .with_to(to),
        ];

        for query in queries {
            assert_eq!(client.economic_calendar(query).await.unwrap().len(), 1);
        }
        assert_eq!(client.market_risk_premium().await.unwrap().len(), 1);

        let auth_tail = if suffix.is_empty() {
            ""
        } else {
            "&apikey=query-secret"
        };
        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!("https://financialmodelingprep.com/stable/economic-calendar{suffix}"),
                format!(
                    "https://financialmodelingprep.com/stable/economic-calendar?country=US{auth_tail}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/economic-calendar?from=2026-01-27{auth_tail}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/economic-calendar?to=2026-04-27{auth_tail}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/economic-calendar?country=US&from=2026-01-27{auth_tail}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/economic-calendar?country=US&to=2026-04-27{auth_tail}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/economic-calendar?from=2026-01-27&to=2026-04-27{auth_tail}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/economic-calendar?country=US&from=2026-01-27&to=2026-04-27{auth_tail}"
                ),
                format!("https://financialmodelingprep.com/stable/market-risk-premium{suffix}"),
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

fn assert_response_types(
    _: &EndpointSpec<EconomicCalendarQuery, Vec<EconomicCalendarEvent>>,
    _: &EndpointSpec<(), Vec<MarketRiskPremium>>,
) {
}
