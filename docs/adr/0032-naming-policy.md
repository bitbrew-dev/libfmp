# ADR 0032: Naming policy across Rust, Python, and Go

- Status: Accepted
- Date: 2026-09-24
- Decision owners: libfmp maintainers
- Scope: method names, row-type names, and Go identifier casing; implemented
  by #343 (cross-SDK) and #344 (Go only)

## Context

Method and type names grew domain by domain (ADRs 0003 to 0029), so the same
idea is spelled several ways: `quote.batch_quote` repeats its namespace,
`mergers_acquisitions_latest` puts the adjective last, and Go mixes `Fmp`,
`Sp500`, and `Ttm` with `URL` and `HTTP`. 1.0 freezes names, so the policy is
set now.

This ADR is the rule. The renames land in #343 and #344; until they merge,
the code, examples, and guides keep the current names.

## Decision

### Where names come from

- Names are normalised at the source: the Rust endpoint functions and client
  methods plus the registry under `crates/fmp-py-gen/registry`.
- Python and Go are generated from that source, so all three SDKs keep one
  name per endpoint.
- Wire endpoint ids and JSON member names never change.

### Method names (all SDKs)

| Rule | Before | After |
| --- | --- | --- |
| Drop the domain noun when the namespace already says it | `quote.batch_quote` | `quote.batch` |
| | `funds.fund_disclosures` | `funds.disclosures` |
| Verb or adjective first | `mergers_acquisitions_latest` | `latest_mergers_acquisitions` |
| | `mergers_acquisitions_search` | `search_mergers_acquisitions` |
| `batch_` is always a prefix | `market_capitalization_batch` | `batch_market_capitalization` |

- Keep the noun where dropping it loses meaning; #343 lists every rename it
  makes and every noun it keeps.
- `LatestX` / `SearchX` in Go follow from the Rust names in exported casing.

### Row-type names (all SDKs)

- Row types are singular: one row is one value (`BulkStockPeers` becomes
  `BulkStockPeer`).
- One suffix convention; any `Record`, `Entry`, `Listing`, or `Result` suffix
  that stays is listed with its reason in #343.

### Go casing (Go only)

Initialisms are all caps everywhere, generated and hand-written, driven by
one initialism table in `gen_go`:

| Group | Initialisms |
| --- | --- |
| Web and data | `ID`, `URL`, `JSON`, `API`, `HTTP` |
| Provider and regulators | `FMP`, `SEC`, `US`, `COT` |
| Instruments and identifiers | `ETF`, `CIK`, `CUSIP`, `ISIN`, `IPO`, `SP500` |
| Metrics | `ESG`, `DCF`, `EPS`, `TTM` |
| Form names | `8K`, `13F` |

| Before | After |
| --- | --- |
| `FmpHeader`, `FmpQuery`, `FmpHeaderFromEnv` | `FMPHeader`, `FMPQuery`, `FMPHeaderFromEnv` |
| `FmpAPIKeyFromEnv` | `APIKeyFromEnv` |
| `Latest8k`, `Form13fFilingDate` | `Latest8K`, `Form13FFilingDate` |
| `IposCalendar`, `Sp500` | `IPOCalendar`, `SP500` |
| `FmpArticle`, `News.FmpArticles` | `Article`, `News.Articles` |
| namespaces `Tipranks`, `Dcf`, `Esg`, `SecFilings` | `TipRanks`, `DCF`, `ESG`, `SECFilings` |

- Namespace field names match their types.
- `EndpointMetadataFor` keys follow the new call paths.
- Struct tags (wire names) are unchanged.

## Alternatives considered

- **Go's generated casing without an initialism table (`Fmp`, `Ttm`).**
  Rejected by the maintainer (#344). Rationale (not stated by the
  maintainer): it reads unlike the standard library (`http.Client`,
  `url.URL`) and the Go code review guidance on initialisms.
- **Rename in each SDK separately.** Rejected: three spellings per endpoint
  is the drift the registry exists to prevent.
- **Keep the current names for 1.0.** Rejected: pre-1.0 is the only window
  where these renames are minor-version changes.

## Consequences

- Breaking for every SDK once #343 and #344 land; each ships with a
  `BREAKING CHANGE:` footer and a rename table.
- ADR 0030's client-surface table names the current Go auth constructors;
  this ADR supersedes those names once #344 lands.
