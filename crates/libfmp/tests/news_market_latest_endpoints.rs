mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        news::{
            LatestCryptoNewsQuery, LatestForexNewsQuery, LatestStockNewsQuery, latest_crypto_news,
            latest_forex_news, latest_stock_news,
        },
    },
    responses::news::NewsArticle,
    transport::HttpMethod,
    types::{Date, Limit, Page},
};

use support::{FixtureExecutor, json_fixture};

const STOCK: &[u8] = include_bytes!("fixtures/latest_stock_news.json");
const CRYPTO: &[u8] = include_bytes!("fixtures/latest_crypto_news.json");
const FOREX: &[u8] = include_bytes!("fixtures/latest_forex_news.json");

#[test]
fn descriptors_use_exact_paths_vec_rows_and_only_documented_metadata() {
    let bounds = EndpointBounds::new().with_response_rows(250).with_page(100);
    let stock: EndpointSpec<LatestStockNewsQuery, Vec<NewsArticle>> =
        latest_stock_news(LatestStockNewsQuery::new());
    let crypto: EndpointSpec<LatestCryptoNewsQuery, Vec<NewsArticle>> =
        latest_crypto_news(LatestCryptoNewsQuery::new());
    let forex: EndpointSpec<LatestForexNewsQuery, Vec<NewsArticle>> =
        latest_forex_news(LatestForexNewsQuery::new());

    assert_facts(&stock, "news/stock-latest", bounds);
    assert_facts(&crypto, "news/crypto-latest", bounds);
    assert_facts(&forex, "news/forex-latest", bounds);

    assert!(bounds.accepts_page(Page(0)));
    assert!(bounds.accepts_page(Page(100)));
    assert!(!bounds.accepts_page(Page(101)));
    assert_eq!(bounds.limit(), None);
    assert!(bounds.accepts_limit(Limit(251)));
    assert_eq!(bounds.date_range_days(), None);
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str, bounds: EndpointBounds) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Unspecified
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), bounds);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn custom_proxy_preserves_exact_queries_auth_headers_fixtures_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(STOCK),
        json_fixture(CRYPTO),
        json_fixture(FOREX),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "market-news")
        .executor(executor.clone())
        .build()
        .unwrap();
    let from = Date::from_str("2026-01-27").unwrap();
    let to = Date::from_str("2026-04-28").unwrap();

    let stock = client
        .latest_stock_news(
            LatestStockNewsQuery::new()
                .with_from(from)
                .with_to(to)
                .with_page(Page(100))
                .with_limit(Limit(251)),
        )
        .await
        .unwrap();
    let crypto = client
        .latest_crypto_news(
            LatestCryptoNewsQuery::new()
                .with_from(from)
                .with_to(to)
                .with_page(Page(0))
                .with_limit(Limit(251)),
        )
        .await
        .unwrap();
    let forex = client
        .latest_forex_news(
            LatestForexNewsQuery::new()
                .with_from(from)
                .with_to(to)
                .with_page(Page(0))
                .with_limit(Limit(251)),
        )
        .await
        .unwrap();

    assert_eq!(stock.len(), 1);
    assert_eq!(stock[0].symbol.as_ref().unwrap().as_str(), "KO");
    assert_eq!(
        stock[0].title,
        "Coca-Cola's Momentum Builds After Strong Q2 Earnings: ETFs to Consider"
    );
    assert_eq!(
        stock[0].url,
        "https://www.zacks.com/stock/news/2964897/coca-cola-s-momentum-builds-after-strong-q2-earnings-etfs-to-consider?cid=CS-STOCKNEWSAPI-FT-etf_news_and_commentary-2964897"
    );
    assert_eq!(crypto.len(), 1);
    assert_eq!(crypto[0].symbol.as_ref().unwrap().as_str(), "UNIUSD");
    assert_eq!(
        crypto[0].url,
        "https://cryptobriefing.com/uniswap-launches-beta-tab-token-launches/"
    );
    assert_eq!(forex.len(), 1);
    assert_eq!(forex[0].symbol.as_ref().unwrap().as_str(), "USDJPY");
    assert_eq!(
        forex[0].url,
        "https://www.fxempire.com/forecasts/article/u-s-dollar-retreats-as-gdp-growth-rate-misses-estimates-analysis-for-eur-usd-gbp-usd-usd-cad-usd-jpy-3-1613891"
    );

    let mut unicode = stock[0].clone();
    unicode.text = "café 中文 📈".to_owned();
    let encoded = serde_json::to_value(&unicode).unwrap();
    let decoded: NewsArticle = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded.text, "café 中文 📈");

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "market-news"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/news/stock-latest?from=2026-01-27&to=2026-04-28&page=100&limit=251",
            "https://proxy.example/router/stable/news/crypto-latest?from=2026-01-27&to=2026-04-28&page=0&limit=251",
            "https://proxy.example/router/stable/news/forex-latest?from=2026-01-27&to=2026-04-28&page=0&limit=251",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_independent_omission_and_page_zero() {
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
            json_fixture(STOCK),
            json_fixture(CRYPTO),
            json_fixture(FOREX),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let from = Date::from_str("2026-01-27").unwrap();
        let to = Date::from_str("2026-04-28").unwrap();

        client
            .latest_stock_news(LatestStockNewsQuery::new().with_from(from))
            .await
            .unwrap();
        client
            .latest_crypto_news(LatestCryptoNewsQuery::new().with_to(to))
            .await
            .unwrap();
        client
            .latest_forex_news(LatestForexNewsQuery::new().with_page(Page(0)))
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
                    "https://financialmodelingprep.com/stable/news/stock-latest?from=2026-01-27{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/news/crypto-latest?to=2026-04-28{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/news/forex-latest?page=0{query_suffix}"
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
