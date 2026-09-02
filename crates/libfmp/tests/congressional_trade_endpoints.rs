mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        congressional::{
            CongressionalTradesByMemberIdQuery, CongressionalTradesByNameQuery,
            CongressionalTradesQuery, LatestCongressionalDisclosuresQuery, house_trades,
            house_trades_by_member_id, house_trades_by_name, latest_house_disclosures,
            latest_senate_disclosures, senate_trades, senate_trades_by_member_id,
            senate_trades_by_name,
        },
        metadata::{AccessRequirement, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::congressional::CongressionalTrade,
    transport::HttpMethod,
    types::{CongressionalMemberId, Limit, Page, SearchTerm, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const FIXTURES: [&[u8]; 8] = [
    include_bytes!("fixtures/congress_senate_latest.json"),
    include_bytes!("fixtures/congress_house_latest.json"),
    include_bytes!("fixtures/congress_senate_trades.json"),
    include_bytes!("fixtures/congress_senate_trades_by_name.json"),
    include_bytes!("fixtures/congress_senate_trades_by_id.json"),
    include_bytes!("fixtures/congress_house_trades.json"),
    include_bytes!("fixtures/congress_house_trades_by_name.json"),
    include_bytes!("fixtures/congress_house_trades_by_id.json"),
];

const PATHS: [&str; 8] = [
    "senate-latest",
    "house-latest",
    "senate-trades",
    "senate-trades-by-name",
    "senate-trades-by-id",
    "house-trades",
    "house-trades-by-name",
    "house-trades-by-id",
];

#[test]
fn descriptors_use_exact_get_paths_typed_rows_and_only_documented_metadata() {
    let latest_senate_disclosures =
        latest_senate_disclosures(LatestCongressionalDisclosuresQuery::new());
    let latest_house_disclosures =
        latest_house_disclosures(LatestCongressionalDisclosuresQuery::new());
    let senate_trades = senate_trades(CongressionalTradesQuery::new(ticker("AAPL")));
    let senate_name = senate_trades_by_name(CongressionalTradesByNameQuery::new(term("Jerry")));
    let senate_id = senate_trades_by_member_id(CongressionalTradesByMemberIdQuery::new());
    let house_trades = house_trades(CongressionalTradesQuery::new(ticker("AAPL")));
    let house_name = house_trades_by_name(CongressionalTradesByNameQuery::new(term("James")));
    let house_id = house_trades_by_member_id(CongressionalTradesByMemberIdQuery::new());

    assert_paginated(&latest_senate_disclosures, PATHS[0]);
    assert_paginated(&latest_house_disclosures, PATHS[1]);
    assert_paginated(&senate_trades, PATHS[2]);
    assert_unbounded(&senate_name, PATHS[3]);
    assert_paginated(&senate_id, PATHS[4]);
    assert_paginated(&house_trades, PATHS[5]);
    assert_unbounded(&house_name, PATHS[6]);
    assert_paginated(&house_id, PATHS[7]);
    assert_response_types(
        &latest_senate_disclosures,
        &latest_house_disclosures,
        &senate_trades,
        &senate_name,
        &senate_id,
        &house_trades,
        &house_name,
        &house_id,
    );
}

fn assert_common<Q>(endpoint: &EndpointSpec<Q, Vec<CongressionalTrade>>, path: &str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    assert_eq!(endpoint.metadata().bounds().limit(), None);
    assert_eq!(endpoint.metadata().bounds().date_range_days(), None);
}

fn assert_paginated<Q>(endpoint: &EndpointSpec<Q, Vec<CongressionalTrade>>, path: &str) {
    assert_common(endpoint, path);
    assert_eq!(
        endpoint
            .metadata()
            .bounds()
            .response_rows()
            .unwrap()
            .maximum(),
        250
    );
    assert_eq!(endpoint.metadata().bounds().page().unwrap().maximum(), 100);
}

fn assert_unbounded<Q>(endpoint: &EndpointSpec<Q, Vec<CongressionalTrade>>, path: &str) {
    assert_common(endpoint, path);
    assert_eq!(endpoint.metadata().bounds().response_rows(), None);
    assert_eq!(endpoint.metadata().bounds().page(), None);
}

#[allow(clippy::too_many_arguments)]
fn assert_response_types(
    _: &EndpointSpec<LatestCongressionalDisclosuresQuery, Vec<CongressionalTrade>>,
    _: &EndpointSpec<LatestCongressionalDisclosuresQuery, Vec<CongressionalTrade>>,
    _: &EndpointSpec<CongressionalTradesQuery, Vec<CongressionalTrade>>,
    _: &EndpointSpec<CongressionalTradesByNameQuery, Vec<CongressionalTrade>>,
    _: &EndpointSpec<CongressionalTradesByMemberIdQuery, Vec<CongressionalTrade>>,
    _: &EndpointSpec<CongressionalTradesQuery, Vec<CongressionalTrade>>,
    _: &EndpointSpec<CongressionalTradesByNameQuery, Vec<CongressionalTrade>>,
    _: &EndpointSpec<CongressionalTradesByMemberIdQuery, Vec<CongressionalTrade>>,
) {
}

fn ticker(value: &str) -> Ticker {
    Ticker::new(value).unwrap()
}

fn term(value: &str) -> SearchTerm {
    SearchTerm::new(value).unwrap()
}

fn fixture_executor() -> Arc<FixtureExecutor> {
    Arc::new(FixtureExecutor::new(FIXTURES.map(json_fixture)))
}

async fn execute_all(client: &Client) -> Vec<CongressionalTrade> {
    let mut rows = Vec::new();
    rows.extend(
        client
            .latest_senate_disclosures(
                LatestCongressionalDisclosuresQuery::new()
                    .with_page(Page(0))
                    .with_limit(Limit(250)),
            )
            .await
            .unwrap(),
    );
    rows.extend(
        client
            .latest_house_disclosures(
                LatestCongressionalDisclosuresQuery::new()
                    .with_page(Page(u32::MAX))
                    .with_limit(Limit(0)),
            )
            .await
            .unwrap(),
    );
    rows.extend(
        client
            .senate_trades(
                CongressionalTradesQuery::new(ticker("BRK.B / Class A"))
                    .with_page(Page(1))
                    .with_limit(Limit(2)),
            )
            .await
            .unwrap(),
    );
    rows.extend(
        client
            .senate_trades_by_name(term("Jerry Moran"))
            .await
            .unwrap(),
    );
    rows.extend(
        client
            .senate_trades_by_member_id(
                CongressionalTradesByMemberIdQuery::new()
                    .with_page(Page(3))
                    .with_limit(Limit(4))
                    .with_member_id(CongressionalMemberId::new("M001242").unwrap()),
            )
            .await
            .unwrap(),
    );
    rows.extend(client.house_trades(ticker("AAPL")).await.unwrap());
    rows.extend(client.house_trades_by_name(term("James A.")).await.unwrap());
    rows.extend(
        client
            .house_trades_by_member_id(CongressionalTradesByMemberIdQuery::new())
            .await
            .unwrap(),
    );
    rows
}

#[tokio::test]
async fn custom_proxy_preserves_exact_query_order_headers_and_fixture_identity() {
    let executor = fixture_executor();
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "congress trades")
        .executor(executor.clone())
        .build()
        .unwrap();

    let rows = execute_all(&client).await;
    assert_eq!(rows.len(), 8);
    assert_eq!(rows[0].member_id.as_str(), "M001242");
    assert_eq!(rows[3].symbol, "");

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "congress trades"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/senate-latest?page=0&limit=250".to_owned(),
            format!(
                "https://proxy.example/router/gateway/stable/house-latest?page={}&limit=0",
                u32::MAX
            ),
            "https://proxy.example/router/gateway/stable/senate-trades?symbol=BRK.B+%2F+Class+A&page=1&limit=2".to_owned(),
            "https://proxy.example/router/gateway/stable/senate-trades-by-name?name=Jerry+Moran".to_owned(),
            "https://proxy.example/router/gateway/stable/senate-trades-by-id?page=3&limit=4&senateID=M001242".to_owned(),
            "https://proxy.example/router/gateway/stable/house-trades?symbol=AAPL".to_owned(),
            "https://proxy.example/router/gateway/stable/house-trades-by-name?name=James+A.".to_owned(),
            "https://proxy.example/router/gateway/stable/house-trades-by-id".to_owned(),
        ]
    );
}

