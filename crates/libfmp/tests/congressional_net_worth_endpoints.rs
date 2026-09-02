mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        congressional::{
            CongressionalNetWorthAggregatedQuery, CongressionalNetWorthQuery,
            congressional_net_worth, congressional_net_worth_aggregated,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::congressional::{
        CongressionalMemberNetWorthAggregate, CongressionalMemberNetWorthEntry,
    },
    transport::HttpMethod,
    types::{CongressionalMemberId, Limit, Page},
};

use support::{FixtureExecutor, json_fixture};

const NET_WORTH: &[u8] = include_bytes!("fixtures/congress_senate_net_worth.json");
const AGGREGATED: &[u8] = include_bytes!("fixtures/congress_senate_net_worth_aggregated.json");

#[test]
fn descriptors_use_exact_get_paths_typed_rows_and_only_documented_metadata() {
    let member_id = || CongressionalMemberId::new("P000197").unwrap();
    let net_worth = congressional_net_worth(CongressionalNetWorthQuery::new(member_id()));
    let aggregated =
        congressional_net_worth_aggregated(CongressionalNetWorthAggregatedQuery::new(member_id()));

    assert_net_worth_facts(&net_worth);
    assert_aggregated_facts(&aggregated);
}

fn assert_net_worth_facts(
    endpoint: &EndpointSpec<CongressionalNetWorthQuery, Vec<CongressionalMemberNetWorthEntry>>,
) {
    assert_common(endpoint, "senate-net-worth");
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new().with_response_rows(250).with_page(100)
    );
}

fn assert_aggregated_facts(
    endpoint: &EndpointSpec<
        CongressionalNetWorthAggregatedQuery,
        Vec<CongressionalMemberNetWorthAggregate>,
    >,
) {
    assert_common(endpoint, "senate-net-worth-aggregated");
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
}

fn assert_common<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
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

#[tokio::test]
async fn custom_proxy_preserves_queries_headers_bare_arrays_and_exact_fixtures() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(NET_WORTH),
        json_fixture(AGGREGATED),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "congressional net worth")
        .executor(executor.clone())
        .build()
        .unwrap();

    let net_worth = client
        .congressional_net_worth(
            CongressionalNetWorthQuery::new(CongressionalMemberId::new("P000197").unwrap())
                .with_page(Page(0))
                .with_limit(Limit(250)),
        )
        .await
        .unwrap();
    let aggregated = client
        .congressional_net_worth_aggregated(
            CongressionalNetWorthAggregatedQuery::new(
                CongressionalMemberId::new("P000197").unwrap(),
            )
            .with_totals_col(""),
        )
        .await
        .unwrap();

    assert_eq!(net_worth[0].member_id.as_str(), "P000197");
    assert_eq!(
        net_worth[0].debt_details.as_ref().unwrap().date_incurred.0,
        "September 2007"
    );
    assert_eq!(net_worth[0].income, None);
    assert_eq!(aggregated[0].total, 225_219_551);
    assert_eq!(
        serde_json::to_value(&net_worth).unwrap(),
        serde_json::from_slice::<serde_json::Value>(NET_WORTH).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&aggregated).unwrap(),
        serde_json::from_slice::<serde_json::Value>(AGGREGATED).unwrap()
    );

    let requests = executor.requests();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "congressional net worth"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/senate-net-worth?senateID=P000197&page=0&limit=250",
            "https://proxy.example/router/gateway/stable/senate-net-worth-aggregated?senateID=P000197&totalsCol=",
        ]
    );
}

