# ADR 0019: SEC filings and industry-classification API names

## Status

Accepted. All twelve SEC filing, company lookup, profile, and
industry-classification descriptors and Rust client methods are implemented.
The future Python facade stays reserved for a later parity slice.

## Decision

The twelve SEC filing, company lookup, profile, and industry-classification
routes use the following final Rust names and reserve the listed future Python
names.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `sec-filings-8k` | `latest_8k_sec_filings` | `Latest8kSecFilingsQuery` | `SecFiling` | `FmpClient.latest_8k_sec_filings`; `fmp.sec_filings.SecFiling` |
| `sec-filings-financials` | `latest_sec_filings` | `LatestSecFilingsQuery` | `SecFiling` | `FmpClient.latest_sec_filings`; `fmp.sec_filings.SecFiling` |
| `sec-filings-search/form-type` | `sec_filings_by_form_type` | `SecFilingsByFormTypeQuery` | `SecFiling` | `FmpClient.sec_filings_by_form_type`; `fmp.sec_filings.SecFiling` |
| `sec-filings-search/symbol` | `sec_filings_by_symbol` | `SecFilingsBySymbolQuery` | `SecFiling` | `FmpClient.sec_filings_by_symbol`; `fmp.sec_filings.SecFiling` |
| `sec-filings-search/cik` | `sec_filings_by_cik` | `SecFilingsByCikQuery` | `SecFiling` | `FmpClient.sec_filings_by_cik`; `fmp.sec_filings.SecFiling` |
| `sec-filings-company-search/name` | `search_sec_companies_by_name` | `SecCompaniesByNameQuery` | `SecCompanySearchResult` | `FmpClient.search_sec_companies_by_name`; `fmp.sec_filings.SecCompanySearchResult` |
| `sec-filings-company-search/symbol` | `search_sec_companies_by_symbol` | `SecCompaniesBySymbolQuery` | `SecCompanySearchResult` | `FmpClient.search_sec_companies_by_symbol`; `fmp.sec_filings.SecCompanySearchResult` |
| `sec-filings-company-search/cik` | `search_sec_companies_by_cik` | `SecCompaniesByCikQuery` | `SecCompanySearchResult` | `FmpClient.search_sec_companies_by_cik`; `fmp.sec_filings.SecCompanySearchResult` |
| `sec-profile` | `sec_company_profile` | `SecCompanyProfileQuery` | `SecCompanyProfile` | `FmpClient.sec_company_profile`; `fmp.sec_filings.SecCompanyProfile` |
| `standard-industrial-classification-list` | `industry_classifications` | `IndustryClassificationsQuery` | `SicClassification` | `FmpClient.industry_classifications`; `fmp.sec_filings.SicClassification` |
| `industry-classification-search` | `search_industry_classifications` | `IndustryClassificationSearchQuery` | `DynamicObject` | `FmpClient.search_industry_classifications`; raw `dict` rows |
| `all-industry-classification` | `all_industry_classifications` | `AllIndustryClassificationsQuery` | `SecCompanySearchResult` | `FmpClient.all_industry_classifications`; `fmp.sec_filings.SecCompanySearchResult` |

All twelve endpoints are US-only `GET` requests returning bare arrays. The five
filing routes deliberately share `SecFiling`; `hasFinancials` is optional so
the documented `true`, `null`, and absent forms all decode. `FormType` is an
open string-backed fundamental rather than a closed enumeration.

Date inputs are independent `Date` values. The contract records the documented
90-day range and pagination maxima as endpoint metadata later; query
construction does not reject or normalize values. Optional page and limit
values retain the full `u32` domain. CIKs remain string-backed, preserving
leading zeroes. The profile query emits the provider's unusual key literally as
`cik-A` without assigning undocumented semantics to it.

Company-search values preserve the documented `"None"` ticker, empty SIC and
industry strings, and raw address text. Profile employees remain a
`NumericString`. The required but null-only `securityType` uses
`Option<DynamicJson>` so a non-null wire type is not invented. The SIC list is
typed, while industry-classification search stays `Vec<DynamicObject>` because
its only documented row is `{}`; no schema is invented from prose.

Python runtime bindings are deferred. The future facade reserves ordinary
`FmpClient` methods and response classes under `fmp.sec_filings`; Rust query
structs will not become Python public classes.
