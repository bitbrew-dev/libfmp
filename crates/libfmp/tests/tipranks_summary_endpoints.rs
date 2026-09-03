mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        tipranks::{
            TipRanksAnalystSummaryQuery, TipRanksFirmSummaryQuery, TipRanksSymbolSummaryQuery,
            tipranks_analyst_summary, tipranks_firm_summary, tipranks_symbol_summary,
        },
    },
    error::ErrorCategory,
    responses::tipranks::{TipRanksAnalystSummary, TipRanksFirmSummary, TipRanksSymbolSummary},
    transport::HttpMethod,
    types::{Date, SearchTerm, Ticker, TipRanksExpertUid},
};

use support::{FixtureExecutor, json_fixture};

const SYMBOL: &[u8] = include_bytes!("fixtures/tipranks_symbol_summary.json");
const ANALYST: &[u8] = include_bytes!("fixtures/tipranks_analyst_summary.json");
const FIRM: &[u8] = include_bytes!("fixtures/tipranks_firm_summary.json");

#[test]
fn descriptors_have_exact_paths_types_and_single_row_tipranks_metadata() {
    let symbol = tipranks_symbol_summary(TipRanksSymbolSummaryQuery::new(
        Ticker::new("AAPL").unwrap(),
    ));
    assert_symbol_type(&symbol);
    assert_descriptor(&symbol, "tipranks-symbol-summary");

    let analyst = tipranks_analyst_summary(TipRanksAnalystSummaryQuery::new(
        TipRanksExpertUid::new("expert").unwrap(),
    ));
    assert_analyst_type(&analyst);
    assert_descriptor(&analyst, "tipranks-analyst-summary");

    let firm = tipranks_firm_summary(TipRanksFirmSummaryQuery::new(
        SearchTerm::new("Morgan Stanley").unwrap(),
    ));
    assert_firm_type(&firm);
    assert_descriptor(&firm, "tipranks-firm-summary");
}

fn assert_descriptor<Q, R>(endpoint: &EndpointSpec<Q, R>, identity: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), identity);
    assert_eq!(endpoint.relative_path(), identity);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Unspecified
    );
    assert_eq!(
        endpoint.metadata().access(),
        AccessRequirement::NamedAddOn("TipRanks")
    );
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new().with_response_rows(1)
    );
    assert!(endpoint.metadata().bounds().accepts_response_rows(1));
    assert!(!endpoint.metadata().bounds().accepts_response_rows(2));
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_symbol_type(_: &EndpointSpec<TipRanksSymbolSummaryQuery, Vec<TipRanksSymbolSummary>>) {}

fn assert_analyst_type(_: &EndpointSpec<TipRanksAnalystSummaryQuery, Vec<TipRanksAnalystSummary>>) {
}

fn assert_firm_type(_: &EndpointSpec<TipRanksFirmSummaryQuery, Vec<TipRanksFirmSummary>>) {}

#[tokio::test]
async fn custom_proxy_sends_one_exact_request_per_method_with_auth_headers_and_encoding() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(SYMBOL),
        json_fixture(ANALYST),
        json_fixture(FIRM),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Bearer ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "tipranks summaries")
        .executor(executor.clone())
        .build()
        .unwrap();
    let from = Date::parse("2025-06-10").unwrap();
    let to = Date::parse("2026-06-10").unwrap();

    let symbol = client
        .tipranks_symbol_summary(
            TipRanksSymbolSummaryQuery::new(Ticker::new("BRK.B").unwrap())
                .with_from(from)
                .with_to(to),
        )
        .await
        .unwrap();
    assert_eq!(symbol.len(), 1);
    assert_eq!(symbol[0].symbol.as_str(), "AAPL");

    let analyst = client
        .tipranks_analyst_summary(
            TipRanksAnalystSummaryQuery::new(TipRanksExpertUid::new("expert / one").unwrap())
                .with_from(from)
                .with_to(to),
        )
        .await
        .unwrap();
    assert_eq!(analyst.len(), 1);
    assert_eq!(
        analyst[0].expert_uid.as_str(),
        "3c6eb8cf1347a4e5757e628fccb684a93abee587"
    );

    let firm = client
        .tipranks_firm_summary(
            TipRanksFirmSummaryQuery::new(SearchTerm::new("Morgan Stanley / Asia").unwrap())
                .with_from(from)
                .with_to(to),
        )
        .await
        .unwrap();
    assert_eq!(firm.len(), 1);
    assert_eq!(firm[0].firm_name, "Morgan Stanley");

    let requests = executor.requests();
    assert_eq!(requests.len(), 3);
    for request in requests.iter() {
        assert_eq!(request.method(), HttpMethod::Get);
        assert_eq!(
            request.expose_headers()["x-router-token"],
            "Bearer proxy-secret"
        );
        assert_eq!(
            request.expose_headers()["x-data-scope"],
            "tipranks summaries"
        );
    }
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://proxy.example/router/gateway/stable/tipranks-symbol-summary?symbol=BRK.B&from=2025-06-10&to=2026-06-10"
    );
    assert_eq!(
        requests[1].expose_url().as_str(),
        "https://proxy.example/router/gateway/stable/tipranks-analyst-summary?expertUID=expert+%2F+one&from=2025-06-10&to=2026-06-10"
    );
    assert_eq!(
        requests[2].expose_url().as_str(),
        "https://proxy.example/router/gateway/stable/tipranks-firm-summary?firmName=Morgan+Stanley+%2F+Asia&from=2025-06-10&to=2026-06-10"
    );
}

#[tokio::test]
async fn required_only_query_auth_is_appended_after_identity_without_date_defaults() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(SYMBOL)]));
    let client = Client::builder()
        .authentication(Authentication::fmp_query("query-secret"))
        .executor(executor.clone())
        .build()
        .unwrap();

    client
        .tipranks_symbol_summary(TipRanksSymbolSummaryQuery::new(
            Ticker::new("AAPL").unwrap(),
        ))
        .await
        .unwrap();

    let requests = executor.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://financialmodelingprep.com/stable/tipranks-symbol-summary?symbol=AAPL&apikey=query-secret"
    );
    assert!(!requests[0].expose_headers().contains_key("apikey"));
}

#[tokio::test]
async fn empty_array_succeeds_and_malformed_root_keeps_summary_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"[]"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();
    let query = || TipRanksFirmSummaryQuery::new(SearchTerm::new("Morgan Stanley").unwrap());

    assert!(
        client
            .tipranks_firm_summary(query())
            .await
            .unwrap()
            .is_empty()
    );
    let error = client.tipranks_firm_summary(query()).await.unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Decode);
    assert_eq!(error.endpoint(), Some("tipranks-firm-summary"));
    assert_eq!(error.status_code(), Some(200));
    assert_eq!(executor.requests().len(), 2);
}
