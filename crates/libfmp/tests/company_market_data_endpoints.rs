mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        company::{
            HistoricalMarketCapitalizationQuery, MarketCapitalizationBatchQuery,
            MarketCapitalizationQuery, SharesFloatAllQuery, SharesFloatQuery,
            historical_market_capitalization, market_capitalization, market_capitalization_batch,
            shares_float, shares_float_all,
        },
        metadata::{AccessRequirement, GeographicAvailability},
    },
    transport::HttpMethod,
    types::{Date, Limit, Page, Ticker, TickerList},
};

use support::{FixtureExecutor, json_fixture};

const MARKET_CAP: &[u8] = include_bytes!("fixtures/company_market_capitalization.json");
const HISTORICAL_MARKET_CAP: &[u8] =
    include_bytes!("fixtures/company_historical_market_capitalization.json");
const SHARE_FLOAT: &[u8] = include_bytes!("fixtures/company_shares_float.json");
const ALL_SHARE_FLOAT: &[u8] = include_bytes!("fixtures/company_shares_float_all.json");
const EMPTY: &[u8] = include_bytes!("fixtures/company_empty.json");

#[test]
fn descriptors_use_exact_paths_queries_geography_and_response_row_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let symbols = TickerList::new(vec![symbol.clone(), Ticker::new("MSFT").unwrap()]).unwrap();
    let historical_query = HistoricalMarketCapitalizationQuery::new(symbol.clone())
        .with_limit(Limit(5_001))
        .with_from(Date::from_str("2026-04-16").unwrap())
        .with_to(Date::from_str("2026-07-16").unwrap());
    let all_query = SharesFloatAllQuery::new()
        .with_page(Page(u32::MAX))
        .with_limit(Limit(5_001));

    assert_eq!(
        MarketCapitalizationQuery::new(symbol.clone())
            .symbol()
            .as_str(),
        "AAPL"
    );
    assert_eq!(
        MarketCapitalizationBatchQuery::new(symbols.clone())
            .symbols()
            .as_slice(),
        symbols.as_slice()
    );
    assert_eq!(SharesFloatQuery::new(symbol.clone()).symbol(), &symbol);
    assert_eq!(historical_query.symbol(), &symbol);
    assert_eq!(historical_query.limit(), Some(Limit(5_001)));
    assert_eq!(historical_query.from().unwrap().to_string(), "2026-04-16");
    assert_eq!(historical_query.to().unwrap().to_string(), "2026-07-16");
    assert_eq!(all_query.page(), Some(Page(u32::MAX)));
    assert_eq!(all_query.limit(), Some(Limit(5_001)));

    let current = market_capitalization(MarketCapitalizationQuery::new(symbol.clone()));
    let batch = market_capitalization_batch(MarketCapitalizationBatchQuery::new(symbols));
    let historical = historical_market_capitalization(historical_query);
    let float = shares_float(SharesFloatQuery::new(symbol));
    let all = shares_float_all(all_query);

    for (endpoint, expected) in [
        (facts(&current), "market-capitalization"),
        (facts(&batch), "market-capitalization-batch"),
        (facts(&historical), "historical-market-capitalization"),
        (facts(&float), "shares-float"),
        (facts(&all), "shares-float-all"),
    ] {
        assert_eq!(endpoint, expected);
    }

    for bounds in [
        current.metadata().bounds(),
        batch.metadata().bounds(),
        float.metadata().bounds(),
    ] {
        assert_eq!(bounds.response_rows(), None);
    }
    for bounds in [historical.metadata().bounds(), all.metadata().bounds()] {
        assert_eq!(bounds.limit(), None);
        assert_eq!(bounds.page(), None);
        assert!(bounds.accepts_limit(Limit(u32::MAX)));
        assert!(bounds.accepts_page(Page(u32::MAX)));
        assert!(bounds.accepts_response_rows(5_000));
        assert!(!bounds.accepts_response_rows(5_001));
    }
}

fn facts<Q, R>(endpoint: &EndpointSpec<Q, R>) -> &'static str {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), endpoint.relative_path());
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    endpoint.id()
}

