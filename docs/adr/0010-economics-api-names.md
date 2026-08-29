# ADR 0010: Economics API names and source ambiguities

## Status

Accepted for issue #19's Rust implementation and reserved future Python facade.

## Decision

The four economics endpoints reserve the following Rust descriptor, client,
query, response-row, and future Python names. Each future free descriptor and
its `Client` method will share the listed Rust method name.

| FMP path | Rust method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `treasury-rates` | `treasury_rates` | `TreasuryRatesQuery` | `TreasuryRate` | `FmpClient.treasury_rates`; `fmp.economics.TreasuryRate` |
| `economic-indicators` | `economic_indicators` | `EconomicIndicatorsQuery` | `EconomicIndicatorObservation` | `FmpClient.economic_indicators`; `fmp.economics.EconomicIndicatorObservation` |
| `economic-calendar` | `economic_calendar` | `EconomicCalendarQuery` | `EconomicCalendarEvent` | `FmpClient.economic_calendar`; `fmp.economics.EconomicCalendarEvent` |
| `market-risk-premium` | `market_risk_premium` | none | `MarketRiskPremium` | `FmpClient.market_risk_premium`; `fmp.economics.MarketRiskPremium` |

`TreasuryRatesQuery` preserves optional `from` and `to` independently.
`EconomicIndicatorsQuery` emits the required `name` first and reuses the
existing `EconomicIndicator` contract, including its 24 explicit provider
spellings and validated `Other` form, before independent dates.
`EconomicCalendarQuery` emits optional `country`, `from`, and `to` in that
order. It uses `CountryCode`; the `MarketRiskPremium.country` response field is
instead a full country-name `String`, as demonstrated by `Zimbabwe`.

The economic-indicators parameter example spans `2025-04-27` through
`2026-04-27`, while the adjacent note explicitly says the maximum date range is
90 days. The query contract preserves both dates without enforcing either
interpretation. The relevant descriptors expose the stated 90-day bound as
metadata, but the SDK does not reject the contradictory documented example at
construction time.

The market-risk-premium prose says data can be accessed for specific dates,
but the section documents no parameters and its endpoint example has no query.
The endpoint is therefore reserved as queryless. Date parameters remain
deferred until a source contract documents their names and wire behavior.

Economic-calendar `previous`, `estimate`, and `actual` are required `f64`
numbers because the sole documented response supplies numeric values for all
three. No missing or null behavior is invented. Rate, change, and percentage
values remain raw provider numbers without percent scaling or range checks.
All response rows are bare arrays, require their documented fields, and accept
unknown fields for forward compatibility.

Python runtime parity remains deferred. The future Python facade will accept
ordinary method arguments and will not expose Rust query structs as Python
public classes.
