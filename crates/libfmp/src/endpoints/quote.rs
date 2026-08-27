//! Stock, index, commodity, forex, and cryptocurrency quote endpoints.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{
            DelayScope, EndpointMetadata, GeographicAvailability, MarketDataDelay, RealtimeAccess,
            UserDeclarationRequirement,
        },
    },
    responses::quote::{AftermarketQuote, AftermarketTrade, Quote, QuoteShort, StockPriceChange},
    types::{ExchangeCode, Ticker, TickerList},
};

macro_rules! symbol_query {
    ($docs:literal, $query:ident) => {
        #[doc = $docs]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            symbol: Ticker,
        }

        impl $query {
            /// Creates a query for one validated provider ticker.
            pub fn new(symbol: Ticker) -> Self {
                Self { symbol }
            }

            /// Borrows the requested ticker.
            pub fn symbol(&self) -> &Ticker {
                &self.symbol
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
            }
        }
    };
}

symbol_query!(
    "Required query parameters for the detailed stock-quote endpoint.",
    QuoteQuery
);
symbol_query!(
    "Required query parameters for the compact stock-quote endpoint.",
    QuoteShortQuery
);
symbol_query!(
    "Required query parameters for the US aftermarket-trade endpoint.",
    AftermarketTradeQuery
);
symbol_query!(
    "Required query parameters for the US aftermarket-quote endpoint.",
    AftermarketQuoteQuery
);
symbol_query!(
    "Required query parameters for the worldwide stock-price-change endpoint.",
    StockPriceChangeQuery
);

macro_rules! symbols_query {
    ($docs:literal, $query:ident) => {
        #[doc = $docs]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            symbols: TickerList,
        }

        impl $query {
            /// Creates a query for a non-empty sequence of validated provider tickers.
            pub fn new(symbols: TickerList) -> Self {
                Self { symbols }
            }

            /// Borrows the requested tickers in provider request order.
            pub fn symbols(&self) -> &TickerList {
                &self.symbols
            }
        }

        impl From<TickerList> for $query {
            fn from(symbols: TickerList) -> Self {
                Self::new(symbols)
            }
        }

        impl From<&TickerList> for $query {
            fn from(symbols: &TickerList) -> Self {
                Self::new(symbols.clone())
            }
        }

        impl QueryParameters for $query {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("symbols", &self.symbols);
            }
        }
    };
}

symbols_query!(
    "Required query parameters for the worldwide detailed stock batch-quote endpoint.",
    BatchQuoteQuery
);
symbols_query!(
    "Required query parameters for the worldwide compact stock batch-quote endpoint.",
    BatchQuoteShortQuery
);
symbols_query!(
    "Required query parameters for the US batch aftermarket-trade endpoint.",
    BatchAftermarketTradeQuery
);
symbols_query!(
    "Required query parameters for the US batch aftermarket-quote endpoint.",
    BatchAftermarketQuoteQuery
);

/// Required query parameters for compact quotes across one exchange.
///
/// This contract intentionally has no configurable `short` field: the
/// documented compact response is always requested with `short=true`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeQuotesQuery {
    exchange: ExchangeCode,
}

impl ExchangeQuotesQuery {
    /// Creates a compact exchange-quote query for one provider exchange code.
    pub fn new(exchange: ExchangeCode) -> Self {
        Self { exchange }
    }

    /// Borrows the requested exchange code.
    pub fn exchange(&self) -> &ExchangeCode {
        &self.exchange
    }
}

impl From<ExchangeCode> for ExchangeQuotesQuery {
    fn from(exchange: ExchangeCode) -> Self {
        Self::new(exchange)
    }
}

impl From<&ExchangeCode> for ExchangeQuotesQuery {
    fn from(exchange: &ExchangeCode) -> Self {
        Self::new(exchange.clone())
    }
}

impl QueryParameters for ExchangeQuotesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("exchange", &self.exchange);
        encoder.required("short", true);
    }
}

/// The closed compact-query contract used by whole-asset quote endpoints.
///
/// It always emits `short=true`; the undocumented `short=false` response
/// shapes are deliberately deferred rather than represented speculatively.
///
/// ```compile_fail
/// let _ = libfmp::endpoints::quote::ShortOnlyQuery::new(false);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShortOnlyQuery;

impl ShortOnlyQuery {
    /// Creates the fixed `short=true` query.
    pub const fn new() -> Self {
        Self
    }
}

impl QueryParameters for ShortOnlyQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("short", true);
    }
}

const NASDAQ_DELAYED_REALTIME: RealtimeAccess = RealtimeAccess::new(
    Some(MarketDataDelay::new(15, DelayScope::Nasdaq)),
    Some(UserDeclarationRequirement::RequiredForRealtime),
);
const WORLDWIDE_REALTIME: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::Worldwide)
    .with_realtime(NASDAQ_DELAYED_REALTIME);
