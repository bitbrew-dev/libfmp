# ADR 0022: Market Hours API names and wire boundaries

## Status

Accepted for issue #31's Rust API. All three Market Hours routes are
implemented; the Python facade remains deferred.

## Decision

The three documented Market Hours endpoints use or reserve these names:

| FMP path | Rust descriptor/client method | Rust query | Rust response row | Future Python module |
| --- | --- | --- | --- | --- |
| `exchange-market-hours` | `exchange_market_hours` | `ExchangeMarketHoursQuery` | `ExchangeMarketHours` | `fmp.market_hours` |
| `holidays-by-exchange` | `holidays_by_exchange` | `HolidaysByExchangeQuery` | `ExchangeHoliday` | `fmp.market_hours` |
| `all-exchange-market-hours` | `all_exchange_market_hours` | `AllExchangeMarketHoursQuery` | `ExchangeMarketHours` | `fmp.market_hours` |

All three routes are worldwide `GET` requests returning bare arrays. No access,
response-row, date-range, or pagination bounds are documented, so none are
invented.

`ExchangeMarketHoursQuery` encodes its required `exchange` first and optional
`timestamp` second. `HolidaysByExchangeQuery` encodes required `exchange`, then
independently optional `from` and `to` dates. `AllExchangeMarketHoursQuery`
encodes only its optional `timestamp`. No undocumented defaults are supplied.

Although the timestamp example resembles Unix seconds, the provider documents
the parameter as a string. `MarketHoursTimestamp` is therefore a validated,
string-backed fundamental value that preserves the exact representation,
including leading zeroes. It is intentionally distinct from the numeric
`UnixSeconds` response type.

`ExchangeMarketHours` preserves `openingHour`, `closingHour`, and `timezone` as
raw strings. The API does not define a stricter grammar, and the crate does not
parse offsets, convert timezones, or add a timezone dependency.

The holiday response documents `adjOpenTime` and `adjCloseTime` as present but
null and gives no non-null shape. Both fields are required-but-nullable
`Option<DynamicJson>` values: missing keys fail decoding, documented nulls are
accepted, and future non-null JSON survives without loss. This avoids guessing
a string or object contract from null-only evidence.

Python runtime parity is deferred. The future facade reserves ordinary
`FmpClient.exchange_market_hours`, `FmpClient.holidays_by_exchange`, and
`FmpClient.all_exchange_market_hours` methods, with query and response names in
`fmp.market_hours` matching the Rust API where they make sense to expose.
