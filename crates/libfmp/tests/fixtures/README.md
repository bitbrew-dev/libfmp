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
  not provider-observed.
- **Observed-shape reproductions.** `quote_short_fractional_volume.json`
  reproduces the one live `quote-short` row that motivated issue #337 (seen
  2026-09-23: a fractional `volume`, `20201922.82733`); the four values are
  the ones reported in the issue, not a recorded response body. It is the
  decode proof for `Volume = f64` in `quote_short_endpoint.rs`,
  `fmp-py/tests/test_client.py`, and `sdk/go/quote_parity_test.go`.
- **No captured live responses yet.** Nothing here was recorded from a live
  call. When one is captured, add it under a `*_captured.json` name, note the
  capture date and plan tier in the owning test, and resolve the matching row
  in `docs/contract-ambiguities.md`.

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
