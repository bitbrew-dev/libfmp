//! Worldwide discounted-cash-flow valuation endpoint contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::dcf::{CustomDcfValuation, CustomLeveredDcfValuation, DcfValuation},
    types::{FiniteDecimal, Ticker},
};

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);

/// Required company symbol shared by the standard and levered DCF routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcfQuery {
    symbol: Ticker,
}

impl DcfQuery {
    /// Creates a DCF query for one company.
    pub const fn new(symbol: Ticker) -> Self {
        Self { symbol }
    }

    /// Borrows the requested company symbol.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }
}

impl From<Ticker> for DcfQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for DcfQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for DcfQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
    }
}

/// Independently optional assumptions accepted by both custom DCF routes.
///
/// Values retain the exact magnitude supplied by the caller. In particular,
/// percentage-like inputs are not scaled between fractional and whole-percent
/// representations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DcfAssumptions {
    revenue_growth_pct: Option<FiniteDecimal>,
    ebitda_pct: Option<FiniteDecimal>,
    depreciation_and_amortization_pct: Option<FiniteDecimal>,
    cash_and_short_term_investments_pct: Option<FiniteDecimal>,
    receivables_pct: Option<FiniteDecimal>,
    inventories_pct: Option<FiniteDecimal>,
    payable_pct: Option<FiniteDecimal>,
    ebit_pct: Option<FiniteDecimal>,
    capital_expenditure_pct: Option<FiniteDecimal>,
    operating_cash_flow_pct: Option<FiniteDecimal>,
    selling_general_and_administrative_expenses_pct: Option<FiniteDecimal>,
    tax_rate: Option<FiniteDecimal>,
    long_term_growth_rate: Option<FiniteDecimal>,
    cost_of_debt: Option<FiniteDecimal>,
    cost_of_equity: Option<FiniteDecimal>,
    market_risk_premium: Option<FiniteDecimal>,
    beta: Option<FiniteDecimal>,
    risk_free_rate: Option<FiniteDecimal>,
}

macro_rules! assumption {
    ($with:ident, $get:ident, $field:ident, $docs:literal) => {
        #[doc = $docs]
        pub const fn $with(mut self, value: FiniteDecimal) -> Self {
            self.$field = Some(value);
            self
        }

        #[doc = $docs]
        pub const fn $get(&self) -> Option<FiniteDecimal> {
            self.$field
        }
    };
}

impl DcfAssumptions {
    /// Creates an assumption set with every input omitted.
    pub const fn new() -> Self {
        Self {
            revenue_growth_pct: None,
            ebitda_pct: None,
            depreciation_and_amortization_pct: None,
            cash_and_short_term_investments_pct: None,
            receivables_pct: None,
            inventories_pct: None,
            payable_pct: None,
            ebit_pct: None,
            capital_expenditure_pct: None,
            operating_cash_flow_pct: None,
            selling_general_and_administrative_expenses_pct: None,
            tax_rate: None,
            long_term_growth_rate: None,
            cost_of_debt: None,
            cost_of_equity: None,
            market_risk_premium: None,
            beta: None,
            risk_free_rate: None,
        }
    }

    assumption!(
        with_revenue_growth_pct,
        revenue_growth_pct,
        revenue_growth_pct,
        "Sets or returns the optional revenue-growth input."
    );
    assumption!(
        with_ebitda_pct,
        ebitda_pct,
        ebitda_pct,
        "Sets or returns the optional EBITDA input."
    );
    assumption!(
        with_depreciation_and_amortization_pct,
        depreciation_and_amortization_pct,
        depreciation_and_amortization_pct,
        "Sets or returns the optional depreciation-and-amortization input."
    );
    assumption!(
        with_cash_and_short_term_investments_pct,
        cash_and_short_term_investments_pct,
        cash_and_short_term_investments_pct,
        "Sets or returns the optional cash-and-short-term-investments input."
    );
    assumption!(
        with_receivables_pct,
        receivables_pct,
        receivables_pct,
        "Sets or returns the optional receivables input."
    );
    assumption!(
        with_inventories_pct,
        inventories_pct,
        inventories_pct,
        "Sets or returns the optional inventories input."
    );
    assumption!(
        with_payable_pct,
        payable_pct,
        payable_pct,
        "Sets or returns the optional payable input."
    );
    assumption!(
        with_ebit_pct,
        ebit_pct,
        ebit_pct,
        "Sets or returns the optional EBIT input."
    );
    assumption!(
        with_capital_expenditure_pct,
        capital_expenditure_pct,
        capital_expenditure_pct,
        "Sets or returns the optional capital-expenditure input."
    );
    assumption!(
        with_operating_cash_flow_pct,
        operating_cash_flow_pct,
        operating_cash_flow_pct,
        "Sets or returns the optional operating-cash-flow input."
    );
    assumption!(
        with_selling_general_and_administrative_expenses_pct,
        selling_general_and_administrative_expenses_pct,
        selling_general_and_administrative_expenses_pct,
        "Sets or returns the optional selling, general, and administrative-expenses input."
    );
    assumption!(
        with_tax_rate,
        tax_rate,
        tax_rate,
        "Sets or returns the optional tax-rate input."
    );
    assumption!(
        with_long_term_growth_rate,
        long_term_growth_rate,
        long_term_growth_rate,
        "Sets or returns the optional long-term-growth-rate input."
    );
    assumption!(
        with_cost_of_debt,
        cost_of_debt,
        cost_of_debt,
        "Sets or returns the optional cost-of-debt input."
    );
    assumption!(
        with_cost_of_equity,
        cost_of_equity,
        cost_of_equity,
        "Sets or returns the optional cost-of-equity input."
    );
    assumption!(
        with_market_risk_premium,
        market_risk_premium,
        market_risk_premium,
        "Sets or returns the optional market-risk-premium input."
    );
    assumption!(
        with_beta,
        beta,
        beta,
        "Sets or returns the optional beta input."
    );
    assumption!(
        with_risk_free_rate,
        risk_free_rate,
        risk_free_rate,
        "Sets or returns the optional risk-free-rate input."
    );

    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("revenueGrowthPct", self.revenue_growth_pct);
        encoder.optional("ebitdaPct", self.ebitda_pct);
        encoder.optional(
            "depreciationAndAmortizationPct",
            self.depreciation_and_amortization_pct,
        );
        encoder.optional(
            "cashAndShortTermInvestmentsPct",
            self.cash_and_short_term_investments_pct,
        );
        encoder.optional("receivablesPct", self.receivables_pct);
        encoder.optional("inventoriesPct", self.inventories_pct);
        encoder.optional("payablePct", self.payable_pct);
        encoder.optional("ebitPct", self.ebit_pct);
        encoder.optional("capitalExpenditurePct", self.capital_expenditure_pct);
        encoder.optional("operatingCashFlowPct", self.operating_cash_flow_pct);
        encoder.optional(
            "sellingGeneralAndAdministrativeExpensesPct",
            self.selling_general_and_administrative_expenses_pct,
        );
        encoder.optional("taxRate", self.tax_rate);
        encoder.optional("longTermGrowthRate", self.long_term_growth_rate);
        encoder.optional("costOfDebt", self.cost_of_debt);
        encoder.optional("costOfEquity", self.cost_of_equity);
        encoder.optional("marketRiskPremium", self.market_risk_premium);
        encoder.optional("beta", self.beta);
        encoder.optional("riskFreeRate", self.risk_free_rate);
    }
}

