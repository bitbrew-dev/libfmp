package fmp

import (
	"errors"
	"fmt"
	"slices"
	"sort"
	"strings"
	"unicode/utf8"
)

// Redacted replaces every secret value removed from a diagnostic.
const Redacted = "[REDACTED]"

// MaxSafeBodyBytes is the largest UTF-8 byte length retained from a provider
// response body inside an *Error.
const MaxSafeBodyBytes = 4096

const ellipsis = "…"

// ErrorCategory is the broad, stable classification shared with the Rust and
// Python SDKs.
type ErrorCategory int

const (
	// CategoryValidation reports an input that failed local validation.
	CategoryValidation ErrorCategory = iota
	// CategoryConfiguration reports invalid client or transport configuration.
	CategoryConfiguration
	// CategoryTransport reports a request the transport could not complete.
	CategoryTransport
	// CategoryStatus reports a non-success HTTP status from the provider.
	CategoryStatus
	// CategoryDecode reports a successful response that could not be decoded.
	CategoryDecode
)

// String returns the stable lowercase value exposed to language bindings.
func (c ErrorCategory) String() string {
	switch c {
	case CategoryValidation:
		return "validation"
	case CategoryConfiguration:
		return "configuration"
	case CategoryTransport:
		return "transport"
	case CategoryStatus:
		return "status"
	case CategoryDecode:
		return "decode"
	default:
		return fmt.Sprintf("ErrorCategory(%d)", int(c))
	}
}

// ConfigurationKind is the machine-readable reason a configuration was
// rejected. It mirrors ConfigurationErrorKind in the Rust crate.
type ConfigurationKind int

const (
	// ConfigurationKindNone marks an error that is not a configuration error.
	ConfigurationKindNone ConfigurationKind = iota
	// ConfigurationKindInvalidBaseURL: the base URL is not an absolute HTTP(S) URL.
	ConfigurationKindInvalidBaseURL
	// ConfigurationKindUnsafeBaseURL: the base URL carries user info, a query, or a fragment.
	ConfigurationKindUnsafeBaseURL
	// ConfigurationKindInvalidPath: a path prefix or endpoint path has unsafe segments.
	ConfigurationKindInvalidPath
	// ConfigurationKindInvalidHeaderName: a header name is not a valid HTTP field name.
	ConfigurationKindInvalidHeaderName
	// ConfigurationKindInvalidHeaderValue: a header value is not a valid HTTP field value.
	ConfigurationKindInvalidHeaderValue
	// ConfigurationKindInvalidQueryName: a query name is empty or has control characters.
	ConfigurationKindInvalidQueryName
	// ConfigurationKindEmptyCredential: a credential is empty.
	ConfigurationKindEmptyCredential
	// ConfigurationKindMissingCredential: direct FMP access has no credential.
	ConfigurationKindMissingCredential
	// ConfigurationKindInsecureAuthentication: a credential over plain HTTP to a non-loopback host.
	ConfigurationKindInsecureAuthentication
	// ConfigurationKindConflictingAuthentication: authentication was configured more than once.
	ConfigurationKindConflictingAuthentication
	// ConfigurationKindProtectedFieldCollision: a caller tried to replace a transport-owned field.
	ConfigurationKindProtectedFieldCollision
	// ConfigurationKindHTTPClient: the underlying HTTP client could not be built.
	ConfigurationKindHTTPClient
)

// Error is the stable error value returned by every client operation.
//
// Endpoint is a logical endpoint id such as "quote-short", never a URL.
// Status is zero when no provider status applies. Body is nil unless a
// provider body was retained; it is always redacted and capped.
type Error struct {
	Category          ErrorCategory
	Message           string
	Endpoint          string
	Status            int
	Body              *SafeBody
	ConfigurationKind ConfigurationKind
	cause             error
}

// Error formats the message, the endpoint id, and the safe body, in that order.
func (e *Error) Error() string {
	var b strings.Builder
	b.WriteString(e.Message)
	if e.Endpoint != "" {
		b.WriteString(" (endpoint: ")
		b.WriteString(e.Endpoint)
		b.WriteString(")")
	}
	if e.Body != nil {
		b.WriteString(": ")
		b.WriteString(e.Body.Text)
	}
	return b.String()
}

// Unwrap returns the redacted cause, when one was retained.
func (e *Error) Unwrap() error {
	return e.cause
}

func configurationError(kind ConfigurationKind, message string) *Error {
	return &Error{Category: CategoryConfiguration, Message: message, ConfigurationKind: kind}
}

