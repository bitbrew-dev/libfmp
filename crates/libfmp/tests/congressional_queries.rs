use libfmp::{
    endpoints::congressional::{
        CongressionalTradesByMemberIdQuery, CongressionalTradesByNameQuery,
        CongressionalTradesQuery, LatestCongressionalDisclosuresQuery,
    },
    types::{CongressionalMemberId, Limit, Page, SearchTerm, Ticker},
};

#[test]
fn latest_query_preserves_omission_zero_and_full_numeric_domains() {
    let empty = LatestCongressionalDisclosuresQuery::new();
    assert_eq!(empty.page(), None);
    assert_eq!(empty.limit(), None);
    let query = empty.with_page(Page(0)).with_limit(Limit(u32::MAX));
    assert_eq!(query.page(), Some(Page(0)));
    assert_eq!(query.limit(), Some(Limit(u32::MAX)));
}

#[test]
fn symbol_query_requires_a_valid_ticker_and_keeps_pagination_optional() {
    assert!(Ticker::new("").is_err());
    let ticker = Ticker::new("000001.SZ").unwrap();
    let query: CongressionalTradesQuery = (&ticker).into();
    assert_eq!(query.symbol(), &ticker);
    assert_eq!(query.page(), None);
    assert_eq!(query.limit(), None);
    let query = query.with_page(Page(u32::MAX)).with_limit(Limit(0));
    assert_eq!(query.page(), Some(Page(u32::MAX)));
    assert_eq!(query.limit(), Some(Limit(0)));
}

#[test]
fn name_and_id_queries_preserve_representation_and_optional_id() {
    let name = SearchTerm::new("  James A.  ").unwrap();
    let query: CongressionalTradesByNameQuery = (&name).into();
    assert_eq!(query.name().as_str(), "  James A.  ");

    let empty = CongressionalTradesByMemberIdQuery::new();
    assert_eq!(empty.page(), None);
    assert_eq!(empty.limit(), None);
    assert_eq!(empty.member_id(), None);
    let query = empty
        .with_page(Page(0))
        .with_limit(Limit(100))
        .with_member_id(CongressionalMemberId::new("P000197").unwrap());
    assert_eq!(query.page(), Some(Page(0)));
    assert_eq!(query.limit(), Some(Limit(100)));
    assert_eq!(query.member_id().unwrap().as_str(), "P000197");
}
