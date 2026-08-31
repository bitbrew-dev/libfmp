use libfmp::{
    endpoints::sec_filings::{
        AllIndustryClassificationsQuery, IndustryClassificationSearchQuery,
        IndustryClassificationsQuery, Latest8kSecFilingsQuery, LatestSecFilingsQuery,
        SecCompaniesByCikQuery, SecCompaniesByNameQuery, SecCompaniesBySymbolQuery,
        SecCompanyProfileQuery, SecFilingsByCikQuery, SecFilingsByFormTypeQuery,
        SecFilingsBySymbolQuery,
    },
    types::{Cik, Date, FormType, Limit, Page, SearchTerm, Ticker},
};

#[test]
fn filing_feed_queries_keep_independent_dates_and_unbounded_pagination() {
    let from = Date::parse("2024-03-01").unwrap();
    let to = Date::parse("2024-01-01").unwrap();

    let latest = Latest8kSecFilingsQuery::new(from, to)
        .with_page(Page(0))
        .with_limit(Limit(u32::MAX));
    assert_eq!(latest.from(), from);
    assert_eq!(latest.to(), to);
    assert_eq!(latest.page(), Some(Page(0)));
    assert_eq!(latest.limit(), Some(Limit(u32::MAX)));

    let financials = LatestSecFilingsQuery::new(from, to);
    assert_eq!(financials.from(), from);
    assert_eq!(financials.to(), to);
    assert_eq!(financials.page(), None);
    assert_eq!(financials.limit(), None);
}

#[test]
fn all_three_filing_searches_preserve_required_keys_dates_and_pagination() {
    let from = Date::parse("2024-01-01").unwrap();
    let to = Date::parse("2024-03-01").unwrap();

    let form = SecFilingsByFormTypeQuery::new(FormType::new("10-K/A amendment").unwrap(), from, to)
        .with_page(Page(u32::MAX))
        .with_limit(Limit(0));
    assert_eq!(form.form_type().as_str(), "10-K/A amendment");
    assert_eq!(form.from(), from);
    assert_eq!(form.to(), to);
    assert_eq!(form.page(), Some(Page(u32::MAX)));
    assert_eq!(form.limit(), Some(Limit(0)));

    let symbol = SecFilingsBySymbolQuery::new(Ticker::new("000001.SZ").unwrap(), from, to);
    assert_eq!(symbol.symbol().as_str(), "000001.SZ");
    assert_eq!(symbol.page(), None);
    assert_eq!(symbol.limit(), None);

    let cik = SecFilingsByCikQuery::new(Cik::new("0000320193").unwrap(), from, to);
    assert_eq!(cik.cik().as_str(), "0000320193");
    assert_eq!(cik.from(), from);
    assert_eq!(cik.to(), to);
}

#[test]
fn company_search_queries_preserve_name_ticker_and_leading_zero_cik() {
    let company = SearchTerm::new("Berkshire, Hathaway").unwrap();
    let by_name: SecCompaniesByNameQuery = (&company).into();
    assert_eq!(by_name.company(), &company);

    let symbol = Ticker::new("BRK.B").unwrap();
    let by_symbol: SecCompaniesBySymbolQuery = symbol.clone().into();
    assert_eq!(by_symbol.symbol(), &symbol);

    let cik = Cik::new("0000320193").unwrap();
    let by_cik: SecCompaniesByCikQuery = (&cik).into();
    assert_eq!(by_cik.cik(), &cik);
}

#[test]
fn profile_and_classification_queries_preserve_omission_and_independent_inputs() {
    let profile: SecCompanyProfileQuery = Ticker::new("AAPL").unwrap().into();
    assert_eq!(profile.symbol().as_str(), "AAPL");
    assert_eq!(profile.cik_a(), None);
    assert_eq!(
        profile
            .with_cik_a(Cik::new("0000320193").unwrap())
            .cik_a()
            .unwrap()
            .as_str(),
        "0000320193"
    );

    let list = IndustryClassificationsQuery::new()
        .with_sic_code(SearchTerm::new("07371").unwrap())
        .with_industry_title(SearchTerm::new("SERVICES, NEC").unwrap());
    assert_eq!(list.sic_code().unwrap().as_str(), "07371");
    assert_eq!(list.industry_title().unwrap().as_str(), "SERVICES, NEC");

    let search = IndustryClassificationSearchQuery::new()
        .with_symbol(Ticker::new("AAPL").unwrap())
        .with_cik(Cik::new("0000320193").unwrap())
        .with_sic_code(SearchTerm::new("3571").unwrap());
    assert_eq!(search.symbol().unwrap().as_str(), "AAPL");
    assert_eq!(search.cik().unwrap().as_str(), "0000320193");
    assert_eq!(search.sic_code().unwrap().as_str(), "3571");

    let all = AllIndustryClassificationsQuery::new()
        .with_page(Page(0))
        .with_limit(Limit(u32::MAX));
    assert_eq!(all.page(), Some(Page(0)));
    assert_eq!(all.limit(), Some(Limit(u32::MAX)));
}
