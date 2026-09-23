//! Worldwide company stock screener endpoint.
//!
//! The provider documents the screener as available for companies worldwide,
//! so the descriptor carries [`GeographicAvailability::Worldwide`]. The Python
//! binding exposes the same route under `client.screener`.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::screener::CompanyScreenerEntry,
    types::{
        CountryCode, ExchangeCode, FiniteDecimal, Industry, Limit, MarketCapitalization, Page,
        Sector,
    },
};

/// Optional filters accepted by the company stock screener.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompanyScreenerQuery {
    market_cap_more_than: Option<MarketCapitalization>,
    market_cap_lower_than: Option<MarketCapitalization>,
    sector: Option<Sector>,
    industry: Option<Industry>,
    beta_more_than: Option<FiniteDecimal>,
    beta_lower_than: Option<FiniteDecimal>,
    price_more_than: Option<FiniteDecimal>,
    price_lower_than: Option<FiniteDecimal>,
    dividend_more_than: Option<FiniteDecimal>,
    dividend_lower_than: Option<FiniteDecimal>,
    volume_more_than: Option<u64>,
    volume_lower_than: Option<u64>,
    exchange: Option<ExchangeCode>,
    country: Option<CountryCode>,
    is_etf: Option<bool>,
    is_fund: Option<bool>,
    is_actively_trading: Option<bool>,
    page: Option<Page>,
    limit: Option<Limit>,
    include_all_share_classes: Option<bool>,
}

macro_rules! value_filter {
    ($with:ident, $get:ident, $field:ident, $type:ty, $docs:literal) => {
        #[doc = $docs]
        pub fn $with(mut self, value: $type) -> Self {
            self.$field = Some(value);
            self
        }

        #[doc = $docs]
        pub const fn $get(&self) -> Option<$type> {
            self.$field
        }
    };
}

macro_rules! string_filter {
    ($with:ident, $get:ident, $field:ident, $type:ty, $docs:literal) => {
        #[doc = $docs]
        pub fn $with(mut self, value: $type) -> Self {
            self.$field = Some(value);
            self
        }

        #[doc = $docs]
        pub fn $get(&self) -> Option<&$type> {
            self.$field.as_ref()
        }
    };
}

impl CompanyScreenerQuery {
    /// Creates a screener query with no filters or undocumented defaults.
    pub const fn new() -> Self {
        Self {
            market_cap_more_than: None,
            market_cap_lower_than: None,
            sector: None,
            industry: None,
            beta_more_than: None,
            beta_lower_than: None,
            price_more_than: None,
            price_lower_than: None,
            dividend_more_than: None,
            dividend_lower_than: None,
            volume_more_than: None,
            volume_lower_than: None,
            exchange: None,
            country: None,
            is_etf: None,
            is_fund: None,
            is_actively_trading: None,
            page: None,
            limit: None,
            include_all_share_classes: None,
        }
    }

