# ADR 0018: ETF and mutual-fund API names and contract boundaries

## Status

Accepted. The Rust holdings, information, allocation, and exposure slices are
implemented; the disclosure slice and future Python facade remain reserved.

## Decision

The nine ETF and mutual-fund endpoints use the following final Rust method,
query, and response-row names and reserve the listed future Python names.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `etf/holdings` | `etf_holdings` | `EtfHoldingsQuery` | `EtfFundHolding` | `FmpClient.etf_holdings`; `fmp.funds.EtfFundHolding` |
| `etf/info` | `etf_info` | `EtfInfoQuery` | `EtfFundInfo` | `FmpClient.etf_info`; `fmp.funds.EtfFundInfo` |
| `etf/country-weightings` | `etf_country_weightings` | `EtfCountryWeightingsQuery` | `EtfCountryWeighting` | `FmpClient.etf_country_weightings`; `fmp.funds.EtfCountryWeighting` |
| `etf/asset-exposure` | `etf_asset_exposure` | `EtfAssetExposureQuery` | `EtfAssetExposure` | `FmpClient.etf_asset_exposure`; `fmp.funds.EtfAssetExposure` |
| `etf/sector-weightings` | `etf_sector_weightings` | `EtfSectorWeightingsQuery` | `EtfSectorWeighting` | `FmpClient.etf_sector_weightings`; `fmp.funds.EtfSectorWeighting` |
| `funds/disclosure-holders-latest` | `latest_fund_disclosure_holders` | `LatestFundDisclosureHoldersQuery` | `FundDisclosureHolder` | `FmpClient.latest_fund_disclosure_holders`; `fmp.funds.FundDisclosureHolder` |
| `funds/disclosure` | `fund_disclosures` | `FundDisclosureQuery` | `FundDisclosure` | `FmpClient.fund_disclosures`; `fmp.funds.FundDisclosure` |
| `funds/disclosure-holders-search` | `search_fund_disclosure_holders` | `FundDisclosureHolderSearchQuery` | `FundDisclosureSearchResult` | `FmpClient.search_fund_disclosure_holders`; `fmp.funds.FundDisclosureSearchResult` |
| `funds/disclosure-dates` | `fund_disclosure_dates` | `FundDisclosureDatesQuery` | `FundDisclosureDate` | `FmpClient.fund_disclosure_dates`; `fmp.funds.FundDisclosureDate` |

All nine endpoints are `GET` requests returning bare arrays. Documented fields
are required and non-null, while unknown fields remain accepted. Query values
have no inferred validation or response caps.

The six symbol-only queries reuse `Ticker`. Disclosure queries encode required
`symbol`, `year`, and `quarter`, followed by optional `cik`, in that exact
order. Dates queries encode required `symbol` followed by optional `cik`.
Name search uses `SearchTerm` so its commas and spaces remain valid input.

Wire representation remains part of the response contract. Country weights
use `PercentString`, while the other percentages are JSON numbers. Share and
balance counts use `u64`; disclosed share changes use `i64`; market values use
decimal-capable `MarketValue`. Identifiers stay string-backed so leading zeroes
and the documented `N/A` CUSIP survive. `cur_cd` is renamed explicitly.
Boolean JSON values remain distinct from the disclosure `Y`/`N` flags, and
numeric-looking strings use `NumericString`.

Temporal types also remain distinct: plain dates use `Date`, space-separated
timestamps use `ApiDateTime`, and the RFC 3339 millisecond information timestamp
uses `IsoTimestamp`. The numeric disclosure year and quarter use
`CalendarYear` and `CalendarQuarter`. `FundDisclosureDate` is an endpoint-facing
alias of the wire-identical existing `Form13fFilingDate` row rather than a
duplicate model.

Python runtime bindings are deferred. The future facade reserves ordinary
`FmpClient` methods and response classes under `fmp.funds`; Rust query structs
will not become Python public classes.