#[tokio::test]
async fn optional_query_values_stay_omitted_and_unknown_response_fields_are_tolerated() {
    const NET_WORTH_WITH_UNKNOWN: &[u8] = br#"[{
        "senateID":"P000197","formType":"House Report","year":2022,
        "filingDate":"2023-05-15","section":"Liabilities","category":"Mortgage",
        "name":"Bank","assetType":"Mortgage","incomeType":null,"owner":"Joint",
        "comment":null,"debtDetails":null,"valueRange":null,"value":-1,
        "incomeRange":{"min":-10,"max":20},"income":null,"link":"https://example.test",
        "futureProviderField":{"nested":true}
    }]"#;
    const AGGREGATED_WITH_UNKNOWN: &[u8] = br#"[{
        "senateID":"P000197","year":2024,"total":-1,"realEstateLiabilities":0,
        "cashAndCashEquivalents":0,"businessAndSelfEmployment":0,"realEstate":0,
        "ownershipInterest":0,"stock":0,"options":0,"revolvingAndCreditLines":0,
        "assetBackedSecurities":0,"businessLiabilities":0,"mutualFundsAndETFs":0,
        "futureProviderField":[1,2,3]
    }]"#;
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(NET_WORTH_WITH_UNKNOWN),
        json_fixture(AGGREGATED_WITH_UNKNOWN),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example")
        .authentication(Authentication::None)
        .executor(executor.clone())
        .build()
        .unwrap();
    let member_id = || CongressionalMemberId::new("P000197").unwrap();

    let rows = client.congressional_net_worth(member_id()).await.unwrap();
    assert_eq!(rows[0].value, -1);
    assert!(rows[0].debt_details.is_none());
    assert_eq!(rows[0].income_range.as_ref().unwrap().min, -10);
    let totals = client
        .congressional_net_worth_aggregated(member_id())
        .await
        .unwrap();
    assert_eq!(totals[0].total, -1);

    assert_eq!(
        executor
            .requests()
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/stable/senate-net-worth?senateID=P000197",
            "https://proxy.example/stable/senate-net-worth-aggregated?senateID=P000197",
        ]
    );
}

#[test]
fn itemized_required_fields_and_nullable_shapes_are_strict() {
    let row = serde_json::from_slice::<serde_json::Value>(NET_WORTH).unwrap()[0].clone();
    for field in [
        "senateID",
        "formType",
        "year",
        "filingDate",
        "section",
        "category",
        "name",
        "assetType",
        "incomeType",
        "owner",
        "comment",
        "debtDetails",
        "valueRange",
        "value",
        "incomeRange",
        "income",
        "link",
    ] {
        let mut missing = row.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<CongressionalMemberNetWorthEntry>(missing).is_err(),
            "itemized field {field} must be present"
        );
    }

    for field in [
        "incomeType",
        "comment",
        "debtDetails",
        "valueRange",
        "incomeRange",
        "income",
    ] {
        let mut nullable = row.clone();
        nullable[field] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<CongressionalMemberNetWorthEntry>(nullable).is_ok(),
            "nullable field {field} must accept null"
        );
    }

    for field in [
        "senateID",
        "formType",
        "year",
        "filingDate",
        "section",
        "category",
        "name",
        "assetType",
        "owner",
        "value",
        "link",
    ] {
        let mut invalid = row.clone();
        invalid[field] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<CongressionalMemberNetWorthEntry>(invalid).is_err(),
            "itemized field {field} must not accept null"
        );
    }
}

#[test]
fn every_aggregated_field_is_required_and_typed() {
    let row = serde_json::from_slice::<serde_json::Value>(AGGREGATED).unwrap()[0].clone();
    for field in [
        "senateID",
        "year",
        "total",
        "realEstateLiabilities",
        "cashAndCashEquivalents",
        "businessAndSelfEmployment",
        "realEstate",
        "ownershipInterest",
        "stock",
        "options",
        "revolvingAndCreditLines",
        "assetBackedSecurities",
        "businessLiabilities",
        "mutualFundsAndETFs",
    ] {
        let mut invalid = row.clone();
        invalid[field] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<CongressionalMemberNetWorthAggregate>(invalid).is_err(),
            "aggregate field {field} must not accept null"
        );
    }
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_apply_to_both_routes() {
    for (authentication, query_auth, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            false,
            Some("header-secret"),
        ),
        (Authentication::fmp_query("query-secret"), true, None),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(NET_WORTH),
            json_fixture(AGGREGATED),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let member_id = || CongressionalMemberId::new("P000197").unwrap();

        client.congressional_net_worth(member_id()).await.unwrap();
        client
            .congressional_net_worth_aggregated(member_id())
            .await
            .unwrap();

        for request in executor.requests().iter() {
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
async fn malformed_non_array_responses_keep_both_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();
    let member_id = || CongressionalMemberId::new("P000197").unwrap();

    let net_worth_error = client
        .congressional_net_worth(member_id())
        .await
        .unwrap_err();
    let aggregated_error = client
        .congressional_net_worth_aggregated(member_id())
        .await
        .unwrap_err();
    for (error, endpoint) in [
        (net_worth_error, "senate-net-worth"),
        (aggregated_error, "senate-net-worth-aggregated"),
    ] {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
