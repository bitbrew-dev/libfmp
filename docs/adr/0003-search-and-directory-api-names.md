# ADR 0003: Search and directory API names

## Status

Accepted for issue #12's Rust implementation and reserved Python facade.

## Decision

The 18 search and directory endpoints use the following public names. Rust
query and response names are implemented now. Python names reserve a future
facade only; this decision does not add Python bindings.

| FMP path | Rust descriptor and `Client` method | Rust query | Rust response row | Reserved Python client method and row model |
| --- | --- | --- | --- | --- |
| `search-symbol` | `search_symbol` | `SymbolSearchQuery` | `SymbolSearchResult` | `FmpClient.search_symbol`; `fmp.search.SymbolSearchResult` |
| `search-name` | `search_name` | `NameSearchQuery` | `NameSearchResult` | `FmpClient.search_name`; `fmp.search.NameSearchResult` |
| `search-cik` | `search_cik` | `CikSearchQuery` | `CikSearchResult` | `FmpClient.search_cik`; `fmp.search.CikSearchResult` |
| `search-cusip` | `search_cusip` | `CusipSearchQuery` | `CusipSearchResult` | `FmpClient.search_cusip`; `fmp.search.CusipSearchResult` |
| `search-isin` | `search_isin` | `IsinSearchQuery` | `IsinSearchResult` | `FmpClient.search_isin`; `fmp.search.IsinSearchResult` |
| `search-exchange-variants` | `search_exchange_variants` | `ExchangeVariantsQuery` | `ExchangeVariant` | `FmpClient.search_exchange_variants`; `fmp.search.ExchangeVariant` |
| `company-screener` | `company_screener` | `CompanyScreenerQuery` | `CompanyScreenerEntry` | `FmpClient.company_screener`; `fmp.screener.CompanyScreenerEntry` |
| `stock-list` | `company_symbols` | none | `CompanySymbol` | `FmpClient.company_symbols`; `fmp.directory.CompanySymbol` |
| `financial-statement-symbol-list` | `financial_statement_symbols` | none | `FinancialStatementSymbol` | `FmpClient.financial_statement_symbols`; `fmp.directory.FinancialStatementSymbol` |
| `cik-list` | `cik_list` | `CikListQuery` | `CikEntry` | `FmpClient.cik_list`; `fmp.directory.CikEntry` |
| `symbol-change` | `symbol_changes` | `SymbolChangesQuery` | `SymbolChange` | `FmpClient.symbol_changes`; `fmp.directory.SymbolChange` |
| `etf-list` | `etf_symbols` | none | `EtfSymbol` | `FmpClient.etf_symbols`; `fmp.directory.EtfSymbol` |
| `actively-trading-list` | `actively_trading` | none | `ActivelyTradingSymbol` | `FmpClient.actively_trading`; `fmp.directory.ActivelyTradingSymbol` |
| `earnings-transcript-list` | `earnings_transcript_list` | none | `EarningsTranscriptAvailability` | `FmpClient.earnings_transcript_list`; `fmp.directory.EarningsTranscriptAvailability` |
| `available-exchanges` | `available_exchanges` | `AvailableExchangesQuery` | `AvailableExchange` | `FmpClient.available_exchanges`; `fmp.directory.AvailableExchange` |
| `available-sectors` | `available_sectors` | none | `AvailableSector` | `FmpClient.available_sectors`; `fmp.directory.AvailableSector` |
| `available-industries` | `available_industries` | none | `AvailableIndustry` | `FmpClient.available_industries`; `fmp.directory.AvailableIndustry` |
| `available-countries` | `available_countries` | none | `AvailableCountry` | `FmpClient.available_countries`; `fmp.directory.AvailableCountry` |

Python may accept ordinary keyword arguments instead of exposing Rust query
objects, consistent with the existing binding boundary.
