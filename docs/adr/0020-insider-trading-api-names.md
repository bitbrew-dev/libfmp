# ADR 0020: Insider-trading API names and contract boundaries

## Status

Accepted for the contract foundation. Endpoint descriptors and Rust client
methods follow in the stacked implementation slices; the future Python facade
remains reserved for the parity work.

## Decision

The six insider-trading endpoints use the following final Rust names and
reserve the listed future Python names.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `insider-trading/latest` | `latest_insider_trades` | `LatestInsiderTradesQuery` | `InsiderTrade` | `FmpClient.latest_insider_trades`; `fmp.insider_trading.InsiderTrade` |
| `insider-trading/search` | `search_insider_trades` | `InsiderTradesSearchQuery` | `InsiderTrade` | `FmpClient.search_insider_trades`; `fmp.insider_trading.InsiderTrade` |
| `insider-trading/reporting-name` | `search_insider_reporting_names` | `InsiderReportingNameSearchQuery` | `InsiderReportingName` | `FmpClient.search_insider_reporting_names`; `fmp.insider_trading.InsiderReportingName` |
| `insider-trading-transaction-type` | `insider_transaction_types` | `()` | `InsiderTransactionType` | `FmpClient.insider_transaction_types`; `fmp.insider_trading.InsiderTransactionType` |
| `insider-trading/statistics` | `insider_trade_statistics` | `InsiderTradeStatisticsQuery` | `InsiderTradeStatistics` | `FmpClient.insider_trade_statistics`; `fmp.insider_trading.InsiderTradeStatistics` |
| `acquisition-of-beneficial-ownership` | `beneficial_ownership_acquisitions` | `BeneficialOwnershipAcquisitionsQuery` | `BeneficialOwnershipAcquisition` | `FmpClient.beneficial_ownership_acquisitions`; `fmp.insider_trading.BeneficialOwnershipAcquisition` |

All six endpoints are US-only `GET` requests returning bare arrays. The latest
and search routes deliberately share the wire-identical `InsiderTrade` row.
Every documented response field is required and non-null, while unknown fields
remain accepted.

The latest query's date, page, and limit are independently optional. Every
search-trades filter is independently optional. The transaction taxonomy has
no documented parameters and therefore uses the existing unit query. The
reporting-name and statistics inputs are required, while beneficial ownership
has a required ticker and optional limit. Query construction preserves zero
and the full `u32` pagination domain; it does not enforce documented response
or page caps.

CIKs and CUSIPs stay string-backed to preserve leading zeroes. Transaction
types use an open, representation-preserving `TransactionTypeCode`, shared by
queries and responses. Trade counts and aggregate counts use `u64`; prices and
ratios remain decimal-capable JSON numbers. Statistics use numeric
`CalendarYear` and validated numeric `CalendarQuarter` values.

Beneficial-ownership `acceptedDate` is a date, not a timestamp. Its voting,
dispositive, amount-owned, and percent fields use `NumericString`, preserving
their required quoted wire representation and exact decimal text.

Python runtime bindings are deferred. The future facade reserves ordinary
`FmpClient` methods and response classes under `fmp.insider_trading`; Rust
query structs will not become Python public classes.
