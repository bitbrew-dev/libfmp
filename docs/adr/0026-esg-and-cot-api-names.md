# ADR 0026: ESG and Commitment of Traders API names

## Status

Accepted for issue #35's Rust API inventory. All six ESG and Commitment of
Traders routes are implemented in Rust; the Python facade remains reserved.

## Decision

The six documented entries use or reserve the following names. Every response
is a bare JSON array.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | State | Future Python API |
| --- | --- | --- | --- | --- | --- |
| `esg-disclosures` | `esg_disclosures` | `EsgSymbolQuery` | `EsgDisclosure` | Implemented | `FmpClient.esg_disclosures`; `fmp.esg.EsgDisclosure` |
| `esg-ratings` | `esg_ratings` | `EsgSymbolQuery` | `EsgRating` | Implemented | `FmpClient.esg_ratings`; `fmp.esg.EsgRating` |
| `esg-benchmark` | `esg_benchmark` | `EsgBenchmarkQuery` | `EsgBenchmark` | Implemented | `FmpClient.esg_benchmark`; `fmp.esg.EsgBenchmark` |
| `commitment-of-traders-report` | `cot_report` | `CotQuery` | `CotReport` | Implemented | `FmpClient.cot_report`; `fmp.commitment_of_traders.CotReport` |
| `commitment-of-traders-analysis` | `cot_analysis` | `CotQuery` | `CotAnalysis` | Implemented | `FmpClient.cot_analysis`; `fmp.commitment_of_traders.CotAnalysis` |
| `commitment-of-traders-list` | `cot_report_list` | unit | `CotReportListing` | Implemented | `FmpClient.cot_report_list`; `fmp.commitment_of_traders.CotReportListing` |

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

The Commitment of Traders contracts live in
`endpoints::commitment_of_traders` and `responses::commitment_of_traders`, while
the descriptor and client methods use concise `cot_*` names. `CotQuery` retains
the independently optional provider order `symbol`, `from`, `to`. Its analysis
route carries the documented 90-calendar-day maximum as metadata without
rejecting construction; the report and list routes carry no inferred bounds.

Public Rust fields correct provider spelling and suffix inconsistencies while
Serde preserves the documented wire contract. In particular, `net_position`
maps to `netPostion`, `change_in_noncomm_spread_all` maps to
`changeInNoncommSpeadAll`, and `traders_noncomm_spread_old` maps to
`tradersNoncommSpeadOl`. The 10 old percentage, eight old trader-count, and
eight old concentration fields use semantic `_old` Rust names and explicit
provider `Ol` wire names. The 54 percentage and concentration fields use
`serde_json::Number` so integer and decimal JSON spellings round-trip exactly.
All documented COT response fields are required and non-null.

Python runtime parity is deferred. Future Python methods and response-module
placement are reserved in the table and intentionally mirror the Rust API.
