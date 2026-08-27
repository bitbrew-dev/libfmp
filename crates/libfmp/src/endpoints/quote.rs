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
    types::{Ticker, TickerList},
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
}
