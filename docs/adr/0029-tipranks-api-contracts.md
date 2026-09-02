# ADR 0029: TipRanks API contracts

## Status

Accepted for issue #38's Rust API inventory. The ratings search, both
point-in-time routes, and all three summary routes are implemented in Rust; the
analyst directory route and the Python facade remain reserved.

## Decision

All seven documented TipRanks entries use or reserve the following public Rust
names. Every response is a bare JSON array. Future Python methods mirror these
snake-case Rust names under `FmpClient`, with models under `fmp.tipranks`.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | State |
| --- | --- | --- | --- | --- |
| `tipranks-search` | `tipranks_ratings_search` | `TipRanksSearchQuery` | `TipRanksRatingSearchResult` | Rust complete |
| `tipranks-pit-symbol` | `tipranks_point_in_time_ratings_by_symbol` | `PointInTimeRatingsBySymbolQuery` | `TipRanksPointInTimeRating` | Rust complete |
| `tipranks-pit-analyst` | `tipranks_point_in_time_ratings_by_analyst` | `PointInTimeRatingsByAnalystQuery` | `TipRanksPointInTimeRating` | Rust complete |
| `tipranks-symbol-summary` | `tipranks_symbol_summary` | `TipRanksSymbolSummaryQuery` | `TipRanksSymbolSummary` | Rust complete |
| `tipranks-analyst-summary` | `tipranks_analyst_summary` | `TipRanksAnalystSummaryQuery` | `TipRanksAnalystSummary` | Rust complete |
| `tipranks-firm-summary` | `tipranks_firm_summary` | `TipRanksFirmSummaryQuery` | `TipRanksFirmSummary` | Rust complete |
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

The symbol point-in-time query requires `symbol`, then preserves the documented
optional `date`, `limit`, `page`, and `nonadjusted` wire order. The analyst
point-in-time prose says to supply `expertUID`, while its parameter table marks
both `expertUID` and `analystName` without a required label. Its query therefore
keeps both selectors optional, permits either or both, and adds no local XOR or
selector requirement. Both queries inject no limit default, preserve page zero
and explicit false, and leave the provider's stated default of 100 to the
server. Their access, conditional three-year history rule, and 5,000-row bounds
match the search endpoint; no client-side historical cutoff is imposed.

Both point-in-time routes share the exact documented 16-field response. The
documentation prose promises `analystRank` and `stockAvgReturn`, but neither is
present in either response example, so neither is invented. Conversely,
`stockReturn` is preserved exactly without an alias. `stockSuccessRate` is a
strict bare JSON number. `priceTarget`, `priceTargetCurrency`, `stockReturn`,
and `beatTarget` are required-present but nullable, matching the analyst
example; missing keys still fail decoding. Number values use
`serde_json::Number` to preserve integer-versus-decimal spelling and reject
numeric strings. Recommendation text stays open and preserves source casing
such as `buy` and `Hold`.

Each summary query requires its documented identity (`symbol`, `expertUID`, or
the exact `firmName` text), followed by independently optional `from` and `to`
dates in wire order. No trailing-twelve-month default or date relationship is
enforced locally. Each route retains a bare array so an empty successful
response remains representable, while metadata records the documented
single-row maximum. The routes require the named `TipRanks` add-on; geography
is unspecified, and the source documents no realtime behavior, pagination,
query limit, date-span bound, conditional plan, or historical cutoff.

The three concrete summary response rows keep their distinct identity fields
required and expose the same 14-field aggregate payload. All fields and both
nested count objects are required and non-null. Count values use the shared
unsigned `Count`, while `averageReturn`, `topReturn`, and `worstReturn` use
`serde_json::Number` to reject numeric strings and preserve signed, fractional,
and integer JSON number tokens. The exact `expertUID` acronym and singular
`analystAction` key are mapped explicitly.

The directory prose says to provide an exact `analystName`, but its parameter
table instead lists `page`, `limit`, and `firmName`. That separate conflict
remains unresolved and must not be silently invented into the reserved query
contract.

Python runtime parity remains deferred and reserved under `fmp.tipranks`.
