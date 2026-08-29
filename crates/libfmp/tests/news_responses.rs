use libfmp::{
    responses::news::{FmpArticle, NewsArticle},
    types::{ApiDateTime, Ticker},
};
use serde::de::DeserializeOwned;

const FMP_ARTICLES: &[u8] = include_bytes!("fixtures/fmp_articles.json");
const LATEST_GENERAL: &[u8] = include_bytes!("fixtures/latest_general_news.json");
const LATEST_PRESS_RELEASES: &[u8] = include_bytes!("fixtures/latest_press_releases.json");
const LATEST_STOCK: &[u8] = include_bytes!("fixtures/latest_stock_news.json");
const LATEST_CRYPTO: &[u8] = include_bytes!("fixtures/latest_crypto_news.json");
const LATEST_FOREX: &[u8] = include_bytes!("fixtures/latest_forex_news.json");
const SEARCH_PRESS_RELEASES: &[u8] = include_bytes!("fixtures/search_press_releases.json");
const SEARCH_STOCK: &[u8] = include_bytes!("fixtures/search_stock_news.json");
const SEARCH_CRYPTO: &[u8] = include_bytes!("fixtures/search_crypto_news.json");
const SEARCH_FOREX: &[u8] = include_bytes!("fixtures/search_forex_news.json");

const FMP_CONTENT: &str = "<ul>\n    <li><strong>Strategic Growth & Operational Excellence:</strong> Centerra Gold is strategically investing in its North American assets, exemplified by a <strong>29%</strong> increase in gold production at the Mount Milligan mine, driving future growth.</li>\n    <li><strong>Robust Financial Health & Increased Guidance:</strong> The company maintains a strong financial position with zero long-term debt and approximately <strong>$450 million</strong> in cash, leading to increased 2026 gold ...";
const PRESS_RELEASE_TEXT: &str = "NEW YORK, July 30, 2026 (GLOBE NEWSWIRE) -- Gainey McKenna & Egleston announces that a securities class action lawsuit has been filed in the United States District Court for the Southern District of New York on behalf of all persons or entities who purchased or otherwise acquired Rackspace Technology, Inc. (“Rackspace” or the “Company”) (NASDAQ: RXT) securities between May 7, 2026 and July 8, 2026, inclusive (the “Class Period”).";
const APPLE_TEXT: &str = "CUPERTINO, Calif.--(BUSINESS WIRE)--Apple® today announced Apple Upgrade, a new product leasing program provided by Klarna for iPhone®, Apple Watch®, Mac®, and iPad® available on the Apple Store® online, in the Apple Store app, and at Apple Store locations in the United States.1 Apple Upgrade makes it even easier for customers to get the Apple products they love with a leasing plan that is right for them. “At Apple, we put the customer at the center of everything we do,” said Karen Rasmussen, A.";

#[test]
fn exact_fmp_fixture_preserves_all_eight_required_opaque_fields() {
    assert_field_count(FMP_ARTICLES, 8);
    let rows: Vec<FmpArticle> = serde_json::from_slice(FMP_ARTICLES).unwrap();
    assert_eq!(
        rows,
        [FmpArticle {
            title: "Centerra Gold (NYSE:CGAU) Drives Growth with North American Investments and Strong Financials".to_owned(),
            date: ApiDateTime::parse("2026-07-30 16:11:45").unwrap(),
            content: FMP_CONTENT.to_owned(),
            tickers: "NYSE:CGAU".to_owned(),
            image: "https://portal.financialmodelingprep.com/positions/6a6b7d9c2514096bb2027277.jpeg".to_owned(),
            link: "https://financialmodelingprep.com/market-news/centerra-gold-cgau-growth-north-american-investments-strong-financials".to_owned(),
            author: "Andrew Wynn".to_owned(),
            site: "Financial Modeling Prep".to_owned(),
        }]
    );
    assert!(rows[0].content.starts_with("<ul>\n    <li><strong>"));
    assert!(rows[0].content.contains("<strong>$450 million</strong>"));
    assert!(rows[0].content.ends_with("gold ..."));
    assert_eq!(rows[0].tickers, "NYSE:CGAU");
}

