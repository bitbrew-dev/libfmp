# Release validation

The Rust crate and Python distribution share their version through `workspace.package.version`. The Python wheel must report distribution name `fmp-py-sdk`; its import package is `fmp` and its private extension is `fmp._native`.

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

As of 2026-08-26, both registry APIs returned HTTP 404 for their exact package lookup, and `cargo search libfmp --limit 10` returned no matches. This indicates that `libfmp` and `fmp-py-sdk` were unregistered at check time; it is not a reservation.

Repeat these checks immediately before publishing:

```console
cargo search libfmp --limit 10
curl --fail-with-body --user-agent "libfmp-name-check/0.1 (https://github.com/bitbrew-dev/libfmp)" https://crates.io/api/v1/crates/libfmp
curl --fail-with-body https://pypi.org/pypi/fmp-py-sdk/json
```

An exact package response means the name is registered. A not-found response indicates only current availability. Never include registry tokens or FMP credentials in validation commands, build logs, or documentation.
