mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        funds::{
            EtfAssetExposureQuery, EtfCountryWeightingsQuery, EtfSectorWeightingsQuery,
            etf_asset_exposure, etf_country_weightings, etf_sector_weightings,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::funds::{EtfAssetExposure, EtfCountryWeighting, EtfSectorWeighting},
    transport::HttpMethod,
    types::Ticker,
};

use support::{FixtureExecutor, json_fixture};

const COUNTRY: &[u8] = include_bytes!("fixtures/etf_country_weightings.json");
const ASSET: &[u8] = include_bytes!("fixtures/etf_asset_exposure.json");
const SECTOR: &[u8] = include_bytes!("fixtures/etf_sector_weightings.json");

#[test]
fn descriptors_use_exact_paths_bare_types_and_only_documented_metadata() {
    let country =
        etf_country_weightings(EtfCountryWeightingsQuery::new(Ticker::new("SPY").unwrap()));
    let asset = etf_asset_exposure(EtfAssetExposureQuery::new(Ticker::new("AAPL").unwrap()));
    let sector = etf_sector_weightings(EtfSectorWeightingsQuery::new(Ticker::new("SPY").unwrap()));

    assert_facts(&country, "etf/country-weightings");
    assert_facts(&asset, "etf/asset-exposure");
    assert_facts(&sector, "etf/sector-weightings");
    assert_response_types(&country, &asset, &sector);
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, Vec<R>>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_response_types(
    _: &EndpointSpec<EtfCountryWeightingsQuery, Vec<EtfCountryWeighting>>,
    _: &EndpointSpec<EtfAssetExposureQuery, Vec<EtfAssetExposure>>,
    _: &EndpointSpec<EtfSectorWeightingsQuery, Vec<EtfSectorWeighting>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_paths_encoding_headers_and_fixture_values() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(COUNTRY),
        json_fixture(ASSET),
        json_fixture(SECTOR),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "fund-allocations")
        .executor(executor.clone())
        .build()
        .unwrap();

    let country = client
        .etf_country_weightings(EtfCountryWeightingsQuery::new(
            Ticker::new("000089.SZ").unwrap(),
        ))
        .await
        .unwrap();
    let asset = client
        .etf_asset_exposure(EtfAssetExposureQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
        ))
        .await
        .unwrap();
    let sector = client
        .etf_sector_weightings(EtfSectorWeightingsQuery::new(
            Ticker::new("ZWT-T.TO").unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(country.len(), 1);
    assert_eq!(country[0].country, "United States");
    assert_eq!(country[0].weight_percentage.as_str(), "97.26%");
    assert_eq!(asset.len(), 1);
    assert_eq!(asset[0].symbol.as_str(), "ZWT-T.TO");
    assert_eq!(asset[0].asset.as_str(), "AAPL");
    assert_eq!(asset[0].shares_number, 42_372.0);
    assert_eq!(asset[0].weight_percentage, 10.1);
    assert_eq!(asset[0].market_value, 20_141_231.66);
    assert_eq!(sector.len(), 1);
    assert_eq!(sector[0].symbol.as_str(), "SPY");
    assert_eq!(sector[0].sector.as_str(), "Basic Materials");
    assert_eq!(sector[0].weight_percentage, 1.6916311902850854);

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "fund-allocations"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/etf/country-weightings?symbol=000089.SZ",
            "https://proxy.example/router/gateway/stable/etf/asset-exposure?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/etf/sector-weightings?symbol=ZWT-T.TO",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_all_exact_urls() {
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
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(COUNTRY),
            json_fixture(ASSET),
            json_fixture(SECTOR),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .etf_country_weightings(EtfCountryWeightingsQuery::new(Ticker::new("SPY").unwrap()))
            .await
            .unwrap();
        client
            .etf_asset_exposure(EtfAssetExposureQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap();
        client
            .etf_sector_weightings(EtfSectorWeightingsQuery::new(Ticker::new("SPY").unwrap()))
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
                    "https://financialmodelingprep.com/stable/etf/country-weightings?symbol=SPY{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/etf/asset-exposure?symbol=AAPL{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/etf/sector-weightings?symbol=SPY{suffix}"
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
async fn malformed_non_arrays_keep_each_endpoint_identity() {
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
            .etf_country_weightings(EtfCountryWeightingsQuery::new(Ticker::new("SPY").unwrap()))
            .await
            .unwrap_err(),
        client
            .etf_asset_exposure(EtfAssetExposureQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap_err(),
        client
            .etf_sector_weightings(EtfSectorWeightingsQuery::new(Ticker::new("SPY").unwrap()))
            .await
            .unwrap_err(),
    ];
    for (error, endpoint) in errors.iter().zip([
        "etf/country-weightings",
        "etf/asset-exposure",
        "etf/sector-weightings",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
