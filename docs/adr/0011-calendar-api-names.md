# ADR 0011: Calendar API names and source ambiguities

## Status

Accepted for issue #20's Rust contract implementation and reserved future
Python facade.

## Decision

The nine calendar endpoints reserve the following Rust descriptor, client,
query, response-row, and future Python names. Each future free descriptor and
its `Client` method will share the listed Rust method name.

| FMP path | Rust method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `dividends` | `dividends` | `DividendsQuery` | `DividendEvent` | `FmpClient.dividends`; `fmp.calendar.DividendEvent` |
| `dividends-calendar` | `dividends_calendar` | `DividendsCalendarQuery` | `DividendEvent` | `FmpClient.dividends_calendar`; `fmp.calendar.DividendEvent` |
| `earnings` | `earnings` | `EarningsQuery` | `EarningsEvent` | `FmpClient.earnings`; `fmp.calendar.EarningsEvent` |
| `earnings-calendar` | `earnings_calendar` | `EarningsCalendarQuery` | `EarningsEvent` | `FmpClient.earnings_calendar`; `fmp.calendar.EarningsEvent` |
| `ipos-calendar` | `ipos_calendar` | `IposCalendarQuery` | `IpoCalendarEvent` | `FmpClient.ipos_calendar`; `fmp.calendar.IpoCalendarEvent` |
| `ipos-disclosure` | `ipos_disclosure` | `IposDisclosureQuery` | `IpoDisclosure` | `FmpClient.ipos_disclosure`; `fmp.calendar.IpoDisclosure` |
| `ipos-prospectus` | `ipos_prospectus` | `IposProspectusQuery` | `IpoProspectus` | `FmpClient.ipos_prospectus`; `fmp.calendar.IpoProspectus` |
| `splits` | `stock_splits` | `StockSplitsQuery` | `StockSplitEvent` | `FmpClient.stock_splits`; `fmp.calendar.StockSplitEvent` |
| `splits-calendar` | `stock_splits_calendar` | `StockSplitsCalendarQuery` | `StockSplitEvent` | `FmpClient.stock_splits_calendar`; `fmp.calendar.StockSplitEvent` |

Every query preserves the parameter order documented for its endpoint.
Optional `from` and `to` values remain independent. Optional
`includeReportTimes` values use `Option<bool>`, keeping omitted, explicit
`false`, and explicit `true` distinct. Page zero and limit zero remain present
values rather than being treated as absent.

The date examples use `2026-03-06` through `2026-06-06`, a 92-day elapsed
span, while the adjacent notes state a maximum date range of 90 days. The SDK
does not silently choose between the contradictory contracts or reject the
documented example at query construction time. The applicable calendar
descriptors expose the stated 90-day limit as metadata, where consumers can
inspect the contradiction without changing query construction.

`DividendEvent.declaration_date` uses `Option<Date>` with the shared
empty-or-null date codec because one exact response supplies a date and the
other supplies an empty string. Explicit null is accepted by that same shared
wire contract. The two earnings `actual` fields are nullable; their estimates
and all remaining earnings fields are required and non-null.

The IPO-calendar source shows `shares`, `priceRange`, and `marketCap` only as
null. These fields are therefore `Option<DynamicJson>` rather than speculative
numeric models. The provider's literal `daa` key is retained as `daa` and uses
the strict `IsoTimestamp` codec; no undocumented expansion of that name is
invented.

The six response rows decode from bare arrays, keep every documented field
present, and accept unknown fields for forward compatibility. Python runtime
parity remains deferred. The future facade will accept ordinary method
arguments and will not expose Rust query structs as Python public classes.
