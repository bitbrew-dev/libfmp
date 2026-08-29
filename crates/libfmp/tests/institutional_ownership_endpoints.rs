mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        institutional_ownership::{
            Form13fFilingDatesQuery, InstitutionalOwnershipExtractQuery,
            LatestInstitutionalOwnershipFilingsQuery, form_13f_filing_dates,
            institutional_ownership_extract, latest_institutional_ownership_filings,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    query::{Quarter, Year},
    responses::institutional_ownership::{
        Form13fFilingDate, InstitutionalHolding, InstitutionalOwnershipFiling,
    },
    transport::HttpMethod,
    types::{Cik, Limit, Page},
};

use support::{FixtureExecutor, json_fixture};

const LATEST: &[u8] = include_bytes!("fixtures/latest_institutional_ownership_filings.json");
const EXTRACT: &[u8] = include_bytes!("fixtures/institutional_ownership_extract.json");
const DATES: &[u8] = include_bytes!("fixtures/form_13f_filing_dates.json");

#[test]
fn descriptors_use_exact_paths_bare_rows_and_only_documented_metadata() {
    let latest =
        latest_institutional_ownership_filings(LatestInstitutionalOwnershipFilingsQuery::new());
    assert_facts(
        &latest,
        "institutional-ownership/latest",
        EndpointBounds::new().with_page(100),
    );
    let bounds = latest.metadata().bounds();
    assert!(bounds.accepts_page(Page(0)));
    assert!(bounds.accepts_page(Page(100)));
    assert!(!bounds.accepts_page(Page(101)));
    assert!(bounds.accepts_limit(Limit(u32::MAX)));

    let extract = institutional_ownership_extract(InstitutionalOwnershipExtractQuery::new(
        Cik::new("0001388838").unwrap(),
        Year(2023),
        Quarter::Q3,
    ));
    assert_facts(
        &extract,
        "institutional-ownership/extract",
        EndpointBounds::new(),
    );

    let dates = form_13f_filing_dates(Cik::new("0001067983").unwrap().into());
    assert_facts(
        &dates,
        "institutional-ownership/dates",
        EndpointBounds::new(),
    );
    assert_response_types(&latest, &extract, &dates);
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
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), bounds);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_response_types(
    _: &EndpointSpec<LatestInstitutionalOwnershipFilingsQuery, Vec<InstitutionalOwnershipFiling>>,
    _: &EndpointSpec<InstitutionalOwnershipExtractQuery, Vec<InstitutionalHolding>>,
    _: &EndpointSpec<Form13fFilingDatesQuery, Vec<Form13fFilingDate>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_paths_query_order_auth_headers_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(LATEST),
        json_fixture(EXTRACT),
        json_fixture(DATES),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "institutional-core")
        .executor(executor.clone())
        .build()
        .unwrap();

    let latest = client
        .latest_institutional_ownership_filings(
            LatestInstitutionalOwnershipFilingsQuery::new()
                .with_page(Page(0))
                .with_limit(Limit(100)),
        )
        .await
        .unwrap();
    let holdings = client
        .institutional_ownership_extract(InstitutionalOwnershipExtractQuery::new(
            Cik::new("0001388838").unwrap(),
            Year(2023),
            Quarter::Q3,
        ))
        .await
        .unwrap();
    let dates = client
        .form_13f_filing_dates(Cik::new("0001067983").unwrap())
        .await
        .unwrap();

    assert_eq!(latest[0].cik.as_str(), "0001803005");
    assert_eq!(holdings[0].security_cusip.as_str(), "674215207");
    assert_eq!(dates[0].quarter.get(), 1);

    let requests = executor.requests();
    assert_eq!(requests.len(), 3);
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "institutional-core"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/institutional-ownership/latest?page=0&limit=100",
            "https://proxy.example/router/gateway/stable/institutional-ownership/extract?cik=0001388838&year=2023&quarter=3",
            "https://proxy.example/router/gateway/stable/institutional-ownership/dates?cik=0001067983",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_page_and_limit_boundary_requests() {
    for (authentication, first_auth, later_auth, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "?apikey=query-secret",
            "&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(LATEST),
            json_fixture(LATEST),
            json_fixture(LATEST),
            json_fixture(LATEST),
            json_fixture(EXTRACT),
            json_fixture(DATES),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        for query in [
            LatestInstitutionalOwnershipFilingsQuery::new(),
            LatestInstitutionalOwnershipFilingsQuery::new().with_page(Page(0)),
            LatestInstitutionalOwnershipFilingsQuery::new().with_page(Page(100)),
            LatestInstitutionalOwnershipFilingsQuery::new()
                .with_page(Page(101))
                .with_limit(Limit(u32::MAX)),
        ] {
            assert_eq!(
                client
                    .latest_institutional_ownership_filings(query)
                    .await
                    .unwrap()
                    .len(),
                1
            );
        }
        client
            .institutional_ownership_extract(InstitutionalOwnershipExtractQuery::new(
                Cik::new("0001388838").unwrap(),
                Year(2023),
                Quarter::Q3,
            ))
            .await
            .unwrap();
        client
            .form_13f_filing_dates(Cik::new("0001067983").unwrap())
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
                    "https://financialmodelingprep.com/stable/institutional-ownership/latest{first_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/latest?page=0{later_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/latest?page=100{later_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/latest?page=101&limit=4294967295{later_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/extract?cik=0001388838&year=2023&quarter=3{later_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/dates?cik=0001067983{later_auth}"
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
