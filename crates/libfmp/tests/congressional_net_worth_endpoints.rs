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
    responses::congressional::{CongressionalMemberNetWorth, CongressionalMemberNetWorthAggregate},
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
    endpoint: &EndpointSpec<CongressionalNetWorthQuery, Vec<CongressionalMemberNetWorth>>,
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
        net_worth[0]
            .debt_details
            .as_ref()
            .unwrap()
            .date_incurred
            .as_ref()
            .unwrap()
            .0,
        "September 2007"
    );
    assert_eq!(net_worth[0].income, None);
    assert_eq!(aggregated[0].total, 225_219_551.0);
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
    assert_eq!(rows[0].value, Some(-1.0));
    assert!(rows[0].debt_details.is_none());
    assert_eq!(rows[0].income_range.as_ref().unwrap().min, -10);
    let totals = client
        .congressional_net_worth_aggregated(member_id())
        .await
        .unwrap();
    assert_eq!(totals[0].total, -1.0);

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
            serde_json::from_value::<CongressionalMemberNetWorth>(missing).is_err(),
            "itemized field {field} must be present"
        );
    }

    for field in [
        "category",
        "name",
        "incomeType",
        "owner",
        "comment",
        "debtDetails",
        "valueRange",
        "value",
        "incomeRange",
        "income",
    ] {
        let mut nullable = row.clone();
        nullable[field] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<CongressionalMemberNetWorth>(nullable).is_ok(),
            "nullable field {field} must accept null"
        );
    }

    for field in [
        "senateID",
        "formType",
        "year",
        "filingDate",
        "section",
        "assetType",
        "link",
    ] {
        let mut invalid = row.clone();
        invalid[field] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<CongressionalMemberNetWorth>(invalid).is_err(),
            "itemized field {field} must not accept null"
        );
    }
}

#[test]
fn itemized_fractional_value_and_null_members_decode_exactly() {
    let mut row = serde_json::from_slice::<serde_json::Value>(NET_WORTH).unwrap()[0].clone();
    row["value"] = serde_json::json!(32500.5);
    row["debtDetails"] = serde_json::json!({});
    let entry = serde_json::from_value::<CongressionalMemberNetWorth>(row.clone()).unwrap();
    assert_eq!(entry.value, Some(32500.5));
    assert!(entry.debt_details.as_ref().unwrap().date_incurred.is_none());
    assert_eq!(serde_json::to_value(&entry).unwrap(), row);

    row["valueRange"]["max"] = serde_json::Value::Null;
    for field in ["category", "name", "owner", "value"] {
        row[field] = serde_json::Value::Null;
    }
    let entry = serde_json::from_value::<CongressionalMemberNetWorth>(row).unwrap();
    assert_eq!(entry.category, None);
    assert_eq!(entry.name, None);
    assert_eq!(entry.owner, None);
    assert_eq!(entry.value, None);
    assert_eq!(entry.value_range.unwrap().max, None);
}

const AGGREGATE_REQUIRED: [&str; 5] = [
    "senateID",
    "year",
    "total",
    "cashAndCashEquivalents",
    "mutualFundsAndETFs",
];

const AGGREGATE_OMITTABLE: [&str; 9] = [
    "realEstate",
    "stock",
    "realEstateLiabilities",
    "businessAndSelfEmployment",
    "ownershipInterest",
    "options",
    "revolvingAndCreditLines",
    "assetBackedSecurities",
    "businessLiabilities",
];

#[test]
fn aggregated_required_fields_are_strict_and_omittable_fields_may_be_absent() {
    let row = serde_json::from_slice::<serde_json::Value>(AGGREGATED).unwrap()[0].clone();
    for field in AGGREGATE_REQUIRED {
        let mut missing = row.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<CongressionalMemberNetWorthAggregate>(missing).is_err(),
            "aggregate field {field} must be present"
        );
    }
    for field in AGGREGATE_REQUIRED.iter().chain(&AGGREGATE_OMITTABLE) {
        let mut invalid = row.clone();
        invalid[field] = serde_json::json!(true);
        assert!(
            serde_json::from_value::<CongressionalMemberNetWorthAggregate>(invalid).is_err(),
            "aggregate field {field} must be typed"
        );
    }
    for field in AGGREGATE_REQUIRED {
        let mut invalid = row.clone();
        invalid[field] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<CongressionalMemberNetWorthAggregate>(invalid).is_err(),
            "aggregate field {field} must not accept null"
        );
    }
    for field in AGGREGATE_OMITTABLE {
        let mut nulled = row.clone();
        nulled[field] = serde_json::Value::Null;
        let totals =
            serde_json::from_value::<CongressionalMemberNetWorthAggregate>(nulled).unwrap();
        let encoded = serde_json::to_value(&totals).unwrap();
        assert!(
            encoded.get(field).is_none(),
            "aggregate field {field} must decode null as None"
        );
    }

    let mut sparse = row.clone();
    for field in AGGREGATE_OMITTABLE {
        sparse.as_object_mut().unwrap().remove(field);
    }
    sparse["total"] = serde_json::json!(59082540.5);
    sparse["cashAndCashEquivalents"] = serde_json::json!(121004.5);
    sparse["mutualFundsAndETFs"] = serde_json::json!(34526531.5);
    let totals =
        serde_json::from_value::<CongressionalMemberNetWorthAggregate>(sparse.clone()).unwrap();
    assert_eq!(totals.total, 59_082_540.5);
    assert_eq!(totals.cash_and_cash_equivalents, 121_004.5);
    assert_eq!(totals.mutual_funds_and_etfs, 34_526_531.5);
    assert_eq!(totals.real_estate, None);
    assert_eq!(totals.stock, None);
    assert_eq!(totals.real_estate_liabilities, None);
    assert_eq!(totals.business_and_self_employment, None);
    assert_eq!(totals.ownership_interest, None);
    assert_eq!(totals.options, None);
    assert_eq!(totals.revolving_and_credit_lines, None);
    assert_eq!(totals.asset_backed_securities, None);
    assert_eq!(totals.business_liabilities, None);
    assert_eq!(serde_json::to_value(&totals).unwrap(), sparse);

    let totals = serde_json::from_value::<CongressionalMemberNetWorthAggregate>(row).unwrap();
    assert_eq!(totals.asset_backed_securities, Some(4_475_006.0));
    assert_eq!(totals.options, Some(0.0));

    let mut fractional = sparse;
    fractional["realEstate"] = serde_json::json!(3000000.5);
    fractional["stock"] = serde_json::json!(8000.5);
    let totals =
        serde_json::from_value::<CongressionalMemberNetWorthAggregate>(fractional).unwrap();
    assert_eq!(totals.real_estate, Some(3_000_000.5));
    assert_eq!(totals.stock, Some(8_000.5));
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
