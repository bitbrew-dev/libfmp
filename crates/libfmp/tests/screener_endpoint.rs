mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        screener::{CompanyScreenerQuery, company_screener},
    },
    transport::HttpMethod,
    types::{CountryCode, ExchangeCode, FiniteDecimal, Industry, Limit, Page, Sector},
};

use support::{FixtureExecutor, json_fixture};

const SCREENER: &[u8] = include_bytes!("fixtures/company_screener.json");
const EMPTY: &[u8] = include_bytes!("fixtures/company_screener_empty.json");
const MULTIPLE: &[u8] = include_bytes!("fixtures/company_screener_multiple.json");
const UNKNOWN: &[u8] = include_bytes!("fixtures/company_screener_unknown.json");

fn decimal(value: f64) -> FiniteDecimal {
    FiniteDecimal::new(value).unwrap()
}

#[test]
fn descriptor_uses_exact_contract_and_only_sourced_worldwide_metadata() {
    let endpoint = company_screener(CompanyScreenerQuery::new());

    assert_eq!(endpoint.id(), "company-screener");
    assert_eq!(endpoint.relative_path(), "company-screener");
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    assert_eq!(endpoint.query(), &CompanyScreenerQuery::default());
}

#[test]
fn fluent_query_exposes_all_twenty_private_filters_without_invented_validation() {
    let query = all_filters();

    assert_eq!(query.market_cap_more_than(), Some(9_007_199_254_740_993));
    assert_eq!(query.market_cap_lower_than(), Some(u64::MAX));
    assert_eq!(query.sector().unwrap().as_str(), "Technology & AI");
    assert_eq!(
        query.industry().unwrap().as_str(),
        "Consumer Electronics / Devices"
    );
    assert_eq!(query.beta_more_than(), Some(decimal(0.5)));
    assert_eq!(query.beta_lower_than(), Some(decimal(1.5)));
    assert_eq!(query.price_more_than(), Some(decimal(10.25)));
    assert_eq!(query.price_lower_than(), Some(decimal(500.0)));
    assert_eq!(query.dividend_more_than(), Some(decimal(0.5)));
    assert_eq!(query.dividend_lower_than(), Some(decimal(2.0)));
    assert_eq!(query.volume_more_than(), Some(1_000));
    assert_eq!(query.volume_lower_than(), Some(u64::MAX));
    assert_eq!(query.exchange().unwrap().as_str(), "NASDAQ Global");
    assert_eq!(query.country().unwrap().as_str(), "US / CA");
    assert_eq!(query.is_etf(), Some(false));
    assert_eq!(query.is_fund(), Some(false));
    assert_eq!(query.is_actively_trading(), Some(true));
    assert_eq!(query.page(), Some(Page(0)));
    assert_eq!(query.limit(), Some(Limit(u32::MAX)));
    assert_eq!(query.include_all_share_classes(), Some(false));
}

#[tokio::test]
async fn client_encodes_no_query_and_all_twenty_filters_in_documented_order() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(SCREENER),
        json_fixture(SCREENER),
    ]));
    let client = proxy_client(executor.clone());

    client
        .company_screener(CompanyScreenerQuery::new())
        .await
        .unwrap();
    client.company_screener(all_filters()).await.unwrap();

    let requests = executor.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://proxy.example/router/stable/company-screener"
    );
    assert_eq!(
        requests[1].expose_url().as_str(),
        "https://proxy.example/router/stable/company-screener?marketCapMoreThan=9007199254740993&marketCapLowerThan=18446744073709551615&sector=Technology+%26+AI&industry=Consumer+Electronics+%2F+Devices&betaMoreThan=0.5&betaLowerThan=1.5&priceMoreThan=10.25&priceLowerThan=500&dividendMoreThan=0.5&dividendLowerThan=2&volumeMoreThan=1000&volumeLowerThan=18446744073709551615&exchange=NASDAQ+Global&country=US+%2F+CA&isEtf=false&isFund=false&isActivelyTrading=true&page=0&limit=4294967295&includeAllShareClasses=false"
    );
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
}

#[tokio::test]
async fn zero_and_false_are_emitted_while_absent_filters_are_omitted() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(EMPTY)]));
    let client = proxy_client(executor.clone());
    let query = CompanyScreenerQuery::new()
        .with_market_cap_more_than(0)
        .with_price_more_than(decimal(0.0))
        .with_volume_more_than(0)
        .with_is_etf(false)
        .with_is_fund(false)
        .with_is_actively_trading(false)
        .with_page(Page(0))
        .with_limit(Limit(0))
        .with_include_all_share_classes(false);

    let rows = client.company_screener(query).await.unwrap();

    assert!(rows.is_empty());
    assert_eq!(
        executor.requests()[0].expose_url().as_str(),
        "https://proxy.example/router/stable/company-screener?marketCapMoreThan=0&priceMoreThan=0&volumeMoreThan=0&isEtf=false&isFund=false&isActivelyTrading=false&page=0&limit=0&includeAllShareClasses=false"
    );
}

#[tokio::test]
async fn client_preserves_empty_multiple_unknown_and_large_number_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(EMPTY),
        json_fixture(MULTIPLE),
        json_fixture(UNKNOWN),
    ]));
    let client = proxy_client(executor);

    let empty = client
        .company_screener(CompanyScreenerQuery::new())
        .await
        .unwrap();
    let multiple = client
        .company_screener(CompanyScreenerQuery::new())
        .await
        .unwrap();
    let unknown = client
        .company_screener(CompanyScreenerQuery::new())
        .await
        .unwrap();

    assert!(empty.is_empty());
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[0].market_cap, 9_007_199_254_740_993);
    assert_eq!(multiple[0].volume, u64::MAX as f64);
    assert_eq!(unknown.len(), 1);
}

#[tokio::test]
async fn direct_fmp_auth_uses_the_same_company_screener_contract() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/company-screener?sector=Technology&limit=1000",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/company-screener?sector=Technology&limit=1000&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(SCREENER)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        let rows = client
            .company_screener(
                CompanyScreenerQuery::new()
                    .with_sector(Sector::new("Technology").unwrap())
                    .with_limit(Limit(1_000)),
            )
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        assert_eq!(requests[0].method(), HttpMethod::Get);
        match expected_header {
            Some((name, value)) => assert_eq!(requests[0].expose_headers()[name], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

fn all_filters() -> CompanyScreenerQuery {
    CompanyScreenerQuery::new()
        .with_market_cap_more_than(9_007_199_254_740_993)
        .with_market_cap_lower_than(u64::MAX)
        .with_sector(Sector::new("Technology & AI").unwrap())
        .with_industry(Industry::new("Consumer Electronics / Devices").unwrap())
        .with_beta_more_than(decimal(0.5))
        .with_beta_lower_than(decimal(1.5))
        .with_price_more_than(decimal(10.25))
        .with_price_lower_than(decimal(500.0))
        .with_dividend_more_than(decimal(0.5))
        .with_dividend_lower_than(decimal(2.0))
        .with_volume_more_than(1_000)
        .with_volume_lower_than(u64::MAX)
        .with_exchange(ExchangeCode::new("NASDAQ Global").unwrap())
        .with_country(CountryCode::new("US / CA").unwrap())
        .with_is_etf(false)
        .with_is_fund(false)
        .with_is_actively_trading(true)
        .with_page(Page(0))
        .with_limit(Limit(u32::MAX))
        .with_include_all_share_classes(false)
}

fn proxy_client(executor: Arc<FixtureExecutor>) -> Client {
    Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::None)
        .executor(executor)
        .build()
        .unwrap()
}
