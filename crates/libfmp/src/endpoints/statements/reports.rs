//! Financial-report discovery, dynamic JSON, and XLSX endpoints.

use crate::{
    Client, Result,
    endpoints::{BinaryResponse, EndpointSpec, QueryEncoder, QueryParameters},
    query::{FiscalPeriod, Year},
    responses::statements::{FinancialReportDate, FinancialReportJson},
    types::Ticker,
};

/// Official XLSX MIME followed by the binary fallback accepted for proxy compatibility.
///
/// This is an SDK interoperability policy. The endpoint does not inspect ZIP
/// magic and rejects every other or missing response content type.
pub const FINANCIAL_REPORTS_XLSX_CONTENT_TYPES: &[&str] = &[
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/octet-stream",
];

/// Required symbol for financial-report period discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinancialReportsDatesQuery {
    symbol: Ticker,
}

impl FinancialReportsDatesQuery {
    /// Creates a report-date query for one ticker.
    pub fn new(symbol: Ticker) -> Self {
        Self { symbol }
    }

    /// Borrows the requested ticker.
    pub fn symbol(&self) -> &Ticker {
        &self.symbol
    }
}

impl From<Ticker> for FinancialReportsDatesQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for FinancialReportsDatesQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for FinancialReportsDatesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
    }
}

macro_rules! financial_report_query {
    ($docs:literal, $query:ident) => {
        #[doc = $docs]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            symbol: Ticker,
            year: Year,
            period: FiscalPeriod,
        }

        impl $query {
            /// Creates a report query from its three required typed values.
            pub fn new(symbol: Ticker, year: Year, period: FiscalPeriod) -> Self {
                Self {
                    symbol,
                    year,
                    period,
                }
            }

            /// Borrows the requested ticker.
            pub fn symbol(&self) -> &Ticker {
                &self.symbol
            }

            /// Returns the numeric report query year.
            pub const fn year(&self) -> Year {
                self.year
            }

            /// Returns the requested fiscal period.
            pub const fn period(&self) -> FiscalPeriod {
                self.period
            }
        }

        impl QueryParameters for $query {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("symbol", &self.symbol);
                encoder.required("year", self.year);
                encoder.required("period", self.period);
            }
        }
    };
}

financial_report_query!(
    "Required query for a dynamic JSON financial report.",
    FinancialReportsJsonQuery
);
financial_report_query!(
    "Required query for an XLSX financial report download.",
    FinancialReportsXlsxQuery
);

/// Describes `GET financial-reports-dates` without binding it to a transport.
pub fn financial_reports_dates(
    query: FinancialReportsDatesQuery,
) -> EndpointSpec<FinancialReportsDatesQuery, Vec<FinancialReportDate>> {
    EndpointSpec::get("financial-reports-dates", "financial-reports-dates", query)
}

/// Describes `GET financial-reports-json` without binding it to a transport.
pub fn financial_reports_json(
    query: FinancialReportsJsonQuery,
) -> EndpointSpec<FinancialReportsJsonQuery, FinancialReportJson> {
    EndpointSpec::get("financial-reports-json", "financial-reports-json", query)
}

/// Describes `GET financial-reports-xlsx` with the SDK's explicit XLSX MIME policy.
pub fn financial_reports_xlsx(
    query: FinancialReportsXlsxQuery,
) -> EndpointSpec<FinancialReportsXlsxQuery, BinaryResponse> {
    EndpointSpec::get_binary(
        "financial-reports-xlsx",
        "financial-reports-xlsx",
        query,
        FINANCIAL_REPORTS_XLSX_CONTENT_TYPES,
    )
}

impl Client {
    /// Retrieves available financial-report periods and protected download links.
    pub async fn financial_reports_dates(
        &self,
        query: impl Into<FinancialReportsDatesQuery>,
    ) -> Result<Vec<FinancialReportDate>> {
        self.execute(&financial_reports_dates(query.into())).await
    }

    /// Retrieves one dynamic JSON financial report as a single object.
    pub async fn financial_reports_json(
        &self,
        query: FinancialReportsJsonQuery,
    ) -> Result<FinancialReportJson> {
        self.execute(&financial_reports_json(query)).await
    }

    /// Downloads one financial report using the endpoint's XLSX MIME policy.
    ///
    /// The download inherits the client's finite response-body limit. Configure
    /// a larger client limit when a known workbook requires it.
    pub async fn financial_reports_xlsx(
        &self,
        query: FinancialReportsXlsxQuery,
    ) -> Result<BinaryResponse> {
        self.execute(&financial_reports_xlsx(query)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        query.encode(&mut QueryEncoder::new(&mut |name, value| {
            pairs.push((name.to_owned(), value.to_owned()));
        }));
        pairs
    }

    #[test]
    fn endpoint_owned_queries_encode_exact_required_order_and_five_periods() {
        let symbol = Ticker::new("BRK.B / Class A").unwrap();
        let dates: FinancialReportsDatesQuery = (&symbol).into();
        assert_eq!(dates.symbol(), &symbol);
        assert_eq!(
            pairs(&dates),
            [("symbol".to_owned(), "BRK.B / Class A".to_owned())]
        );

        let periods = [
            FiscalPeriod::Q1,
            FiscalPeriod::Q2,
            FiscalPeriod::Q3,
            FiscalPeriod::Q4,
            FiscalPeriod::FullYear,
        ];
        for period in periods {
            let json = FinancialReportsJsonQuery::new(symbol.clone(), Year(2022), period);
            let xlsx = FinancialReportsXlsxQuery::new(symbol.clone(), Year(2022), period);
            assert_eq!(json.symbol(), &symbol);
            assert_eq!(json.year(), Year(2022));
            assert_eq!(json.period(), period);
            assert_eq!(xlsx.symbol(), &symbol);
            assert_eq!(xlsx.year(), Year(2022));
            assert_eq!(xlsx.period(), period);

            let expected = [
                ("symbol".to_owned(), "BRK.B / Class A".to_owned()),
                ("year".to_owned(), "2022".to_owned()),
                ("period".to_owned(), period.to_string()),
            ];
            assert_eq!(pairs(&json), expected);
            assert_eq!(pairs(&xlsx), expected);
        }
    }

    #[test]
    fn xlsx_descriptor_inherits_the_finite_client_response_limit() {
        let query = FinancialReportsXlsxQuery::new(
            Ticker::new("AAPL").unwrap(),
            Year(2025),
            FiscalPeriod::FullYear,
        );
        assert_eq!(
            financial_reports_xlsx(query).max_response_body_bytes(),
            None
        );
    }
}
