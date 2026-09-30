mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        calendar::{
            IposCalendarQuery, IposDisclosureQuery, IposProspectusQuery, ipos_calendar,
            ipos_disclosure, ipos_prospectus,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    responses::calendar::{IpoCalendarEvent, IpoDisclosure, IpoProspectus},
    transport::HttpMethod,
    types::{Date, DateRange},
};

use support::{FixtureExecutor, json_fixture};

const IPOS_CALENDAR: &[u8] = include_bytes!("fixtures/ipos_calendar.json");
const IPOS_DISCLOSURE: &[u8] = include_bytes!("fixtures/ipos_disclosure.json");
const IPOS_PROSPECTUS: &[u8] = include_bytes!("fixtures/ipos_prospectus.json");

#[test]
fn descriptors_use_exact_paths_rows_and_only_documented_metadata() {
    let calendar: EndpointSpec<IposCalendarQuery, Vec<IpoCalendarEvent>> =
        ipos_calendar(IposCalendarQuery::new());
    let disclosure: EndpointSpec<IposDisclosureQuery, Vec<IpoDisclosure>> =
        ipos_disclosure(IposDisclosureQuery::new());
    let prospectus: EndpointSpec<IposProspectusQuery, Vec<IpoProspectus>> =
        ipos_prospectus(IposProspectusQuery::new());

    assert_facts(
        &calendar,
        "ipos-calendar",
        GeographicAvailability::Worldwide,
        EndpointBounds::new().with_date_range_days(90),
    );
    assert_facts(
        &disclosure,
        "ipos-disclosure",
        GeographicAvailability::UsOnly,
        EndpointBounds::new(),
    );
    assert_facts(
        &prospectus,
        "ipos-prospectus",
        GeographicAvailability::UsOnly,
        EndpointBounds::new(),
    );

    let bounds = calendar.metadata().bounds();
    let range_90_days = DateRange::new(
        Date::from_str("2026-03-06").unwrap(),
        Date::from_str("2026-06-04").unwrap(),
    )
    .unwrap();
    let range_91_days = DateRange::new(
        Date::from_str("2026-03-06").unwrap(),
        Date::from_str("2026-06-05").unwrap(),
    )
    .unwrap();
    let documented_92_day_example = DateRange::new(
        Date::from_str("2026-03-06").unwrap(),
        Date::from_str("2026-06-06").unwrap(),
    )
    .unwrap();
    assert!(bounds.accepts_date_range(&range_90_days));
    assert!(!bounds.accepts_date_range(&range_91_days));
    assert!(!bounds.accepts_date_range(&documented_92_day_example));
}

fn assert_facts<Q, R>(
    endpoint: &EndpointSpec<Q, Vec<R>>,
    path: &'static str,
    geography: GeographicAvailability,
    bounds: EndpointBounds,
) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(endpoint.metadata().geography(), geography);
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), bounds);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn custom_proxy_preserves_exact_queries_auth_headers_and_source_rows() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(IPOS_CALENDAR),
        json_fixture(IPOS_DISCLOSURE),
        json_fixture(IPOS_PROSPECTUS),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "ipos")
        .executor(executor.clone())
        .build()
        .unwrap();
    let from = Date::from_str("2026-03-06").unwrap();
    let to = Date::from_str("2026-06-06").unwrap();

    let calendar = client
        .ipos_calendar(IposCalendarQuery::new().with_from(from).with_to(to))
        .await
        .unwrap();
    let disclosure = client
        .ipos_disclosure(IposDisclosureQuery::new().with_from(from).with_to(to))
        .await
        .unwrap();
    let prospectus = client
        .ipos_prospectus(IposProspectusQuery::new().with_from(from).with_to(to))
        .await
        .unwrap();

    assert_eq!(calendar.len(), 1);
    assert_eq!(calendar[0].daa.as_str(), "2026-07-29T04:00:00.000Z");
    assert_eq!(calendar[0].shares, None);
    assert_eq!(calendar[0].price_range, None);
    assert_eq!(calendar[0].market_cap, None);
    assert_eq!(disclosure.len(), 1);
    assert_eq!(disclosure[0].accepted_date.to_string(), "2026-07-30");
    assert_eq!(prospectus.len(), 1);
    let offering_values: [f64; 6] = [
        prospectus[0].price_public_per_share,
        prospectus[0].price_public_total,
        prospectus[0].discounts_and_commissions_per_share.unwrap(),
        prospectus[0].discounts_and_commissions_total.unwrap(),
        prospectus[0].proceeds_before_expenses_per_share,
        prospectus[0].proceeds_before_expenses_total,
    ];
    assert_eq!(offering_values, [1.0, 434.0, 0.0, 82_251.0, 1.0, 82_251.0]);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "ipos"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/ipos-calendar?from=2026-03-06&to=2026-06-06",
            "https://proxy.example/router/gateway/stable/ipos-disclosure?from=2026-03-06&to=2026-06-06",
            "https://proxy.example/router/gateway/stable/ipos-prospectus?from=2026-03-06&to=2026-06-06",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_all_independent_date_states() {
    let from = Date::from_str("2026-03-06").unwrap();
    let to = Date::from_str("2026-06-06").unwrap();

    for (authentication, empty_suffix, value_suffix, expected_header) in [
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
        let executor = Arc::new(FixtureExecutor::new(
            std::iter::repeat_with(|| json_fixture(IPOS_CALENDAR))
                .take(4)
                .chain(std::iter::repeat_with(|| json_fixture(IPOS_DISCLOSURE)).take(4))
                .chain(std::iter::repeat_with(|| json_fixture(IPOS_PROSPECTUS)).take(4)),
        ));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        for query in [
            IposCalendarQuery::new(),
            IposCalendarQuery::new().with_from(from),
            IposCalendarQuery::new().with_to(to),
            IposCalendarQuery::new().with_from(from).with_to(to),
        ] {
            assert_eq!(client.ipos_calendar(query).await.unwrap().len(), 1);
        }
        for query in [
            IposDisclosureQuery::new(),
            IposDisclosureQuery::new().with_from(from),
            IposDisclosureQuery::new().with_to(to),
            IposDisclosureQuery::new().with_from(from).with_to(to),
        ] {
            assert_eq!(client.ipos_disclosure(query).await.unwrap().len(), 1);
        }
        for query in [
            IposProspectusQuery::new(),
            IposProspectusQuery::new().with_from(from),
            IposProspectusQuery::new().with_to(to),
            IposProspectusQuery::new().with_from(from).with_to(to),
        ] {
            assert_eq!(client.ipos_prospectus(query).await.unwrap().len(), 1);
        }

        let requests = executor.requests();
        let urls = requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>();
        let mut expected = Vec::new();
        for path in ["ipos-calendar", "ipos-disclosure", "ipos-prospectus"] {
            expected.extend([
                format!("https://financialmodelingprep.com/stable/{path}{empty_suffix}"),
                format!(
                    "https://financialmodelingprep.com/stable/{path}?from=2026-03-06{value_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/{path}?to=2026-06-06{value_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/{path}?from=2026-03-06&to=2026-06-06{value_suffix}"
                ),
            ]);
        }
        assert_eq!(urls, expected);
        for request in requests.iter() {
            match expected_header {
                Some(value) => assert_eq!(request.expose_headers()["apikey"], value),
                None => assert!(!request.expose_headers().contains_key("apikey")),
            }
        }
    }
}
