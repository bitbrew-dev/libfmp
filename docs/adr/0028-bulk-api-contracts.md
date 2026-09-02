# ADR 0028: Bulk API contracts

## Status

Accepted for issue #37's Rust API inventory. The first seven routes are
implemented in Rust; the remaining eleven routes and the Python facade are
reserved.

## Decision

All 18 documented bulk entries use or reserve the following public names.
Every response is a bare JSON array. Future Python methods mirror these
snake-case Rust names under `FmpClient`, with models under `fmp.bulk`.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | State |
| --- | --- | --- | --- | --- |
| `profile-bulk` | `bulk_company_profiles` | `BulkPartQuery` | existing `CompanyProfile` | Rust complete |
| `rating-bulk` | `bulk_stock_ratings` | `()` | `BulkStockRating` | Rust complete |
| `dcf-bulk` | `bulk_dcf_valuations` | `()` | `BulkDcfValuation` | Rust complete |
| `scores-bulk` | `bulk_financial_scores` | `()` | `BulkFinancialScore` | Rust complete |
| `price-target-summary-bulk` | `bulk_price_target_summaries` | `()` | `BulkPriceTargetSummary` | Rust complete |
| `etf-holder-bulk` | `bulk_etf_holdings` | `BulkPartQuery` | `BulkEtfHolding` | Rust complete |
| `upgrades-downgrades-consensus-bulk` | `bulk_upgrades_downgrades_consensus` | `()` | `BulkUpgradesDowngradesConsensus` | Rust complete |
| `key-metrics-ttm-bulk` | `bulk_key_metrics_ttm` | `()` | `BulkKeyMetricsTtm` | Reserved |
| `ratios-ttm-bulk` | `bulk_financial_ratios_ttm` | `()` | `BulkFinancialRatiosTtm` | Reserved |
| `peers-bulk` | `bulk_stock_peers` | `()` | `BulkStockPeers` | Reserved |
| `earnings-surprises-bulk` | `bulk_earnings_surprises` | `BulkYearQuery` | `BulkEarningsSurprise` | Reserved |
| `income-statement-bulk` | `bulk_income_statements` | `BulkStatementQuery` | `BulkIncomeStatement` | Reserved |
| `income-statement-growth-bulk` | `bulk_income_statement_growth` | `BulkStatementQuery` | `BulkIncomeStatementGrowth` | Reserved |
| `balance-sheet-statement-bulk` | `bulk_balance_sheet_statements` | `BulkStatementQuery` | `BulkBalanceSheetStatement` | Reserved |
| `balance-sheet-statement-growth-bulk` | `bulk_balance_sheet_statement_growth` | `BulkStatementQuery` | `BulkBalanceSheetStatementGrowth` | Reserved |
| `cash-flow-statement-bulk` | `bulk_cash_flow_statements` | `BulkStatementQuery` | `BulkCashFlowStatement` | Reserved |
| `cash-flow-statement-growth-bulk` | `bulk_cash_flow_statement_growth` | `BulkStatementQuery` | `BulkCashFlowStatementGrowth` | Reserved |
| `eod-bulk` | `bulk_eod` | `BulkEodQuery` | `BulkEodBar` | Reserved |

The endpoint contracts live in `endpoints::bulk`. Response contracts live in
`responses::bulk`, with snapshot-like rows separated into
`responses::bulk::snapshots` internally. `BulkPartQuery` has exactly one
required `part` key backed by the open, representation-preserving `BulkPart`
fundamental type. The source calls it a string and only provides `0` and `1` as
examples, so the crate validates nonempty, control-safe text but infers neither
a numeric representation nor a finite range.

The first seven methods each issue exactly one HTTP request. They do not
automatically enumerate partitions, retry a failed partition, fan out requests,
or join results. A bulk response is buffered as one complete body and decoded
as a bare `Vec`; the existing configurable client timeout applies. This release
does not impose an undocumented byte cap and does not offer streaming decode.

Only `profile-bulk` reuses an existing response model because its documented
36-key row exactly matches `CompanyProfile`. The other six rows remain distinct
bulk contracts. Every documented key is required and non-null. Numeric strings
use `NumericString`, preserving their provider spelling and rejecting JSON
numbers. The consensus route deliberately keeps `symbol` as `String` because
the documented value is empty.

Source defects are preserved rather than repaired. `BulkDcfValuation` maps the
exact key `Stock Price` to `stock_price`. `BulkPriceTargetSummary.publishers`
retains the malformed embedded publisher text verbatim. `BulkEtfHolding` maps
the exact key `lastUpdated"` to `last_updated_raw` and keeps the documented
trailing quote in its string value; no corrected alias or date parser is added.
The empty ETF CUSIP remains a plain string.

The source marks only `price-target-summary-bulk` as US-only. The other six
implemented endpoints are worldwide. No response bounds, pagination, access
requirement, conditional plan, or realtime behavior is inferred.

Python runtime parity remains deferred. Only the future `fmp.bulk` namespace
and the method and model names listed above are reserved.
