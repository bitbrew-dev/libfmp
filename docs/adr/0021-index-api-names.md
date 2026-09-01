# ADR 0021: Index API names and contract boundaries

## Status

Accepted for issue #30's Rust API. All 15 index routes are implemented; the
Python facade remains deferred.

## Decision

The 15 documented Index endpoints use or reserve the following Rust names.
Generic quote and chart contracts are reused rather than duplicated.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | State |
| --- | --- | --- | --- | --- |
| `index-list` | `index_list` | unit (`()`) | `IndexListing` | Implemented here |
| `quote` | `index_quote` | `QuoteQuery` | `Quote` | Index facade over shared route |
| `quote-short` | `index_quote_short` | `QuoteShortQuery` | `QuoteShort` | Index facade over shared route |
| `batch-index-quotes` | `index_quotes` | `ShortOnlyQuery` | `QuoteShort` | Existing client method; index descriptor facade added here |
| `historical-price-eod/light` | `index_chart_light` | `IndexChartQuery` | `StockChartLightBar` | Implemented here; shared row |
| `historical-price-eod/full` | `index_chart_full` | `IndexChartQuery` | `StockChartFullBar` | Implemented here; shared row |
| `historical-chart/1min` | `index_chart_one_minute` | `IndexChartQuery` | `StockChartIntradayBar` | Implemented here; shared row |
| `historical-chart/5min` | `index_chart_five_minutes` | `IndexChartQuery` | `StockChartIntradayBar` | Implemented here; shared row |
| `historical-chart/1hour` | `index_chart_one_hour` | `IndexChartQuery` | `StockChartIntradayBar` | Implemented here; shared row |
| `sp500-constituent` | `sp500_constituents` | unit (`()`) | `IndexConstituent` | Implemented here |
| `nasdaq-constituent` | `nasdaq_constituents` | unit (`()`) | `IndexConstituent` | Implemented here |
| `dowjones-constituent` | `dow_jones_constituents` | unit (`()`) | `IndexConstituent` | Implemented here |
| `historical-sp500-constituent` | `historical_sp500_constituents` | unit (`()`) | `HistoricalIndexConstituent` | Implemented here |
| `historical-nasdaq-constituent` | `historical_nasdaq_constituents` | unit (`()`) | `HistoricalIndexConstituent` | Implemented here |
| `historical-dowjones-constituent` | `historical_dow_jones_constituents` | unit (`()`) | `HistoricalIndexConstituent` | Implemented here |

The dedicated quote descriptor names make the generic provider routes
discoverable from `endpoints::indexes`. They delegate to the existing quote
descriptors and reuse `Quote`, `QuoteShort`, `QuoteQuery`, and `QuoteShortQuery`
exactly. The `Client` gains `index_quote` and `index_quote_short`; its existing
`index_quotes` method already covers the batch route and is not duplicated.
Symbols remain validated, representation-preserving `Ticker` values, including
caret-prefixed values such as `^VIX`.

The five index chart descriptors share one narrow `IndexChartQuery`, which
encodes `symbol`, then the independently optional `from` and `to` dates. The
index documentation does not expose the stock chart's `nonadjusted` or
`extended` flags, so they are intentionally absent. The two end-of-day routes
record only the documented 5,000-row response maximum; the three intraday
routes have no invented bounds. Response rows are reused exactly from the chart
module rather than introducing index-specific duplicates.

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

The six constituent methods share a private `IndexKind` routing abstraction,
but the public API stays discoverable through the names listed above. Current
constituents share `IndexConstituent`; their documented `dateFirstAdded` is
required but nullable. Historical changes share `HistoricalIndexConstituent`;
their human-readable `dateAdded` is representation-preserving text, while
`removedTicker` and `removedSecurity` are required but nullable. CIK values
remain representation-preserving, including leading zeroes. No geography or
bounds are inferred for these six routes.

Python runtime bindings are deferred. The future facade reserves
ordinary `FmpClient` method names and `fmp.indexes.IndexListing`,
`IndexConstituent`, and `HistoricalIndexConstituent`; generic quote and chart
rows remain in their existing Python modules.
