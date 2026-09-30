# ADR 0034: Date members that can arrive as a bare year

- Status: Accepted
- Date: 2026-09-30
- Decision owners: libfmp maintainers
- Scope: `IpoProspectus.ipo_date` in Rust, Python, and Go; implemented by
  [#379](https://github.com/bitbrew-dev/libfmp/issues/379) (1.2.0)

## Context

`IpoProspectus.ipoDate` is documented as a `YYYY-MM-DD` date. The #368 live
audit decoded 239 `ipos-prospectus` rows and found 7 that carry a bare year
(`"2020"`, `"2008"`, seen over `from=2020-03-01&to=2020-03-31`). The member was
never `null`. ADR 0033 leaves "a year where a date is documented" to a
separate type fix, and one such row fails the whole response today.

## Decision

A member seen as either a full date or a bare year uses `codecs::DateOrYear`,
built like `DateOrDateTime`: the exact wire length picks the form, and each
form is validated strictly.

| Wire value | Rust | Python | Go |
| --- | --- | --- | --- |
| `"2026-07-28"` | `DateOrYear::Date(Date)` | `datetime.date` | `fmp.DateOrYear`, `.Date()` reports it |
| `"2020"` | `DateOrYear::Year(2020)` (`u16`) | `int` | `fmp.DateOrYear`, `.Year()` reports it |
| anything else | decode error | `FmpDecodeError` | `*InvalidTemporalValueError` |

- A year stays a year. `"2020"` is never widened to `2020-01-01`, which would
  invent a month and a day.
- Only exactly four ASCII digits are a year; a 10-byte value must be a real
  `YYYY-MM-DD` date. Errors name the expected shapes and never include the
  value. Both forms re-encode as the exact wire string.
- The Python member is `datetime.date | int`, so full dates keep the
  `datetime.date` type they had; the hand-written `models::convert::DateOrYear`
  converts both ways. Go gets its own validating type instead of the
  `string` used for `DateOrDateTime`, so a malformed value still fails.
- Evidence decides, as in ADR 0033: only `IpoProspectus.ipo_date` changes.
  The other `ipoDate` members (company profiles, search, SEC filings) were
  never seen as a bare year and stay `Date`.
- The change is breaking and ships in a minor release with a
  `BREAKING CHANGE:` footer.

## Alternatives considered

- **Plain `String`.** Rejected: full dates would lose their type in every
  language, and garbage would decode silently in Go.
- **Coerce the year to January 1.** Rejected: it invents information.
- **`Option<Date>` with the year dropped to `None`.** Rejected: it discards
  the year the provider did send.