const US_REALTIME: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::UsOnly)
    .with_realtime(NASDAQ_DELAYED_REALTIME);
const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);
const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);
const UNSPECIFIED: EndpointMetadata = EndpointMetadata::new();

/// Describes `GET quote` without binding it to a transport.
pub fn quote(query: QuoteQuery) -> EndpointSpec<QuoteQuery, Vec<Quote>> {
    EndpointSpec::get("quote", "quote", query).with_metadata(WORLDWIDE_REALTIME)
}

/// Describes `GET quote-short` without binding it to a transport.
pub fn quote_short(query: QuoteShortQuery) -> EndpointSpec<QuoteShortQuery, Vec<QuoteShort>> {
    EndpointSpec::get("quote-short", "quote-short", query).with_metadata(WORLDWIDE_REALTIME)
}

/// Describes `GET aftermarket-trade` without binding it to a transport.
pub fn aftermarket_trade(
    query: AftermarketTradeQuery,
) -> EndpointSpec<AftermarketTradeQuery, Vec<AftermarketTrade>> {
    EndpointSpec::get("aftermarket-trade", "aftermarket-trade", query).with_metadata(US_REALTIME)
}

/// Describes `GET aftermarket-quote` without binding it to a transport.
pub fn aftermarket_quote(
    query: AftermarketQuoteQuery,
) -> EndpointSpec<AftermarketQuoteQuery, Vec<AftermarketQuote>> {
    EndpointSpec::get("aftermarket-quote", "aftermarket-quote", query).with_metadata(US_REALTIME)
}

/// Describes `GET stock-price-change` without binding it to a transport.
pub fn stock_price_change(
    query: StockPriceChangeQuery,
) -> EndpointSpec<StockPriceChangeQuery, Vec<StockPriceChange>> {
    EndpointSpec::get("stock-price-change", "stock-price-change", query).with_metadata(WORLDWIDE)
}

/// Describes `GET batch-quote` without binding it to a transport.
pub fn batch_quote(query: BatchQuoteQuery) -> EndpointSpec<BatchQuoteQuery, Vec<Quote>> {
    EndpointSpec::get("batch-quote", "batch-quote", query).with_metadata(WORLDWIDE_REALTIME)
}

/// Describes `GET batch-quote-short` without binding it to a transport.
pub fn batch_quote_short(
    query: BatchQuoteShortQuery,
) -> EndpointSpec<BatchQuoteShortQuery, Vec<QuoteShort>> {
    EndpointSpec::get("batch-quote-short", "batch-quote-short", query)
        .with_metadata(WORLDWIDE_REALTIME)
}

/// Describes `GET batch-aftermarket-trade` without binding it to a transport.
pub fn batch_aftermarket_trade(
    query: BatchAftermarketTradeQuery,
) -> EndpointSpec<BatchAftermarketTradeQuery, Vec<AftermarketTrade>> {
    EndpointSpec::get("batch-aftermarket-trade", "batch-aftermarket-trade", query)
        .with_metadata(US_REALTIME)
}

/// Describes `GET batch-aftermarket-quote` without binding it to a transport.
pub fn batch_aftermarket_quote(
    query: BatchAftermarketQuoteQuery,
) -> EndpointSpec<BatchAftermarketQuoteQuery, Vec<AftermarketQuote>> {
    EndpointSpec::get("batch-aftermarket-quote", "batch-aftermarket-quote", query)
        .with_metadata(US_REALTIME)
}

/// Describes compact `GET batch-exchange-quote` without binding it to a transport.
pub fn exchange_quotes(
    query: ExchangeQuotesQuery,
) -> EndpointSpec<ExchangeQuotesQuery, Vec<QuoteShort>> {
    EndpointSpec::get("batch-exchange-quote", "batch-exchange-quote", query)
        .with_metadata(WORLDWIDE_REALTIME)
}

/// Describes compact `GET batch-mutualfund-quotes` without binding it to a transport.
pub fn mutual_fund_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    EndpointSpec::get(
        "batch-mutualfund-quotes",
        "batch-mutualfund-quotes",
        ShortOnlyQuery::new(),
    )
    .with_metadata(US_ONLY)
}

/// Describes compact `GET batch-etf-quotes` without binding it to a transport.
pub fn etf_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    EndpointSpec::get(
        "batch-etf-quotes",
        "batch-etf-quotes",
        ShortOnlyQuery::new(),
    )
    .with_metadata(WORLDWIDE)
}

/// Describes compact `GET batch-commodity-quotes` without binding it to a transport.
pub fn commodity_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    EndpointSpec::get(
        "batch-commodity-quotes",
        "batch-commodity-quotes",
        ShortOnlyQuery::new(),
    )
    .with_metadata(UNSPECIFIED)
}

