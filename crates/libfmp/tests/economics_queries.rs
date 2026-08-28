use std::str::FromStr;

use libfmp::{
    endpoints::economics::{EconomicCalendarQuery, EconomicIndicatorsQuery, TreasuryRatesQuery},
    query::EconomicIndicator,
    types::{CountryCode, Date},
};

#[test]
fn public_query_builders_preserve_required_values_and_independent_options() {
    let from = Date::from_str("2026-01-27").unwrap();
    let to = Date::from_str("2026-04-27").unwrap();

    let treasury = TreasuryRatesQuery::new().with_from(from);
    assert_eq!(treasury.from(), Some(from));
    assert_eq!(treasury.to(), None);

    let indicators: EconomicIndicatorsQuery = EconomicIndicator::Gdp.into();
    assert_eq!(indicators.name(), &EconomicIndicator::Gdp);
    assert_eq!(indicators.from(), None);
    assert_eq!(indicators.to(), None);

    let calendar = EconomicCalendarQuery::new()
        .with_country(CountryCode::new("US").unwrap())
        .with_to(to);
    assert_eq!(calendar.country().unwrap().as_str(), "US");
    assert_eq!(calendar.from(), None);
    assert_eq!(calendar.to(), Some(to));
}

#[test]
fn indicator_query_reuses_all_documented_and_open_indicator_values() {
    for indicator in EconomicIndicator::DOCUMENTED {
        let expected = indicator.as_str().to_owned();
        let query = EconomicIndicatorsQuery::new(indicator);
        assert_eq!(query.name().as_str(), expected);
    }

    let query =
        EconomicIndicatorsQuery::new(EconomicIndicator::new("futureProviderIndicator").unwrap());
    assert_eq!(query.name().as_str(), "futureProviderIndicator");
}