    value_filter!(
        with_market_cap_more_than,
        market_cap_more_than,
        market_cap_more_than,
        MarketCapitalization,
        "Sets or returns the optional lower market-cap filter."
    );
    value_filter!(
        with_market_cap_lower_than,
        market_cap_lower_than,
        market_cap_lower_than,
        MarketCapitalization,
        "Sets or returns the optional upper market-cap filter."
    );
    string_filter!(
        with_sector,
        sector,
        sector,
        Sector,
        "Sets or borrows the optional open sector filter."
    );
    string_filter!(
        with_industry,
        industry,
        industry,
        Industry,
        "Sets or borrows the optional open industry filter."
    );
    value_filter!(
        with_beta_more_than,
        beta_more_than,
        beta_more_than,
        FiniteDecimal,
        "Sets or returns the optional lower beta filter."
    );
    value_filter!(
        with_beta_lower_than,
        beta_lower_than,
        beta_lower_than,
        FiniteDecimal,
        "Sets or returns the optional upper beta filter."
    );
    value_filter!(
        with_price_more_than,
        price_more_than,
        price_more_than,
        FiniteDecimal,
        "Sets or returns the optional lower price filter."
    );
    value_filter!(
        with_price_lower_than,
        price_lower_than,
        price_lower_than,
        FiniteDecimal,
        "Sets or returns the optional upper price filter."
    );
    value_filter!(
        with_dividend_more_than,
        dividend_more_than,
        dividend_more_than,
        FiniteDecimal,
        "Sets or returns the optional lower dividend filter."
    );
    value_filter!(
        with_dividend_lower_than,
        dividend_lower_than,
        dividend_lower_than,
        FiniteDecimal,
        "Sets or returns the optional upper dividend filter."
    );
    value_filter!(
        with_volume_more_than,
        volume_more_than,
        volume_more_than,
        u64,
        "Sets or returns the optional lower volume filter (a whole number of shares)."
    );
    value_filter!(
        with_volume_lower_than,
        volume_lower_than,
        volume_lower_than,
        u64,
        "Sets or returns the optional upper volume filter (a whole number of shares)."
    );
    string_filter!(
        with_exchange,
        exchange,
        exchange,
        ExchangeCode,
        "Sets or borrows the optional open exchange-code filter."
    );
    string_filter!(
        with_country,
        country,
        country,
        CountryCode,
        "Sets or borrows the optional open country-code filter."
    );
    value_filter!(
        with_is_etf,
        is_etf,
        is_etf,
        bool,
        "Sets or returns the optional ETF classification filter."
    );
    value_filter!(
        with_is_fund,
        is_fund,
        is_fund,
        bool,
        "Sets or returns the optional fund classification filter."
    );
    value_filter!(
        with_is_actively_trading,
        is_actively_trading,
        is_actively_trading,
        bool,
        "Sets or returns the optional actively-trading filter."
    );
    value_filter!(
        with_page,
        page,
        page,
        Page,
        "Sets or returns the optional provider page index."
    );
    value_filter!(
        with_limit,
        limit,
        limit,
        Limit,
        "Sets or returns the optional provider result limit."
    );
    value_filter!(
        with_include_all_share_classes,
        include_all_share_classes,
        include_all_share_classes,
        bool,
        "Sets or returns the optional all-share-classes filter."
    );
}

impl QueryParameters for CompanyScreenerQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("marketCapMoreThan", self.market_cap_more_than);
        encoder.optional("marketCapLowerThan", self.market_cap_lower_than);
        encoder.optional("sector", self.sector.as_ref());
        encoder.optional("industry", self.industry.as_ref());
        encoder.optional("betaMoreThan", self.beta_more_than);
        encoder.optional("betaLowerThan", self.beta_lower_than);
        encoder.optional("priceMoreThan", self.price_more_than);
        encoder.optional("priceLowerThan", self.price_lower_than);
        encoder.optional("dividendMoreThan", self.dividend_more_than);
        encoder.optional("dividendLowerThan", self.dividend_lower_than);
        encoder.optional("volumeMoreThan", self.volume_more_than);
        encoder.optional("volumeLowerThan", self.volume_lower_than);
        encoder.optional("exchange", self.exchange.as_ref());
        encoder.optional("country", self.country.as_ref());
        encoder.optional("isEtf", self.is_etf);
        encoder.optional("isFund", self.is_fund);
        encoder.optional("isActivelyTrading", self.is_actively_trading);
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
        encoder.optional("includeAllShareClasses", self.include_all_share_classes);
    }
}

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);

/// Describes `GET company-screener` without binding it to a transport.
pub fn company_screener(
    query: CompanyScreenerQuery,
) -> EndpointSpec<CompanyScreenerQuery, Vec<CompanyScreenerEntry>> {
    EndpointSpec::get("company-screener", "company-screener", query).with_metadata(WORLDWIDE)
}

impl Client {
    /// Screens worldwide companies using the supplied optional filters.
    pub async fn company_screener(
        &self,
        query: CompanyScreenerQuery,
    ) -> Result<Vec<CompanyScreenerEntry>> {
        self.execute(&company_screener(query)).await
    }
}
