//! Worldwide exchange market-hours endpoint contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::market_hours::{ExchangeHoliday, ExchangeMarketHours},
    types::{Date, ExchangeCode, MarketHoursTimestamp},
};

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);

/// Required exchange and optional opaque timestamp for one exchange's hours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeMarketHoursQuery {
    exchange: ExchangeCode,
    timestamp: Option<MarketHoursTimestamp>,
}

impl ExchangeMarketHoursQuery {
    /// Creates a query without an undocumented timestamp default.
    pub const fn new(exchange: ExchangeCode) -> Self {
        Self {
            exchange,
            timestamp: None,
        }
    }

    /// Sets the optional opaque provider timestamp.
    pub fn with_timestamp(mut self, timestamp: MarketHoursTimestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Borrows the requested exchange code.
    pub const fn exchange(&self) -> &ExchangeCode {
        &self.exchange
    }

    /// Borrows the optional opaque provider timestamp.
    pub const fn timestamp(&self) -> Option<&MarketHoursTimestamp> {
        self.timestamp.as_ref()
    }
}

impl From<ExchangeCode> for ExchangeMarketHoursQuery {
    fn from(exchange: ExchangeCode) -> Self {
        Self::new(exchange)
    }
}

impl From<&ExchangeCode> for ExchangeMarketHoursQuery {
    fn from(exchange: &ExchangeCode) -> Self {
        Self::new(exchange.clone())
    }
}

impl QueryParameters for ExchangeMarketHoursQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("exchange", &self.exchange);
        encoder.optional("timestamp", self.timestamp.as_ref());
    }
}

/// Required exchange and independently optional holiday date filters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolidaysByExchangeQuery {
    exchange: ExchangeCode,
    from: Option<Date>,
    to: Option<Date>,
}

impl HolidaysByExchangeQuery {
    /// Creates a query without undocumented date defaults.
    pub const fn new(exchange: ExchangeCode) -> Self {
        Self {
            exchange,
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

    /// Borrows the requested exchange code.
    pub const fn exchange(&self) -> &ExchangeCode {
        &self.exchange
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

impl From<ExchangeCode> for HolidaysByExchangeQuery {
    fn from(exchange: ExchangeCode) -> Self {
        Self::new(exchange)
    }
}

impl From<&ExchangeCode> for HolidaysByExchangeQuery {
    fn from(exchange: &ExchangeCode) -> Self {
        Self::new(exchange.clone())
    }
}

impl QueryParameters for HolidaysByExchangeQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("exchange", &self.exchange);
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
    }
}

/// Optional opaque timestamp for all exchanges' hours.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AllExchangeMarketHoursQuery {
    timestamp: Option<MarketHoursTimestamp>,
}

impl AllExchangeMarketHoursQuery {
    /// Creates a query without an undocumented timestamp default.
    pub const fn new() -> Self {
        Self { timestamp: None }
    }

    /// Sets the optional opaque provider timestamp.
    pub fn with_timestamp(mut self, timestamp: MarketHoursTimestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Borrows the optional opaque provider timestamp.
    pub const fn timestamp(&self) -> Option<&MarketHoursTimestamp> {
        self.timestamp.as_ref()
    }
}

impl QueryParameters for AllExchangeMarketHoursQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("timestamp", self.timestamp.as_ref());
    }
}

/// Describes `GET exchange-market-hours` without binding it to a transport.
pub fn exchange_market_hours(
    query: ExchangeMarketHoursQuery,
) -> EndpointSpec<ExchangeMarketHoursQuery, Vec<ExchangeMarketHours>> {
    EndpointSpec::get("exchange-market-hours", "exchange-market-hours", query)
        .with_metadata(WORLDWIDE)
}

/// Describes `GET holidays-by-exchange` without binding it to a transport.
pub fn holidays_by_exchange(
    query: HolidaysByExchangeQuery,
) -> EndpointSpec<HolidaysByExchangeQuery, Vec<ExchangeHoliday>> {
    EndpointSpec::get("holidays-by-exchange", "holidays-by-exchange", query)
        .with_metadata(WORLDWIDE)
}

/// Describes `GET all-exchange-market-hours` without binding it to a transport.
pub fn all_exchange_market_hours(
    query: AllExchangeMarketHoursQuery,
) -> EndpointSpec<AllExchangeMarketHoursQuery, Vec<ExchangeMarketHours>> {
    EndpointSpec::get(
        "all-exchange-market-hours",
        "all-exchange-market-hours",
        query,
    )
    .with_metadata(WORLDWIDE)
}

impl Client {
    /// Retrieves trading hours for one exchange.
    pub async fn exchange_market_hours(
        &self,
        query: impl Into<ExchangeMarketHoursQuery>,
    ) -> Result<Vec<ExchangeMarketHours>> {
        self.execute(&exchange_market_hours(query.into())).await
    }

    /// Retrieves holidays for one exchange.
    pub async fn holidays_by_exchange(
        &self,
        query: impl Into<HolidaysByExchangeQuery>,
    ) -> Result<Vec<ExchangeHoliday>> {
        self.execute(&holidays_by_exchange(query.into())).await
    }

    /// Retrieves trading hours for all exchanges.
    pub async fn all_exchange_market_hours(
        &self,
        query: AllExchangeMarketHoursQuery,
    ) -> Result<Vec<ExchangeMarketHours>> {
        self.execute(&all_exchange_market_hours(query)).await
    }
}
