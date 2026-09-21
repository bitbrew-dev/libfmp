# Scripts

| Script | Purpose |
| --- | --- |
| `check-crate-package.sh` | Rejects a `libfmp` crate package that ships test trees or exceeds the size budget (see `docs/releasing.md`). |
| `set-version.sh` | Sets the workspace version in `Cargo.toml` and `Cargo.lock`; invoked by semantic-release. |
| `gen-endpoint-coverage.py` | Regenerates `docs/endpoint-coverage.md` from the endpoint registry; pass `--oracle artifacts/documentations/outer.md` for the documented-entry counts. |
| `check_go_sdk.sh` | Verification gate for the Go SDK at `sdk/go`: pins Go 1.27.1 and golangci-lint v2, then runs `gofmt`, `go vet`, `golangci-lint run`, `go test -race`, and the examples with `CGO_ENABLED=0`. Backs the `go-sdk-tests` pre-commit hook and the `go-sdk.yml` workflow. |
