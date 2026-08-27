# ADR 0005: Quote API names and compact universe contracts

## Status

Accepted for issue #14's Rust implementation and reserved future Python facade.

## Decision

The 16 quote endpoints use the following Rust method, query, and response-row
names. The same method name applies to the free descriptor and `Client` method.

| FMP path | Rust method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `quote` | `quote` | `QuoteQuery` | `Quote` | `FmpClient.quote`; `fmp.quote.Quote` |
| `quote-short` | `quote_short` | `QuoteShortQuery` | `QuoteShort` | `FmpClient.quote_short`; `fmp.quote.QuoteShort` |
| `aftermarket-trade` | `aftermarket_trade` | `AftermarketTradeQuery` | `AftermarketTrade` | `FmpClient.aftermarket_trade`; `fmp.quote.AftermarketTrade` |
| `aftermarket-quote` | `aftermarket_quote` | `AftermarketQuoteQuery` | `AftermarketQuote` | `FmpClient.aftermarket_quote`; `fmp.quote.AftermarketQuote` |
| `stock-price-change` | `stock_price_change` | `StockPriceChangeQuery` | `StockPriceChange` | `FmpClient.stock_price_change`; `fmp.quote.StockPriceChange` |
| `batch-quote` | `batch_quote` | `BatchQuoteQuery` | `Quote` | `FmpClient.batch_quote`; `fmp.quote.Quote` |
| `batch-quote-short` | `batch_quote_short` | `BatchQuoteShortQuery` | `QuoteShort` | `FmpClient.batch_quote_short`; `fmp.quote.QuoteShort` |
| `batch-aftermarket-trade` | `batch_aftermarket_trade` | `BatchAftermarketTradeQuery` | `AftermarketTrade` | `FmpClient.batch_aftermarket_trade`; `fmp.quote.AftermarketTrade` |
| `batch-aftermarket-quote` | `batch_aftermarket_quote` | `BatchAftermarketQuoteQuery` | `AftermarketQuote` | `FmpClient.batch_aftermarket_quote`; `fmp.quote.AftermarketQuote` |
| `batch-exchange-quote` | `exchange_quotes` | `ExchangeQuotesQuery` | `QuoteShort` | `FmpClient.exchange_quotes`; `fmp.quote.QuoteShort` |
| `batch-mutualfund-quotes` | `mutual_fund_quotes` | `ShortOnlyQuery` | `QuoteShort` | `FmpClient.mutual_fund_quotes`; `fmp.quote.QuoteShort` |
| `batch-etf-quotes` | `etf_quotes` | `ShortOnlyQuery` | `QuoteShort` | `FmpClient.etf_quotes`; `fmp.quote.QuoteShort` |
| `batch-commodity-quotes` | `commodity_quotes` | `ShortOnlyQuery` | `QuoteShort` | `FmpClient.commodity_quotes`; `fmp.quote.QuoteShort` |
| `batch-crypto-quotes` | `cryptocurrency_quotes` | `ShortOnlyQuery` | `QuoteShort` | `FmpClient.cryptocurrency_quotes`; `fmp.quote.QuoteShort` |
| `batch-forex-quotes` | `forex_quotes` | `ShortOnlyQuery` | `QuoteShort` | `FmpClient.forex_quotes`; `fmp.quote.QuoteShort` |
| `batch-index-quotes` | `index_quotes` | `ShortOnlyQuery` | `QuoteShort` | `FmpClient.index_quotes`; `fmp.quote.QuoteShort` |

The exchange and whole-asset endpoints always emit `short=true`. Their typed
contracts expose neither a boolean argument nor a setter. The documentation
shows only the compact response while leaving the `short=false` shapes
unspecified, so those shapes remain deferred until a reliable contract exists.

Python runtime parity remains deferred while the Rust API is prioritized. A
future facade may accept ordinary method arguments, but it will not expose the
Rust query objects as Python public classes.
