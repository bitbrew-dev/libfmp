# ADR 0033: Nullable response members and empty-string sentinels

- Status: Accepted
- Date: 2026-09-30
- Decision owners: libfmp maintainers
- Scope: response models in Rust, Python, and Go; implemented by #368
  (1.2.0) and its follow-up fix issues

## Context

Response members were typed from the `outer.md` examples, which rarely show
a null. The 2026-09-30 live audit on
[#368](https://github.com/bitbrew-dev/libfmp/issues/368) (every registry path, 4012
requests) found 194 rows that fail to decode today: a member the examples
show as a value arrives as `null`, or a typed code such as a country code,
ISIN, or CUSIP arrives as `""`, which the non-empty string newtypes of
ADR 0001 reject. One such member fails the whole response.

ADR 0001 allowed sentinels to become `None` only where `outer.md`
establishes it, and ADR 0030 lists the field codecs Go mirrors. Observed wire
evidence is now a second source, so both need a rule.

## Decision

| Wire value observed on a member | Rust | Python | Go |
| --- | --- | --- | --- |
| `null` | `Option<T>`, key still required through the file's `required_option` | `Optional[T]` | `*T` |
| `""` on a typed code (`CountryCode`, `Isin`, `Ticker`, ...) | `Option<T>` through `codecs::empty_or_null` | `None` | `nil` |
| `""` on a date | `Option<Date>` through `codecs::empty_or_null_date` | `None` | `nil` |
| `""` on a plain `String` | unchanged: stays `""` | `""` | `""` |
| `"NULL"` text on a `NumericString` (`FundDisclosureSearchResult.entityOrgType`, #380) | `Option<NumericString>` through `codecs::null_text` | `None` | `nil` |
| key absent from the object | plain `Option<T>` with `skip_serializing_if = "Option::is_none"` (no `required_option`) | `None` | `*T` with `omitzero` |

- Evidence decides: a member changes only when the audit (or a later
  report) saw it null or empty. Parity siblings are not widened.
- `codecs::empty_or_null` accepts `""` and null as `None` and parses any
  other string with the type's validation, so a whitespace-only or
  control-bearing value is still an error; its message never includes the
  value. `None` re-encodes as `null`.
- No invented values: a null amount never becomes `0` or `NaN`, and a row
  is never dropped to hide a bad member.
- `codecs::null_text` accepts the exact text `"NULL"` and null as `None`
  and parses any other string as a `NumericString`, so `""`, `"null"`, or a
  padded `" NULL"` is still an error that never includes the value. It is
  used only where the evidence shows `"NULL"` standing in for an absent
  value (the live rows that send it on `entityOrgType` also send a null
  `address`); no other codec treats `"NULL"` as absent.
- A member is optional-when-absent only when the audit saw its key
  missing, not merely null; every other member keeps a required key.
  First case, [#377](https://github.com/bitbrew-dev/libfmp/issues/377):
  `CongressionalMemberNetWorthAggregate` omits `assetBackedSecurities`,
  `businessAndSelfEmployment`, `businessLiabilities`, `options`,
  `ownershipInterest`, `realEstateLiabilities` and
  `revolvingAndCreditLines` on all 11 audited `senate-net-worth-aggregated`
  rows, and a 14-member live probe (131 rows) also saw `realEstate` and
  `stock` absent; only `senateID`, `year`, `total`,
  `cashAndCashEquivalents` and `mutualFundsAndETFs` were always sent.
  `CongressionalDebtDetails` omits `dateIncurred` on 30 audited
  `senate-net-worth` rows. The same audit saw fractional amounts
  (`32500.5`) on members typed as integers; those became `MarketValue`
  (`f64`) under the next rule.
- A member whose non-null value has the wrong type (a year where a date is
  documented) is a separate type fix, not an `Option`.
- Every change is breaking and ships in a minor release with a
  `BREAKING CHANGE:` footer naming the members.

### Go

`gen_go` maps `deserialize_with = "crate::codecs::empty_or_null::deserialize"`
on an `Option` of a string-backed type to `Codec::EmptyOrNullString`: the
shadow member keeps the raw `jsontext.Value`, the key is required, and
`""` or null decode to `nil`. `crate::codecs::null_text::deserialize` maps
the same way to `Codec::NullTextString`, where the text `"NULL"` or null
decode to `nil`. It fails on any other shape rather than guessing.
A plain `Option<T>` without a codec is `Codec::Plain`: the key
may be absent or null, and `skip_serializing_if = "Option::is_none"` adds
`omitzero` so a re-encoded row omits it as serde does.

## Alternatives considered

- **Reject `""` on typed codes.** Rejected: the audit saw it on live rows
  (ETF holdings, fund disclosures, insider Form 3 filings), so every such
  response would fail.
- **Keep `""` as a value of the typed code.** Rejected: it breaks the
  non-empty guarantee of ADR 0001 and makes callers test two spellings of
  absent.
- **Default null amounts to `0.0` or `NaN`.** Rejected: that invents a
  financial figure.
- **Make every member optional.** Rejected: it hides the documented
  contract and pushes checks onto every caller for members never seen null.

## Consequences

- ADR 0001's sentinel rule and ADR 0030's codec list point here.
- More `Option` members across the three SDKs; callers handle `None`.
- New evidence of a null or `""` is a small, local model change under this
  rule, with a decode test proving it.