// validationError reports a query argument that failed local validation. The
// message is "<argument>: <reason>", the shape the Python binding uses.
func validationError(argument string, reason error) *Error {
	return &Error{Category: CategoryValidation, Message: argument + ": " + reason.Error(), cause: reason}
}

// missingMemberError reports a JSON object that omitted a required member or
// carried null for it. serde rejects both for a non-optional field; the
// generated UnmarshalJSONFrom of every model returns this for the first
// required member it cannot find. The endpoint id is filled in by getJSON,
// which wraps the decoder failure.
func missingMemberError(model, member string) *Error {
	return &Error{
		Category: CategoryDecode,
		Message:  fmt.Sprintf("required member %q of %s is missing or null", member, model),
	}
}

// invalidMemberError reports a JSON member whose kind the Rust decoder would
// reject, such as a non-object value for a DynamicObject field.
func invalidMemberError(model, member, expected string) *Error {
	return &Error{
		Category: CategoryDecode,
		Message:  fmt.Sprintf("member %q of %s must be a JSON %s", member, model, expected),
	}
}

func transportError(endpoint, message string, cause error) *Error {
	return &Error{Category: CategoryTransport, Message: message, Endpoint: endpoint, cause: cause}
}

func statusError(endpoint string, status int, body *SafeBody) *Error {
	return &Error{
		Category: CategoryStatus,
		Message:  fmt.Sprintf("provider returned HTTP status %d", status),
		Endpoint: endpoint,
		Status:   status,
		Body:     body,
	}
}

func decodeError(endpoint string, status int, body *SafeBody, message string, cause error) *Error {
	return &Error{
		Category: CategoryDecode,
		Message:  message,
		Endpoint: endpoint,
		Status:   status,
		Body:     body,
		cause:    cause,
	}
}

// redactedCause is a transport or decoder cause whose text has been passed
// through the Redactor. It keeps context sentinels reachable via errors.Is.
type redactedCause struct {
	text     string
	sentinel error
}

func (c *redactedCause) Error() string { return c.text }

func (c *redactedCause) Is(target error) bool {
	return c.sentinel != nil && target == c.sentinel
}

// SafeBody is a redacted, bounded provider response body.
type SafeBody struct {
	Text      string
	Truncated bool
}

// NewSafeBody redacts and bounds a body before it is retained in an Error.
func NewSafeBody(body string, redactor *Redactor) SafeBody {
	text := redactor.Redact(body)
	truncated := len(text) > MaxSafeBodyBytes
	if truncated {
		end := MaxSafeBodyBytes - len(ellipsis)
		for end > 0 && !utf8.RuneStart(text[end]) {
			end--
		}
		text = text[:end] + ellipsis
	}
	return SafeBody{Text: text, Truncated: truncated}
}

// String returns the safe retained text.
func (b SafeBody) String() string { return b.Text }

// Errors returned by Redactor when a secret name is rejected.
var (
	ErrEmptySecretName   = errors.New("secret name must not be empty")
	ErrInvalidSecretName = errors.New("secret name contains unsupported characters")
)

// Redactor removes registered secrets and the values of known secret query
// parameters from diagnostic text.
type Redactor struct {
	secrets           []string
	secretQueryNames  map[string]struct{}
	secretHeaderNames map[string]struct{}
}

// NewRedactor creates a policy with common authentication and signing names
// already protected.
func NewRedactor() *Redactor {
	return &Redactor{
		secretQueryNames: nameSet(
			"access_token", "api_key", "apikey", "key", "sig", "signature",
			"token", "x-amz-signature", "x-goog-signature",
		),
		secretHeaderNames: nameSet("authorization", "proxy-authorization", "x-api-key", "apikey"),
	}
}

func nameSet(names ...string) map[string]struct{} {
	set := make(map[string]struct{}, len(names))
	for _, name := range names {
		set[name] = struct{}{}
	}
	return set
}

// AddSecret registers a value to remove from arbitrary diagnostic text.
// Empty values are ignored.
func (r *Redactor) AddSecret(secret string) {
	if secret == "" || slices.Contains(r.secrets, secret) {
		return
	}
	r.secrets = append(r.secrets, secret)
}

// AddSecretHeaderName marks a header name as secret, compared case-insensitively.
func (r *Redactor) AddSecretHeaderName(name string) error {
	if err := validateSecretName(name, isHeaderNameByte); err != nil {
		return err
	}
	r.secretHeaderNames[asciiLower(name)] = struct{}{}
	return nil
}

