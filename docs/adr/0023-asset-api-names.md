# ADR 0023: Commodity, forex, and cryptocurrency API names

## Status

Accepted for issue #32's Rust API inventory. All 27 catalog, quote, and chart
entries are implemented in Rust; the Python facade remains deferred.

## Decision

The 27 documented Commodity, Forex, and Crypto market-data entries use or
reserve the following names. Shared quote and chart contracts are reused rather
than duplicated.

| Domain | FMP path | Rust descriptor/client method | Rust query | Rust response row | State | Future Python API |
| --- | --- | --- | --- | --- | --- | --- |
| Commodity | `commodities-list` | `commodities_list` | unit (`()`) | `CommodityListing` | Implemented | `FmpClient.commodities_list`; `fmp.commodities.CommodityListing` |
| Commodity | `quote` | `commodity_quote` | `QuoteQuery` | `Quote` | Implemented facade | `FmpClient.commodity_quote`; `fmp.quote.Quote` |
| Commodity | `quote-short` | `commodity_quote_short` | `QuoteShortQuery` | `QuoteShort` | Implemented facade | `FmpClient.commodity_quote_short`; `fmp.quote.QuoteShort` |
| Commodity | `batch-commodity-quotes` | `commodity_quotes` | `ShortOnlyQuery` | `QuoteShort` | Existing client method; domain descriptor facade added | `FmpClient.commodity_quotes`; `fmp.quote.QuoteShort` |
| Commodity | `historical-price-eod/light` | `commodity_chart_light` | `AssetChartQuery` | `StockChartLightBar` | Implemented facade | `FmpClient.commodity_chart_light`; `fmp.chart.StockChartLightBar` |
| Commodity | `historical-price-eod/full` | `commodity_chart_full` | `AssetChartQuery` | `StockChartFullBar` | Implemented facade | `FmpClient.commodity_chart_full`; `fmp.chart.StockChartFullBar` |
| Commodity | `historical-chart/1min` | `commodity_chart_one_minute` | `AssetChartQuery` | `StockChartIntradayBar` | Implemented facade | `FmpClient.commodity_chart_one_minute`; `fmp.chart.StockChartIntradayBar` |
| Commodity | `historical-chart/5min` | `commodity_chart_five_minutes` | `AssetChartQuery` | `StockChartIntradayBar` | Implemented facade | `FmpClient.commodity_chart_five_minutes`; `fmp.chart.StockChartIntradayBar` |
| Commodity | `historical-chart/1hour` | `commodity_chart_one_hour` | `AssetChartQuery` | `StockChartIntradayBar` | Implemented facade | `FmpClient.commodity_chart_one_hour`; `fmp.chart.StockChartIntradayBar` |
| Forex | `forex-list` | `forex_list` | unit (`()`) | `ForexPair` | Implemented | `FmpClient.forex_list`; `fmp.forex.ForexPair` |
| Forex | `quote` | `forex_quote` | `QuoteQuery` | `Quote` | Implemented facade | `FmpClient.forex_quote`; `fmp.quote.Quote` |
| Forex | `quote-short` | `forex_quote_short` | `QuoteShortQuery` | `QuoteShort` | Implemented facade | `FmpClient.forex_quote_short`; `fmp.quote.QuoteShort` |
| Forex | `batch-forex-quotes` | `forex_quotes` | `ShortOnlyQuery` | `QuoteShort` | Existing client method; domain descriptor facade added | `FmpClient.forex_quotes`; `fmp.quote.QuoteShort` |
| Forex | `historical-price-eod/light` | `forex_chart_light` | `AssetChartQuery` | `StockChartLightBar` | Implemented facade | `FmpClient.forex_chart_light`; `fmp.chart.StockChartLightBar` |
| Forex | `historical-price-eod/full` | `forex_chart_full` | `AssetChartQuery` | `StockChartFullBar` | Implemented facade | `FmpClient.forex_chart_full`; `fmp.chart.StockChartFullBar` |
| Forex | `historical-chart/1min` | `forex_chart_one_minute` | `AssetChartQuery` | `StockChartIntradayBar` | Implemented facade | `FmpClient.forex_chart_one_minute`; `fmp.chart.StockChartIntradayBar` |
| Forex | `historical-chart/5min` | `forex_chart_five_minutes` | `AssetChartQuery` | `StockChartIntradayBar` | Implemented facade | `FmpClient.forex_chart_five_minutes`; `fmp.chart.StockChartIntradayBar` |
| Forex | `historical-chart/1hour` | `forex_chart_one_hour` | `AssetChartQuery` | `StockChartIntradayBar` | Implemented facade | `FmpClient.forex_chart_one_hour`; `fmp.chart.StockChartIntradayBar` |
| Crypto | `cryptocurrency-list` | `cryptocurrency_list` | unit (`()`) | `CryptocurrencyListing` | Implemented | `FmpClient.cryptocurrency_list`; `fmp.crypto.CryptocurrencyListing` |
| Crypto | `quote` | `cryptocurrency_quote` | `QuoteQuery` | `Quote` | Implemented facade | `FmpClient.cryptocurrency_quote`; `fmp.quote.Quote` |
| Crypto | `quote-short` | `cryptocurrency_quote_short` | `QuoteShortQuery` | `QuoteShort` | Implemented facade | `FmpClient.cryptocurrency_quote_short`; `fmp.quote.QuoteShort` |
| Crypto | `batch-crypto-quotes` | `cryptocurrency_quotes` | `ShortOnlyQuery` | `QuoteShort` | Existing client method; domain descriptor facade added | `FmpClient.cryptocurrency_quotes`; `fmp.quote.QuoteShort` |
| Crypto | `historical-price-eod/light` | `cryptocurrency_chart_light` | `AssetChartQuery` | `StockChartLightBar` | Implemented facade | `FmpClient.cryptocurrency_chart_light`; `fmp.chart.StockChartLightBar` |
| Crypto | `historical-price-eod/full` | `cryptocurrency_chart_full` | `AssetChartQuery` | `StockChartFullBar` | Implemented facade | `FmpClient.cryptocurrency_chart_full`; `fmp.chart.StockChartFullBar` |
| Crypto | `historical-chart/1min` | `cryptocurrency_chart_one_minute` | `AssetChartQuery` | `StockChartIntradayBar` | Implemented facade | `FmpClient.cryptocurrency_chart_one_minute`; `fmp.chart.StockChartIntradayBar` |
| Crypto | `historical-chart/5min` | `cryptocurrency_chart_five_minutes` | `AssetChartQuery` | `StockChartIntradayBar` | Implemented facade | `FmpClient.cryptocurrency_chart_five_minutes`; `fmp.chart.StockChartIntradayBar` |
| Crypto | `historical-chart/1hour` | `cryptocurrency_chart_one_hour` | `AssetChartQuery` | `StockChartIntradayBar` | Implemented facade | `FmpClient.cryptocurrency_chart_one_hour`; `fmp.chart.StockChartIntradayBar` |

