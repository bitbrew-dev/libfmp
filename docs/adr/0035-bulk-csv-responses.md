# ADR 0035: Bulk CSV responses

- Status: Accepted
- Date: 2026-10-01
- Decision owners: libfmp maintainers
- Scope: the eighteen `*-bulk` routes in Rust, Python, and Go; implemented
  by [#376](https://github.com/bitbrew-dev/libfmp/issues/376). Supersedes
  ADR 0028's "bare JSON array" wire format, its "every documented key is
  required and non-null" rule for bulk rows, and its quoted ETF
  `lastUpdated"` key.

## Context

ADR 0028 typed the bulk routes from the `outer.md` examples, which show JSON
arrays. The 2026-09-30 live audit (#368 rerun) found that every bulk route
answers `200 text/csv` and ignores `Accept: application/json`, so every bulk
call failed to decode in all three SDKs. Live probes on 2026-09-30 and
2026-10-01 established the wire shape:

- A headed CSV body with LF line endings and no byte-order mark. Strings are
  quoted, numbers are bare (including exponent forms such as `6.9e-9`),
  quoted cells can carry commas and doubled quotes, and an absent value is an
  empty cell.
- A request with no data (for example `earnings-surprises-bulk?year=1900`)
  answers `200 text/csv` with an empty body. A bad parameter answers
  `400 application/json`, which the existing status path handles.
- The ETF header is a clean `lastUpdated`, not the documented `lastUpdated"`.
- Bodies are large: one `profile-bulk` part is about 30 MB, and the full
  `ratios-ttm-bulk` body measured 68,755,170 bytes (65.6 MiB), above the
  64 MiB general default.
- A census of over a million live rows (full bodies for most routes, income
  statements for two periods, samples for the other statement routes) found
  empty cells on many numeric columns, sparse and varying by period: for example `netIncomeDeductions` empty on 24 of 55,315 FY2024
  income rows and on none of 18,512 Q1 2025 rows; `grahamNumberTTM` empty on
  11,072 of 30,041 key-metrics rows; `dcf` on 1,653 of 35,029 rows. Typed
  codes were also empty: `reportedCurrency` (scores, 313 of 62,655), ETF
  `asset` and `isin`, and profile `isin`, `country` (with `change` and
  `changePercentage` on two rows).

## Decision

### Contract

- `EndpointSpec::get_csv` and `ResponseContract::<Vec<Row>>::csv()` describe
  a CSV route; `ExpectedContentType::Csv` accepts `text/csv`
  case-insensitively, parameters ignored. All eighteen bulk descriptors use
  it, and gen_go renders them as `getCSV[Row]`.
- A `200` with any other media type, including `application/json`, is the
  existing "unexpected content type" decode error. The provider-message check
  of ADR 0031 (#381) stays on the JSON path: a header-only CSV body such as
  `symbol,price` is data, not an error line.
- An empty body or a header-only body decodes to no rows.

### Decoding

- Rust: the `csv` crate (1.4, RFC 4180 records, quoted cells with commas,
  doubled quotes and newlines, no trimming, equal field counts) only splits
  records. Each record is handed to the row's `Deserialize` as a map from
  header name to cell through a small cell deserializer, so serde renames
  and field codecs apply unchanged and `serde_path_to_error` reports
  `[row].member`, the JSON decoder's path format. The csv crate's own serde
  layer was not used: it lost member paths and infers types for
  `deserialize_any`.
- Go: `encoding/csv` splits records; each record is written as a JSON object
  whose cell kinds come from the model's fields (numbers and booleans raw,
  everything else quoted) and decoded with the model's own JSON rules, so
  shadows, codecs, and redaction apply and a failure reports
  `/<row>/<member>`.
- Every model member, `Option` or not, must be a header column, checked on
  the first record, so a dropped or renamed provider column is a
  `missing_member` error instead of a silent `None`. Columns the model does
  not read are ignored.
- Error messages are classified into a `DecodeErrorKind` and dropped; no cell
  value reaches an error (ADR 0031).

### Cell rules

| Cell | Option member | Plain `String` | Number or bool | Typed code or date, required |
| --- | --- | --- | --- | --- |
| empty | `None` / `None` / `nil` | `""` | `null` decode error | `null` decode error |
| text | the member's own parse | verbatim | parsed; anything else is `wrong_type` | the type's validation |

`NumericString` keeps the cell text verbatim (`6.9148336e-9`, values beyond
`u64`). Booleans accept exactly `true` and `false`. Go is more lenient on one
point it already had for JSON: a required plain `string` (Go's
`NumericString` and code types) takes an empty cell as `""`.

### Nullability (bulk CSV only)

CSV has no null: an empty cell is its only absence marker, and the census
shows sparse empties that change by period, where one empty cell would fail a
50,000-row response. So, for bulk CSV rows only:

- Every numeric metric member (`NumericString`) is `Option<NumericString>`,
  Python `Optional[str]`, Go `*string`.
- Identity members stay required: `symbol`, `date`, and on statement rows
  `reportedCurrency`, `cik`, `filingDate`, `acceptedDate`, `fiscalYear`, and
  `period`; `lastUpdated` on ETF holdings and earnings surprises.
- Typed codes follow the evidence through `codecs::empty_or_null`:
  `BulkFinancialScore.reported_currency`, `BulkEtfHolding.asset` and
  `isin`.
- Text members (`BulkStockPeer.peers`, ETF `cusip` and `name`) stay strings;
  an empty cell is `""`.

JSON endpoints keep ADR 0033's evidence-only rule. `CompanyProfile` is shared
with the JSON `company.profile` route; the evidence rule widened its `isin`
and `country` (`empty_or_null`) and `change` and `change_percentage`
(`required_option`), which affects both routes.

`BulkEtfHolding.last_updated_raw: String` (key `lastUpdated"`) became
`last_updated: Date` (key `lastUpdated`).

### Body limit and buffering

CSV bulk descriptors default to `DEFAULT_BULK_MAX_RESPONSE_BODY_BYTES`
(256 MiB) while the client limit is unset; every other endpoint keeps the
64 MiB default. Once a caller sets the client limit
(`ClientBuilder::max_response_body_bytes`, Go `WithMaxResponseBodyBytes`,
Python `max_response_body_bytes=`), it applies to every endpoint, bulk
included, and an endpoint override still wins. Bodies stay buffered, as in
ADR 0028, and decode record by record from the buffer; there is no streaming
API.

## Alternatives considered

- **Evidence-only `Option`s for bulk rows.** Rejected: about forty members
  today, and any new year or period can add an empty cell that fails a whole
  response.
- **Transcode CSV to JSON in Rust and reuse `decode_json`.** Rejected: CSV
  cells carry no types, so the transcoder would need the model's schema,
  which serde already provides.
- **A separate `BulkCompanyProfile`.** Rejected: a 36-field copy in three
  SDKs for the same provider data, where the JSON profile shows the same
  gaps.
- **Streaming decode.** Deferred: the bodies fit the buffered limit and every
  method returns a `Vec`.

## Consequences

- Breaking in all three SDKs: the bulk methods require `text/csv`, most bulk
  row members are optional, the ETF field is renamed, and four
  `CompanyProfile` members are optional.
- The shared fixtures under `crates/libfmp/tests/fixtures/bulk_*.csv` are
  trimmed live captures; the Go fixture check covers `.csv` files.
- A new empty identity cell is a decode error by design and a small model
  change under this ADR.
