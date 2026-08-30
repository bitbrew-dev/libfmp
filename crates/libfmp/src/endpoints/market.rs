//! Market-performance endpoint contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::market::{IndustryPe, IndustryPerformance, SectorPe, SectorPerformance},
    types::{Date, ExchangeCode, Industry, Sector},
};

macro_rules! snapshot_query {
    ($name:ident, $filter_ty:ty, $field:ident, $setter:ident, $wire:literal, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            date: Date,
            exchange: Option<ExchangeCode>,
            $field: Option<$filter_ty>,
        }

        impl $name {
            /// Creates a dated snapshot query without optional filters.
            pub const fn new(date: Date) -> Self {
                Self {
                    date,
                    exchange: None,
                    $field: None,
                }
            }

            /// Sets the optional exchange filter.
            pub fn with_exchange(mut self, exchange: ExchangeCode) -> Self {
                self.exchange = Some(exchange);
                self
            }

            #[doc = concat!("Sets the optional ", $wire, " filter.")]
            pub fn $setter(mut self, value: $filter_ty) -> Self {
                self.$field = Some(value);
                self
            }

            /// Returns the required snapshot date.
            pub const fn date(&self) -> Date {
                self.date
            }

            /// Borrows the optional exchange filter.
            pub const fn exchange(&self) -> Option<&ExchangeCode> {
                self.exchange.as_ref()
            }

            #[doc = concat!("Borrows the optional ", $wire, " filter.")]
            pub const fn $field(&self) -> Option<&$filter_ty> {
                self.$field.as_ref()
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("date", self.date);
                encoder.optional("exchange", self.exchange.as_ref());
                encoder.optional($wire, self.$field.as_ref());
            }
        }
    };
}

snapshot_query!(
    SectorPerformanceSnapshotQuery,
    Sector,
    sector,
    with_sector,
    "sector",
    "Required date and optional exchange and sector filters for a performance snapshot."
);
snapshot_query!(
    IndustryPerformanceSnapshotQuery,
    Industry,
    industry,
    with_industry,
    "industry",
    "Required date and optional exchange and industry filters for a performance snapshot."
);
snapshot_query!(
    SectorPeSnapshotQuery,
    Sector,
    sector,
    with_sector,
    "sector",
    "Required date and optional exchange and sector filters for a P/E snapshot."
);
snapshot_query!(
    IndustryPeSnapshotQuery,
    Industry,
    industry,
    with_industry,
    "industry",
    "Required date and optional exchange and industry filters for a P/E snapshot."
);

macro_rules! historical_sector_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            from: Option<Date>,
            exchange: Option<ExchangeCode>,
            sector: Sector,
            to: Option<Date>,
        }

        impl $name {
            /// Creates a query for one sector without optional filters.
            pub const fn new(sector: Sector) -> Self {
                Self {
                    from: None,
                    exchange: None,
                    sector,
                    to: None,
                }
            }

            /// Sets the optional independent start date.
            pub const fn with_from(mut self, from: Date) -> Self {
                self.from = Some(from);
                self
            }

            /// Sets the optional exchange filter.
            pub fn with_exchange(mut self, exchange: ExchangeCode) -> Self {
                self.exchange = Some(exchange);
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

            /// Borrows the optional exchange filter.
            pub const fn exchange(&self) -> Option<&ExchangeCode> {
                self.exchange.as_ref()
            }

            /// Borrows the required sector.
            pub const fn sector(&self) -> &Sector {
                &self.sector
            }

            /// Returns the optional independent end date.
            pub const fn to(&self) -> Option<Date> {
                self.to
            }
        }

        impl From<Sector> for $name {
            fn from(sector: Sector) -> Self {
                Self::new(sector)
            }
        }

        impl From<&Sector> for $name {
            fn from(sector: &Sector) -> Self {
                Self::new(sector.clone())
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.optional("from", self.from);
                encoder.optional("exchange", self.exchange.as_ref());
                encoder.required("sector", &self.sector);
                encoder.optional("to", self.to);
            }
        }
    };
}