Each domain owns a thin endpoint and response namespace:
`endpoints::commodities`/`responses::commodities`, `endpoints::forex`/
`responses::forex`, and `endpoints::crypto`/`responses::crypto`. The quote
facades reuse `QuoteQuery`, `QuoteShortQuery`, `Quote`, and `QuoteShort`. This
keeps the public API discoverable by asset without creating divergent copies of
the generic wire contracts.

Commodity and forex single-symbol full and short quotes are explicitly US-only
in the documentation. Their domain descriptors record that geography but no
market-data delay or realtime declaration: those details are present on the
stock quote documentation, not these asset sections. Cryptocurrency quote
geography and realtime metadata remain unspecified. The three catalog and
three batch endpoints likewise receive no inferred geography, delay, access,
or bounds.

The batch descriptors delegate to the existing quote-module descriptors, and
their existing `Client` methods are not duplicated. `ShortOnlyQuery` continues
to emit fixed `short=true`; the provider documents only the compact four-field
batch result. A future full-shape variant requires a documented response rather
than a guessed `Quote` contract.

`CommodityListing.exchange` is required but nullable. A missing key therefore
fails decoding, documented `null` is retained as `None`, and future non-null
exchange codes remain accepted. `CryptocurrencyListing.ico_date` is a typed
`Date`, while both supply counts are `u64` so values beyond 32-bit ranges are
preserved. `Quote.market_cap` remains nullable and the shared unsigned market
capitalization and volume types preserve the documented large crypto values.

The 15 chart methods use one narrow shared `AssetChartQuery` because
all three documentation sections expose the same required `symbol` followed by
independently optional `from` and `to` dates. The response rows remain the
existing chart models. Thin domain facades delegate to one path engine. The two
end-of-day routes in each domain carry only the documented 5,000-row bound; no
bounds are attached to intraday routes.

Python runtime parity is deferred. Future client methods use the ordinary names
in the table. Catalog rows live in their matching asset modules; generic quote
and chart rows remain in `fmp.quote` and `fmp.chart` rather than being copied.