#[test]
fn all_nine_exact_provider_fixtures_share_the_same_eight_field_row() {
    let cases = [
        (LATEST_GENERAL, None, "Seeking Alpha"),
        (LATEST_PRESS_RELEASES, Some("RXT"), "GlobeNewsWire"),
        (LATEST_STOCK, Some("KO"), "Zacks Investment Research"),
        (LATEST_CRYPTO, Some("UNIUSD"), "Crypto Briefing"),
        (LATEST_FOREX, Some("USDJPY"), "FXEmpire"),
        (SEARCH_PRESS_RELEASES, Some("AAPL"), "Business Wire"),
        (SEARCH_STOCK, Some("AAPL"), "CNBC Television"),
        (SEARCH_CRYPTO, Some("BTCUSD"), "AMBCrypto"),
        (SEARCH_FOREX, Some("EURUSD"), "FXEmpire"),
    ];

    for (fixture, symbol, publisher) in cases {
        assert_field_count(fixture, 8);
        let rows: Vec<NewsArticle> = serde_json::from_slice(fixture).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].symbol.as_ref().map(Ticker::as_str), symbol);
        assert_eq!(rows[0].publisher, publisher);
        assert!(rows[0].image.starts_with("https://"));
        assert!(rows[0].url.starts_with("https://"));
    }
}

#[test]
fn exact_unicode_html_and_url_strings_are_not_normalized_or_parsed() {
    let latest_press: Vec<NewsArticle> = serde_json::from_slice(LATEST_PRESS_RELEASES).unwrap();
    assert_eq!(latest_press[0].text, PRESS_RELEASE_TEXT);
    assert!(latest_press[0].text.contains("“Rackspace”"));

    let search_press: Vec<NewsArticle> = serde_json::from_slice(SEARCH_PRESS_RELEASES).unwrap();
    assert_eq!(search_press[0].text, APPLE_TEXT);
    assert_eq!(search_press[0].text.matches('®').count(), 6);
    assert!(search_press[0].text.contains("“At Apple"));

    let stock: Vec<NewsArticle> = serde_json::from_slice(LATEST_STOCK).unwrap();
    assert_eq!(
        stock[0].url,
        "https://www.zacks.com/stock/news/2964897/coca-cola-s-momentum-builds-after-strong-q2-earnings-etfs-to-consider?cid=CS-STOCKNEWSAPI-FT-etf_news_and_commentary-2964897"
    );
    let searched: Vec<NewsArticle> = serde_json::from_slice(SEARCH_STOCK).unwrap();
    assert_eq!(
        searched[0].url,
        "https://www.youtube.com/watch?v=ZKMD80U8dRM"
    );
}

#[test]
fn symbol_key_is_required_but_explicit_null_is_preserved() {
    let general: Vec<NewsArticle> = serde_json::from_slice(LATEST_GENERAL).unwrap();
    assert_eq!(general[0].symbol, None);

    let mut missing: serde_json::Value = serde_json::from_slice(LATEST_GENERAL).unwrap();
    missing[0].as_object_mut().unwrap().remove("symbol");
    assert!(serde_json::from_value::<Vec<NewsArticle>>(missing).is_err());

    let encoded = serde_json::to_value(&general[0]).unwrap();
    assert!(encoded.get("symbol").is_some());
    assert!(encoded["symbol"].is_null());
}

#[test]
fn documented_fields_are_required_nullability_is_exact_and_unknowns_are_accepted() {
    assert_contract::<FmpArticle>(FMP_ARTICLES, &[]);
    for fixture in [
        LATEST_GENERAL,
        LATEST_PRESS_RELEASES,
        LATEST_STOCK,
        LATEST_CRYPTO,
        LATEST_FOREX,
        SEARCH_PRESS_RELEASES,
        SEARCH_STOCK,
        SEARCH_CRYPTO,
        SEARCH_FOREX,
    ] {
        assert_contract::<NewsArticle>(fixture, &["symbol"]);
    }
}

