mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        transcripts::{
            EarningsTranscriptDatesQuery, EarningsTranscriptQuery, LatestEarningsTranscriptsQuery,
            earnings_transcript, earnings_transcript_dates, earnings_transcript_list,
            latest_earnings_transcripts,
        },
    },
    query::{FiscalPeriod, Quarter, Year},
    responses::transcripts::EarningsTranscript,
    transport::HttpMethod,
    types::{CalendarYear, Limit, Page, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const LATEST: &[u8] = include_bytes!("fixtures/latest_earnings_transcripts.json");
const TRANSCRIPT: &[u8] = include_bytes!("fixtures/earnings_transcript.json");
const DATES: &[u8] = include_bytes!("fixtures/earnings_transcript_dates.json");
const AVAILABILITY: &[u8] = include_bytes!("fixtures/directory_earnings_transcript_list.json");

#[test]
fn descriptor_uses_exact_path_query_row_and_worldwide_metadata_without_bounds() {
    let query = EarningsTranscriptQuery::new(Ticker::new("AAPL").unwrap(), Year(2020), Quarter::Q3)
        .with_limit(Limit(u32::MAX));
    let endpoint: EndpointSpec<EarningsTranscriptQuery, Vec<EarningsTranscript>> =
        earnings_transcript(query);

    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), "earning-call-transcript");
    assert_eq!(endpoint.relative_path(), "earning-call-transcript");
    assert_eq!(endpoint.query().symbol().as_str(), "AAPL");
    assert_eq!(endpoint.query().year(), Year(2020));
    assert_eq!(endpoint.query().quarter(), Quarter::Q3);
    assert_eq!(endpoint.query().limit(), Some(Limit(u32::MAX)));
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert!(endpoint.metadata().bounds().accepts_limit(Limit(u32::MAX)));
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_integration_covers_all_four_transcript_paths_without_duplication() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(LATEST),
        json_fixture(TRANSCRIPT),
        json_fixture(DATES),
        json_fixture(AVAILABILITY),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "transcripts")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("AAPL").unwrap();

    let latest = client
        .latest_earnings_transcripts(
            LatestEarningsTranscriptsQuery::new()
                .with_limit(Limit(100))
                .with_page(Page(0)),
        )
        .await
        .unwrap();
    let transcripts = client
        .earnings_transcript(
            EarningsTranscriptQuery::new(symbol.clone(), Year(2020), Quarter::Q3)
                .with_limit(Limit(1)),
        )
        .await
        .unwrap();
    let dates = client.earnings_transcript_dates(symbol).await.unwrap();
    let availability = client.earnings_transcript_list().await.unwrap();

    assert_eq!(latest[0].fiscal_year, CalendarYear(2026));
    assert_eq!(transcripts.len(), 1);
    assert_eq!(transcripts[0].period, FiscalPeriod::Q3);
    assert_eq!(transcripts[0].year, CalendarYear(2020));
    assert!(transcripts[0].content.contains("\nTejas Gala:"));
    assert!(transcripts[0].content.ends_with("Aft..."));
    assert_eq!(dates[0].fiscal_year, CalendarYear(2026));
    assert_eq!(availability[0].no_of_transcripts.as_str(), "6");

    let transcript_json: serde_json::Value = serde_json::from_slice(TRANSCRIPT).unwrap();
    assert!(transcript_json[0]["year"].is_u64());
    assert!(transcript_json[0].get("fiscalYear").is_none());
    assert_eq!(
        transcripts[0].content,
        transcript_json[0]["content"].as_str().unwrap()
    );

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "transcripts"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/earning-call-transcript-latest?limit=100&page=0",
            "https://proxy.example/router/gateway/stable/earning-call-transcript?symbol=AAPL&year=2020&quarter=3&limit=1",
            "https://proxy.example/router/gateway/stable/earning-call-transcript-dates?symbol=AAPL",
            "https://proxy.example/router/gateway/stable/earnings-transcript-list",
        ]
    );

    let list = earnings_transcript_list();
    assert_eq!(list.metadata().geography(), GeographicAvailability::UsOnly);
    assert_eq!(list.id(), "earnings-transcript-list");
    assert_eq!(
        latest_earnings_transcripts(LatestEarningsTranscriptsQuery::new()).id(),
        "earning-call-transcript-latest"
    );
    assert_eq!(
        earnings_transcript_dates(EarningsTranscriptDatesQuery::new(
            Ticker::new("AAPL").unwrap()
        ))
        .id(),
        "earning-call-transcript-dates"
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_required_order_and_optional_limit() {
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
            json_fixture(TRANSCRIPT),
            json_fixture(TRANSCRIPT),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .earnings_transcript(EarningsTranscriptQuery::new(
                Ticker::new("AAPL").unwrap(),
                Year(2020),
                Quarter::Q3,
            ))
            .await
            .unwrap();
        client
            .earnings_transcript(
                EarningsTranscriptQuery::new(Ticker::new("AAPL").unwrap(), Year(2020), Quarter::Q3)
                    .with_limit(Limit(u32::MAX)),
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
                    "https://financialmodelingprep.com/stable/earning-call-transcript?symbol=AAPL&year=2020&quarter=3{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/earning-call-transcript?symbol=AAPL&year=2020&quarter=3&limit=4294967295{query_suffix}"
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
