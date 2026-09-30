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

- Evidence decides: a member changes only when the audit (or a later
  report) saw it null or empty. Parity siblings are not widened.
- `codecs::empty_or_null` accepts `""` and null as `None` and parses any
  other string with the type's validation, so a whitespace-only or
  control-bearing value is still an error; its message never includes the
  value. `None` re-encodes as `null`.
- No invented values: a null amount never becomes `0` or `NaN`, and a row
  is never dropped to hide a bad member.
- A member whose non-null value has the wrong type (a year where a date is
  documented, the text `"NULL"`) is a separate type fix, not an `Option`.
- Every change is breaking and ships in a minor release with a
  `BREAKING CHANGE:` footer naming the members.

### Go

`gen_go` maps `deserialize_with = "crate::codecs::empty_or_null::deserialize"`
on an `Option` of a string-backed type to `Codec::EmptyOrNullString`: the
shadow member keeps the raw `jsontext.Value`, the key is required, and
`""` or null decode to `nil`. It fails on any other shape rather than
guessing.

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
