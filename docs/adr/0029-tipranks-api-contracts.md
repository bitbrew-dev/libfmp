# ADR 0029: TipRanks API contracts

## Status

Accepted for issue #38's Rust API inventory. The ratings search is implemented
in Rust; the other six routes and the Python facade remain reserved.

## Decision

All seven documented TipRanks entries use or reserve the following public Rust
names. Every response is a bare JSON array. Future Python methods mirror these
snake-case Rust names under `FmpClient`, with models under `fmp.tipranks`.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | State |
| --- | --- | --- | --- | --- |
| `tipranks-search` | `tipranks_ratings_search` | `TipRanksSearchQuery` | `TipRanksRatingSearchResult` | Rust complete |
| `tipranks-pit-symbol` | `tipranks_ratings_by_symbol` | `TipRanksSymbolQuery` | `TipRanksPointInTimeRating` | Reserved |
| `tipranks-pit-analyst` | `tipranks_ratings_by_analyst` | `TipRanksAnalystQuery` | `TipRanksPointInTimeRating` | Reserved |
| `tipranks-symbol-summary` | `tipranks_symbol_summary` | `TipRanksSymbolSummaryQuery` | `TipRanksRatingsSummary` | Reserved |
| `tipranks-analyst-summary` | `tipranks_analyst_summary` | `TipRanksAnalystSummaryQuery` | `TipRanksRatingsSummary` | Reserved |
| `tipranks-firm-summary` | `tipranks_firm_summary` | `TipRanksFirmSummaryQuery` | `TipRanksRatingsSummary` | Reserved |
| `tipranks-analysts` | `tipranks_analysts` | `TipRanksAnalystsQuery` | `TipRanksAnalyst` | Reserved |

The endpoint contracts live in `endpoints::tipranks`, and response contracts
live in `responses::tipranks`. `TipRanksSearchQuery` exposes all seven optional
filters in the provider's documented wire order: `expertUID`, `symbol`, `from`,
`to`, `limit`, `page`, and `nonadjusted`. It injects no defaults. The date
filters are independent, page zero remains representable, and the optional
boolean distinguishes omission from explicit false. `expertUID` is backed by
the open, nonempty, control-safe, representation-preserving
`TipRanksExpertUid` type; the acronym's exact wire spelling is mapped
explicitly. Symbols, dates, pages, and limits use the existing narrow shared
types.

The search endpoint requires the named `TipRanks` add-on. Geography is not
specified, and no realtime behavior is documented. The endpoint records both
the documented maximum limit and maximum response size of 5,000. Bounds remain
endpoint metadata and do not create local request rejection. The source says
ratings history goes back three years and that older historical backfill
requires an Enterprise plan. This is represented as a conditional plan fact,
not as hard date validation, because the request dates remain independently
optional and availability is a provider capability.

The exact search response is a required, non-null 13-field row. Its event
`date` is a strict representation-preserving RFC 3339 `IsoTimestamp`, while
`recommendationDate` is a strict calendar `Date`. Price target uses
`serde_json::Number` so the strict bare JSON number preserves integer versus
fractional representation without coercion through `f64`; currency is an open
validated code, and recommendation, action, article, site, and URL values
remain open strings. The exact `expertUID` key is preserved on serialization.

The point-in-time analyst example contains nullable price-target, currency,
return, and beat-target values, so later work must not derive that response's
nullability from the non-null search row. The directory prose says to provide
an exact `analystName`, but its parameter table instead lists `page`, `limit`,
and `firmName`. That conflict remains unresolved and must not be silently
invented into the reserved query contract.

Python runtime parity remains deferred and reserved under `fmp.tipranks`.
