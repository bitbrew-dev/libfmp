mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        sec_filings::{
            SecCompaniesByCikQuery, SecCompaniesByNameQuery, SecCompaniesBySymbolQuery,
            SecCompanyProfileQuery, search_sec_companies_by_cik, search_sec_companies_by_name,
            search_sec_companies_by_symbol, sec_company_profile,
        },
    },
    error::ErrorCategory,
    responses::sec_filings::{SecCompanyProfile, SecCompanySearchResult},
    transport::HttpMethod,
    types::{Cik, SearchTerm, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const BY_NAME: &[u8] = include_bytes!("fixtures/sec_companies_by_name.json");
const BY_SYMBOL: &[u8] = include_bytes!("fixtures/sec_companies_by_symbol.json");
const BY_CIK: &[u8] = include_bytes!("fixtures/sec_companies_by_cik.json");
const PROFILE: &[u8] = include_bytes!("fixtures/sec_company_profile.json");

#[test]
fn descriptors_use_exact_get_paths_bare_rows_and_only_us_metadata() {
    let by_name = search_sec_companies_by_name(SecCompaniesByNameQuery::new(
        SearchTerm::new("Berkshire").unwrap(),
    ));
    let by_symbol = search_sec_companies_by_symbol(SecCompaniesBySymbolQuery::new(
        Ticker::new("AAPL").unwrap(),
    ));
    let by_cik =
        search_sec_companies_by_cik(SecCompaniesByCikQuery::new(Cik::new("0000320193").unwrap()));
    let profile = sec_company_profile(SecCompanyProfileQuery::new(Ticker::new("AAPL").unwrap()));

    assert_search_facts(&by_name, "sec-filings-company-search/name");
    assert_search_facts(&by_symbol, "sec-filings-company-search/symbol");
    assert_search_facts(&by_cik, "sec-filings-company-search/cik");
    assert_profile_facts(&profile, "sec-profile");
    assert_response_types(&by_name, &by_symbol, &by_cik, &profile);
}

fn assert_metadata<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_search_facts<Q>(
    endpoint: &EndpointSpec<Q, Vec<SecCompanySearchResult>>,
    path: &'static str,
) {
    assert_metadata(endpoint, path);
}

fn assert_profile_facts(
    endpoint: &EndpointSpec<SecCompanyProfileQuery, Vec<SecCompanyProfile>>,
    path: &'static str,
) {
    assert_metadata(endpoint, path);
}

fn assert_response_types(
    _: &EndpointSpec<SecCompaniesByNameQuery, Vec<SecCompanySearchResult>>,
    _: &EndpointSpec<SecCompaniesBySymbolQuery, Vec<SecCompanySearchResult>>,
    _: &EndpointSpec<SecCompaniesByCikQuery, Vec<SecCompanySearchResult>>,
    _: &EndpointSpec<SecCompanyProfileQuery, Vec<SecCompanyProfile>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_paths_encoding_headers_and_fixture_identities() {
    let executor = Arc::new(FixtureExecutor::new(company_fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "sec-companies")
        .executor(executor.clone())
        .build()
        .unwrap();

    let by_name = client
        .search_sec_companies_by_name(SecCompaniesByNameQuery::new(
            SearchTerm::new("Berkshire, Hathaway / Fund").unwrap(),
        ))
        .await
        .unwrap();
    let by_symbol = client
        .search_sec_companies_by_symbol(SecCompaniesBySymbolQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
        ))
        .await
        .unwrap();
    let by_cik = client
        .search_sec_companies_by_cik(SecCompaniesByCikQuery::new(Cik::new("0000320193").unwrap()))
        .await
        .unwrap();
    let profile = client
        .sec_company_profile(
            SecCompanyProfileQuery::new(Ticker::new("AAPL").unwrap())
                .with_cik_a(Cik::new("0000320193").unwrap()),
        )
        .await
        .unwrap();

    assert_eq!(by_name[0].symbol.as_str(), "None");
    assert_eq!(by_name[0].cik.as_str(), "0001418405");
    assert_eq!(by_name[0].sic_code, "");
    assert_eq!(by_name[0].industry_title, "");
    assert_eq!(
        by_name[0].business_address,
        "c/o Berkshire Property Advisors LLC, Boston MA 02108"
    );
    assert_eq!(by_symbol[0].symbol.as_str(), "AAPL");
    assert_eq!(by_symbol[0].name, "APPLE INC.");
    assert_eq!(by_cik[0].cik.as_str(), "0000320193");
    assert_eq!(by_cik[0], by_symbol[0]);
    assert_eq!(profile[0].symbol.as_str(), "AAPL");
    assert_eq!(profile[0].cik.as_str(), "0000320193");
    assert_eq!(profile[0].security_type, None);
    assert_eq!(
        serde_json::to_value(&profile[0])
            .unwrap()
            .as_object()
            .unwrap()
            .len(),
        35
    );

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "sec-companies"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/sec-filings-company-search/name?company=Berkshire%2C+Hathaway+%2F+Fund",
            "https://proxy.example/router/gateway/stable/sec-filings-company-search/symbol?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/sec-filings-company-search/cik?cik=0000320193",
            "https://proxy.example/router/gateway/stable/sec-profile?symbol=AAPL&cik-A=0000320193",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_profile_cik_a_omission() {
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
        let executor = Arc::new(FixtureExecutor::new(company_fixtures()));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .search_sec_companies_by_name(SecCompaniesByNameQuery::new(
                SearchTerm::new("Berkshire").unwrap(),
            ))
            .await
            .unwrap();
        client
            .search_sec_companies_by_symbol(SecCompaniesBySymbolQuery::new(
                Ticker::new("AAPL").unwrap(),
            ))
            .await
            .unwrap();
        client
            .search_sec_companies_by_cik(SecCompaniesByCikQuery::new(
                Cik::new("0000320193").unwrap(),
            ))
            .await
            .unwrap();
        client
            .sec_company_profile(SecCompanyProfileQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            request_urls(&requests),
            [
                format!(
                    "https://financialmodelingprep.com/stable/sec-filings-company-search/name?company=Berkshire{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/sec-filings-company-search/symbol?symbol=AAPL{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/sec-filings-company-search/cik?cik=0000320193{suffix}"
                ),
                format!("https://financialmodelingprep.com/stable/sec-profile?symbol=AAPL{suffix}"),
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
async fn malformed_non_array_responses_keep_all_four_endpoint_identities() {
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
            .search_sec_companies_by_name(SecCompaniesByNameQuery::new(
                SearchTerm::new("Berkshire").unwrap(),
            ))
            .await
            .unwrap_err(),
        client
            .search_sec_companies_by_symbol(SecCompaniesBySymbolQuery::new(
                Ticker::new("AAPL").unwrap(),
            ))
            .await
            .unwrap_err(),
        client
            .search_sec_companies_by_cik(SecCompaniesByCikQuery::new(
                Cik::new("0000320193").unwrap(),
            ))
            .await
            .unwrap_err(),
        client
            .sec_company_profile(SecCompanyProfileQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap_err(),
    ];

    for (error, id) in errors.iter().zip([
        "sec-filings-company-search/name",
        "sec-filings-company-search/symbol",
        "sec-filings-company-search/cik",
        "sec-profile",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
}

fn company_fixtures() -> [libfmp::transport::TransportResponse; 4] {
    [
        json_fixture(BY_NAME),
        json_fixture(BY_SYMBOL),
        json_fixture(BY_CIK),
        json_fixture(PROFILE),
    ]
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}
