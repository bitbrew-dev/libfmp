# ADR 0024: Discounted cash flow API names

## Status

Accepted for issue #33's Rust API inventory. The standard and levered routes
are implemented in Rust; custom routes and the Python facade remain deferred.

## Decision

The four documented Discounted Cash Flow entries use or reserve the following
names. All routes are worldwide GET endpoints with bare-array responses.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | State | Future Python API |
| --- | --- | --- | --- | --- | --- |
| `discounted-cash-flow` | `discounted_cash_flow` | `DcfQuery` | `DcfValuation` | Implemented | `FmpClient.discounted_cash_flow`; `fmp.dcf.DcfValuation` |
| `levered-discounted-cash-flow` | `levered_discounted_cash_flow` | `DcfQuery` | `DcfValuation` | Implemented | `FmpClient.levered_discounted_cash_flow`; `fmp.dcf.DcfValuation` |
| `custom-discounted-cash-flow` | `custom_discounted_cash_flow` | Reserved custom DCF query | Reserved custom DCF row | Reserved | `FmpClient.custom_discounted_cash_flow`; row in `fmp.dcf` |
| `custom-levered-discounted-cash-flow` | `custom_levered_discounted_cash_flow` | Reserved custom DCF query | Reserved custom levered DCF row | Reserved | `FmpClient.custom_levered_discounted_cash_flow`; row in `fmp.dcf` |

The implemented endpoints live in `endpoints::dcf`; their response model lives
in `responses::dcf`. `DcfQuery` contains only the documented required `Ticker`.
Both implemented routes share `DcfValuation` because their documented rows
have the same four required fields: `symbol`, `date`, `dcf`, and the provider's
unusual exact key `Stock Price`. The Rust field is `stock_price` with an
explicit Serde rename, so neither camel-case nor snake-case spelling is
accepted as the wire key.

No response-row, pagination, date-range, access-plan, or realtime bounds are
inferred. The custom endpoints require their own typed parameter and response
contracts because their documented shapes differ substantially from the two
simple valuations. Their names are reserved here to prevent later naming drift.

Python runtime parity is deferred. Future Python methods mirror the Rust
methods in the table, while response rows belong in `fmp.dcf`.
