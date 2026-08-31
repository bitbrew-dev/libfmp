# ADR 0017: Technical indicators API names and contract boundaries

## Status

Accepted for issue #26's Rust core implementation and reserved future Python
facade.

## Decision

The nine Technical Indicators endpoints use the following final Rust
descriptor, client, shared-query, and response-row names and reserve the listed
future Python names.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `technical-indicators/sma` | `simple_moving_average` | `TechnicalIndicatorQuery` | `SimpleMovingAverageBar` | `FmpClient.simple_moving_average`; `fmp.technical_indicators.SimpleMovingAverageBar` |
| `technical-indicators/ema` | `exponential_moving_average` | `TechnicalIndicatorQuery` | `ExponentialMovingAverageBar` | `FmpClient.exponential_moving_average`; `fmp.technical_indicators.ExponentialMovingAverageBar` |
| `technical-indicators/wma` | `weighted_moving_average` | `TechnicalIndicatorQuery` | `WeightedMovingAverageBar` | `FmpClient.weighted_moving_average`; `fmp.technical_indicators.WeightedMovingAverageBar` |
| `technical-indicators/dema` | `double_exponential_moving_average` | `TechnicalIndicatorQuery` | `DoubleExponentialMovingAverageBar` | `FmpClient.double_exponential_moving_average`; `fmp.technical_indicators.DoubleExponentialMovingAverageBar` |
| `technical-indicators/tema` | `triple_exponential_moving_average` | `TechnicalIndicatorQuery` | `TripleExponentialMovingAverageBar` | `FmpClient.triple_exponential_moving_average`; `fmp.technical_indicators.TripleExponentialMovingAverageBar` |
| `technical-indicators/rsi` | `relative_strength_index` | `TechnicalIndicatorQuery` | `RelativeStrengthIndexBar` | `FmpClient.relative_strength_index`; `fmp.technical_indicators.RelativeStrengthIndexBar` |
| `technical-indicators/standarddeviation` | `standard_deviation` | `TechnicalIndicatorQuery` | `StandardDeviationBar` | `FmpClient.standard_deviation`; `fmp.technical_indicators.StandardDeviationBar` |
| `technical-indicators/williams` | `williams` | `TechnicalIndicatorQuery` | `WilliamsBar` | `FmpClient.williams`; `fmp.technical_indicators.WilliamsBar` |
| `technical-indicators/adx` | `average_directional_index` | `TechnicalIndicatorQuery` | `AverageDirectionalIndexBar` | `FmpClient.average_directional_index`; `fmp.technical_indicators.AverageDirectionalIndexBar` |

All nine endpoints are worldwide `GET` requests returning bare arrays.
Documented fields are required and non-null, while unknown fields remain
accepted for forward compatibility. Each response row stays a distinct flat
public struct even though the six OHLCV fields are shared; callers therefore
do not have to navigate a provider-invented nested bar.

The endpoints share one query with exact wire order `symbol`, `periodLength`,
`timeframe`, optional `from`, then optional `to`. Symbols reuse `Ticker`.
Period lengths reuse the strictly positive `PeriodLength` while preserving the
full positive `u32` domain. Timeframes reuse `ChartTimeframe`, whose seven exact
wire values are `1min`, `5min`, `15min`, `30min`, `1hour`, `4hour`, and `1day`.
The two dates are independent: the SDK does not require a pair, reorder a
reversed interval, impose a span, or invent an undocumented cap.

Response timestamps reuse the strict timezone-less `ApiDateTime`. OHLC values
and the SMA, EMA, WMA, DEMA, TEMA, and standard-deviation metrics use `Price`.
Volume uses the full `Volume` (`u64`) domain. RSI, Williams, and ADX remain raw
`f64` values, preserving negative Williams readings and accepting integer or
fractional JSON numbers. The standard-deviation row preserves the exact
`standardDeviation` provider key.

Python runtime bindings are deferred. The future facade reserves ordinary
`FmpClient` methods and the nine response classes under
`fmp.technical_indicators`; the Rust query struct will not become a Python
public class.

This foundation defines the shared query and nine response rows. Descriptors
and client methods are intentionally implemented by later vertical slices.