#[tokio::test]
async fn proxy_client_preserves_exact_omission_order_encoding_and_independent_dates() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(MARKET_CAP),
        json_fixture(MARKET_CAP),
        json_fixture(HISTORICAL_MARKET_CAP),
        json_fixture(HISTORICAL_MARKET_CAP),
        json_fixture(HISTORICAL_MARKET_CAP),
        json_fixture(HISTORICAL_MARKET_CAP),
        json_fixture(SHARE_FLOAT),
        json_fixture(ALL_SHARE_FLOAT),
        json_fixture(ALL_SHARE_FLOAT),
    ]));
    let client = proxy_client(executor.clone());
    let from = Date::from_str("2026-04-16").unwrap();
    let to = Date::from_str("2026-07-16").unwrap();

    client
        .market_capitalization(Ticker::new("BRK.B / Class A").unwrap())
        .await
        .unwrap();
    client
        .market_capitalization_batch(
            TickerList::new(vec![
                Ticker::new("AAPL").unwrap(),
                Ticker::new("^VIX").unwrap(),
                Ticker::new("000001.SZ").unwrap(),
            ])
            .unwrap(),
        )
        .await
        .unwrap();
    client
        .historical_market_capitalization(Ticker::new("AAPL").unwrap())
        .await
        .unwrap();
    client
        .historical_market_capitalization(
            HistoricalMarketCapitalizationQuery::new(Ticker::new("AAPL").unwrap()).with_from(from),
        )
        .await
        .unwrap();
    client
        .historical_market_capitalization(
            HistoricalMarketCapitalizationQuery::new(Ticker::new("AAPL").unwrap()).with_to(to),
        )
        .await
        .unwrap();
    client
        .historical_market_capitalization(
            HistoricalMarketCapitalizationQuery::new(Ticker::new("AAPL").unwrap())
                .with_limit(Limit(5_001))
                .with_from(to)
                .with_to(from),
        )
        .await
        .unwrap();
    client
        .shares_float(Ticker::new("AAPL / USD").unwrap())
        .await
        .unwrap();
    client
        .shares_float_all(SharesFloatAllQuery::new())
        .await
        .unwrap();
    client
        .shares_float_all(
            SharesFloatAllQuery::new()
                .with_page(Page(0))
                .with_limit(Limit(5_001)),
        )
        .await
        .unwrap();

    let requests = executor.requests();
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/market-capitalization?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/market-capitalization-batch?symbols=AAPL%2C%5EVIX%2C000001.SZ",
            "https://proxy.example/router/stable/historical-market-capitalization?symbol=AAPL",
            "https://proxy.example/router/stable/historical-market-capitalization?symbol=AAPL&from=2026-04-16",
            "https://proxy.example/router/stable/historical-market-capitalization?symbol=AAPL&to=2026-07-16",
            "https://proxy.example/router/stable/historical-market-capitalization?symbol=AAPL&limit=5001&from=2026-07-16&to=2026-04-16",
            "https://proxy.example/router/stable/shares-float?symbol=AAPL+%2F+USD",
            "https://proxy.example/router/stable/shares-float-all",
            "https://proxy.example/router/stable/shares-float-all?page=0&limit=5001",
        ]
    );
    assert!(!urls[2].contains("limit="));
    assert!(!urls[2].contains("from="));
    assert!(!urls[2].contains("to="));
    assert!(!urls[7].contains('?'));
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
}

#[tokio::test]
async fn every_market_data_client_method_preserves_empty_arrays() {
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| json_fixture(EMPTY)).take(5),
    ));
    let client = proxy_client(executor);

    assert!(
        client
            .market_capitalization(Ticker::new("AAPL").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .market_capitalization_batch(
                TickerList::new(vec![Ticker::new("AAPL").unwrap()]).unwrap()
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .historical_market_capitalization(Ticker::new("AAPL").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .shares_float(Ticker::new("AAPL").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .shares_float_all(SharesFloatAllQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_use_the_same_market_data_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/shares-float?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/market-capitalization-batch?symbols=AAPL%2CMSFT&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            SHARE_FLOAT
        } else {
            MARKET_CAP
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            client
                .shares_float(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
        } else {
            client
                .market_capitalization_batch(
                    TickerList::new(vec![
                        Ticker::new("AAPL").unwrap(),
                        Ticker::new("MSFT").unwrap(),
                    ])
                    .unwrap(),
                )
                .await
                .unwrap();
        }

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some((name, value)) => assert_eq!(requests[0].expose_headers()[name], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
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
