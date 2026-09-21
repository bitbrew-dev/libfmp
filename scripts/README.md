# Scripts

| Script | Purpose |
| --- | --- |
| `check-crate-package.sh` | Rejects a `libfmp` crate package that ships test trees or exceeds the size budget (see `docs/releasing.md`). |
| `set-version.sh` | Sets the workspace version in `Cargo.toml` and `Cargo.lock`; invoked by semantic-release. |
| `gen-endpoint-coverage.py` | Regenerates `docs/endpoint-coverage.md` from the endpoint registry; pass `--oracle artifacts/documentations/outer.md` for the documented-entry counts. The `Go methods` column and the per-endpoint `Go` state come from `check_go_coverage.py`. |
| `check_go_coverage.py` | Audits that every endpoint registry method has a generated Go method on its `<Domain>Namespace` (stdlib only, no Go toolchain). Prints `Go methods: N of 271` and the missing list by domain; `--strict` exits 1 while any method is missing (the phase 3 exit gate, #282). Unit tests: `python3 -m unittest discover -s scripts -p 'check_go_coverage_test.py'`. |
| `check_go_sdk.sh` | Verification gate for the Go SDK at `sdk/go`: pins Go 1.27.1 and golangci-lint v2, then runs `gofmt`, `go vet`, `golangci-lint run`, `go test -race`, the examples with `CGO_ENABLED=0`, the `gen_go` regeneration gate, and the non-strict Go coverage audit. Backs the `go-sdk-tests` pre-commit hook and the `go-sdk.yml` workflow. |