/// Describes compact `GET batch-crypto-quotes` without binding it to a transport.
pub fn cryptocurrency_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    EndpointSpec::get(
        "batch-crypto-quotes",
        "batch-crypto-quotes",
        ShortOnlyQuery::new(),
    )
    .with_metadata(UNSPECIFIED)
}

/// Describes compact `GET batch-forex-quotes` without binding it to a transport.
pub fn forex_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    EndpointSpec::get(
        "batch-forex-quotes",
        "batch-forex-quotes",
        ShortOnlyQuery::new(),
    )
    .with_metadata(UNSPECIFIED)
}

/// Describes compact `GET batch-index-quotes` without binding it to a transport.
pub fn index_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    EndpointSpec::get(
        "batch-index-quotes",
        "batch-index-quotes",
        ShortOnlyQuery::new(),
    )
    .with_metadata(UNSPECIFIED)
}

impl Client {
    /// Retrieves detailed worldwide stock quotes for one ticker.
    pub async fn quote(&self, query: impl Into<QuoteQuery>) -> Result<Vec<Quote>> {
        self.execute(&quote(query.into())).await
    }

    /// Retrieves compact worldwide stock quotes for one ticker.
    pub async fn quote_short(&self, query: impl Into<QuoteShortQuery>) -> Result<Vec<QuoteShort>> {
        self.execute(&quote_short(query.into())).await
    }

    /// Retrieves US aftermarket trades for one ticker.
    pub async fn aftermarket_trade(
        &self,
        query: impl Into<AftermarketTradeQuery>,
    ) -> Result<Vec<AftermarketTrade>> {
        self.execute(&aftermarket_trade(query.into())).await
    }

    /// Retrieves US aftermarket bid-and-ask quotes for one ticker.
    pub async fn aftermarket_quote(
        &self,
        query: impl Into<AftermarketQuoteQuery>,
    ) -> Result<Vec<AftermarketQuote>> {
        self.execute(&aftermarket_quote(query.into())).await
    }

    /// Retrieves worldwide percentage price changes for one stock.
    pub async fn stock_price_change(
        &self,
        query: impl Into<StockPriceChangeQuery>,
    ) -> Result<Vec<StockPriceChange>> {
        self.execute(&stock_price_change(query.into())).await
    }

    /// Retrieves detailed worldwide stock quotes for multiple tickers.
    pub async fn batch_quote(&self, query: impl Into<BatchQuoteQuery>) -> Result<Vec<Quote>> {
        self.execute(&batch_quote(query.into())).await
    }

    /// Retrieves compact worldwide stock quotes for multiple tickers.
    pub async fn batch_quote_short(
        &self,
        query: impl Into<BatchQuoteShortQuery>,
    ) -> Result<Vec<QuoteShort>> {
        self.execute(&batch_quote_short(query.into())).await
    }

    /// Retrieves US aftermarket trades for multiple tickers.
    pub async fn batch_aftermarket_trade(
        &self,
        query: impl Into<BatchAftermarketTradeQuery>,
    ) -> Result<Vec<AftermarketTrade>> {
        self.execute(&batch_aftermarket_trade(query.into())).await
    }

    /// Retrieves US aftermarket bid-and-ask quotes for multiple tickers.
    pub async fn batch_aftermarket_quote(
        &self,
        query: impl Into<BatchAftermarketQuoteQuery>,
    ) -> Result<Vec<AftermarketQuote>> {
        self.execute(&batch_aftermarket_quote(query.into())).await
    }

    /// Retrieves compact quotes for every stock on one exchange.
    pub async fn exchange_quotes(
        &self,
        query: impl Into<ExchangeQuotesQuery>,
    ) -> Result<Vec<QuoteShort>> {
        self.execute(&exchange_quotes(query.into())).await
    }

    /// Retrieves compact quotes for the documented mutual-fund universe.
    pub async fn mutual_fund_quotes(&self) -> Result<Vec<QuoteShort>> {
        self.execute(&mutual_fund_quotes()).await
    }

    /// Retrieves compact quotes for the documented ETF universe.
    pub async fn etf_quotes(&self) -> Result<Vec<QuoteShort>> {
        self.execute(&etf_quotes()).await
    }

    /// Retrieves compact quotes for the documented commodity universe.
    pub async fn commodity_quotes(&self) -> Result<Vec<QuoteShort>> {
        self.execute(&commodity_quotes()).await
    }

    /// Retrieves compact quotes for the documented cryptocurrency universe.
    pub async fn cryptocurrency_quotes(&self) -> Result<Vec<QuoteShort>> {
        self.execute(&cryptocurrency_quotes()).await
    }

    /// Retrieves compact quotes for the documented forex universe.
    pub async fn forex_quotes(&self) -> Result<Vec<QuoteShort>> {
        self.execute(&forex_quotes()).await
    }

    /// Retrieves compact quotes for the documented index universe.
    pub async fn index_quotes(&self) -> Result<Vec<QuoteShort>> {
        self.execute(&index_quotes()).await
    }
}