#[tokio::test]
async fn direct_fmp_auth_supports_header_and_query_modes_on_all_routes() {
    for (authentication, query_auth, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            false,
            Some("header-secret"),
        ),
        (Authentication::fmp_query("query-secret"), true, None),
    ] {
        let executor = fixture_executor();
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        assert_eq!(execute_all(&client).await.len(), 8);
        let requests = executor.requests();
        assert_eq!(requests.len(), 8);
        for (request, path) in requests.iter().zip(PATHS) {
            assert!(request.expose_url().path().ends_with(path));
            let api_keys = request
                .expose_url()
                .query_pairs()
                .filter(|(name, _)| name == "apikey")
                .map(|(_, value)| value.into_owned())
                .collect::<Vec<_>>();
            if query_auth {
                assert_eq!(api_keys, ["query-secret"]);
            } else {
                assert!(api_keys.is_empty());
            }
            match expected_header {
                Some(value) => assert_eq!(request.expose_headers()["apikey"], value),
                None => assert!(!request.expose_headers().contains_key("apikey")),
            }
        }
    }
}

#[tokio::test]
async fn malformed_non_array_responses_keep_all_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new((0..8).map(|_| json_fixture(b"{}"))));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let errors = [
        client
            .latest_senate_disclosures(LatestCongressionalDisclosuresQuery::new())
            .await
            .unwrap_err(),
        client
            .latest_house_disclosures(LatestCongressionalDisclosuresQuery::new())
            .await
            .unwrap_err(),
        client.senate_trades(ticker("AAPL")).await.unwrap_err(),
        client
            .senate_trades_by_name(term("Jerry"))
            .await
            .unwrap_err(),
        client
            .senate_trades_by_member_id(CongressionalTradesByMemberIdQuery::new())
            .await
            .unwrap_err(),
        client.house_trades(ticker("AAPL")).await.unwrap_err(),
        client
            .house_trades_by_name(term("James"))
            .await
            .unwrap_err(),
        client
            .house_trades_by_member_id(CongressionalTradesByMemberIdQuery::new())
            .await
            .unwrap_err(),
    ];
    for (error, endpoint) in errors.iter().zip(PATHS) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
