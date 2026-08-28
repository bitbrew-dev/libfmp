//! Economics endpoint and query contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata},
    },
    query::EconomicIndicator,
    responses::economics::{
        EconomicCalendarEvent, EconomicIndicatorObservation, MarketRiskPremium, TreasuryRate,
    },
    types::{CountryCode, Date},
};

/// Optional independent date bounds for Treasury-rate observations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TreasuryRatesQuery {
    from: Option<Date>,
    to: Option<Date>,
}

impl TreasuryRatesQuery {
    /// Creates a query without undocumented date defaults.
    pub const fn new() -> Self {
        Self {
            from: None,
            to: None,
        }
    }

    /// Sets the optional independent start date.
    pub const fn with_from(mut self, from: Date) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the optional independent end date.
    pub const fn with_to(mut self, to: Date) -> Self {
        self.to = Some(to);
        self
    }

    /// Returns the optional independent start date.
    pub const fn from(&self) -> Option<Date> {
        self.from
    }

    /// Returns the optional independent end date.
    pub const fn to(&self) -> Option<Date> {
        self.to
    }
}

impl QueryParameters for TreasuryRatesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
    }
}

/// Required indicator name and optional independent observation date bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EconomicIndicatorsQuery {
    name: EconomicIndicator,
    from: Option<Date>,
    to: Option<Date>,
}

impl EconomicIndicatorsQuery {
    /// Creates a query for one documented or validated open indicator name.
    pub const fn new(name: EconomicIndicator) -> Self {
        Self {
            name,
            from: None,
            to: None,
        }
    }

    /// Sets the optional independent start date.
    pub const fn with_from(mut self, from: Date) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the optional independent end date.
    pub const fn with_to(mut self, to: Date) -> Self {
        self.to = Some(to);
        self
    }

    /// Borrows the requested indicator name.
    pub const fn name(&self) -> &EconomicIndicator {
        &self.name
    }

    /// Returns the optional independent start date.
    pub const fn from(&self) -> Option<Date> {
        self.from
    }

    /// Returns the optional independent end date.
    pub const fn to(&self) -> Option<Date> {
        self.to
    }
}

impl From<EconomicIndicator> for EconomicIndicatorsQuery {
    fn from(name: EconomicIndicator) -> Self {
        Self::new(name)
    }
}

impl From<&EconomicIndicator> for EconomicIndicatorsQuery {
    fn from(name: &EconomicIndicator) -> Self {
        Self::new(name.clone())
    }
}

impl QueryParameters for EconomicIndicatorsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("name", &self.name);
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
    }
}

const NINETY_DAY_RANGE: EndpointMetadata =
    EndpointMetadata::new().with_bounds(EndpointBounds::new().with_date_range_days(90));

/// Describes `GET treasury-rates` without binding a transport.
pub fn treasury_rates(
    query: TreasuryRatesQuery,
) -> EndpointSpec<TreasuryRatesQuery, Vec<TreasuryRate>> {
    EndpointSpec::get("treasury-rates", "treasury-rates", query).with_metadata(NINETY_DAY_RANGE)
}

/// Describes `GET economic-indicators` without binding a transport.
pub fn economic_indicators(
    query: EconomicIndicatorsQuery,
) -> EndpointSpec<EconomicIndicatorsQuery, Vec<EconomicIndicatorObservation>> {
    EndpointSpec::get("economic-indicators", "economic-indicators", query)
        .with_metadata(NINETY_DAY_RANGE)
}

impl Client {
    /// Retrieves Treasury-rate observations within the documented 90-day range.
    pub async fn treasury_rates(
        &self,
        query: impl Into<TreasuryRatesQuery>,
    ) -> Result<Vec<TreasuryRate>> {
        self.execute(&treasury_rates(query.into())).await
    }

    /// Retrieves observations for one economic indicator within 90 days.
    pub async fn economic_indicators(
        &self,
        query: impl Into<EconomicIndicatorsQuery>,
    ) -> Result<Vec<EconomicIndicatorObservation>> {
        self.execute(&economic_indicators(query.into())).await
    }
}

/// Optional country and independent date bounds for economic-calendar events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EconomicCalendarQuery {
    country: Option<CountryCode>,
    from: Option<Date>,
    to: Option<Date>,
}

impl EconomicCalendarQuery {
    /// Creates a query without undocumented filter defaults.
    pub const fn new() -> Self {
        Self {
            country: None,
            from: None,
            to: None,
        }
    }

    /// Sets the optional provider country code.
    pub fn with_country(mut self, country: CountryCode) -> Self {
        self.country = Some(country);
        self
    }

    /// Sets the optional independent start date.
    pub const fn with_from(mut self, from: Date) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the optional independent end date.
    pub const fn with_to(mut self, to: Date) -> Self {
        self.to = Some(to);
        self
    }

