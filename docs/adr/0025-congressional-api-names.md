# ADR 0025: Congressional API names

## Status

Accepted for issue #34's Rust API inventory. All 12 congressional routes are
implemented in Rust; the Python facade remains reserved.

## Decision

The 12 documented entries in the Senate section use or reserve the following
names. All are US-only GET endpoints with bare-array responses.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | State | Future Python API |
| --- | --- | --- | --- | --- | --- |
| `senate-latest` | `latest_senate_disclosures` | `LatestCongressionalDisclosuresQuery` | `CongressionalTrade` | Implemented | `FmpClient.latest_senate_disclosures`; `fmp.congressional.CongressionalTrade` |
| `house-latest` | `latest_house_disclosures` | `LatestCongressionalDisclosuresQuery` | `CongressionalTrade` | Implemented | `FmpClient.latest_house_disclosures`; `fmp.congressional.CongressionalTrade` |
| `senate-trades` | `senate_trades` | `CongressionalTradesQuery` | `CongressionalTrade` | Implemented | `FmpClient.senate_trades`; `fmp.congressional.CongressionalTrade` |
| `senate-trades-by-name` | `senate_trades_by_name` | `CongressionalTradesByNameQuery` | `CongressionalTrade` | Implemented | `FmpClient.senate_trades_by_name`; `fmp.congressional.CongressionalTrade` |
| `senate-trades-by-id` | `senate_trades_by_member_id` | `CongressionalTradesByMemberIdQuery` | `CongressionalTrade` | Implemented | `FmpClient.senate_trades_by_member_id`; `fmp.congressional.CongressionalTrade` |
| `house-trades` | `house_trades` | `CongressionalTradesQuery` | `CongressionalTrade` | Implemented | `FmpClient.house_trades`; `fmp.congressional.CongressionalTrade` |
| `house-trades-by-name` | `house_trades_by_name` | `CongressionalTradesByNameQuery` | `CongressionalTrade` | Implemented | `FmpClient.house_trades_by_name`; `fmp.congressional.CongressionalTrade` |
| `house-trades-by-id` | `house_trades_by_member_id` | `CongressionalTradesByMemberIdQuery` | `CongressionalTrade` | Implemented | `FmpClient.house_trades_by_member_id`; `fmp.congressional.CongressionalTrade` |
| `senate-profile` | `congressional_profiles` | `CongressionalProfilesQuery` | `CongressionalMemberProfile` | Implemented | `FmpClient.congressional_profiles`; `fmp.congressional.CongressionalMemberProfile` |
| `senate-positions` | `congressional_positions` | `CongressionalPositionsQuery` | `CongressionalMemberPosition` | Implemented | `FmpClient.congressional_positions`; `fmp.congressional.CongressionalMemberPosition` |
| `senate-net-worth` | `congressional_net_worth` | `CongressionalNetWorthQuery` | `CongressionalMemberNetWorthEntry` | Implemented | `FmpClient.congressional_net_worth`; `fmp.congressional.CongressionalMemberNetWorthEntry` |
| `senate-net-worth-aggregated` | `congressional_net_worth_aggregated` | `CongressionalNetWorthAggregatedQuery` | `CongressionalMemberNetWorthAggregate` | Implemented | `FmpClient.congressional_net_worth_aggregated`; `fmp.congressional.CongressionalMemberNetWorthAggregate` |

The provider uses the wire key `senateID` for both Senate and House members.
The public fundamental is therefore the chamber-neutral
`CongressionalMemberId`, and response/query fields are named `member_id` with
explicit `senateID` wire renames. This avoids embedding a provider naming leak
in callers while preserving the request and response contracts exactly.

The implemented endpoints and row live in `endpoints::congressional` and
`responses::congressional`. The two latest routes share one independently
optional page/limit query. The two symbol routes share a required `Ticker`
followed by optional page/limit. The two name routes use a required,
representation-preserving `SearchTerm`. The ID routes preserve the documented
optional order `page`, `limit`, `senateID`.

All eight routes share `CongressionalTrade`. Its response `symbol` is a
required `String`, rather than `Ticker`, because the documented
`senate-trades-by-name` row contains an empty string. The provider omits
`capitalGainsOver200USD` from the latest Senate response and sends the exact
title-case string `"False"` in the other documented rows, so that field is an
optional `TitleCaseBoolFlag` and is omitted again when serialized as `None`.
All other documented fields remain required.

Only routes whose documentation gives both caps carry the 250-response and
page-100 metadata. The two name searches have no inferred bounds. No access,
realtime, or other metadata is inferred.

The profile and position routes use independently optional filters and preserve
their documented wire order. Open provider party and position values remain
raw strings. The profile endpoint carries only its documented 500-row and
page-20 bounds; the position endpoint carries only its documented 300-row and
page-50 bounds. Profile fields are required exactly as shown. Position
`endDate` alone is nullable in the documented row.

The itemized net-worth endpoint requires a member ID. Its parameter table omits
pagination, but its documented endpoint URL includes `page=0&limit=250` and the
notes specify both a 250-row maximum and page 100 maximum. The Rust query
therefore exposes optional page and limit values after the required `senateID`
and attaches those two documented bounds; it does not inject the example
values as defaults. The aggregated query requires `senateID` and preserves the
optional raw `totalsCol` string, including the explicitly documented empty
value. No bounds are inferred for that route.

The itemized example is House-shaped despite its Senate-prefixed path: it uses
`"House Report"` and a House Clerk disclosure link. The neutral public model
names and the existing `CongressionalMemberId` deliberately reflect this
cross-chamber provider behavior. Nullable fields remain key-required and
accept JSON null. Opaque debt dates such as `"September 2007"` are preserved
without date parsing, and signed integers keep all monetary values exact while
allowing negative net worth.

Python runtime parity is deferred. Future Python methods and response-module
placement are reserved in the table and intentionally mirror the Rust API.
