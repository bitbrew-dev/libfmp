# ADR 0002: Shared wire types and Python conversions

## Status

Accepted for the P1 shared-contract foundation.

## Context

`artifacts/documentations/outer.md` documents multiple wire representations for
the same conceptual value. Examples include fiscal years as integers and
strings, financial values as numbers and numeric strings, percentages as
numbers, numeric strings, and percent-suffixed strings, and several unrelated
date and boolean formats. Coercing these values into one permissive type would
lose provider information and make later endpoint models unreliable.

## Decision

The Rust core exposes narrow query enums and lossless response codecs. Similar
wire forms remain separate when their endpoint semantics differ: fiscal periods
are not retrieval frequencies, numeric quarters are not fiscal-period strings,
and RFC 3339 timestamps are not accepted by date or naive-datetime fields.
Endpoint-specific availability, add-on requirements, conditional plan
requirements, delay declarations, and bounds are attached as additive metadata.
Named add-on entitlement and conditional plan entitlement compose: for example,
TipRanks access can require the TipRanks add-on generally while only history
older than three years requires the Enterprise plan. There is no global limit,
page, or nonzero rule.

The Python binding will apply this conversion policy when these Rust contracts
reach public endpoint models:

- JSON integers remain Python `int`; values are never routed through `f64`.
- Fractional JSON numbers become Python `float`.
- Numeric strings and percent-suffixed strings remain Python `str` and are not
  scaled.
- A fiscal year becomes `int | str`, matching its wire representation.
- Native JSON booleans become Python `bool`; `Y`/`N`, `Yes`/`No`, lowercase
  `true`/`false`, and title-case `True`/`False` forms remain validated strings
  unless a field's public model deliberately exposes its typed flag enum.
- `YYYY-MM-DD` values become `datetime.date`; naive
  `YYYY-MM-DD HH:MM:SS` values become naive `datetime.datetime`; RFC 3339 values
  become timezone-aware `datetime.datetime`. Date-or-datetime fields preserve
  which of those two documented forms was received. Empty and null optional
  dates become `None`. Opaque human or partial date text remains `str`.
- Dynamic JSON recursively becomes native Python dictionaries, lists, strings,
  integers, floats, booleans, and `None`.
- Binary response bodies become Python `bytes`.
- Query enums may initially be accepted as validated Python strings while Rust
  retains the exact enum vocabulary.

The existing `BinaryBody` contract remains unchanged. The source does not
establish one unambiguous XLSX MIME type, so this foundation does not invent one
or add a header-aware binary wrapper.
