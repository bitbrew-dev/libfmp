package fmp

// Version is the SDK version reported in the default User-Agent header.
//
// It tracks the libfmp workspace version. Release plumbing that keeps it in
// sync with Cargo.toml is a later phase; until then it is maintained by hand.
const Version = "0.8.0"

const defaultUserAgent = "libfmp-go/" + Version
