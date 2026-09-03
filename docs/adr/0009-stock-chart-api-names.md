# ADR 0009: Stock chart API names and shared contracts

## Status

Accepted. The Rust implementation is complete; the Python facade remains
deferred.

## Decision

The ten stock chart endpoints use the following Rust descriptor, client, query,
and response-row names. Each free descriptor and its `Client` method share the
listed Rust method name.

| FMP path | Rust method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `historical-price-eod/light` | `stock_chart_light` | `StockChartEodQuery` | `StockChartLightBar` | `FmpClient.stock_chart_light`; `fmp.chart.StockChartLightBar` |
| `historical-price-eod/full` | `stock_chart_full` | `StockChartEodQuery` | `StockChartFullBar` | `FmpClient.stock_chart_full`; `fmp.chart.StockChartFullBar` |
| `historical-price-eod/non-split-adjusted` | `stock_chart_non_split_adjusted` | `StockChartEodQuery` | `StockChartAdjustedBar` | `FmpClient.stock_chart_non_split_adjusted`; `fmp.chart.StockChartAdjustedBar` |
| `historical-price-eod/dividend-adjusted` | `stock_chart_dividend_adjusted` | `StockChartEodQuery` | `StockChartAdjustedBar` | `FmpClient.stock_chart_dividend_adjusted`; `fmp.chart.StockChartAdjustedBar` |
| `historical-chart/1min` | `stock_chart_one_minute` | `StockChartIntradayQuery` | `StockChartIntradayBar` | `FmpClient.stock_chart_one_minute`; `fmp.chart.StockChartIntradayBar` |
| `historical-chart/5min` | `stock_chart_five_minutes` | `StockChartIntradayQuery` | `StockChartIntradayBar` | `FmpClient.stock_chart_five_minutes`; `fmp.chart.StockChartIntradayBar` |
| `historical-chart/15min` | `stock_chart_fifteen_minutes` | `StockChartIntradayQuery` | `StockChartIntradayBar` | `FmpClient.stock_chart_fifteen_minutes`; `fmp.chart.StockChartIntradayBar` |
| `historical-chart/30min` | `stock_chart_thirty_minutes` | `StockChartIntradayQuery` | `StockChartIntradayBar` | `FmpClient.stock_chart_thirty_minutes`; `fmp.chart.StockChartIntradayBar` |
| `historical-chart/1hour` | `stock_chart_one_hour` | `StockChartIntradayQuery` | `StockChartIntradayBar` | `FmpClient.stock_chart_one_hour`; `fmp.chart.StockChartIntradayBar` |
| `historical-chart/4hour` | `stock_chart_four_hours` | `StockChartIntradayQuery` | `StockChartIntradayBar` | `FmpClient.stock_chart_four_hours`; `fmp.chart.StockChartIntradayBar` |

The four end-of-day endpoints share `StockChartEodQuery`. Its optional `from`
and `to` dates are independent because the provider documents each separately;
the SDK does not require both or synthesize a `DateRange` constraint.

The six intraday endpoints share `StockChartIntradayQuery`. In addition to the
independent dates, `nonadjusted` and `extended` are `Option<bool>` values. An
omitted flag is absent from the request, while explicit `false` remains present.
The interval is fixed by each endpoint path rather than exposed through the
query, preventing path/query contradictions.

The non-split-adjusted and dividend-adjusted endpoints share
`StockChartAdjustedBar` because both documented responses have the same exact
seven fields, including the `adjOpen`, `adjHigh`, `adjLow`, and `adjClose`
spellings. All six intraday endpoints share `StockChartIntradayBar`. Its
timezone-less `date` uses `ApiDateTime`, and the model deliberately has no
symbol or timezone field because neither appears in the documented rows.

Unknown response fields remain forward-compatible. Documented fields remain
required and are not widened to nullable values. Python runtime parity remains
deferred; the future facade will accept ordinary method arguments and will not
expose the Rust query structs as Python public classes.
