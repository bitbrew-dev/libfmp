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

The Python binding applies this conversion policy to public endpoint models
(the mapping lives in `crates/fmp-py-gen/src/bin/gen_models/classify.rs` and
`crates/fmp-py/src/convert.rs`):

- JSON integers remain Python `int`; values are never routed through `f64`.
  Integer literals beyond `u64` stay exact through the dynamic path.
- Fractional JSON numbers become Python `float`.
- Numeric strings and percent-suffixed strings remain Python `str` and are not
  scaled.
- A fiscal year matches its wire representation per field: `FiscalYearString`
  fields become `str` and `CalendarYear` fields become `int`. The
  `FiscalYear` union is not used by any public model, and the generator has no
  mapping for it; a model that needs it would also need generator work.
- Native JSON booleans become Python `bool`; `Y`/`N`, `Yes`/`No`, lowercase
  `true`/`false`, and title-case `True`/`False` forms remain validated strings
  unless a field's public model deliberately exposes its typed flag enum.
- `YYYY-MM-DD` values become `datetime.date`; naive
  `YYYY-MM-DD HH:MM:SS` values become naive `datetime.datetime`; RFC 3339
  `IsoTimestamp` values remain `str` holding the exact wire text, so the
  offset spelling and millisecond precision are preserved rather than
  normalised into a timezone-aware `datetime.datetime`. Date-or-datetime
  fields remain `str`, preserving which of the two documented forms was
  received. Empty and null optional dates become `None`. Opaque human or
  partial date text remains `str`.
- Dynamic JSON recursively becomes native Python dictionaries, lists, strings,
  integers, floats, booleans, and `None`.
- Binary response bytes become Python `bytes` on `fmp.BinaryPayload`, which
  carries the validated `content_type` and optional `content_disposition`
  alongside `data`.
- Query enums are accepted as validated Python strings while Rust retains the
  exact enum vocabulary. Matching is case-insensitive and `quarterly` is an
  accepted alias, but the encoded wire value is always the documented spelling.

Amendment (2026-09-21, audit of issue #11): the timestamp, fiscal-year, binary,
and query-enum bullets above were revised to describe the shipped binding. The
original text promised timezone-aware datetimes for RFC 3339 values and an
`int | str` fiscal-year union; neither shipped, and the domain ADRs (0011,
0018, 0029) together with the binding tests assert the representation-preserving
`str` behaviour instead.

Dynamic object-root payloads use `DynamicObject`. It preserves arbitrary member
names and recursively lossless JSON data, including arbitrary-precision integer
tokens, but does not treat semantically insignificant object-member order as
data. Duplicate names follow the JSON map model. Consequently, this foundation
does not enable an order-preservation dependency.

Binary downloads use the header-aware `BinaryResponse` value. `BinaryBody`
remains a source-compatible alias. Both names provide `as_bytes()` and
`into_bytes()`; `BinaryResponse` additionally provides the exact validated
`content_type()` and an optional textual `content_disposition()`. Its `Debug`
implementation includes only the byte length, validated base media type, and
whether a disposition is present. It never prints body bytes, disposition
contents, or content-type parameters.

Each endpoint owns its accepted MIME allow-list because the provider source does
not establish one global binary MIME policy. The client validates `Content-Type`
against that endpoint contract before constructing `BinaryResponse`; optional
`Content-Disposition` is retained only when it is valid header text.

`PartialEq` and `Eq` compare bytes, exact content type, and optional content
disposition. Metadata is part of binary response identity rather than incidental
transport state. The compatibility alias preserves source compatibility, while
the equality semantics intentionally reflect the enriched response value.
