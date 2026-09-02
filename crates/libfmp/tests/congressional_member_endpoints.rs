mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        congressional::{
            CongressionalPositionsQuery, CongressionalProfilesQuery, congressional_positions,
            congressional_profiles,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::congressional::{CongressionalMemberPosition, CongressionalMemberProfile},
    transport::HttpMethod,
    types::{CongressionalMemberId, Limit, Page},
};

use support::{FixtureExecutor, json_fixture};

const PROFILE: &[u8] = include_bytes!("fixtures/congress_senate_profile.json");
const POSITIONS: &[u8] = include_bytes!("fixtures/congress_senate_positions.json");

#[test]
fn descriptors_use_exact_get_paths_typed_rows_and_only_documented_metadata() {
    let profiles = congressional_profiles(CongressionalProfilesQuery::new());
    let positions = congressional_positions(CongressionalPositionsQuery::new());

    assert_profile_facts(&profiles);
    assert_position_facts(&positions);
}

fn assert_profile_facts(
    endpoint: &EndpointSpec<CongressionalProfilesQuery, Vec<CongressionalMemberProfile>>,
) {
    assert_common(endpoint, "senate-profile");
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new().with_response_rows(500).with_page(20)
    );
}

fn assert_position_facts(
    endpoint: &EndpointSpec<CongressionalPositionsQuery, Vec<CongressionalMemberPosition>>,
) {
    assert_common(endpoint, "senate-positions");
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new().with_response_rows(300).with_page(50)
    );
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
async fn custom_proxy_preserves_wire_order_headers_bare_arrays_and_exact_fixtures() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(PROFILE),
        json_fixture(POSITIONS),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "congressional members")
        .executor(executor.clone())
        .build()
        .unwrap();

    let profiles = client
        .congressional_profiles(
            CongressionalProfilesQuery::new()
                .with_active(false)
                .with_member_id(CongressionalMemberId::new("P000197").unwrap())
                .with_latest_party("Independent / Other")
                .with_latest_position("Representative At-Large")
                .with_page(Page(0))
                .with_limit(Limit(500)),
        )
        .await
        .unwrap();
    let positions = client
        .congressional_positions(
            CongressionalPositionsQuery::new()
                .with_member_id(CongressionalMemberId::new("P000197").unwrap())
                .with_party("Republican / Other")
                .with_position("Representative At-Large")
                .with_page(Page(0))
                .with_limit(Limit(300)),
        )
        .await
        .unwrap();

    assert_eq!(profiles[0].member_id.as_str(), "L000397");
    assert!(profiles[0].active);
    assert_eq!(positions[0].member_id.as_str(), "Z000018");
    assert_eq!(positions[0].end_date, None);
    assert_eq!(
        serde_json::to_value(&profiles).unwrap(),
        serde_json::from_slice::<serde_json::Value>(PROFILE).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&positions).unwrap(),
        serde_json::from_slice::<serde_json::Value>(POSITIONS).unwrap()
    );

    let requests = executor.requests();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "congressional members"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/senate-profile?active=false&senateID=P000197&latestParty=Independent+%2F+Other&latestPosition=Representative+At-Large&page=0&limit=500",
            "https://proxy.example/router/gateway/stable/senate-positions?senateID=P000197&party=Republican+%2F+Other&position=Representative+At-Large&page=0&limit=300",
        ]
    );
}

#[tokio::test]
async fn omitted_options_stay_omitted_and_unknown_response_fields_are_tolerated() {
    const PROFILE_WITH_UNKNOWN: &[u8] = br#"[{
        "senateID":"L000397","firstName":"Zoe","lastName":"Lofgren",
        "birthDate":"1947-12-20","latestParty":"Democrat","latestState":"CA",
        "latestPosition":"Representative","image":"https://example.test/member.jpg",
        "active":true,"yearsActive":31.6,"futureProviderField":{"nested":true}
    }]"#;
    const POSITION_WITH_UNKNOWN: &[u8] = br#"[{
        "senateID":"Z000018","congressNumber":119,"startDate":"2025-01-02",
        "endDate":null,"party":"Republican","position":"Representative",
        "state":"MT","yearsInTerm":0.7,"futureProviderField":[1,2,3]
    }]"#;

    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(PROFILE_WITH_UNKNOWN),
        json_fixture(POSITION_WITH_UNKNOWN),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example")
        .authentication(Authentication::None)
        .executor(executor.clone())
        .build()
        .unwrap();

    assert_eq!(
        client
            .congressional_profiles(CongressionalProfilesQuery::new())
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        client
            .congressional_positions(CongressionalPositionsQuery::new())
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        executor
            .requests()
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/stable/senate-profile",
            "https://proxy.example/stable/senate-positions",
        ]
    );
}

#[test]
fn documented_fields_have_strict_types_and_only_position_end_date_is_nullable() {
    let profile = serde_json::from_slice::<serde_json::Value>(PROFILE).unwrap()[0].clone();
    for field in [
        "senateID",
        "firstName",
        "lastName",
        "birthDate",
        "latestParty",
        "latestState",
        "latestPosition",
        "image",
        "active",
        "yearsActive",
    ] {
        let mut invalid = profile.clone();
        invalid[field] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<CongressionalMemberProfile>(invalid).is_err(),
            "profile field {field} must not accept null"
        );
    }

    let position = serde_json::from_slice::<serde_json::Value>(POSITIONS).unwrap()[0].clone();
    assert!(serde_json::from_value::<CongressionalMemberPosition>(position.clone()).is_ok());
    let mut missing_end_date = position.clone();
    missing_end_date.as_object_mut().unwrap().remove("endDate");
    assert!(
        serde_json::from_value::<CongressionalMemberPosition>(missing_end_date).is_err(),
        "position endDate must be present even when nullable"
    );
    for field in [
        "senateID",
        "congressNumber",
        "startDate",
        "party",
        "position",
        "state",
        "yearsInTerm",
    ] {
        let mut invalid = position.clone();
        invalid[field] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<CongressionalMemberPosition>(invalid).is_err(),
            "position field {field} must not accept null"
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
            json_fixture(PROFILE),
            json_fixture(POSITIONS),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .congressional_profiles(CongressionalProfilesQuery::new().with_active(true))
            .await
            .unwrap();
        client
            .congressional_positions(
                CongressionalPositionsQuery::new()
                    .with_member_id(CongressionalMemberId::new("P000197").unwrap()),
            )
            .await
            .unwrap();

        let requests = executor.requests();
        for request in requests.iter() {
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

    let profile_error = client
        .congressional_profiles(CongressionalProfilesQuery::new())
        .await
        .unwrap_err();
    let position_error = client
        .congressional_positions(CongressionalPositionsQuery::new())
        .await
        .unwrap_err();
    for (error, endpoint) in [
        (profile_error, "senate-profile"),
        (position_error, "senate-positions"),
    ] {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
