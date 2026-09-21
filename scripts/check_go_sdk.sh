#!/usr/bin/env bash
# Verification gate for the Go SDK at sdk/go (ADR 0030, "Verification gates").
# Runs from the pre-commit hook `go-sdk-tests` and the `go-sdk.yml` workflow.
# Later phases append steps at the end: the coverage audit follows the gen_go
# regeneration gate. Keep the script linear so it slots in after the tests.
set -euo pipefail
cd "$(dirname "$0")/.."
root=$(pwd)

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

# 5. Regeneration gate (ADR 0030, "Generator idempotency"). gen_go rewrites
#    every generated domain plus queries.go and namespaces.go; the tree must
#    come back byte-identical, so a hand edit of a generated file or a Rust
#    change without a regeneration fails here. Runs from the repository root.
#    The diff is against the index, so a pre-commit run compares the staged
#    tree with fresh output; untracked files catch a newly generated domain.
cd "$root"
cargo run -q -p fmp-py-gen --bin gen_go
untracked=$(git ls-files --others --exclude-standard -- sdk/go)
if ! git diff --exit-code --stat -- sdk/go || [[ -n "$untracked" ]]; then
  printf 'gen_go: sdk/go is not byte-identical after regeneration; run `cargo run -p fmp-py-gen --bin gen_go` and commit the result.\n' >&2
  [[ -n "$untracked" ]] && printf 'untracked generated files:\n%s\n' "$untracked" >&2
  exit 1
fi
