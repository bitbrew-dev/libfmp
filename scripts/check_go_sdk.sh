#!/usr/bin/env bash
# Verification gate for the Go SDK at sdk/go (ADR 0030, "Verification gates").
# Runs from the pre-commit hook `go-sdk-tests` and the `go-sdk.yml` workflow.
# Later phases append steps at the end: the gen_go regeneration gate and the
# coverage audit. Keep the script linear so they slot in after the tests.
set -euo pipefail
cd "$(dirname "$0")/.."

# 1. Toolchain pins. The Go version must match go.mod exactly; golangci-lint
#    must be a v2 release because .golangci.yml uses the v2 schema.
go_version=$(go env GOVERSION)
if [[ "$go_version" != go1.27.1 ]]; then
  printf 'Go 1.27.1 is required; found %s. Install the version pinned in sdk/go/go.mod.\n' "$go_version" >&2
  exit 1
fi
if ! command -v golangci-lint >/dev/null 2>&1; then
  printf 'golangci-lint v2 is required; it is not on PATH. See https://golangci-lint.run/docs/welcome/install/\n' >&2
  exit 1
fi
lint_version=$(golangci-lint version --short)
if [[ "${lint_version%%.*}" != 2 ]]; then
  printf 'golangci-lint v2 is required; found %s.\n' "$lint_version" >&2
  exit 1
fi

# 2. Formatting.
unformatted=$(gofmt -l sdk/go)
if [[ -n "$unformatted" ]]; then
  printf 'gofmt: the following files are not formatted:\n%s\n' "$unformatted" >&2
  exit 1
fi

# 3. Vet, lint, and race-detected tests. CGO_ENABLED=0 proves the module is
#    pure Go; the race detector links without cgo on the supported platforms.
cd sdk/go
CGO_ENABLED=0 go vet ./...
golangci-lint run ./...
CGO_ENABLED=0 go test -race -count=1 ./...

# 4. Examples must compile and run offline. The glob is empty until the first
#    example lands, so the loop tolerates a missing directory.
for example in examples/*/; do
  [[ -d "$example" ]] || continue
  CGO_ENABLED=0 go run "./$example"
done
