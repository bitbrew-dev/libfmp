package fmp

// Version is the SDK version reported in the default User-Agent header.
//
// It equals the libfmp workspace version ([workspace.package].version in
// Cargo.toml). scripts/set-version.sh rewrites it at release time and the
// release workflow tags sdk/go/v<Version> at the release commit, so it must
// never be edited by hand; version_test.go fails when it drifts.
const Version = "1.0.0"

const defaultUserAgent = "libfmp-go/" + Version
