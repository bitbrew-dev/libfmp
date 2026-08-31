mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    codecs::DynamicObject,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        sec_filings::{
            AllIndustryClassificationsQuery, IndustryClassificationSearchQuery,
            IndustryClassificationsQuery, all_industry_classifications, industry_classifications,
            search_industry_classifications,
        },
    },
    error::ErrorCategory,
    responses::sec_filings::{SecCompanySearchResult, SicClassification},
    transport::HttpMethod,
    types::{Cik, Limit, Page, SearchTerm, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const CLASSIFICATIONS: &[u8] = include_bytes!("fixtures/industry_classifications.json");
const SEARCH: &[u8] = include_bytes!("fixtures/industry_classification_search.json");
const ALL: &[u8] = include_bytes!("fixtures/all_industry_classifications.json");

#[test]
fn descriptors_use_exact_get_paths_bare_rows_and_only_us_metadata() {
    let classifications = industry_classifications(IndustryClassificationsQuery::new());
    let search = search_industry_classifications(IndustryClassificationSearchQuery::new());
    let all = all_industry_classifications(AllIndustryClassificationsQuery::new());

    assert_facts(&classifications, "standard-industrial-classification-list");
    assert_facts(&search, "industry-classification-search");
    assert_facts(&all, "all-industry-classification");
    assert_response_types(&classifications, &search, &all);
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
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

fn assert_response_types(
    _: &EndpointSpec<IndustryClassificationsQuery, Vec<SicClassification>>,
    _: &EndpointSpec<IndustryClassificationSearchQuery, Vec<DynamicObject>>,
    _: &EndpointSpec<AllIndustryClassificationsQuery, Vec<SecCompanySearchResult>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_paths_query_order_encoding_headers_and_fixture_identity() {
    let executor = Arc::new(FixtureExecutor::new(classification_fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "sec-classifications")
        .executor(executor.clone())
        .build()
        .unwrap();

    let classifications = client
        .industry_classifications(
            IndustryClassificationsQuery::new()
                .with_industry_title(SearchTerm::new("SERVICES, NEC / OTHER").unwrap())
                .with_sic_code(SearchTerm::new("07371").unwrap()),
        )
        .await
        .unwrap();
    let search = client
        .search_industry_classifications(
            IndustryClassificationSearchQuery::new()
                .with_symbol(Ticker::new("BRK.B / Class A").unwrap())
                .with_cik(Cik::new("0000320193").unwrap())
                .with_sic_code(SearchTerm::new("07371").unwrap()),
        )
        .await
        .unwrap();
    let all = client
        .all_industry_classifications(
            AllIndustryClassificationsQuery::new()
                .with_page(Page(0))
                .with_limit(Limit(u32::MAX)),
        )
        .await
        .unwrap();

    assert_eq!(classifications[0].office, "Office of Life Sciences");
    assert_eq!(classifications[0].sic_code, "100");
    assert_eq!(
        classifications[0].industry_title,
        "AGRICULTURAL PRODUCTION-CROPS"
    );
    assert_eq!(search, [DynamicObject::new()]);
    assert_eq!(all[0].symbol.as_str(), "0Q16.L");
    assert_eq!(all[0].cik.as_str(), "0000070858");
    assert_eq!(
        all[0].business_address,
        "['BANK OF AMERICA CORPORATE CENTER', 'CHARLOTTE NC 28255']"
    );

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "sec-classifications"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/standard-industrial-classification-list?industryTitle=SERVICES%2C+NEC+%2F+OTHER&sicCode=07371",
            "https://proxy.example/router/gateway/stable/industry-classification-search?symbol=BRK.B+%2F+Class+A&cik=0000320193&sicCode=07371",
            "https://proxy.example/router/gateway/stable/all-industry-classification?page=0&limit=4294967295",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_omission_and_zero_values() {
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
        let executor = Arc::new(FixtureExecutor::new(classification_fixtures()));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .industry_classifications(IndustryClassificationsQuery::new())
            .await
            .unwrap();
        client
            .search_industry_classifications(IndustryClassificationSearchQuery::new())
            .await
            .unwrap();
        client
            .all_industry_classifications(
                AllIndustryClassificationsQuery::new()
                    .with_page(Page(0))
                    .with_limit(Limit(0)),
            )
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            request_urls(&requests),
            [
                format!(
                    "https://financialmodelingprep.com/stable/standard-industrial-classification-list{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/industry-classification-search{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/all-industry-classification?page=0&limit=0{}",
                    if suffix.is_empty() {
                        ""
                    } else {
                        "&apikey=query-secret"
                    }
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
async fn malformed_non_array_responses_keep_each_endpoint_identity() {
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

    let errors = [
        client
            .industry_classifications(IndustryClassificationsQuery::new())
            .await
            .unwrap_err(),
        client
            .search_industry_classifications(IndustryClassificationSearchQuery::new())
            .await
            .unwrap_err(),
        client
            .all_industry_classifications(AllIndustryClassificationsQuery::new())
            .await
            .unwrap_err(),
    ];

    for (error, id) in errors.iter().zip([
        "standard-industrial-classification-list",
        "industry-classification-search",
        "all-industry-classification",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
}

fn classification_fixtures() -> [libfmp::transport::TransportResponse; 3] {
    [
        json_fixture(CLASSIFICATIONS),
        json_fixture(SEARCH),
        json_fixture(ALL),
    ]
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}
