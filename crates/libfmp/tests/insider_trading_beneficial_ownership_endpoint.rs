mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        insider_trading::{
            BeneficialOwnershipAcquisitionsQuery, beneficial_ownership_acquisitions,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::insider_trading::BeneficialOwnershipAcquisition,
    transport::HttpMethod,
    types::{Date, Limit, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const OWNERSHIP: &[u8] = include_bytes!("fixtures/beneficial_ownership_acquisitions.json");
const PATH: &str = "acquisition-of-beneficial-ownership";

#[test]
fn descriptor_uses_exact_path_bare_rows_and_only_documented_metadata() {
    let endpoint = beneficial_ownership_acquisitions(BeneficialOwnershipAcquisitionsQuery::new(
        Ticker::new("AAPL").unwrap(),
    ));

    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), PATH);
    assert_eq!(endpoint.relative_path(), PATH);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    assert_response_type(&endpoint);
}

fn assert_response_type(
    _: &EndpointSpec<BeneficialOwnershipAcquisitionsQuery, Vec<BeneficialOwnershipAcquisition>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_order_encoding_headers_and_exact_fixture_identity() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(OWNERSHIP)]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "beneficial ownership")
        .executor(executor.clone())
        .build()
        .unwrap();

    let rows = client
        .beneficial_ownership_acquisitions(
            BeneficialOwnershipAcquisitionsQuery::new(Ticker::new("BRK.B / Class A").unwrap())
                .with_limit(Limit(u32::MAX)),
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.cik.as_str(), "0000320193");
    assert_eq!(row.cusip.as_ref().unwrap().as_str(), "037833100");
    assert_eq!(row.filing_date, Date::parse("2026-04-29").unwrap());
    assert_eq!(row.accepted_date, Date::parse("2026-04-29").unwrap());
    assert_eq!(row.sole_voting_power.as_str(), "0");
    assert_eq!(row.shared_voting_power.as_ref().unwrap().as_str(), "0");
    assert_eq!(row.sole_dispositive_power.as_str(), "0");
    assert_eq!(row.shared_dispositive_power.as_str(), "0");
    assert_eq!(row.amount_beneficially_owned.as_str(), "1099168953");
    assert_eq!(row.percent_of_class.as_str(), "7.48");
    assert_eq!(
        serde_json::to_value(&rows).unwrap(),
        serde_json::from_slice::<serde_json::Value>(OWNERSHIP).unwrap()
    );

    let requests = executor.requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request.method(), HttpMethod::Get);
    assert_eq!(
        request.expose_headers()["x-router-token"],
        "Token proxy-secret"
    );
    assert_eq!(
        request.expose_headers()["x-data-scope"],
        "beneficial ownership"
    );
    assert_eq!(
        request.expose_url().as_str(),
        format!(
            "https://proxy.example/router/gateway/stable/{PATH}?symbol=BRK.B+%2F+Class+A&limit={}",
            u32::MAX
        )
    );
}

#[tokio::test]
async fn direct_auth_preserves_omission_and_zero_limit() {
    for (authentication, limit, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            None,
            format!("https://financialmodelingprep.com/stable/{PATH}?symbol=AAPL"),
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            Some(Limit(0)),
            format!(
                "https://financialmodelingprep.com/stable/{PATH}?symbol=AAPL&limit=0&apikey=query-secret"
            ),
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(OWNERSHIP)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let mut query = BeneficialOwnershipAcquisitionsQuery::new(Ticker::new("AAPL").unwrap());
        if let Some(limit) = limit {
            query = query.with_limit(limit);
        }

        client
            .beneficial_ownership_acquisitions(query)
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some(value) => assert_eq!(requests[0].expose_headers()["apikey"], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

#[tokio::test]
async fn malformed_non_array_response_keeps_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(b"{}")]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let error = client
        .beneficial_ownership_acquisitions(Ticker::new("AAPL").unwrap())
        .await
        .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Decode);
    assert_eq!(error.endpoint(), Some(PATH));
    assert_eq!(error.status_code(), Some(200));
}
