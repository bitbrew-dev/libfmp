# ADR 0008: As-reported, financial-report, and segmentation API contracts

## Status

Accepted for issue #17's Rust implementation and reserved future Python facade.

## Decision

The nine issue #17 endpoints use the following Rust descriptor, client, query,
and response names. Each free descriptor and its `Client` method share the
listed Rust method name.

| FMP path | Rust method | Rust query | Rust response | Future Python API |
| --- | --- | --- | --- | --- |
| `income-statement-as-reported` | `income_statement_as_reported` | `IncomeStatementAsReportedQuery` | `AsReportedFinancialStatement` | `FmpClient.income_statement_as_reported`; `fmp.statements.AsReportedFinancialStatement` |
| `balance-sheet-statement-as-reported` | `balance_sheet_statement_as_reported` | `BalanceSheetStatementAsReportedQuery` | `AsReportedFinancialStatement` | `FmpClient.balance_sheet_statement_as_reported`; `fmp.statements.AsReportedFinancialStatement` |
| `cash-flow-statement-as-reported` | `cash_flow_statement_as_reported` | `CashFlowStatementAsReportedQuery` | `AsReportedFinancialStatement` | `FmpClient.cash_flow_statement_as_reported`; `fmp.statements.AsReportedFinancialStatement` |
| `financial-statement-full-as-reported` | `financial_statement_full_as_reported` | `FinancialStatementFullAsReportedQuery` | `AsReportedFinancialStatement` | `FmpClient.financial_statement_full_as_reported`; `fmp.statements.AsReportedFinancialStatement` |
| `financial-reports-dates` | `financial_reports_dates` | `FinancialReportsDatesQuery` | `FinancialReportDate` | `FmpClient.financial_reports_dates`; `fmp.statements.FinancialReportDate` |
| `financial-reports-json` | `financial_reports_json` | `FinancialReportsJsonQuery` | `FinancialReportJson` | `FmpClient.financial_reports_json`; `fmp.statements.FinancialReportJson` |
| `financial-reports-xlsx` | `financial_reports_xlsx` | `FinancialReportsXlsxQuery` | `BinaryResponse` | `FmpClient.financial_reports_xlsx`; `fmp.statements.BinaryResponse` |
| `revenue-product-segmentation` | `revenue_product_segmentation` | `RevenueProductSegmentationQuery` | `RevenueSegmentation` | `FmpClient.revenue_product_segmentation`; `fmp.statements.RevenueSegmentation` |
| `revenue-geographic-segmentation` | `revenue_geographic_segmentation` | `RevenueGeographicSegmentationQuery` | `RevenueSegmentation` | `FmpClient.revenue_geographic_segmentation`; `fmp.statements.RevenueSegmentation` |

The four as-reported endpoints share a typed
`AsReportedFinancialStatement` envelope for `symbol`, numeric `fiscalYear`,
fiscal `period`, `reportedCurrency`, `date`, and `data`. The provider-native
`data` member remains an open `DynamicObject`: issuer taxonomy keys and JSON
value kinds are preserved without normalization or a fixed financial schema.

The current `outer.md` full-as-reported example contains 300 unique `data`
keys; a mechanical raw-member scan finds no duplicate key in the current
source. JSON objects and `DynamicObject` cannot represent duplicate members.
If a provider or proxy supplies duplicate names, standard JSON object parsing
retains only the effective final member rather than inventing a second SDK
field.

The XLSX documentation's response example is copied JSON report content even
though the endpoint is documented as an XLSX download. The SDK therefore uses
`BinaryResponse`, accepting the official XLSX MIME type
`application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` and
`application/octet-stream` for binary-compatible proxies; it does not model
the copied JSON example as the XLSX response contract.

`FinancialReportDate` links use `SecretUrl` so embedded API keys remain
redacted in debug and display surfaces; implicit serialization is disallowed.
Revenue segmentation uses the shared dynamic row type and the closed documented query value
`SegmentationStructure::Flat`; no unsupported structure spelling is invented.

Python runtime parity remains deferred while the Rust API is prioritized. The
future Python facade will accept ordinary method arguments; Rust query objects
will not be exposed as Python public classes.
