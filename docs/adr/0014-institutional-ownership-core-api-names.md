# ADR 0014: Institutional-ownership core API names and semantics

## Status

Accepted for issue #23's Rust core implementation and reserved future Python
facade.

## Decision

The first three Form 13F endpoints use the following final Rust descriptor,
client, query, and response-row names and reserve the listed future Python
names.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `institutional-ownership/latest` | `latest_institutional_ownership_filings` | `LatestInstitutionalOwnershipFilingsQuery` | `InstitutionalOwnershipFiling` | `FmpClient.latest_institutional_ownership_filings`; `fmp.institutional_ownership.InstitutionalOwnershipFiling` |
| `institutional-ownership/extract` | `institutional_ownership_extract` | `InstitutionalOwnershipExtractQuery` | `InstitutionalHolding` | `FmpClient.institutional_ownership_extract`; `fmp.institutional_ownership.InstitutionalHolding` |
| `institutional-ownership/dates` | `form_13f_filing_dates` | `Form13fFilingDatesQuery` | `Form13fFilingDate` | `FmpClient.form_13f_filing_dates`; `fmp.institutional_ownership.Form13fFilingDate` |

All three endpoints are US-only `GET` requests returning bare arrays with
required, non-null documented fields and unknown-field tolerance. CIK and
CUSIP values reuse the representation-preserving `Cik` and `Cusip`
fundamentals, so leading zeroes remain data rather than numeric padding.

The latest-filing row distinguishes its reporting `date` (`Date`) from
`filingDate` and `acceptedDate` (`ApiDateTime`). The extract response documents
all three temporal fields as date-only strings, so each uses `Date`. Its
required `putCallShare` is an ordinary `String` and preserves the documented
empty value. Extract queries reuse `Year` and the closed textual query
`Quarter`, while dates responses reuse numeric `CalendarYear` and validated
numeric `CalendarQuarter`.

The latest endpoint documents a maximum page number of 100 and no maximum for
the optional `limit`. Page 100 is therefore descriptor metadata; constructors
remain lossless and allow zero, page 101, and high limit values so callers can
observe future provider behavior. No limit or response-row bound is invented.

Extracted `shares` and filing `value` are exact non-negative JSON integers and
use `u64`. This preserves large documented-domain values without floating-point
coercion. The SDK does not infer currency units or scale the filing value.

Later holder analytics, summaries, industry endpoints, and Python runtime
bindings are deferred. The future Python facade will accept ordinary method
arguments and will not expose Rust query structs as Python public classes.
