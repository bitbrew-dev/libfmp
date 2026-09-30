mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        calendar::{EarningsCalendarQuery, EarningsQuery, earnings, earnings_calendar},
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    responses::calendar::EarningsEvent,
    transport::HttpMethod,
    types::{Date, DateRange, Limit, Page, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const EARNINGS: &[u8] = include_bytes!("fixtures/earnings.json");
const EARNINGS_CALENDAR: &[u8] = include_bytes!("fixtures/earnings_calendar.json");

#[test]
fn descriptors_use_exact_paths_vec_rows_and_only_documented_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let company_bounds = EndpointBounds::new().with_response_rows(1_000);
    let calendar_bounds = EndpointBounds::new()
        .with_response_rows(4_000)
        .with_date_range_days(90);

    let company: EndpointSpec<EarningsQuery, Vec<EarningsEvent>> =
        earnings(EarningsQuery::new(symbol));
    let calendar: EndpointSpec<EarningsCalendarQuery, Vec<EarningsEvent>> =
        earnings_calendar(EarningsCalendarQuery::new());
    assert_facts(&company, "earnings", company_bounds);
    assert_facts(&calendar, "earnings-calendar", calendar_bounds);

    assert_eq!(company_bounds.limit(), None);
    assert!(company_bounds.accepts_limit(Limit(u32::MAX)));
    assert_eq!(calendar_bounds.page(), None);
    assert!(calendar_bounds.accepts_page(Page(u32::MAX)));

    let range_90_days = DateRange::new(
        Date::from_str("2026-04-27").unwrap(),
        Date::from_str("2026-07-26").unwrap(),
    )
    .unwrap();
    let range_91_days = DateRange::new(
        Date::from_str("2026-04-27").unwrap(),
        Date::from_str("2026-07-27").unwrap(),
    )
    .unwrap();
    let documented_92_day_example = DateRange::new(
        Date::from_str("2026-03-06").unwrap(),
        Date::from_str("2026-06-06").unwrap(),
    )
    .unwrap();
    assert!(calendar_bounds.accepts_date_range(&range_90_days));
    assert!(!calendar_bounds.accepts_date_range(&range_91_days));
    assert!(!calendar_bounds.accepts_date_range(&documented_92_day_example));
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str, bounds: EndpointBounds) {
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

#[tokio::test]
async fn custom_proxy_preserves_query_states_order_auth_headers_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(EARNINGS),
        json_fixture(EARNINGS),
        json_fixture(EARNINGS_CALENDAR),
        json_fixture(EARNINGS_CALENDAR),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "earnings")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("AAPL").unwrap();
    let from = Date::from_str("2026-04-27").unwrap();
    let to = Date::from_str("2026-07-26").unwrap();

    let omitted = client.earnings(symbol.clone()).await.unwrap();
    let explicit_false = client
        .earnings(
            EarningsQuery::new(symbol)
                .with_limit(Limit(u32::MAX))
                .with_include_report_times(false),
        )
        .await
        .unwrap();
    let calendar_omitted = client
        .earnings_calendar(EarningsCalendarQuery::new())
        .await
        .unwrap();
    let calendar_true = client
        .earnings_calendar(
            EarningsCalendarQuery::new()
                .with_from(from)
                .with_to(to)
                .with_page(Page(0))
                .with_include_report_times(true),
        )
        .await
        .unwrap();

    assert_eq!(omitted.len(), 1);
    assert_eq!(omitted[0].eps_actual, None);
    assert_eq!(omitted[0].revenue_actual, None);
    assert_eq!(omitted[0].revenue_estimated, Some(109_038_900_000.0));
    assert_eq!(explicit_false, omitted);
    assert_eq!(calendar_omitted.len(), 1);
    assert_eq!(calendar_omitted[0].eps_actual, Some(0.549));
    assert_eq!(calendar_omitted[0].revenue_actual, Some(1_101_500_000.0));
    assert_eq!(calendar_omitted[0].revenue_estimated, Some(1_086_300_000.0));
    assert_eq!(calendar_true, calendar_omitted);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "earnings"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/earnings?symbol=AAPL",
            "https://proxy.example/router/stable/earnings?symbol=AAPL&limit=4294967295&includeReportTimes=false",
            "https://proxy.example/router/stable/earnings-calendar",
            "https://proxy.example/router/stable/earnings-calendar?from=2026-04-27&to=2026-07-26&page=0&includeReportTimes=true",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_apply_to_both_earnings_contracts() {
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
            json_fixture(EARNINGS),
            json_fixture(EARNINGS_CALENDAR),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbol = Ticker::new("AAPL").unwrap();
        let to = Date::from_str("2026-07-26").unwrap();

        client
            .earnings(EarningsQuery::new(symbol).with_include_report_times(true))
            .await
            .unwrap();
        client
            .earnings_calendar(
                EarningsCalendarQuery::new()
                    .with_to(to)
                    .with_page(Page(0))
                    .with_include_report_times(false),
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
                    "https://financialmodelingprep.com/stable/earnings?symbol=AAPL&includeReportTimes=true{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/earnings-calendar?to=2026-07-26&page=0&includeReportTimes=false{query_suffix}"
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
