# Response fixtures

JSON bodies that the Rust `tests/*_responses.rs` and `tests/*_endpoints.rs`
suites `include_bytes!`/`include_str!`, and that the Python suite reads through
`conftest.load_fixture`. Every fixture is served through a deterministic
executor or a loopback server; no test in this directory's users opens a
network connection.

## Provenance

- **Documented rows.** The default `<endpoint>.json` file is the response
  sample copied from the captured provider documentation,
  `artifacts/documentations/outer.md` (gitignored; the documentation oracle
  pinned by the README). Values are copied verbatim, including provider
  defects such as the bulk ETF holder `lastUpdated"` key, the DCF
  `"Stock Price"` key, and `null`-only fields.
- **Hand-derived variants.** `*_empty.json` (`[]`), `*_multiple.json`, and
  `*_unknown.json` (a documented row plus a future field) are edited copies
  of the documented row. They prove empty arrays, provider order, and
  unknown-field tolerance; null and missing-field variants are built inline
  in the `*_responses.rs` tests from the same documented row.
- **Synthetic and dynamic shapes.** `*_synthetic.json` and `*_dynamic.json`
  are constructed by hand for edge cases the documentation does not show
  (mixed numeric forms, dynamic JSON recursion). Their file name marks them as
  not provider-observed. The three `*_fractional_synthetic.json` files
  (`company_screener`, `cryptocurrency_list`, `stock_splits`) carry
  fractional and `1.0`-form numbers in integer-documented fields (the
  JSON formatter hook rewrites exponent spellings, so exponent-form cases are
  inline in the Rust and Go tests); they are the issue #339 decode proof in
  `integral_f64_responses.rs`, the fmp-py screener, crypto, and calendar
  tests, and `sdk/go/integral_f64_parity_test.go`. The nine issue #340
  `*_fractional_synthetic.json` files (`latest_insider_trades`,
  `fund_disclosures`, `company_shares_float`, `fundraising_by_cik`,
  `earnings_calendar`, `custom_discounted_cash_flow`, `income_statement`,
  `aftermarket_trade`, `institutional_ownership_extract`) do the same for the
  `Quantity` and `MarketValue` fields, including a negative fractional fund
  `balance`, in the same three test suites. Issue #341 gave
  `income_statement_fractional_synthetic.json` a fractional `revenue` and an
  integral-float `ebitda` to prove `StatementAmount` decoding the same way.
- **Decode-path proof.** `company_screener_null_beta_synthetic.json` is the
  one null variant kept as a file: 40 documented rows with `"beta": null` in
  row 37, so the offending member lands past the `MAX_SAFE_BODY_BYTES` body
  excerpt and the Go and Python suites can share it. It is the issue #366
  decode-path proof in `decode_path.rs`, `sdk/go/decode_path_test.go`, and
  the fmp-py screener tests, which also swap in a string value to prove it
  never reaches the error.
- **Observed-shape reproductions.** `quote_short_fractional_volume.json`
  reproduces the one live `quote-short` row that motivated issue #337 (seen
  2026-09-23: a fractional `volume`, `20201922.82733`); the four values are
  the ones reported in the issue, not a recorded response body. It is the
  decode proof for `Volume = f64` in `quote_short_endpoint.rs`,
  `fmp-py/tests/test_client.py`, and `sdk/go/quote_parity_test.go`.
- **Trimmed live capture.** `financial_reports_json.json` is the one fixture
  taken from a live call (AAPL, 2023, Q1, captured 2026-09-30 for issue
  #374): the three headers plus three of its 48 sections, verbatim. It proves
  the endpoint returns one bare object, not an array. It keeps the endpoint
  name because the documented sample it replaced had the wrong shape.
- **Other captured live responses.** When one is captured, add it under a
  `*_captured.json` name, note the capture date and plan tier in the owning
  test, and resolve the matching row in `docs/contract-ambiguities.md`.

Formatting is not wire-faithful: the `pretty-format-json` pre-commit hook
re-indents every file. Only the content (keys, value kinds, spelling,
ordering) is what the tests assert.

## Redaction policy

- **No fixture contains a credential.** The only file mentioning `apikey` is
  `financial_reports_dates.json`, whose documented download URLs already carry
  `apikey=[REDACTED]` in the provider sample; keep that literal text. Verify
  with `grep -rl apikey crates/libfmp/tests/fixtures`.
- **Captured responses must be redacted before commit.** Strip or replace any
  key, token, session id, account e-mail, or signed URL with `[REDACTED]`;
  keep the key name so the shape stays observable.
- **The runtime redacts the rest.** Diagnostics never carry a configured
  secret: `SecretString`, `SecretUrl` (`Debug` prints
  `SecretUrl([REDACTED URL])`), and `SafeBody` in `src/error.rs` apply the
  `Redactor` defaults: every configured secret value, the secret header names
  (`authorization`, `proxy-authorization`, `x-api-key`, `apikey`), the secret
  query names (`access_token`, `api_key`, `apikey`, `key`, `sig`, `signature`,
  `token`, `x-amz-signature`, `x-goog-signature`), any custom secret name,
  and a `MAX_SAFE_BODY_BYTES` bound on retained bodies.
  `tests/transport.rs` (`status_decode_connection_and_timeout_failures_are_safe`,
  `overlapping_secrets_are_redacted_from_status_and_decode_bodies`,
  `debug_output_never_contains_configured_secrets_or_urls`) and
  `tests/errors.rs` pin that behaviour.

## Live tests

Opt-in live checks live outside this directory: `tests/live_opt_in.rs`
(`#[ignore]`, gated on `FMP_LIVE_TESTS=1`) and
`crates/fmp-py/tests/test_live_opt_in.py` (`pytest.mark.skipif`). They exercise
direct `apikey` header authentication and the README proxy route against
`quote-short` and assert the secret never appears in error text. They are the
path for capturing new fixtures; they never write files themselves.