    /// Borrows the optional provider country code.
    pub const fn country(&self) -> Option<&CountryCode> {
        self.country.as_ref()
    }

    /// Returns the optional independent start date.
    pub const fn from(&self) -> Option<Date> {
        self.from
    }

    /// Returns the optional independent end date.
    pub const fn to(&self) -> Option<Date> {
        self.to
    }
}

impl QueryParameters for EconomicCalendarQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("country", self.country.as_ref());
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
    }
}

/// Describes `GET economic-calendar` without binding a transport.
pub fn economic_calendar(
    query: EconomicCalendarQuery,
) -> EndpointSpec<EconomicCalendarQuery, Vec<EconomicCalendarEvent>> {
    EndpointSpec::get("economic-calendar", "economic-calendar", query)
        .with_metadata(NINETY_DAY_RANGE)
}

/// Describes queryless `GET market-risk-premium` without binding a transport.
pub fn market_risk_premium() -> EndpointSpec<(), Vec<MarketRiskPremium>> {
    EndpointSpec::get("market-risk-premium", "market-risk-premium", ())
}

impl Client {
    /// Retrieves economic-calendar events within the documented 90-day range.
    pub async fn economic_calendar(
        &self,
        query: impl Into<EconomicCalendarQuery>,
    ) -> Result<Vec<EconomicCalendarEvent>> {
        self.execute(&economic_calendar(query.into())).await
    }

    /// Retrieves the provider's queryless country risk-premium rows.
    pub async fn market_risk_premium(&self) -> Result<Vec<MarketRiskPremium>> {
        self.execute(&market_risk_premium()).await
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn pairs(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        query.encode(&mut QueryEncoder::new(&mut |name, value| {
            pairs.push((name.to_owned(), value.to_owned()));
        }));
        pairs
    }

    #[test]
    fn treasury_dates_are_independent_and_encode_in_exact_order() {
        let from = Date::from_str("2026-01-27").unwrap();
        let to = Date::from_str("2026-04-27").unwrap();
        assert!(pairs(&TreasuryRatesQuery::new()).is_empty());
        assert_eq!(TreasuryRatesQuery::new().with_from(from).to(), None);
        assert_eq!(TreasuryRatesQuery::new().with_to(to).from(), None);

        let query = TreasuryRatesQuery::new().with_from(from).with_to(to);
        assert_eq!(query.from(), Some(from));
        assert_eq!(query.to(), Some(to));
        assert_eq!(
            pairs(&query),
            [
                ("from".to_owned(), "2026-01-27".to_owned()),
                ("to".to_owned(), "2026-04-27".to_owned()),
            ]
        );
    }

    #[test]
    fn indicator_name_precedes_dates_and_preserves_known_and_open_values() {
        let from = Date::from_str("2025-04-27").unwrap();
        let to = Date::from_str("2026-04-27").unwrap();
        let query = EconomicIndicatorsQuery::new(EconomicIndicator::Gdp)
            .with_from(from)
            .with_to(to);
        assert_eq!(query.name(), &EconomicIndicator::Gdp);
        assert_eq!(query.from(), Some(from));
        assert_eq!(query.to(), Some(to));
        assert_eq!(
            pairs(&query),
            [
                ("name".to_owned(), "GDP".to_owned()),
                ("from".to_owned(), "2025-04-27".to_owned()),
                ("to".to_owned(), "2026-04-27".to_owned()),
            ]
        );

        let open = EconomicIndicator::new("futureProviderIndicator").unwrap();
        let converted: EconomicIndicatorsQuery = (&open).into();
        assert_eq!(
            pairs(&converted),
            [("name".to_owned(), "futureProviderIndicator".to_owned())]
        );
    }

    #[test]
    fn calendar_omits_absent_filters_and_encodes_country_then_dates() {
        let from = Date::from_str("2026-01-27").unwrap();
        let to = Date::from_str("2026-04-27").unwrap();
        assert!(pairs(&EconomicCalendarQuery::new()).is_empty());
        assert_eq!(EconomicCalendarQuery::new().with_from(from).to(), None);
        assert_eq!(EconomicCalendarQuery::new().with_to(to).from(), None);

        let query = EconomicCalendarQuery::new()
            .with_country(CountryCode::new("US").unwrap())
            .with_from(from)
            .with_to(to);
        assert_eq!(query.country().unwrap().as_str(), "US");
        assert_eq!(query.from(), Some(from));
        assert_eq!(query.to(), Some(to));
        assert_eq!(
            pairs(&query),
            [
                ("country".to_owned(), "US".to_owned()),
                ("from".to_owned(), "2026-01-27".to_owned()),
                ("to".to_owned(), "2026-04-27".to_owned()),
            ]
        );
    }
}