// AddSecretQueryName marks a query name as secret, compared case-insensitively.
func (r *Redactor) AddSecretQueryName(name string) error {
	if err := validateSecretName(name, isQueryNameByte); err != nil {
		return err
	}
	r.secretQueryNames[asciiLower(name)] = struct{}{}
	return nil
}

// RedactHeader produces a safe header value for diagnostics.
func (r *Redactor) RedactHeader(name, value string) string {
	if _, secret := r.secretHeaderNames[asciiLower(name)]; secret {
		return Redacted
	}
	return r.Redact(value)
}

// Redact removes registered values and the values of secret query parameters.
func (r *Redactor) Redact(value string) string {
	return redactQueryValues(redactSecretValues(value, r.secrets), r.secretQueryNames)
}

// Format prints counts and names only, never a registered value.
func (r *Redactor) Format(f fmt.State, _ rune) {
	_, _ = fmt.Fprintf(f, "Redactor{registered_secret_count: %d, secret_query_names: %v, secret_header_names: %v}",
		len(r.secrets), sortedNames(r.secretQueryNames), sortedNames(r.secretHeaderNames))
}

func sortedNames(set map[string]struct{}) []string {
	names := make([]string, 0, len(set))
	for name := range set {
		names = append(names, name)
	}
	sort.Strings(names)
	return names
}

func validateSecretName(name string, allowed func(byte) bool) error {
	if name == "" {
		return ErrEmptySecretName
	}
	for i := 0; i < len(name); i++ {
		if !allowed(name[i]) {
			return ErrInvalidSecretName
		}
	}
	return nil
}

func isAlphanumeric(b byte) bool {
	return (b >= '0' && b <= '9') || (b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z')
}

// isHeaderNameByte reports whether b is an RFC 9110 tchar.
func isHeaderNameByte(b byte) bool {
	return isAlphanumeric(b) || strings.IndexByte("!#$%&'*+-.^_`|~", b) >= 0
}

func isQueryNameByte(b byte) bool {
	return isAlphanumeric(b) || strings.IndexByte("-._~", b) >= 0
}

func asciiLower(value string) string {
	return strings.Map(func(r rune) rune {
		if r >= 'A' && r <= 'Z' {
			return r + ('a' - 'A')
		}
		return r
	}, value)
}

type byteRange struct{ start, end int }

func redactSecretValues(value string, secrets []string) string {
	var ranges []byteRange
	for _, secret := range secrets {
		searchStart := 0
		for {
			offset := strings.Index(value[searchStart:], secret)
			if offset < 0 {
				break
			}
			start := searchStart + offset
			ranges = append(ranges, byteRange{start, start + len(secret)})
			_, width := utf8.DecodeRuneInString(value[start:])
			if width == 0 {
				break
			}
			searchStart = start + width
		}
	}
	return replaceRanges(value, ranges)
}

func redactQueryValues(value string, names map[string]struct{}) string {
	lowercase := asciiLower(value)
	var ranges []byteRange
	for _, name := range sortedNames(names) {
		needle := name + "="
		for from := 0; ; {
			offset := strings.Index(lowercase[from:], needle)
			if offset < 0 {
				break
			}
			start := from + offset
			from = start + 1
			if start > 0 && strings.IndexByte("?&; '\"", value[start-1]) < 0 {
				continue
			}
			valueStart := start + len(needle)
			valueEnd := len(value)
			if stop := strings.IndexAny(value[valueStart:], "&;# \n\r\t'\""); stop >= 0 {
				valueEnd = valueStart + stop
			}
			ranges = append(ranges, byteRange{valueStart, valueEnd})
		}
	}
	return replaceRanges(value, ranges)
}

func replaceRanges(value string, ranges []byteRange) string {
	if len(ranges) == 0 {
		return value
	}
	sort.Slice(ranges, func(i, j int) bool {
		if ranges[i].start != ranges[j].start {
			return ranges[i].start < ranges[j].start
		}
		return ranges[i].end < ranges[j].end
	})
	disjoint := ranges[:0]
	for _, r := range ranges {
		if n := len(disjoint); n > 0 && r.start <= disjoint[n-1].end {
			disjoint[n-1].end = max(disjoint[n-1].end, r.end)
			continue
		}
		disjoint = append(disjoint, r)
	}
	var b strings.Builder
	last := 0
	for _, r := range disjoint {
		b.WriteString(value[last:r.start])
		b.WriteString(Redacted)
		last = r.end
	}
	b.WriteString(value[last:])
	return b.String()
}
