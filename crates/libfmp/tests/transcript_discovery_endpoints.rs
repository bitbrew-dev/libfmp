mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec, directory,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        transcripts::{
            EarningsTranscriptDatesQuery, LatestEarningsTranscriptsQuery,
            earnings_transcript_dates, earnings_transcript_list, latest_earnings_transcripts,
        },
    },
    query::FiscalPeriod,
    responses::transcripts::{
        EarningsTranscriptAvailability, EarningsTranscriptDate, LatestEarningsTranscript,
    },
    transport::HttpMethod,
    types::{CalendarQuarter, CalendarYear, Date, Limit, Page, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const LATEST: &[u8] = include_bytes!("fixtures/latest_earnings_transcripts.json");
const DATES: &[u8] = include_bytes!("fixtures/earnings_transcript_dates.json");
const AVAILABILITY: &[u8] = include_bytes!("fixtures/directory_earnings_transcript_list.json");

#[test]
fn descriptors_use_exact_paths_vec_rows_and_only_documented_metadata() {
    let latest = latest_earnings_transcripts(LatestEarningsTranscriptsQuery::new());
    assert_facts(
        &latest,
        "earning-call-transcript-latest",
        EndpointBounds::new().with_response_rows(100).with_page(100),
    );
    assert_facts(
        &earnings_transcript_dates(EarningsTranscriptDatesQuery::new(
            Ticker::new("AAPL").unwrap(),
        )),
        "earning-call-transcript-dates",
        EndpointBounds::new(),
    );

    let latest_bounds = latest.metadata().bounds();
    assert_eq!(latest_bounds.limit(), None);
    assert!(latest_bounds.accepts_limit(Limit(u32::MAX)));
    assert!(latest_bounds.accepts_page(Page(0)));
    assert!(latest_bounds.accepts_page(Page(100)));
    assert!(!latest_bounds.accepts_page(Page(101)));
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
        json_fixture(LATEST),
        json_fixture(DATES),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "transcript-discovery")
        .executor(executor.clone())
        .build()
        .unwrap();

    let latest = client
        .latest_earnings_transcripts(
            LatestEarningsTranscriptsQuery::new()
                .with_limit(Limit(101))
                .with_page(Page(100)),
        )
        .await
        .unwrap();
    let dates = client
        .earnings_transcript_dates(Ticker::new("AAPL").unwrap())
        .await
        .unwrap();

    assert_eq!(
        latest,
        [LatestEarningsTranscript {
            symbol: Ticker::new("VLO").unwrap(),
            period: FiscalPeriod::Q2,
            fiscal_year: CalendarYear(2026),
            date: Date::from_str("2026-07-30").unwrap(),
        }]
    );
    assert_eq!(
        dates,
        [EarningsTranscriptDate {
            quarter: CalendarQuarter::new(2).unwrap(),
            fiscal_year: CalendarYear(2026),
            date: Date::from_str("2026-04-30").unwrap(),
        }]
    );
    let latest_json: serde_json::Value = serde_json::from_slice(LATEST).unwrap();
    let dates_json: serde_json::Value = serde_json::from_slice(DATES).unwrap();
    assert!(latest_json[0]["fiscalYear"].is_u64());
    assert!(dates_json[0]["quarter"].is_u64());
    assert!(dates_json[0]["fiscalYear"].is_u64());

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "transcript-discovery"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/earning-call-transcript-latest?limit=101&page=100",
            "https://proxy.example/router/stable/earning-call-transcript-dates?symbol=AAPL",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_independent_latest_options() {
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
            json_fixture(LATEST),
            json_fixture(LATEST),
            json_fixture(DATES),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .latest_earnings_transcripts(
                LatestEarningsTranscriptsQuery::new().with_limit(Limit(100)),
            )
            .await
            .unwrap();
        client
            .latest_earnings_transcripts(LatestEarningsTranscriptsQuery::new().with_page(Page(0)))
            .await
            .unwrap();
        client
            .earnings_transcript_dates(Ticker::new("AAPL").unwrap())
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
                    "https://financialmodelingprep.com/stable/earning-call-transcript-latest?limit=100{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/earning-call-transcript-latest?page=0{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/earning-call-transcript-dates?symbol=AAPL{query_suffix}"
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
async fn transcript_list_reexport_keeps_the_single_shipped_us_only_contract() {
    let reexported = earnings_transcript_list();
    let directory_owned = directory::earnings_transcript_list();
    assert_eq!(reexported.id(), directory_owned.id());
    assert_eq!(reexported.relative_path(), directory_owned.relative_path());
    assert_eq!(reexported.metadata(), directory_owned.metadata());
    assert_eq!(reexported.id(), "earnings-transcript-list");
    assert_eq!(
        reexported.metadata().geography(),
        GeographicAvailability::UsOnly
    );

    let executor = Arc::new(FixtureExecutor::new([json_fixture(AVAILABILITY)]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("list-secret"))
        .executor(executor.clone())
        .build()
        .unwrap();
    let rows: Vec<EarningsTranscriptAvailability> =
        client.earnings_transcript_list().await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].symbol.as_str(), "INBS");
    assert_eq!(rows[0].no_of_transcripts.as_str(), "6");
    assert_eq!(
        executor.requests()[0].expose_url().as_str(),
        "https://financialmodelingprep.com/stable/earnings-transcript-list"
    );
}
