# Scripts

| Script | Purpose |
| --- | --- |
| `check-crate-package.sh` | Rejects a `libfmp` crate package that ships test trees or exceeds the size budget (see `docs/releasing.md`). |
| `set-version.sh` | Sets the workspace version in `Cargo.toml` and `Cargo.lock`; invoked by semantic-release. |
| `gen-endpoint-coverage.py` | Regenerates `docs/endpoint-coverage.md` from the endpoint registry; pass `--oracle artifacts/documentations/outer.md` for the documented-entry counts. |
