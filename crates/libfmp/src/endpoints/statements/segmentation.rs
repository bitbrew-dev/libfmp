//! Product and geographic revenue-segmentation endpoints.
//!
//! Segmentation queries accept retrieval frequencies, not fiscal-period
//! selectors.
//!
//! ```compile_fail
//! use libfmp::{
//!     endpoints::statements::RevenueProductSegmentationQuery,
//!     query::FiscalPeriod,
//!     types::Ticker,
//! };
//! let query = RevenueProductSegmentationQuery::new(Ticker::new("AAPL").unwrap());
//! let _ = query.with_period(FiscalPeriod::Q1);
//! ```

use std::fmt;

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata},
    },
    query::RetrievalFrequency,
    responses::statements::RevenueSegmentation,
    types::Ticker,
};

/// A documented revenue-segmentation response structure.
///
/// The provider currently documents only `flat`; arbitrary strings are not
/// accepted by this closed contract.
///
/// ```compile_fail
/// let _ = libfmp::endpoints::statements::SegmentationStructure::new("nested");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SegmentationStructure {
    Flat,
}

impl SegmentationStructure {
    /// Returns the exact provider query representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Flat => "flat",
        }
    }
}

impl fmt::Display for SegmentationStructure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

macro_rules! segmentation_query {
    ($docs:literal, $query:ident) => {
        #[doc = $docs]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            symbol: Ticker,
            period: Option<RetrievalFrequency>,
            structure: Option<SegmentationStructure>,
        }

        impl $query {
            /// Creates a query for one ticker without undocumented defaults.
            pub fn new(symbol: Ticker) -> Self {
                Self {
                    symbol,
                    period: None,
                    structure: None,
                }
            }

            /// Sets the optional annual or quarterly retrieval frequency.
            pub const fn with_period(mut self, period: RetrievalFrequency) -> Self {
                self.period = Some(period);
                self
            }

            /// Sets the optional documented response structure.
            pub const fn with_structure(mut self, structure: SegmentationStructure) -> Self {
                self.structure = Some(structure);
                self
            }

            /// Borrows the requested ticker.
            pub fn symbol(&self) -> &Ticker {
                &self.symbol
            }

            /// Returns the optional annual or quarterly retrieval frequency.
            pub const fn period(&self) -> Option<RetrievalFrequency> {
                self.period
            }

            /// Returns the optional documented response structure.
            pub const fn structure(&self) -> Option<SegmentationStructure> {
                self.structure
            }
        }

        impl From<Ticker> for $query {
            fn from(symbol: Ticker) -> Self {
                Self::new(symbol)
            }
        }

        impl From<&Ticker> for $query {
            fn from(symbol: &Ticker) -> Self {
                Self::new(symbol.clone())
            }
        }

        impl QueryParameters for $query {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("symbol", &self.symbol);
                encoder.optional("period", self.period);
                encoder.optional("structure", self.structure);
            }
        }
    };
}

segmentation_query!(
    "Query parameters for product revenue segmentation.",
    RevenueProductSegmentationQuery
);
segmentation_query!(
    "Query parameters for geographic revenue segmentation.",
    RevenueGeographicSegmentationQuery
);

const SEGMENTATION_METADATA: EndpointMetadata =
    EndpointMetadata::new().with_bounds(EndpointBounds::new().with_response_rows(1_000));

/// Describes `GET revenue-product-segmentation` without binding it to a transport.
pub fn revenue_product_segmentation(
    query: RevenueProductSegmentationQuery,
) -> EndpointSpec<RevenueProductSegmentationQuery, Vec<RevenueSegmentation>> {
    EndpointSpec::get(
        "revenue-product-segmentation",
        "revenue-product-segmentation",
        query,
    )
    .with_metadata(SEGMENTATION_METADATA)
}

/// Describes `GET revenue-geographic-segmentation` without binding it to a transport.
pub fn revenue_geographic_segmentation(
    query: RevenueGeographicSegmentationQuery,
) -> EndpointSpec<RevenueGeographicSegmentationQuery, Vec<RevenueSegmentation>> {
    EndpointSpec::get(
        "revenue-geographic-segmentation",
        "revenue-geographic-segmentation",
        query,
    )
    .with_metadata(SEGMENTATION_METADATA)
}

impl Client {
    /// Retrieves product revenue segmentation for one company.
    pub async fn revenue_product_segmentation(
        &self,
        query: impl Into<RevenueProductSegmentationQuery>,
    ) -> Result<Vec<RevenueSegmentation>> {
        self.execute(&revenue_product_segmentation(query.into()))
            .await
    }

    /// Retrieves geographic revenue segmentation for one company.
    pub async fn revenue_geographic_segmentation(
        &self,
        query: impl Into<RevenueGeographicSegmentationQuery>,
    ) -> Result<Vec<RevenueSegmentation>> {
        self.execute(&revenue_geographic_segmentation(query.into()))
            .await
    }
}
