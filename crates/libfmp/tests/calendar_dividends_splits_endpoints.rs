mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        calendar::{
            DividendsCalendarQuery, DividendsQuery, StockSplitsCalendarQuery, StockSplitsQuery,
            dividends, dividends_calendar, stock_splits, stock_splits_calendar,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    transport::HttpMethod,
    types::{Date, DateRange, Limit, Page, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const DIVIDENDS: &[u8] = include_bytes!("fixtures/dividends.json");
const DIVIDENDS_CALENDAR: &[u8] = include_bytes!("fixtures/dividends_calendar.json");
const STOCK_SPLITS: &[u8] = include_bytes!("fixtures/stock_splits.json");
const STOCK_SPLITS_CALENDAR: &[u8] = include_bytes!("fixtures/stock_splits_calendar.json");

#[test]
fn descriptors_use_exact_paths_vec_rows_and_only_documented_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let company_bounds = EndpointBounds::new().with_response_rows(1_000);
    let calendar_bounds = EndpointBounds::new()
        .with_response_rows(4_000)
        .with_date_range_days(90);

    assert_facts(
        &dividends(DividendsQuery::new(symbol.clone())),
        "dividends",
        company_bounds,
    );
    assert_facts(
        &dividends_calendar(DividendsCalendarQuery::new()),
        "dividends-calendar",
        calendar_bounds,
    );
    assert_facts(
        &stock_splits(StockSplitsQuery::new(symbol)),
        "splits",
        company_bounds,
    );
    assert_facts(
        &stock_splits_calendar(StockSplitsCalendarQuery::new()),
        "splits-calendar",
        calendar_bounds,
    );

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
    let contradictory_92_day_sample = DateRange::new(
        Date::from_str("2026-03-06").unwrap(),
        Date::from_str("2026-06-06").unwrap(),
    )
    .unwrap();
    assert!(calendar_bounds.accepts_date_range(&range_90_days));
    assert!(!calendar_bounds.accepts_date_range(&range_91_days));
    assert!(!calendar_bounds.accepts_date_range(&contradictory_92_day_sample));
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
async fn custom_proxy_preserves_exact_query_order_auth_headers_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(DIVIDENDS),
        json_fixture(DIVIDENDS_CALENDAR),
        json_fixture(STOCK_SPLITS),
        json_fixture(STOCK_SPLITS_CALENDAR),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "dividends-splits")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("AAPL").unwrap();
    let from = Date::from_str("2026-01-27").unwrap();
    let to = Date::from_str("2026-04-27").unwrap();

    let dividends_rows = client
        .dividends(DividendsQuery::new(symbol.clone()).with_limit(Limit(1_001)))
        .await
        .unwrap();
    let dividend_calendar_rows = client
        .dividends_calendar(
            DividendsCalendarQuery::new()
                .with_from(from)
                .with_to(to)
                .with_page(Page(0)),
        )
        .await
        .unwrap();
    let split_rows = client
        .stock_splits(StockSplitsQuery::new(symbol).with_limit(Limit(0)))
        .await
        .unwrap();
    let split_calendar_rows = client
        .stock_splits_calendar(
            StockSplitsCalendarQuery::new()
                .with_from(from)
                .with_to(to)
                .with_page(Page(0)),
        )
        .await
        .unwrap();

    assert_eq!(dividends_rows.len(), 1);
    assert_eq!(dividends_rows[0].frequency, "Quarterly");
    assert_eq!(dividend_calendar_rows.len(), 1);
    assert_eq!(dividend_calendar_rows[0].symbol.as_str(), "5871.TW");
    assert_eq!(split_rows.len(), 1);
    assert_eq!(split_rows[0].numerator, 4.0);
    assert_eq!(split_calendar_rows.len(), 1);
    assert_eq!(split_calendar_rows[0].denominator, 5.0);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "dividends-splits"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/dividends?symbol=AAPL&limit=1001",
            "https://proxy.example/router/stable/dividends-calendar?from=2026-01-27&to=2026-04-27&page=0",
            "https://proxy.example/router/stable/splits?symbol=AAPL&limit=0",
            "https://proxy.example/router/stable/splits-calendar?from=2026-01-27&to=2026-04-27&page=0",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_independent_omission_and_page_zero() {
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
            json_fixture(DIVIDENDS),
            json_fixture(DIVIDENDS_CALENDAR),
            json_fixture(STOCK_SPLITS),
            json_fixture(STOCK_SPLITS_CALENDAR),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbol = Ticker::new("AAPL").unwrap();
        let from = Date::from_str("2026-01-27").unwrap();
        let to = Date::from_str("2026-04-27").unwrap();

        client.dividends(symbol.clone()).await.unwrap();
        client
            .dividends_calendar(
                DividendsCalendarQuery::new()
                    .with_from(from)
                    .with_page(Page(0)),
            )
            .await
            .unwrap();
        client.stock_splits(symbol).await.unwrap();
        client
            .stock_splits_calendar(
                StockSplitsCalendarQuery::new()
                    .with_to(to)
                    .with_page(Page(0)),
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
                    "https://financialmodelingprep.com/stable/dividends?symbol=AAPL{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/dividends-calendar?from=2026-01-27&page=0{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/splits?symbol=AAPL{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/splits-calendar?to=2026-04-27&page=0{query_suffix}"
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
