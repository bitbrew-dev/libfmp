#!/usr/bin/env sh
# Set the workspace version in Cargo.toml, the workspace-member entries in
# Cargo.lock, and the Go module's Version constant in sdk/go/version.go.
# Invoked by semantic-release (@semantic-release/exec prepareCmd) so every
# SDK tracks the release; all three files are committed by
# @semantic-release/git (all three must be in its assets list).
#
#   sh scripts/set-version.sh 1.5.1
#
# Uses awk only (no cargo / GNU-sed extensions) so it runs in the minimal
# semantic-release container image, which has no cargo. The Cargo.lock pass
# rewrites the version of every source-less [[package]] block (the local
# workspace crates - registry deps carry a source line); the lockfile-format
# "version = N" line sits above the first [[package]] and is left untouched.
# The version.go pass rewrites the first `const Version = "..."` line; the
# sdk/go/vX.Y.Z tag job later asserts it equals the release tag, so the pass
# fails loudly when the constant is missing rather than tagging a stale file.
set -eu

version="${1:?usage: set-version.sh <version>}"

tmp="$(mktemp)"
awk -v v="$version" '
  /^\[/            { in_pkg = ($0 == "[workspace.package]") }
  in_pkg && !done && /^version[[:space:]]*=/ {
      print "version = \"" v "\""
      done = 1
      next
  }
  { print }
' Cargo.toml > "$tmp"
mv "$tmp" Cargo.toml

tmp2="$(mktemp)"
awk -v v="$version" '
  function flush(   i) {
      for (i = 1; i <= n; i++) {
          if (!has_source && buf[i] ~ /^version[[:space:]]*=/)
              buf[i] = "version = \"" v "\""
          print buf[i]
      }
      n = 0; has_source = 0
  }
  /^\[/ { flush(); buf[++n] = $0; next }
  {
      if (n > 0) {
          if ($0 ~ /^source[[:space:]]*=/) has_source = 1
          buf[++n] = $0
      } else {
          print
      }
  }
  END { flush() }
' Cargo.lock > "$tmp2"
mv "$tmp2" Cargo.lock

go_version_file="sdk/go/version.go"
tmp3="$(mktemp)"
awk -v v="$version" '
  !done && /^const Version[[:space:]]*=[[:space:]]*"/ {
      print "const Version = \"" v "\""
      done = 1
      next
  }
  { print }
' "$go_version_file" > "$tmp3"
mv "$tmp3" "$go_version_file"
if ! grep -q "^const Version = \"$version\"\$" "$go_version_file"; then
    echo "set-version.sh: no 'const Version = \"...\"' line found in $go_version_file" >&2
    exit 1
fi

echo "set [workspace.package] version = $version in Cargo.toml, Cargo.lock, and $go_version_file"
