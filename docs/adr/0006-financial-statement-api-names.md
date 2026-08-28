# ADR 0006: Financial-statement API names and row contracts

## Status

Accepted for issue #15's Rust implementation and reserved future Python facade.

## Decision

The six financial-statement endpoints use the following Rust descriptor,
client, query, and response-row names. Each free descriptor and its `Client`
method share the listed Rust method name.

| FMP path | Rust method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `income-statement` | `income_statement` | `IncomeStatementQuery` | `IncomeStatement` | `FmpClient.income_statement`; `fmp.statements.IncomeStatement` |
| `balance-sheet-statement` | `balance_sheet_statement` | `BalanceSheetStatementQuery` | `BalanceSheetStatement` | `FmpClient.balance_sheet_statement`; `fmp.statements.BalanceSheetStatement` |
| `cash-flow-statement` | `cash_flow_statement` | `CashFlowStatementQuery` | `CashFlowStatement` | `FmpClient.cash_flow_statement`; `fmp.statements.CashFlowStatement` |
| `income-statement-ttm` | `income_statement_ttm` | `IncomeStatementTtmQuery` | `IncomeStatement` | `FmpClient.income_statement_ttm`; `fmp.statements.IncomeStatement` |
| `balance-sheet-statement-ttm` | `balance_sheet_statement_ttm` | `BalanceSheetStatementTtmQuery` | `BalanceSheetStatementTtm` | `FmpClient.balance_sheet_statement_ttm`; `fmp.statements.BalanceSheetStatementTtm` |
| `cash-flow-statement-ttm` | `cash_flow_statement_ttm` | `CashFlowStatementTtmQuery` | `CashFlowStatement` | `FmpClient.cash_flow_statement_ttm`; `fmp.statements.CashFlowStatement` |

Historical and TTM income statements share `IncomeStatement`, and historical
and TTM cash-flow statements share `CashFlowStatement`, because each pair has
the same documented wire fields. Balance-sheet TTM deliberately uses the
distinct `BalanceSheetStatementTtm`: its documented row omits
`capitalLeaseObligationsNonCurrent` from the historical shape.

`latest-financial-statements` is excluded from issue #15 and this naming
decision. Python runtime parity remains deferred while the Rust API is
prioritized. The future Python facade will accept ordinary method arguments;
the Rust query objects will not be exposed as Python public classes.