/// Required company symbol and custom assumptions shared by both custom DCF routes.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomDcfQuery {
    symbol: Ticker,
    assumptions: DcfAssumptions,
}

impl CustomDcfQuery {
    /// Creates a custom DCF query without introducing assumption defaults.
    pub const fn new(symbol: Ticker, assumptions: DcfAssumptions) -> Self {
        Self {
            symbol,
            assumptions,
        }
    }

    /// Borrows the requested company symbol.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Borrows the independently optional custom assumptions.
    pub const fn assumptions(&self) -> &DcfAssumptions {
        &self.assumptions
    }
}

impl QueryParameters for CustomDcfQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        self.assumptions.encode(encoder);
    }
}

/// Describes `GET discounted-cash-flow` without binding it to a transport.
pub fn discounted_cash_flow(query: DcfQuery) -> EndpointSpec<DcfQuery, Vec<DcfValuation>> {
    EndpointSpec::get("discounted-cash-flow", "discounted-cash-flow", query)
        .with_metadata(WORLDWIDE)
}

/// Describes `GET levered-discounted-cash-flow` without binding it to a transport.
pub fn levered_discounted_cash_flow(query: DcfQuery) -> EndpointSpec<DcfQuery, Vec<DcfValuation>> {
    EndpointSpec::get(
        "levered-discounted-cash-flow",
        "levered-discounted-cash-flow",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET custom-discounted-cash-flow` without binding it to a transport.
pub fn custom_discounted_cash_flow(
    query: CustomDcfQuery,
) -> EndpointSpec<CustomDcfQuery, Vec<CustomDcfValuation>> {
    EndpointSpec::get(
        "custom-discounted-cash-flow",
        "custom-discounted-cash-flow",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET custom-levered-discounted-cash-flow` without binding it to a transport.
pub fn custom_levered_discounted_cash_flow(
    query: CustomDcfQuery,
) -> EndpointSpec<CustomDcfQuery, Vec<CustomLeveredDcfValuation>> {
    EndpointSpec::get(
        "custom-levered-discounted-cash-flow",
        "custom-levered-discounted-cash-flow",
        query,
    )
    .with_metadata(WORLDWIDE)
}

impl Client {
    /// Retrieves the standard discounted-cash-flow valuation for one company.
    pub async fn discounted_cash_flow(
        &self,
        query: impl Into<DcfQuery>,
    ) -> Result<Vec<DcfValuation>> {
        self.execute(&discounted_cash_flow(query.into())).await
    }

    /// Retrieves the levered discounted-cash-flow valuation for one company.
    pub async fn levered_discounted_cash_flow(
        &self,
        query: impl Into<DcfQuery>,
    ) -> Result<Vec<DcfValuation>> {
        self.execute(&levered_discounted_cash_flow(query.into()))
            .await
    }

    /// Retrieves a custom unlevered discounted-cash-flow valuation.
    pub async fn custom_discounted_cash_flow(
        &self,
        query: CustomDcfQuery,
    ) -> Result<Vec<CustomDcfValuation>> {
        self.execute(&custom_discounted_cash_flow(query)).await
    }

    /// Retrieves a custom levered discounted-cash-flow valuation.
    pub async fn custom_levered_discounted_cash_flow(
        &self,
        query: CustomDcfQuery,
    ) -> Result<Vec<CustomLeveredDcfValuation>> {
        self.execute(&custom_levered_discounted_cash_flow(query))
            .await
    }
}
