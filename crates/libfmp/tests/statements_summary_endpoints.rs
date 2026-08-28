mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        statements::{
            EnterpriseValuesQuery, FinancialScoresQuery, LatestFinancialStatementsQuery,
            OwnerEarningsQuery, enterprise_values, financial_scores, latest_financial_statements,
            owner_earnings,
        },
    },
    query::{FiscalPeriod, RetrievalFrequency},
    transport::HttpMethod,
    types::{Limit, Page, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const LATEST: &[u8] = include_bytes!("fixtures/latest_financial_statements.json");
const SCORES: &[u8] = include_bytes!("fixtures/financial_scores.json");
const OWNER: &[u8] = include_bytes!("fixtures/owner_earnings.json");
const ENTERPRISE: &[u8] = include_bytes!("fixtures/enterprise_values.json");

#[test]
fn descriptors_use_exact_get_paths_and_only_documented_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();

    let latest = latest_financial_statements(
        LatestFinancialStatementsQuery::new()
            .with_page(Page(0))
            .with_limit(Limit(250)),
    );
    assert_facts(
        &latest,
        "latest-financial-statements",
        EndpointBounds::new().with_response_rows(250).with_page(100),
    );
    assert!(latest.metadata().bounds().accepts_page(Page(0)));
    assert!(latest.metadata().bounds().accepts_page(Page(100)));
    assert!(!latest.metadata().bounds().accepts_page(Page(101)));
    assert!(latest.metadata().bounds().accepts_response_rows(250));
    assert!(!latest.metadata().bounds().accepts_response_rows(251));
    assert_facts(
        &financial_scores(FinancialScoresQuery::new(symbol.clone())),
        "financial-scores",
        EndpointBounds::new(),
    );
    assert_facts(
        &owner_earnings(OwnerEarningsQuery::new(symbol.clone())),
        "owner-earnings",
        EndpointBounds::new(),
    );
    assert_facts(
        &enterprise_values(
            EnterpriseValuesQuery::new(symbol)
                .with_limit(Limit(1_000))
                .with_period(RetrievalFrequency::Annual),
        ),
        "enterprise-values",
        EndpointBounds::new().with_response_rows(1_000),
    );
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str, bounds: EndpointBounds) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), bounds);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[test]
fn endpoint_queries_expose_exact_values_and_preserve_documented_order() {
    let symbol = Ticker::new("BRK.B / Class A").unwrap();
    let latest_query = LatestFinancialStatementsQuery::new()
        .with_page(Page(0))
        .with_limit(Limit(250));
    let scores_query: FinancialScoresQuery = (&symbol).into();
    let owner_query = OwnerEarningsQuery::new(symbol.clone()).with_limit(Limit(5));
    let enterprise_query = EnterpriseValuesQuery::new(symbol.clone())
        .with_limit(Limit(7))
        .with_period(FiscalPeriod::Q4);

    assert_eq!(latest_query.page(), Some(Page(0)));
    assert_eq!(latest_query.limit(), Some(Limit(250)));
    assert_eq!(scores_query.symbol(), &symbol);
    assert_eq!(owner_query.symbol(), &symbol);
    assert_eq!(owner_query.limit(), Some(Limit(5)));
    assert_eq!(enterprise_query.symbol(), &symbol);
    assert_eq!(enterprise_query.limit(), Some(Limit(7)));
    assert_eq!(enterprise_query.period(), Some(FiscalPeriod::Q4.into()));

    assert_eq!(
        latest_financial_statements(latest_query).query().page(),
        Some(Page(0))
    );
    assert_eq!(financial_scores(scores_query).query().symbol(), &symbol);
    assert_eq!(owner_earnings(owner_query).query().limit(), Some(Limit(5)));
    assert_eq!(
        enterprise_values(enterprise_query).query().period(),
        Some(FiscalPeriod::Q4.into())
    );
}

#[tokio::test]
async fn custom_proxy_auth_headers_and_routing_preserve_all_four_contracts() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(LATEST),
        json_fixture(SCORES),
        json_fixture(OWNER),
        json_fixture(ENTERPRISE),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .default_header("x-data-scope", "financial-summaries")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let latest = client
        .latest_financial_statements(
            LatestFinancialStatementsQuery::new()
                .with_page(Page(0))
                .with_limit(Limit(250)),
        )
        .await
        .unwrap();
    let scores = client.financial_scores(&symbol).await.unwrap();
    let owner = client
        .owner_earnings(OwnerEarningsQuery::new(symbol.clone()).with_limit(Limit(5)))
        .await
        .unwrap();
    let enterprise = client
        .enterprise_values(
            EnterpriseValuesQuery::new(symbol)
                .with_limit(Limit(7))
                .with_period(RetrievalFrequency::Quarterly),
        )
        .await
        .unwrap();

    assert_eq!(latest[0].calendar_year.get(), 2026);
    assert_eq!(scores[0].market_cap, 5_042_169_135_511);
    assert_eq!(owner[0].growth_capex, -2_130_994_500);
    assert_eq!(enterprise[0].enterprise_value, 3_895_186_810_000);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "financial-summaries"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/latest-financial-statements?page=0&limit=250",
            "https://proxy.example/router/stable/financial-scores?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/owner-earnings?symbol=BRK.B+%2F+Class+A&limit=5",
            "https://proxy.example/router/stable/enterprise-values?symbol=BRK.B+%2F+Class+A&limit=7&period=quarter",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_apply_to_every_compact_endpoint() {
    for (authentication, query_suffix, expected_header) in [
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
            json_fixture(LATEST),
            json_fixture(SCORES),
            json_fixture(OWNER),
            json_fixture(ENTERPRISE),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbol = Ticker::new("AAPL").unwrap();

        client
            .latest_financial_statements(LatestFinancialStatementsQuery::new().with_page(Page(0)))
            .await
            .unwrap();
        client.financial_scores(&symbol).await.unwrap();
        client.owner_earnings(&symbol).await.unwrap();
        client
            .enterprise_values(
                EnterpriseValuesQuery::new(symbol).with_period(FiscalPeriod::FullYear),
            )
            .await
            .unwrap();

        let requests = executor.requests();
        let expected = [
            format!(
                "https://financialmodelingprep.com/stable/latest-financial-statements?page=0{query_suffix}"
            ),
            format!(
                "https://financialmodelingprep.com/stable/financial-scores?symbol=AAPL{query_suffix}"
            ),
            format!(
                "https://financialmodelingprep.com/stable/owner-earnings?symbol=AAPL{query_suffix}"
            ),
            format!(
                "https://financialmodelingprep.com/stable/enterprise-values?symbol=AAPL&period=FY{query_suffix}"
            ),
        ];
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            expected
        );
        for request in requests.iter() {
            match expected_header {
                Some(value) => assert_eq!(request.expose_headers()["apikey"], value),
                None => assert!(!request.expose_headers().contains_key("apikey")),
            }
        }
    }
}
