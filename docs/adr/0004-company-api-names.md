# ADR 0004: Company API names

## Status

Accepted for issue #13's Rust implementation and reserved future Python facade.

## Decision

The 17 company endpoints use the following Rust method, query, and response-row
names. The same method name applies to the free descriptor and `Client` method.

| FMP path | Rust method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `profile` | `profile` | `ProfileQuery` | `CompanyProfile` | `FmpClient.profile`; `fmp.company.CompanyProfile` |
| `profile-cik` | `profile_by_cik` | `ProfileByCikQuery` | `CompanyProfile` | `FmpClient.profile_by_cik`; `fmp.company.CompanyProfile` |
| `company-notes` | `company_notes` | `CompanyNotesQuery` | `CompanyNote` | `FmpClient.company_notes`; `fmp.company.CompanyNote` |
| `stock-peers` | `stock_peers` | `StockPeersQuery` | `StockPeer` | `FmpClient.stock_peers`; `fmp.company.StockPeer` |
| `delisted-companies` | `delisted_companies` | `DelistedCompaniesQuery` | `DelistedCompany` | `FmpClient.delisted_companies`; `fmp.company.DelistedCompany` |
| `employee-count` | `employee_count` | `EmployeeCountQuery` | `EmployeeCount` | `FmpClient.employee_count`; `fmp.company.EmployeeCount` |
| `historical-employee-count` | `historical_employee_count` | `HistoricalEmployeeCountQuery` | `EmployeeCount` | `FmpClient.historical_employee_count`; `fmp.company.EmployeeCount` |
| `market-capitalization` | `market_capitalization` | `MarketCapitalizationQuery` | `MarketCapitalizationRecord` | `FmpClient.market_capitalization`; `fmp.company.MarketCapitalizationRecord` |
| `market-capitalization-batch` | `market_capitalization_batch` | `MarketCapitalizationBatchQuery` | `MarketCapitalizationRecord` | `FmpClient.market_capitalization_batch`; `fmp.company.MarketCapitalizationRecord` |
| `historical-market-capitalization` | `historical_market_capitalization` | `HistoricalMarketCapitalizationQuery` | `MarketCapitalizationRecord` | `FmpClient.historical_market_capitalization`; `fmp.company.MarketCapitalizationRecord` |
| `shares-float` | `shares_float` | `SharesFloatQuery` | `CompanyShareFloat` | `FmpClient.shares_float`; `fmp.company.CompanyShareFloat` |
| `shares-float-all` | `shares_float_all` | `SharesFloatAllQuery` | `AllSharesFloatRecord` | `FmpClient.shares_float_all`; `fmp.company.AllSharesFloatRecord` |
| `mergers-acquisitions-latest` | `mergers_acquisitions_latest` | `MergersAcquisitionsLatestQuery` | `MergerAcquisition` | `FmpClient.mergers_acquisitions_latest`; `fmp.company.MergerAcquisition` |
| `mergers-acquisitions-search` | `mergers_acquisitions_search` | `MergersAcquisitionsSearchQuery` | `MergerAcquisition` | `FmpClient.mergers_acquisitions_search`; `fmp.company.MergerAcquisition` |
| `key-executives` | `key_executives` | `KeyExecutivesQuery` | `CompanyExecutive` | `FmpClient.key_executives`; `fmp.company.CompanyExecutive` |
| `governance-executive-compensation` | `executive_compensation` | `ExecutiveCompensationQuery` | `ExecutiveCompensation` | `FmpClient.executive_compensation`; `fmp.company.ExecutiveCompensation` |
| `executive-compensation-benchmark` | `executive_compensation_benchmark` | `ExecutiveCompensationBenchmarkQuery` | `ExecutiveCompensationBenchmark` | `FmpClient.executive_compensation_benchmark`; `fmp.company.ExecutiveCompensationBenchmark` |

Python runtime parity remains deferred. A future facade may accept ordinary
method arguments, but it will not expose the Rust query objects as Python
public classes.
