# ADR 0027: Fundraising API names

## Status

Accepted for issue #36's Rust API inventory. The two search routes are
implemented in Rust; four detailed routes and the Python facade remain reserved.

## Decision

All six documented fundraising entries use or reserve the following names.
Every response is a bare JSON array, and future Python methods mirror these
snake-case Rust names under `FmpClient`, with models under `fmp.fundraising`.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | State |
| --- | --- | --- | --- | --- |
| `crowdfunding-offerings-search` | `search_crowdfunding_offerings` | `OfferingSearchQuery` | `CrowdfundingOfferingSearchResult` | Implemented |
| `crowdfunding-offerings-latest` | `latest_crowdfunding_offerings` | `LatestCrowdfundingOfferingsQuery` | `CrowdfundingOffering` | Reserved |
| `crowdfunding-offerings` | `crowdfunding_offerings_by_cik` | `OfferingByCikQuery` | `CrowdfundingOffering` | Reserved |
| `fundraising-search` | `search_regulation_d_offerings` | `OfferingSearchQuery` | `RegulationDOfferingSearchResult` | Implemented |
| `fundraising-latest` | `latest_regulation_d_offerings` | `LatestRegulationDOfferingsQuery` | `RegulationDOffering` | Reserved |
| `fundraising` | `regulation_d_offerings_by_cik` | `OfferingByCikQuery` | `RegulationDOffering` | Reserved |

The endpoint and response contracts live in `endpoints::fundraising` and
`responses::fundraising`. Both search routes share `OfferingSearchQuery`: its
required validated `SearchTerm` is emitted as the sole wire key `name`, and no
default is injected. Both routes carry only the documented US-only geography;
no pagination, response bounds, plan access, or realtime behavior is inferred.

The compact response structs remain separate because their `date` contracts
differ. The crowdfunding search sample documents only JSON null. Its key is
required-present, but its possible non-null format is unproven, so Rust uses
`Option<DynamicJson>` and deliberately defers a narrower representation until
the source documentation supplies one. The Regulation D search sample proves
the exact `YYYY-MM-DD HH:MM:SS` form and therefore uses `ApiDateTime`. Both CIKs
use the representation-preserving `Cik` type so leading zeroes survive.

The four reserved detailed routes will complete issue #36's Rust surface in a
later stack branch. Their shared `OfferingByCikQuery` requires `Cik`; their
route-specific latest queries preserve only parameters documented for each
family. Python runtime parity remains deferred.
