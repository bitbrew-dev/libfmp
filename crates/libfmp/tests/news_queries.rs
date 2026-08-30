use std::str::FromStr;

use libfmp::{
    endpoints::news::{
        FmpArticlesQuery, LatestCryptoNewsQuery, LatestForexNewsQuery, LatestGeneralNewsQuery,
        LatestPressReleasesQuery, LatestStockNewsQuery, SearchCryptoNewsQuery,
        SearchForexNewsQuery, SearchPressReleasesQuery, SearchStockNewsQuery,
    },
    types::{Date, Limit, Page, Ticker, TickerList},
};

#[test]
fn fmp_articles_preserves_omission_page_zero_and_unbounded_limit_units() {
    let empty = FmpArticlesQuery::new();
    assert_eq!(empty.page(), None);
    assert_eq!(empty.limit(), None);

    let query = empty.with_page(Page(0)).with_limit(Limit(u32::MAX));
    assert_eq!(query.page(), Some(Page(0)));
    assert_eq!(query.limit(), Some(Limit(u32::MAX)));
}

#[test]
fn every_latest_query_preserves_independent_dates_and_zero_values() {
    let from = Date::from_str("2026-01-27").unwrap();
    let to = Date::from_str("2026-04-28").unwrap();

    macro_rules! assert_latest {
        ($query:ty) => {{
            let empty = <$query>::new();
            assert_eq!(empty.from(), None);
            assert_eq!(empty.to(), None);
            assert_eq!(empty.page(), None);
            assert_eq!(empty.limit(), None);

            let from_only = empty.with_from(from);
            assert_eq!(from_only.from(), Some(from));
            assert_eq!(from_only.to(), None);

            let to_only = <$query>::new().with_to(to);
            assert_eq!(to_only.from(), None);
            assert_eq!(to_only.to(), Some(to));

            let all = <$query>::new()
                .with_from(to)
                .with_to(from)
                .with_page(Page(0))
                .with_limit(Limit(0));
            assert_eq!(all.from(), Some(to));
            assert_eq!(all.to(), Some(from));
            assert_eq!(all.page(), Some(Page(0)));
            assert_eq!(all.limit(), Some(Limit(0)));
        }};
    }

    assert_latest!(LatestGeneralNewsQuery);
    assert_latest!(LatestPressReleasesQuery);
    assert_latest!(LatestStockNewsQuery);
    assert_latest!(LatestCryptoNewsQuery);
    assert_latest!(LatestForexNewsQuery);
}

#[test]
fn every_search_query_requires_nonempty_symbols_and_preserves_all_options() {
    assert!(TickerList::new(Vec::new()).is_err());
    let symbols = TickerList::new(vec![
        Ticker::new("AAPL").unwrap(),
        Ticker::new("MSFT").unwrap(),
    ])
    .unwrap();
    let from = Date::from_str("2026-01-27").unwrap();
    let to = Date::from_str("2026-04-28").unwrap();

    macro_rules! assert_search {
        ($query:ty) => {{
            let owned: $query = symbols.clone().into();
            assert_eq!(owned.symbols(), &symbols);
            assert_eq!(owned.from(), None);
            assert_eq!(owned.to(), None);
            assert_eq!(owned.page(), None);
            assert_eq!(owned.limit(), None);

            let borrowed: $query = (&symbols).into();
            assert_eq!(borrowed.symbols(), &symbols);

            let from_only = <$query>::new(symbols.clone()).with_from(from);
            assert_eq!(from_only.from(), Some(from));
            assert_eq!(from_only.to(), None);

            let to_only = <$query>::new(symbols.clone()).with_to(to);
            assert_eq!(to_only.from(), None);
            assert_eq!(to_only.to(), Some(to));

            let reversed = <$query>::new(symbols.clone())
                .with_from(to)
                .with_to(from)
                .with_page(Page(0))
                .with_limit(Limit(u32::MAX));
            assert_eq!(reversed.from(), Some(to));
            assert_eq!(reversed.to(), Some(from));
            assert_eq!(reversed.page(), Some(Page(0)));
            assert_eq!(reversed.limit(), Some(Limit(u32::MAX)));
        }};
    }

    assert_search!(SearchPressReleasesQuery);
    assert_search!(SearchStockNewsQuery);
    assert_search!(SearchCryptoNewsQuery);
    assert_search!(SearchForexNewsQuery);
}
