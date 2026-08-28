# ADR 0012: Earnings-transcript API names and source conflicts

## Status

Accepted for issue #21's Rust contract implementation and reserved future
Python facade.

## Decision

The earnings-transcript endpoints reserve the following Rust descriptor,
client, query, response-row, and future Python names. The first three
descriptors and client methods are deferred; this issue implements their query
and response contracts only.

| FMP path | Rust method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `earning-call-transcript-latest` | `latest_earnings_transcripts` | `LatestEarningsTranscriptsQuery` | `LatestEarningsTranscript` | `FmpClient.latest_earnings_transcripts`; `fmp.transcripts.LatestEarningsTranscript` |
| `earning-call-transcript` | `earnings_transcript` | `EarningsTranscriptQuery` | `EarningsTranscript` | `FmpClient.earnings_transcript`; `fmp.transcripts.EarningsTranscript` |
| `earning-call-transcript-dates` | `earnings_transcript_dates` | `EarningsTranscriptDatesQuery` | `EarningsTranscriptDate` | `FmpClient.earnings_transcript_dates`; `fmp.transcripts.EarningsTranscriptDate` |
| `earnings-transcript-list` | existing `earnings_transcript_list` | none | existing `EarningsTranscriptAvailability` | existing `FmpClient.earnings_transcript_list`; `fmp.directory.EarningsTranscriptAvailability` |

The transcript modules re-export the existing directory descriptor and row for
`earnings-transcript-list`; they do not duplicate its descriptor, client
method, model, or fixture. The original directory documentation and shipped
metadata classify this endpoint as US-only, while the later transcript section
calls the same path worldwide. This conflict does not justify silently changing
an existing public contract, so the re-export preserves the shipped US-only
metadata.

The full-transcript parameter table calls `year` and `quarter` strings. Its
example uses the textual URL values `2020` and `3`, so the query reuses `Year`
and the closed textual `Quarter` vocabulary. Transcript response years are
numeric JSON values and therefore use `CalendarYear`. The dates response uses
the new validated numeric `CalendarQuarter`, accepting only JSON integers one
through four. This keeps query text distinct from response numbers.

The latest endpoint prose describes a transcript-availability list with counts,
but its documented response contains only symbol, period, numeric fiscal year,
and date. `LatestEarningsTranscript` follows the exact response rather than the
mismatched prose. Complete transcript `content` is an exact, unbounded `String`;
the SDK does not truncate or impose an undocumented length limit.

The latest endpoint documents at most 100 responses and a maximum page number
of 100. Those facts are reserved for its future descriptor metadata, not
constructor rejection: `Limit` and `Page` remain endpoint-neutral query units.
The full-transcript endpoint documents an optional limit without a maximum, and
the transcript-dates endpoint documents no bound. No limit or pagination rules
are invented for either endpoint.

All three new response rows are bare arrays with required non-null documented
fields and unknown-field tolerance. Python runtime parity remains deferred; the
future facade will accept ordinary method arguments and will not expose Rust
query structs as Python public classes.
