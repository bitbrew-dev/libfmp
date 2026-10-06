package fmp

import (
	"maps"
	"math"
	"net/http"
	"slices"
	"strconv"
	"strings"
	"time"
)

// MaxSafeHeaders is the largest number of response header values retained
// in an *Error.
const MaxSafeHeaders = 16

// MaxSafeHeaderValueBytes is the largest response header value retained in
// an *Error; longer values are dropped, never truncated.
const MaxSafeHeaderValueBytes = 256

// IsRetainedHeaderName reports whether a response header may be retained for
// callers. It is the single allowlist shared with the Rust and Python SDKs:
// Retry-After, plus every name starting with X-Proxy- or X-RateLimit-,
// compared case-insensitively. Everything else, including Set-Cookie,
// Authorization and apikey, is never retained.
func IsRetainedHeaderName(name string) bool {
	name = asciiLower(name)
	return name == "retry-after" || strings.HasPrefix(name, "x-proxy-") ||
		strings.HasPrefix(name, "x-ratelimit-")
}

// retainedHeaders keeps the allowlisted headers of a response, redacted and
// bounded, or returns nil when none qualify. Values that are not visible
// ASCII or exceed MaxSafeHeaderValueBytes are dropped.
func retainedHeaders(header http.Header, redactor *Redactor) http.Header {
	var kept http.Header
	count := 0
	for _, name := range slices.Sorted(maps.Keys(header)) {
		values := header[name]
		if !IsRetainedHeaderName(name) {
			continue
		}
		for _, value := range values {
			if count == MaxSafeHeaders || !visibleHeaderText(value) {
				continue
			}
			value = redactor.RedactHeader(name, value)
			if len(value) > MaxSafeHeaderValueBytes {
				continue
			}
			if kept == nil {
				kept = make(http.Header)
			}
			kept.Add(name, value)
			count++
		}
	}
	return kept
}

// RetryAfter returns the Retry-After delay relative to now. It accepts
// delta-seconds and every HTTP-date form of RFC 9110; a date at or before now
// yields zero, meaning retry at once. The bool is false when the header is
// absent or unparsable.
func (e *Error) RetryAfter() (time.Duration, bool) {
	return retryAfterAt(e.Headers.Get("Retry-After"), time.Now())
}

// ProxyError returns the X-Proxy-Error reason a proxy such as valet sent, or
// "" when there is none.
func (e *Error) ProxyError() string {
	return e.Headers.Get("X-Proxy-Error")
}

func retryAfterAt(value string, now time.Time) (time.Duration, bool) {
	value = strings.TrimSpace(value)
	if value == "" {
		return 0, false
	}
	if strings.Trim(value, "0123456789") == "" {
		seconds, err := strconv.ParseInt(value, 10, 64)
		if err != nil || seconds > int64(math.MaxInt64/time.Second) {
			return math.MaxInt64, true
		}
		return time.Duration(seconds) * time.Second, true
	}
	date, err := http.ParseTime(value)
	if err != nil {
		return 0, false
	}
	return max(date.Sub(now), 0), true
}
