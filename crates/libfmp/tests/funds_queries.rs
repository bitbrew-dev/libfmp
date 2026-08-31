use libfmp::{
    endpoints::funds::{
        EtfAssetExposureQuery, EtfCountryWeightingsQuery, EtfHoldingsQuery, EtfInfoQuery,
        EtfSectorWeightingsQuery, FundDisclosureDatesQuery, FundDisclosureHolderSearchQuery,
        FundDisclosureQuery, LatestFundDisclosureHoldersQuery,
    },
    query::{Quarter, Year},
    types::{Cik, SearchTerm, Ticker},
};

#[test]
fn all_six_symbol_only_queries_preserve_owned_and_borrowed_tickers() {
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    macro_rules! assert_symbol_query {
        ($query:ty) => {{
            let owned: $query = symbol.clone().into();
            assert_eq!(owned.symbol(), &symbol);
            let borrowed: $query = (&symbol).into();
            assert_eq!(borrowed.symbol(), &symbol);
        }};
    }

    assert_symbol_query!(EtfHoldingsQuery);
    assert_symbol_query!(EtfInfoQuery);
    assert_symbol_query!(EtfCountryWeightingsQuery);
    assert_symbol_query!(EtfAssetExposureQuery);
    assert_symbol_query!(EtfSectorWeightingsQuery);
    assert_symbol_query!(LatestFundDisclosureHoldersQuery);
}

#[test]
fn disclosure_query_preserves_required_values_and_optional_cik() {
    let symbol = Ticker::new("000089.SZ").unwrap();
    let query = FundDisclosureQuery::new(symbol.clone(), Year(0), Quarter::Q4);
    assert_eq!(query.symbol(), &symbol);
    assert_eq!(query.year(), Year(0));
    assert_eq!(query.quarter(), Quarter::Q4);
    assert_eq!(query.cik(), None);

    let cik = Cik::new("0000857489").unwrap();
    let query = query.with_cik(cik.clone());
    assert_eq!(query.cik(), Some(&cik));
}

#[test]
fn holder_search_uses_a_representation_preserving_search_term() {
    let name = SearchTerm::new("Federated Hermes Government Income Securities, Inc.").unwrap();
    let owned: FundDisclosureHolderSearchQuery = name.clone().into();
    let borrowed: FundDisclosureHolderSearchQuery = (&name).into();
    assert_eq!(owned.name(), &name);
    assert_eq!(borrowed.name(), &name);
}

#[test]
fn disclosure_dates_preserve_symbol_and_optional_leading_zero_cik() {
    let symbol = Ticker::new("VWO").unwrap();
    let query: FundDisclosureDatesQuery = (&symbol).into();
    assert_eq!(query.symbol(), &symbol);
    assert_eq!(query.cik(), None);

    let cik = Cik::new("0000036405").unwrap();
    assert_eq!(query.with_cik(cik.clone()).cik(), Some(&cik));
}
