# ADR 0021: Index API names and contract boundaries

## Status

Accepted for issue #30's Rust API. The index directory and quote facade are
implemented in this slice; shared chart routes are recorded here, while the
constituent routes and Python facade remain deferred.

## Decision

The 15 documented Index endpoints use or reserve the following Rust names.
Generic quote and chart contracts are reused rather than duplicated.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | State |
| --- | --- | --- | --- | --- |
| `index-list` | `index_list` | unit (`()`) | `IndexListing` | Implemented here |
| `quote` | `index_quote` | `QuoteQuery` | `Quote` | Index facade over shared route |
| `quote-short` | `index_quote_short` | `QuoteShortQuery` | `QuoteShort` | Index facade over shared route |
| `batch-index-quotes` | `index_quotes` | `ShortOnlyQuery` | `QuoteShort` | Existing client method; index descriptor facade added here |
| `historical-price-eod/light` | `stock_chart_light` | `StockChartEodQuery` | `StockChartLightBar` | Existing shared route |
| `historical-price-eod/full` | `stock_chart_full` | `StockChartEodQuery` | `StockChartFullBar` | Existing shared route |
| `historical-chart/1min` | `stock_chart_one_minute` | `StockChartIntradayQuery` | `StockChartIntradayBar` | Existing shared route |
| `historical-chart/5min` | `stock_chart_five_minutes` | `StockChartIntradayQuery` | `StockChartIntradayBar` | Existing shared route |
| `historical-chart/1hour` | `stock_chart_one_hour` | `StockChartIntradayQuery` | `StockChartIntradayBar` | Existing shared route |
| `sp500-constituent` | `sp500_constituents` | unit (`()`) | `IndexConstituent` | Deferred |
| `nasdaq-constituent` | `nasdaq_constituents` | unit (`()`) | `IndexConstituent` | Deferred |
| `dowjones-constituent` | `dow_jones_constituents` | unit (`()`) | `IndexConstituent` | Deferred |
| `historical-sp500-constituent` | `historical_sp500_constituents` | unit (`()`) | `HistoricalIndexConstituent` | Deferred |
| `historical-nasdaq-constituent` | `historical_nasdaq_constituents` | unit (`()`) | `HistoricalIndexConstituent` | Deferred |
| `historical-dowjones-constituent` | `historical_dow_jones_constituents` | unit (`()`) | `HistoricalIndexConstituent` | Deferred |

The dedicated quote descriptor names make the generic provider routes
discoverable from `endpoints::indexes`. They delegate to the existing quote
descriptors and reuse `Quote`, `QuoteShort`, `QuoteQuery`, and `QuoteShortQuery`
exactly. The `Client` gains `index_quote` and `index_quote_short`; its existing
`index_quotes` method already covers the batch route and is not duplicated.
Symbols remain validated, representation-preserving `Ticker` values, including
caret-prefixed values such as `^VIX`.

`batch-index-quotes` remains a closed compact contract. `ShortOnlyQuery` always
emits `short=true` and has no boolean setter. Although the parameter name implies
another mode, `outer.md` documents only the four-field compact response, so the
undocumented full batch shape is not exposed or guessed. A future typed full
variant requires an explicit documented response contract.

All 15 routes are `GET` requests returning bare arrays. `index-list` documents
worldwide coverage and required, non-null `symbol`, `name`, `exchange`, and
`currency` fields. Its `IndexListing` row is the only new response type in this
slice. Unknown fields remain accepted for forward compatibility. No query or
response caps are documented for the directory or quote routes.

The six future constituent methods may share a private `IndexKind` routing
abstraction, but the public API stays discoverable through the names listed
above. Python runtime bindings are deferred. The future facade reserves
ordinary `FmpClient` method names and `fmp.indexes.IndexListing`,
`IndexConstituent`, and `HistoricalIndexConstituent`; generic quote and chart
rows remain in their existing Python modules.
