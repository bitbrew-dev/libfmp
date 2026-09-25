# ADR 0015: Analyst API names and contract boundaries

## Status

Accepted for issue #24's Rust core implementation and reserved future Python
facade.

## Decision

The eight Analyst endpoints use the following final Rust descriptor, client,
query, and response-row names and reserve the listed future Python names.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `analyst-estimates` | `financial_estimates` | `FinancialEstimatesQuery` | `FinancialEstimate` | `FmpClient.financial_estimates`; `fmp.analyst.FinancialEstimate` |
| `ratings-snapshot` | `ratings_snapshot` | `RatingsSnapshotQuery` | `RatingSnapshot` | `FmpClient.ratings_snapshot`; `fmp.analyst.RatingSnapshot` |
| `ratings-historical` | `historical_ratings` | `HistoricalRatingsQuery` | `HistoricalRating` | `FmpClient.historical_ratings`; `fmp.analyst.HistoricalRating` |
| `price-target-summary` | `price_target_summary` | `PriceTargetSummaryQuery` | `PriceTargetSummary` | `FmpClient.price_target_summary`; `fmp.analyst.PriceTargetSummary` |
| `price-target-consensus` | `price_target_consensus` | `PriceTargetConsensusQuery` | `PriceTargetConsensus` | `FmpClient.price_target_consensus`; `fmp.analyst.PriceTargetConsensus` |
| `grades` | `stock_grades` | `StockGradesQuery` | `StockGrade` | `FmpClient.stock_grades`; `fmp.analyst.StockGrade` |
| `grades-historical` | `historical_stock_grades` | `HistoricalStockGradesQuery` | `HistoricalStockGrade` | `FmpClient.historical_stock_grades`; `fmp.analyst.HistoricalStockGrade` |
| `grades-consensus` | `stock_grades_summary` | `StockGradesSummaryQuery` | `StockGradesSummary` | `FmpClient.stock_grades_summary`; `fmp.analyst.StockGradesSummary` |

All eight endpoints are `GET` requests returning bare arrays. Documented
fields are required and non-null, while unknown fields remain accepted for
forward compatibility. Symbols reuse `Ticker` and response dates reuse
`Date`. Snapshot responses remain arrays even where the documentation states
that only one row is returned.

Financial-estimate queries own required `symbol` and `period`, followed by
optional `page` and `limit`. Their period is the narrow
`RetrievalFrequency`, whose exact wire values are `annual` and `quarter`; the
broader fiscal-period vocabulary is not accepted. The documented maximum of
1,000 is a response-row fact rather than a query-limit bound. Page and limit
constructors therefore preserve zero and their full shared scalar domains.

The 15 forecast currency fields use signed `StatementAmount` values, avoiding
unsigned rejection of future negative estimates. `StatementAmount` became
`f64` in #341 (ADR 0031), so values above `2^53` round to the nearest `f64`. EPS values remain raw `f64`, while analyst
counts use `Count` (`u64`). No currency unit or scale is inferred.

Rating snapshot and history rows are distinct because only history contains a
date. Their rating labels remain open strings and their seven score fields use
`Count`. Historical-rating queries own an optional limit after the required
symbol. The documented 10,000-row maximum is response metadata and does not
become constructor validation.

The two price-target endpoints are US-only; the other six are worldwide. Price
values use `Price` and accept either integer or decimal JSON numbers. The price
target summary's required `publishers` value remains an exact opaque `String`:
although its contents are JSON-array text, the provider sends that text inside
a JSON string and the SDK does not normalize, reorder, or parse it implicitly.

Current stock-grade labels and actions remain open strings. Historical grade
rows preserve the `analystRatingsStrongBuy` through
`analystRatingsStrongSell` wire-key family, while summary rows preserve the
distinct `strongBuy` through `strongSell` family. Both use exact `Count`
values. Historical-grade queries own an optional limit after the required
symbol; the documented 1,000-row maximum does not constrain that query value.

Python runtime bindings are deferred. The future facade reserves ordinary
`FmpClient` methods and response classes under `fmp.analyst`; Rust query
structs will not become Python public classes.

The Rust descriptors and client methods for all eight Analyst endpoints are
implemented.