macro_rules! historical_industry_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            industry: Industry,
            exchange: Option<ExchangeCode>,
            from: Option<Date>,
            to: Option<Date>,
        }

        impl $name {
            /// Creates a query for one industry without optional filters.
            pub const fn new(industry: Industry) -> Self {
                Self {
                    industry,
                    exchange: None,
                    from: None,
                    to: None,
                }
            }

            /// Sets the optional exchange filter.
            pub fn with_exchange(mut self, exchange: ExchangeCode) -> Self {
                self.exchange = Some(exchange);
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

            /// Borrows the required industry.
            pub const fn industry(&self) -> &Industry {
                &self.industry
            }

            /// Borrows the optional exchange filter.
            pub const fn exchange(&self) -> Option<&ExchangeCode> {
                self.exchange.as_ref()
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

        impl From<Industry> for $name {
            fn from(industry: Industry) -> Self {
                Self::new(industry)
            }
        }

        impl From<&Industry> for $name {
            fn from(industry: &Industry) -> Self {
                Self::new(industry.clone())
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("industry", &self.industry);
                encoder.optional("exchange", self.exchange.as_ref());
                encoder.optional("from", self.from);
                encoder.optional("to", self.to);
            }
        }
    };
}

historical_sector_query!(
    HistoricalSectorPerformanceQuery,
    "Required sector and optional date and exchange filters for historical performance."
);
historical_industry_query!(
    HistoricalIndustryPerformanceQuery,
    "Required industry and optional exchange and date filters for historical performance."
);
historical_sector_query!(
    HistoricalSectorPeQuery,
    "Required sector and optional date and exchange filters for historical P/E ratios."
);
historical_industry_query!(
    HistoricalIndustryPeQuery,
    "Required industry and optional exchange and date filters for historical P/E ratios."
);

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);