#[test]
fn response_string_and_datetime_kinds_are_strict() {
    let mut article: serde_json::Value = serde_json::from_slice(FMP_ARTICLES).unwrap();
    article[0]["date"] = serde_json::json!(20260730161145_u64);
    assert!(serde_json::from_value::<Vec<FmpArticle>>(article).is_err());

    for replacement in [serde_json::json!(0), serde_json::json!(1.5)] {
        let mut news: serde_json::Value = serde_json::from_slice(LATEST_STOCK).unwrap();
        news[0]["publishedDate"] = replacement.clone();
        assert!(serde_json::from_value::<Vec<NewsArticle>>(news).is_err());

        let mut news: serde_json::Value = serde_json::from_slice(LATEST_STOCK).unwrap();
        news[0]["title"] = replacement;
        assert!(serde_json::from_value::<Vec<NewsArticle>>(news).is_err());
    }

    for invalid in [
        "2026-07-30T13:15:49",
        "2026-07-30 13:15:49Z",
        "2026-07-30 13:15",
    ] {
        let mut news: serde_json::Value = serde_json::from_slice(LATEST_STOCK).unwrap();
        news[0]["publishedDate"] = serde_json::json!(invalid);
        assert!(serde_json::from_value::<Vec<NewsArticle>>(news).is_err());
    }
}

#[test]
fn both_rows_are_bare_arrays_preserving_empty_multiple_and_large_opaque_strings() {
    assert!(
        serde_json::from_slice::<Vec<FmpArticle>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<NewsArticle>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_value::<Vec<NewsArticle>>(serde_json::json!({ "articles": [] })).is_err()
    );

    let mut multiple_fmp: serde_json::Value = serde_json::from_slice(FMP_ARTICLES).unwrap();
    let duplicate_fmp = multiple_fmp[0].clone();
    multiple_fmp.as_array_mut().unwrap().push(duplicate_fmp);
    assert_eq!(
        serde_json::from_value::<Vec<FmpArticle>>(multiple_fmp)
            .unwrap()
            .len(),
        2
    );

    let mut multiple: serde_json::Value = serde_json::from_slice(LATEST_STOCK).unwrap();
    let duplicate = multiple[0].clone();
    multiple.as_array_mut().unwrap().push(duplicate);
    assert_eq!(
        serde_json::from_value::<Vec<NewsArticle>>(multiple)
            .unwrap()
            .len(),
        2
    );

    let payload = "<p>café 中文 📈 &amp; exact</p>".repeat(40_000);
    let mut article: serde_json::Value = serde_json::from_slice(FMP_ARTICLES).unwrap();
    article[0]["content"] = serde_json::json!(payload);
    let rows: Vec<FmpArticle> = serde_json::from_value(article).unwrap();
    assert_eq!(rows[0].content, payload);
}

fn assert_contract<T: DeserializeOwned>(fixture: &[u8], nullable: &[&str]) {
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let keys = source[0]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    for key in keys {
        let mut missing = source.clone();
        missing[0].as_object_mut().unwrap().remove(&key);
        assert!(
            serde_json::from_value::<Vec<T>>(missing).is_err(),
            "accepted missing {key}"
        );

        let mut null = source.clone();
        null[0][&key] = serde_json::Value::Null;
        let decoded = serde_json::from_value::<Vec<T>>(null);
        assert_eq!(
            decoded.is_ok(),
            nullable.contains(&key.as_str()),
            "unexpected null behavior for {key}"
        );
    }

    let mut forward = source;
    forward[0]["futureProviderField"] = serde_json::json!({ "nested": [1, true, null] });
    assert_eq!(serde_json::from_value::<Vec<T>>(forward).unwrap().len(), 1);
}

fn assert_field_count(fixture: &[u8], expected: usize) {
    let value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), expected);
}
