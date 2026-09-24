# Release validation

The Rust crate, the Python distribution, and the Go module share their version through `workspace.package.version`; `scripts/set-version.sh` writes it to `Cargo.toml`, `Cargo.lock`, and `sdk/go/version.go` on every release. The Python wheel must report distribution name `fmp-py-sdk`; its import package is `fmp` and its private extension is `fmp._native`.

## License

The repository, Rust crate, and Python distribution are licensed under the [MIT License](../LICENSE). Artifact validation must confirm that both package metadata and the packaged license text remain consistent before publishing.

## Validate artifacts

From the repository root:

```console
cargo metadata --no-deps --format-version 1 --offline
cargo package -p libfmp --list
cargo package -p libfmp --offline
scripts/check-crate-package.sh
maturin build --manifest-path crates/fmp-py/Cargo.toml --release --offline
maturin sdist --manifest-path crates/fmp-py/Cargo.toml --out target/wheels
```

Confirm that Cargo metadata reports `license = "MIT"`, the crate file list contains `LICENSE`, and the wheel and source distribution metadata contain both `License-Expression: MIT` and `License-File: LICENSE`. Each artifact must contain the complete license text.

The crate package check rejects published test or fixture trees and rejects a
compressed `.crate` larger than 512 KiB. Set `LIBFMP_CRATE_MAX_KIB` when a
reviewed release intentionally changes that budget. This standalone check is
suitable for local release validation now and can be called unchanged by a
future publication gate.

Install the wheel into a clean Python environment and verify the names, shared version, and installed license metadata:

```console
python -m venv .venv-release-check
.venv-release-check/bin/python -m pip install target/wheels/fmp_py_sdk-*.whl
.venv-release-check/bin/python -c 'from importlib.metadata import metadata, version; import fmp; package = metadata("fmp-py-sdk"); assert version("fmp-py-sdk") == fmp.__version__; assert package["License-Expression"] == "MIT"; assert package.get_all("License-File") == ["LICENSE"]'
```

## Check names immediately before publishing

As of 2026-08-26, both registry APIs returned HTTP 404 for their exact package lookup, and `cargo search libfmp --limit 10` returned no matches, so the names were unregistered at that time. Since the first releases on 2026-08-27 (0.1.0 on crates.io, 0.1.1 on PyPI) this project owns both names, and every release publishes the same workspace version to crates.io and PyPI; `Cargo.toml` holds the current one.

Repeat these checks immediately before publishing to confirm the registries still report this project's latest version:

```console
cargo search libfmp --limit 10
curl --fail-with-body --user-agent "libfmp-name-check/0.1 (https://github.com/bitbrew-dev/libfmp)" https://crates.io/api/v1/crates/libfmp
curl --fail-with-body https://pypi.org/pypi/fmp-py-sdk/json
```

An exact package response means the name is registered. A not-found response indicates only current availability. Never include registry tokens or FMP credentials in validation commands, build logs, or documentation.

## Release notes checklist

`semantic-release` generates the GitHub release notes from the conventional commit messages on `main`, and the same workflow publishes the crate and the wheels. Once the run finishes, edit the notes of the new release (`gh release edit <tag> --notes-file <file>`) so that every release states the following:

- Breaking-name avoidance: confirm that no public Rust or Python name changed without a `!` or `BREAKING CHANGE` commit, and list any renamed or deprecated names with their replacements. Review the `fmp-py/artifact` regeneration commits for changed existing signatures, since regenerated stubs follow every Rust rename.
- Entitlement-limited endpoints: point at the `gated` rows of [endpoint-coverage.md](endpoint-coverage.md) (the TipRanks add-on and the fixed `short=true` quote universes) so users know which methods need more than the standard plan, and at the quote rows whose note names the Nasdaq real-time user declaration.
- Unresolved upstream response gaps: point at the `deferred` and `raw` rows of the same table and at the open hardening issue (#40) so users know which documented ambiguities are mirrored verbatim rather than corrected.

Regenerate the coverage table before tagging when the registry changed (`python3 scripts/gen-endpoint-coverage.py --oracle artifacts/documentations/outer.md`) so the linked rows match the release.

## Go module checklist

The `publish-go-tag` job of `semantic-release.yml` runs after the crates.io and PyPI jobs of the same user-dispatched run. It checks out the release tag (`X.Y.Z`, no `v`), asserts that `sdk/go/go.mod` exists and that `sdk/go/version.go` carries `Version = "X.Y.Z"` at that commit, then creates the annotated tag `sdk/go/vX.Y.Z` there and pushes it. Go requires the `sdk/go/` prefix for a module in a subdirectory ([Mapping versions to commits](https://go.dev/ref/mod#vcs-version)); the module version itself is `vX.Y.Z`. The job never creates a second GitHub release and never moves an existing tag: a rerun that finds `sdk/go/vX.Y.Z` at the release commit is a no-op, and one that finds it elsewhere fails. Its last steps always verify the pushed tag directly (`GOPROXY=direct` with the workflow token, asserting that `go list -m` returns `vX.Y.Z`), then warm `proxy.golang.org` only when the repository is public; the proxy caches the version and from then on it is immutable, so a failed version can only be superseded by a new release. While the repository is private the job prints a notice instead of warming the proxy. A proxy timeout in that step is safe to rerun.

`sdk/go/v*` tags are a user decision: only a release the user dispatches creates one, and nothing else pushes one.

After the run finishes:

- Verify the tag sits on the release commit: `git ls-remote --tags origin 'refs/tags/sdk/go/vX.Y.Z*'` lists the tag and its peeled commit; the peeled commit must equal `git rev-list -n 1 X.Y.Z`.
- Verify the proxy serves it: `GOPROXY=https://proxy.golang.org go list -m -versions github.com/bitbrew-dev/libfmp/sdk/go` must list `vX.Y.Z`, and `go list -m github.com/bitbrew-dev/libfmp/sdk/go@vX.Y.Z` must print the same version. Run both from a directory without a `go.mod`.
- Remember that the proxy checks apply only to a public repository. While the repository is private, run the same `go list -m` commands with the `GOPRIVATE` environment and git credentials from the [Go README](../sdk/go/README.md#installation); `proxy.golang.org` will report the version as not found.
- Verify `go test ./...` from `sdk/go` at the tag passes `TestVersionMatchesWorkspace`, which compares `Version` with `Cargo.toml`.
- Remember that the served version is immutable: a wrong tag is fixed by the next release, never by retagging.