/// Describes `GET sector-performance-snapshot` without binding a transport.
pub fn sector_performance_snapshot(
    query: SectorPerformanceSnapshotQuery,
) -> EndpointSpec<SectorPerformanceSnapshotQuery, Vec<SectorPerformance>> {
    EndpointSpec::get(
        "sector-performance-snapshot",
        "sector-performance-snapshot",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET industry-performance-snapshot` without binding a transport.
pub fn industry_performance_snapshot(
    query: IndustryPerformanceSnapshotQuery,
) -> EndpointSpec<IndustryPerformanceSnapshotQuery, Vec<IndustryPerformance>> {
    EndpointSpec::get(
        "industry-performance-snapshot",
        "industry-performance-snapshot",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET sector-pe-snapshot` without binding a transport.
pub fn sector_pe_snapshot(
    query: SectorPeSnapshotQuery,
) -> EndpointSpec<SectorPeSnapshotQuery, Vec<SectorPe>> {
    EndpointSpec::get("sector-pe-snapshot", "sector-pe-snapshot", query).with_metadata(WORLDWIDE)
}

/// Describes `GET industry-pe-snapshot` without binding a transport.
pub fn industry_pe_snapshot(
    query: IndustryPeSnapshotQuery,
) -> EndpointSpec<IndustryPeSnapshotQuery, Vec<IndustryPe>> {
    EndpointSpec::get("industry-pe-snapshot", "industry-pe-snapshot", query)
        .with_metadata(WORLDWIDE)
}

impl Client {
    /// Retrieves a worldwide dated sector-performance snapshot.
    pub async fn sector_performance_snapshot(
        &self,
        query: SectorPerformanceSnapshotQuery,
    ) -> Result<Vec<SectorPerformance>> {
        self.execute(&sector_performance_snapshot(query)).await
    }

    /// Retrieves a worldwide dated industry-performance snapshot.
    pub async fn industry_performance_snapshot(
        &self,
        query: IndustryPerformanceSnapshotQuery,
    ) -> Result<Vec<IndustryPerformance>> {
        self.execute(&industry_performance_snapshot(query)).await
    }

    /// Retrieves a worldwide dated sector P/E snapshot.
    pub async fn sector_pe_snapshot(&self, query: SectorPeSnapshotQuery) -> Result<Vec<SectorPe>> {
        self.execute(&sector_pe_snapshot(query)).await
    }

    /// Retrieves a worldwide dated industry P/E snapshot.
    pub async fn industry_pe_snapshot(
        &self,
        query: IndustryPeSnapshotQuery,
    ) -> Result<Vec<IndustryPe>> {
        self.execute(&industry_pe_snapshot(query)).await
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn pairs(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut values = Vec::new();
        query.encode(&mut QueryEncoder::new(&mut |name, value| {
            values.push((name.to_owned(), value.to_owned()));
        }));
        values
    }

    #[test]
    fn snapshot_queries_encode_exact_documented_order_and_omit_absent_filters() {
        let date = Date::from_str("2024-02-01").unwrap();
        assert_eq!(
            pairs(
                &SectorPerformanceSnapshotQuery::new(date)
                    .with_exchange(ExchangeCode::new("NEW / EXCHANGE").unwrap())
                    .with_sector(Sector::new("Future & Energy").unwrap())
            ),
            [
                ("date".to_owned(), "2024-02-01".to_owned()),
                ("exchange".to_owned(), "NEW / EXCHANGE".to_owned()),
                ("sector".to_owned(), "Future & Energy".to_owned()),
            ]
        );
        assert_eq!(
            pairs(&IndustryPeSnapshotQuery::new(date)),
            [("date".to_owned(), "2024-02-01".to_owned())]
        );
    }

    #[test]
    fn sector_history_encodes_from_exchange_sector_to_with_independent_dates() {
        let from = Date::from_str("2024-02-01").unwrap();
        let to = Date::from_str("2024-03-01").unwrap();
        let sector = Sector::new("Energy").unwrap();
        assert_eq!(
            pairs(
                &HistoricalSectorPerformanceQuery::new(sector.clone())
                    .with_from(from)
                    .with_exchange(ExchangeCode::new("NASDAQ").unwrap())
                    .with_to(to)
            ),
            [
                ("from".to_owned(), "2024-02-01".to_owned()),
                ("exchange".to_owned(), "NASDAQ".to_owned()),
                ("sector".to_owned(), "Energy".to_owned()),
                ("to".to_owned(), "2024-03-01".to_owned()),
            ]
        );
        assert_eq!(
            pairs(&HistoricalSectorPeQuery::new(sector).with_to(to)),
            [
                ("sector".to_owned(), "Energy".to_owned()),
                ("to".to_owned(), "2024-03-01".to_owned()),
            ]
        );
    }

    #[test]
    fn industry_history_encodes_industry_exchange_from_to_with_independent_dates() {
        let from = Date::from_str("2024-02-01").unwrap();
        let to = Date::from_str("2024-03-01").unwrap();
        let industry = Industry::new("Biotechnology").unwrap();
        assert_eq!(
            pairs(
                &HistoricalIndustryPerformanceQuery::new(industry.clone())
                    .with_exchange(ExchangeCode::new("NASDAQ").unwrap())
                    .with_from(from)
                    .with_to(to)
            ),
            [
                ("industry".to_owned(), "Biotechnology".to_owned()),
                ("exchange".to_owned(), "NASDAQ".to_owned()),
                ("from".to_owned(), "2024-02-01".to_owned()),
                ("to".to_owned(), "2024-03-01".to_owned()),
            ]
        );
        assert_eq!(
            pairs(&HistoricalIndustryPeQuery::new(industry).with_from(from)),
            [
                ("industry".to_owned(), "Biotechnology".to_owned()),
                ("from".to_owned(), "2024-02-01".to_owned()),
            ]
        );
    }
}
