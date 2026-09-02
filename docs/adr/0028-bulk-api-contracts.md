# ADR 0028: Bulk API contracts

## Status

Accepted for issue #37's Rust API inventory. The first fifteen routes are
implemented in Rust; the remaining three routes and the Python facade are
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
| `key-metrics-ttm-bulk` | `bulk_key_metrics_ttm` | `()` | `BulkKeyMetricsTtm` | Rust complete |
| `ratios-ttm-bulk` | `bulk_financial_ratios_ttm` | `()` | `BulkFinancialRatiosTtm` | Rust complete |
| `peers-bulk` | `bulk_stock_peers` | `()` | `BulkStockPeers` | Rust complete |
| `earnings-surprises-bulk` | `bulk_earnings_surprises` | `BulkYearQuery` | `BulkEarningsSurprise` | Rust complete |
| `income-statement-bulk` | `bulk_income_statements` | `BulkStatementQuery` | `BulkIncomeStatement` | Rust complete |
| `income-statement-growth-bulk` | `bulk_income_statement_growth` | `BulkStatementQuery` | `BulkIncomeStatementGrowth` | Rust complete |
| `balance-sheet-statement-bulk` | `bulk_balance_sheet_statements` | `BulkStatementQuery` | `BulkBalanceSheetStatement` | Rust complete |
| `balance-sheet-statement-growth-bulk` | `bulk_balance_sheet_statement_growth` | `BulkStatementQuery` | `BulkBalanceSheetStatementGrowth` | Rust complete |
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

The two trailing-twelve-month contracts explicitly map every TTM-bearing field
to its documented wire key. Serde's ordinary camel-case conversion would emit
`Ttm`, while the provider uses uppercase `TTM`, including the compound keys
`evToEBITDATTM`, `netDebtToEBITDATTM`, and `netIncomePerEBTTTM`. The provider
typo `researchAndDevelopementToRevenueTTM` is preserved on the wire while the
public Rust field uses the corrected `development` spelling. Every documented
metric and ratio remains a `NumericString`, preserving zero, negative, large,
and decimal representations and rejecting JSON numbers. The two diluted P/E
fields absent from the bulk fixture are not inferred from related APIs.

`BulkStockPeers.peers` remains the documented scalar `String`; it is neither
split nor promoted to a ticker collection. `BulkYearQuery` has exactly one
required `year` key backed by the existing open `Year` type. No range or
default is inferred. Earnings-surprise EPS fields remain numeric strings, and
both documented dates use the strict `Date` type.

`BulkStatementQuery` has exactly the required `year` and `period` keys, in that
wire order. It uses the existing open `Year` and the narrower `FiscalPeriod`
whose complete documented set is `Q1`, `Q2`, `Q3`, `Q4`, and `FY`; it does not
reuse the ordinary statement query's retrieval-frequency alternatives or
documented 1,000-row response bound. No default, option, or year range is
inferred.

The two income bulk rows remain separate from the normalized statement models.
Every documented monetary, count, EPS, and growth metric is `NumericString`,
which preserves integer-like, decimal, negative, zero, and large quoted values
while rejecting JSON numbers. Identity fields use the proven narrow types,
including a representation-preserving `Cik` so the documented ten leading
zeroes survive. The exact acronym keys `growthEBITDA`, `growthEPS`,
`growthEPSDiluted`, and `growthEBIT` are mapped explicitly. The request examples
ask for year 2026 while both response examples identify fiscal year 2025; the
response is preserved as documented rather than made to agree with the query.

The two balance-sheet bulk rows likewise remain distinct from the normalized
statement models, and every documented metric is a required `NumericString`.
The provider typo `growthOthertotalStockholdersEquity` is preserved on the wire
while the public Rust field uses the corrected `other_total` spelling. The
distinct documented key `growthTotalLiabilitiesAndStockholdersEquity` is not
silently changed to the non-growth row's `totalLiabilitiesAndTotalEquity`
wording.

The source marks only `price-target-summary-bulk` as US-only. The other fourteen
implemented endpoints are worldwide. No response bounds, pagination, access
requirement, conditional plan, or realtime behavior is inferred.

Python runtime parity remains deferred. Only the future `fmp.bulk` namespace
and the method and model names listed above are reserved.
