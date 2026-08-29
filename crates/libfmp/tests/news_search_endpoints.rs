mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        news::{
            SearchCryptoNewsQuery, SearchForexNewsQuery, SearchPressReleasesQuery,
            SearchStockNewsQuery, search_crypto_news, search_forex_news, search_press_releases,
            search_stock_news,
        },
    },
    responses::news::NewsArticle,
    transport::HttpMethod,
    types::{Date, Limit, Page, Ticker, TickerList},
};

use support::{FixtureExecutor, json_fixture};

const PRESS: &[u8] = include_bytes!("fixtures/search_press_releases.json");
const STOCK: &[u8] = include_bytes!("fixtures/search_stock_news.json");
const CRYPTO: &[u8] = include_bytes!("fixtures/search_crypto_news.json");
const FOREX: &[u8] = include_bytes!("fixtures/search_forex_news.json");

#[test]
fn descriptors_use_exact_paths_vec_rows_and_only_documented_metadata() {
    let tickers = symbols(["AAPL"]);
    let press: EndpointSpec<SearchPressReleasesQuery, Vec<NewsArticle>> =
        search_press_releases(SearchPressReleasesQuery::new(tickers.clone()));
    let stock: EndpointSpec<SearchStockNewsQuery, Vec<NewsArticle>> =
        search_stock_news(SearchStockNewsQuery::new(tickers.clone()));
    let crypto: EndpointSpec<SearchCryptoNewsQuery, Vec<NewsArticle>> =
        search_crypto_news(SearchCryptoNewsQuery::new(tickers.clone()));
    let forex: EndpointSpec<SearchForexNewsQuery, Vec<NewsArticle>> =
        search_forex_news(SearchForexNewsQuery::new(tickers));
    let bounds = EndpointBounds::new().with_response_rows(250).with_page(100);

    assert_facts(
        &press,
        "news/press-releases",
        GeographicAvailability::UsOnly,
        bounds,
    );
    assert_facts(
        &stock,
        "news/stock",
        GeographicAvailability::Unspecified,
        bounds,
    );
    assert_facts(
        &crypto,
        "news/crypto",
        GeographicAvailability::Unspecified,
        bounds,
    );
    assert_facts(
        &forex,
        "news/forex",
        GeographicAvailability::Unspecified,
        bounds,
    );

    assert!(bounds.accepts_page(Page(0)));
    assert!(bounds.accepts_page(Page(100)));
    assert!(!bounds.accepts_page(Page(101)));
    assert_eq!(bounds.limit(), None);
    assert!(bounds.accepts_limit(Limit(251)));
    assert_eq!(bounds.date_range_days(), None);
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
async fn proxy_preserves_complex_symbol_order_exact_queries_headers_and_unicode_rows() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(PRESS),
        json_fixture(STOCK),
        json_fixture(CRYPTO),
        json_fixture(FOREX),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "news-search")
        .executor(executor.clone())
        .build()
        .unwrap();
    let from = Date::from_str("2026-01-27").unwrap();
    let to = Date::from_str("2026-04-28").unwrap();

    let press = client
        .search_press_releases(
            SearchPressReleasesQuery::new(symbols(["AAPL", "BRK.B / Class A"]))
                .with_from(from)
                .with_to(to)
                .with_page(Page(0))
                .with_limit(Limit(251)),
        )
        .await
        .unwrap();
    let stock = client
        .search_stock_news(
            SearchStockNewsQuery::new(symbols(["MSFT", "BRK.B / Class A"]))
                .with_from(from)
                .with_to(to)
                .with_page(Page(0))
                .with_limit(Limit(251)),
        )
        .await
        .unwrap();
    let crypto = client
        .search_crypto_news(
            SearchCryptoNewsQuery::new(symbols(["BTCUSD", "ETH/USD"]))
                .with_from(from)
                .with_to(to)
                .with_page(Page(0))
                .with_limit(Limit(251)),
        )
        .await
        .unwrap();
    let forex = client
        .search_forex_news(
            SearchForexNewsQuery::new(symbols(["EURUSD", "USD/JPY"]))
                .with_from(from)
                .with_to(to)
                .with_page(Page(0))
                .with_limit(Limit(251)),
        )
        .await
        .unwrap();

    assert_eq!(press.len(), 1);
    assert_eq!(press[0].symbol.as_ref().unwrap().as_str(), "AAPL");
    assert_eq!(press[0].text.matches('®').count(), 6);
    assert!(press[0].text.contains("“At Apple"));
    assert_eq!(stock[0].publisher, "CNBC Television");
    assert_eq!(crypto[0].symbol.as_ref().unwrap().as_str(), "BTCUSD");
    assert_eq!(forex[0].symbol.as_ref().unwrap().as_str(), "EURUSD");

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "news-search"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/news/press-releases?symbols=AAPL%2CBRK.B+%2F+Class+A&from=2026-01-27&to=2026-04-28&page=0&limit=251",
            "https://proxy.example/router/gateway/stable/news/stock?symbols=MSFT%2CBRK.B+%2F+Class+A&from=2026-01-27&to=2026-04-28&page=0&limit=251",
            "https://proxy.example/router/gateway/stable/news/crypto?symbols=BTCUSD%2CETH%2FUSD&from=2026-01-27&to=2026-04-28&page=0&limit=251",
            "https://proxy.example/router/gateway/stable/news/forex?symbols=EURUSD%2CUSD%2FJPY&from=2026-01-27&to=2026-04-28&page=0&limit=251",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_independent_optional_fields_and_page_zero() {
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
            json_fixture(PRESS),
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
            .search_press_releases(SearchPressReleasesQuery::new(symbols(["AAPL", "MSFT"])))
            .await
            .unwrap();
        client
            .search_stock_news(SearchStockNewsQuery::new(symbols(["AAPL", "MSFT"])).with_from(from))
            .await
            .unwrap();
        client
            .search_crypto_news(
                SearchCryptoNewsQuery::new(symbols(["BTCUSD", "ETHUSD"])).with_to(to),
            )
            .await
            .unwrap();
        client
            .search_forex_news(
                SearchForexNewsQuery::new(symbols(["EURUSD", "USDJPY"])).with_page(Page(0)),
            )
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
                    "https://financialmodelingprep.com/stable/news/press-releases?symbols=AAPL%2CMSFT{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/news/stock?symbols=AAPL%2CMSFT&from=2026-01-27{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/news/crypto?symbols=BTCUSD%2CETHUSD&to=2026-04-28{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/news/forex?symbols=EURUSD%2CUSDJPY&page=0{query_suffix}"
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

fn symbols<const N: usize>(values: [&str; N]) -> TickerList {
    TickerList::new(
        values
            .into_iter()
            .map(|value| Ticker::new(value).unwrap())
            .collect(),
    )
    .unwrap()
}
