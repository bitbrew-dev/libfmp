mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{
            AccessRequirement, ConditionalPlanRequirement, EndpointBounds, GeographicAvailability,
            PlanCondition,
        },
        tipranks::{TipRanksSearchQuery, tipranks_ratings_search},
    },
    error::ErrorCategory,
    responses::tipranks::TipRanksRatingSearchResult,
    transport::HttpMethod,
    types::{Date, Limit, Page, StringValueError, Ticker, TipRanksExpertUid},
};

use support::{FixtureExecutor, json_fixture};

const SEARCH: &[u8] = include_bytes!("fixtures/tipranks_ratings_search.json");

#[test]
fn descriptor_has_exact_path_response_type_and_tipranks_access_metadata() {
    let endpoint = tipranks_ratings_search(TipRanksSearchQuery::new());
    assert_response_type(&endpoint);
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), "tipranks-search");
    assert_eq!(endpoint.relative_path(), "tipranks-search");
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Unspecified
    );
    assert_eq!(
        endpoint.metadata().access(),
        AccessRequirement::NamedAddOn("TipRanks")
    );
    assert_eq!(
        endpoint.metadata().conditional_plan(),
        Some(ConditionalPlanRequirement::new(
            "Enterprise",
            PlanCondition::HistoryOlderThanYears(3)
        ))
    );
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new()
            .with_limit(5_000)
            .with_response_rows(5_000)
    );
    assert!(endpoint.metadata().bounds().accepts_limit(Limit(5_000)));
    assert!(!endpoint.metadata().bounds().accepts_limit(Limit(5_001)));
    assert!(endpoint.metadata().bounds().accepts_response_rows(5_000));
    assert!(!endpoint.metadata().bounds().accepts_response_rows(5_001));
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_response_type(_: &EndpointSpec<TipRanksSearchQuery, Vec<TipRanksRatingSearchResult>>) {}

#[test]
fn expert_uid_is_open_validated_and_representation_preserving() {
    for invalid in ["", "  "] {
        assert_eq!(
            TipRanksExpertUid::new(invalid).unwrap_err(),
            StringValueError::Empty
        );
    }
    assert_eq!(
        TipRanksExpertUid::new("expert\nuid").unwrap_err(),
        StringValueError::ControlCharacter
    );

    for value in ["9d6962", "0001", "expert uid/alpha", "a,b", "-1"] {
        let expert_uid = TipRanksExpertUid::new(value).unwrap();
        assert_eq!(expert_uid.as_str(), value);
        assert_eq!(expert_uid.to_string(), value);
        assert_eq!(
            serde_json::to_string(&expert_uid).unwrap(),
            format!("\"{value}\"")
        );
        let decoded: TipRanksExpertUid = serde_json::from_str(&format!("\"{value}\"")).unwrap();
        assert_eq!(decoded, expert_uid);
    }
}

#[tokio::test]
async fn custom_proxy_sends_one_exact_full_query_request_with_auth_header_and_fixture() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(SEARCH)]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Bearer ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "tipranks ratings")
        .executor(executor.clone())
        .build()
        .unwrap();
    let query = TipRanksSearchQuery::new()
        .with_expert_uid(TipRanksExpertUid::new("expert / one").unwrap())
        .with_symbol(Ticker::new("RR.L").unwrap())
        .with_from(Date::parse("2025-06-10").unwrap())
        .with_to(Date::parse("2026-06-10").unwrap())
        .with_limit(Limit(5_000))
        .with_page(Page(0))
        .with_nonadjusted(false);

    let rows = client.tipranks_ratings_search(query).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].symbol.as_str(), "RR.L");
    assert_eq!(
        rows[0].expert_uid.as_str(),
        "9d6962cbd29862b8d70de0a2ddb3eb0bdfedc2b7"
    );

    let requests = executor.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method(), HttpMethod::Get);
    assert_eq!(
        requests[0].expose_headers()["x-router-token"],
        "Bearer proxy-secret"
    );
    assert_eq!(
        requests[0].expose_headers()["x-data-scope"],
        "tipranks ratings"
    );
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://proxy.example/router/gateway/stable/tipranks-search?expertUID=expert+%2F+one&symbol=RR.L&from=2025-06-10&to=2026-06-10&limit=5000&page=0&nonadjusted=false"
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_omission_and_exact_auth_position() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/tipranks-search",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/tipranks-search?apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(SEARCH)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        let rows = client
            .tipranks_ratings_search(TipRanksSearchQuery::new())
            .await
            .unwrap();
        assert_eq!(rows[0].date.as_str(), "2026-07-30T16:40:58.403Z");

        let requests = executor.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some(value) => assert_eq!(requests[0].expose_headers()["apikey"], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

#[tokio::test]
async fn bare_empty_array_decodes_and_malformed_root_keeps_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"[]"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();

    assert!(
        client
            .tipranks_ratings_search(TipRanksSearchQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
    let error = client
        .tipranks_ratings_search(TipRanksSearchQuery::new())
        .await
        .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Decode);
    assert_eq!(error.endpoint(), Some("tipranks-search"));
    assert_eq!(error.status_code(), Some(200));
    assert_eq!(executor.requests().len(), 2);
}
