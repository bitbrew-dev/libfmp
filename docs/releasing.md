# Release validation

The Rust crate and Python distribution share version `0.1.0` through `workspace.package.version`. The Python wheel must report distribution name `fmp-py-sdk`; its import package remains `fmp_py` and its private extension remains `fmp_py._native`.

## Publication blocker

No project license has been selected. Do not publish either package until a license is chosen and matching repository, Cargo, and Python package metadata are added. This repository intentionally has no speculative `license`, `license-file`, or Python license classifier.

## Validate artifacts

From the repository root:

```console
cargo metadata --no-deps --format-version 1 --offline
cargo package -p libfmp --list
cargo package -p libfmp --offline
maturin build --manifest-path crates/fmp-py/Cargo.toml --release --offline
maturin sdist --manifest-path crates/fmp-py/Cargo.toml --out target/wheels
```

Install the wheel into a clean Python environment and verify both names and the shared version:

```console
python -m venv .venv-release-check
.venv-release-check/bin/python -m pip install target/wheels/fmp_py_sdk-0.1.0-*.whl
.venv-release-check/bin/python -c 'from importlib.metadata import version; import fmp_py; assert version("fmp-py-sdk") == fmp_py.__version__ == "0.1.0"'
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
