# ADR 0026: ESG and Commitment of Traders API names

## Status

Accepted for issue #35's Rust API inventory. The three ESG routes are
implemented in Rust; the three Commitment of Traders routes and the Python
facade remain reserved.

## Decision

The six documented entries use or reserve the following names. Every response
is a bare JSON array.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | State | Future Python API |
| --- | --- | --- | --- | --- | --- |
| `esg-disclosures` | `esg_disclosures` | `EsgSymbolQuery` | `EsgDisclosure` | Implemented | `FmpClient.esg_disclosures`; `fmp.esg.EsgDisclosure` |
| `esg-ratings` | `esg_ratings` | `EsgSymbolQuery` | `EsgRating` | Implemented | `FmpClient.esg_ratings`; `fmp.esg.EsgRating` |
| `esg-benchmark` | `esg_benchmark` | `EsgBenchmarkQuery` | `EsgBenchmark` | Implemented | `FmpClient.esg_benchmark`; `fmp.esg.EsgBenchmark` |
| `commitment-of-traders-report` | `commitment_of_traders_report` | `CotQuery` | `CotReport` | Reserved | `FmpClient.commitment_of_traders_report`; `fmp.cot.CotReport` |
| `commitment-of-traders-analysis` | `commitment_of_traders_analysis` | `CotQuery` | `CotAnalysis` | Reserved | `FmpClient.commitment_of_traders_analysis`; `fmp.cot.CotAnalysis` |
| `commitment-of-traders-list` | `commitment_of_traders_list` | unit | `CotReportListing` | Reserved | `FmpClient.commitment_of_traders_list`; `fmp.cot.CotReportListing` |

The ESG contracts live in `endpoints::esg` and `responses::esg`. Disclosures
and ratings share one required, validated `Ticker` query. Benchmarks expose the
provider's independently optional string `year` through the existing
representation-preserving `BenchmarkYear`; no default is injected.

The provider spells the response keys `ESGScore` and `ESGRiskRating` with an
uppercase acronym. Public Rust fields follow snake case as `esg_score` and
`esg_risk_rating`, with explicit Serde renames so serialization preserves the
provider contract exactly. All documented ESG response fields are required and
non-null. The three routes carry only their documented US-only geography; no
pagination, access, realtime, or other bounds are inferred.

The future Commitment of Traders slice is reserved under the concise `cot`
module while keeping fully expanded client method names. `CotQuery` will retain
the independently optional provider order `symbol`, `from`, `to`. Its analysis
route will carry the documented 90-calendar-day maximum, without applying that
bound to the report or list routes.

Python runtime parity is deferred. Future Python methods and response-module
placement are reserved in the table and intentionally mirror the Rust API.
