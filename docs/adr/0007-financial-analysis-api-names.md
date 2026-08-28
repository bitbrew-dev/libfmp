# ADR 0007: Financial-analysis API names and growth contracts

## Status

Accepted for issue #16's Rust implementation and reserved future Python facade.

## Decision

The twelve issue #16 endpoints use the following Rust descriptor, client,
query, and response-row names. Each free descriptor and its `Client` method
share the listed Rust method name.

| FMP path | Rust method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `latest-financial-statements` | `latest_financial_statements` | `LatestFinancialStatementsQuery` | `LatestFinancialStatement` | `FmpClient.latest_financial_statements`; `fmp.statements.LatestFinancialStatement` |
| `key-metrics` | `key_metrics` | `KeyMetricsQuery` | `KeyMetrics` | `FmpClient.key_metrics`; `fmp.statements.KeyMetrics` |
| `ratios` | `financial_ratios` | `FinancialRatiosQuery` | `FinancialRatios` | `FmpClient.financial_ratios`; `fmp.statements.FinancialRatios` |
| `key-metrics-ttm` | `key_metrics_ttm` | `KeyMetricsTtmQuery` | `KeyMetricsTtm` | `FmpClient.key_metrics_ttm`; `fmp.statements.KeyMetricsTtm` |
| `ratios-ttm` | `financial_ratios_ttm` | `FinancialRatiosTtmQuery` | `FinancialRatiosTtm` | `FmpClient.financial_ratios_ttm`; `fmp.statements.FinancialRatiosTtm` |
| `financial-scores` | `financial_scores` | `FinancialScoresQuery` | `FinancialScore` | `FmpClient.financial_scores`; `fmp.statements.FinancialScore` |
| `owner-earnings` | `owner_earnings` | `OwnerEarningsQuery` | `OwnerEarnings` | `FmpClient.owner_earnings`; `fmp.statements.OwnerEarnings` |
| `enterprise-values` | `enterprise_values` | `EnterpriseValuesQuery` | `EnterpriseValue` | `FmpClient.enterprise_values`; `fmp.statements.EnterpriseValue` |
| `income-statement-growth` | `income_statement_growth` | `IncomeStatementGrowthQuery` | `IncomeStatementGrowth` | `FmpClient.income_statement_growth`; `fmp.statements.IncomeStatementGrowth` |
| `balance-sheet-statement-growth` | `balance_sheet_statement_growth` | `BalanceSheetStatementGrowthQuery` | `BalanceSheetStatementGrowth` | `FmpClient.balance_sheet_statement_growth`; `fmp.statements.BalanceSheetStatementGrowth` |
| `cash-flow-statement-growth` | `cash_flow_statement_growth` | `CashFlowStatementGrowthQuery` | `CashFlowStatementGrowth` | `FmpClient.cash_flow_statement_growth`; `fmp.statements.CashFlowStatementGrowth` |
| `financial-growth` | `financial_statement_growth` | `FinancialStatementGrowthQuery` | `FinancialStatementGrowth` | `FmpClient.financial_statement_growth`; `fmp.statements.FinancialStatementGrowth` |

The semantic `financial_statement_growth` name intentionally differs from
the provider's abbreviated `financial-growth` path. It identifies the combined
income-statement, balance-sheet, and cash-flow growth response without leaking
an ambiguous provider spelling into the public SDK.

Regular and trailing-twelve-month key metrics use the distinct `KeyMetrics`
and `KeyMetricsTtm` row types. Regular and trailing-twelve-month financial
ratios likewise use distinct `FinancialRatios` and `FinancialRatiosTtm` row
types. The four growth endpoints also use four distinct response types because
their documented fields are not interchangeable.

All documented growth values remain raw `f64` ratios. The SDK does not scale
them into percentages or constrain them to a percentage-like range.

Python runtime parity remains deferred while the Rust API is prioritized. The
future Python facade will accept ordinary method arguments; Rust query objects
will not be exposed as Python public classes.
