package fmp

import (
	"fmt"
	"os"
	"strings"
)

// EnvAPIKey is the process environment variable read by FMPHeaderFromEnv.
const EnvAPIKey = "FMP_API_KEY"

const (
	fmpHeaderName = "apikey"
	fmpQueryName  = "apikey"
	bearerHeader  = "authorization"
	bearerPrefix  = "Bearer "
)

type authMode uint8

const (
	authNone authMode = iota
	authFMPHeader
	authFMPQuery
	authBearer
	authCustomHeader
	authCustomQuery
)

// Authentication selects how the transport attaches a credential to every
// request. The zero value sends no credential. Formatting an Authentication
// with any fmt verb never reveals the secret.
type Authentication struct {
	mode   authMode
	name   string
	prefix string
	secret string
}

// FMPHeader sends the FMP API key in the exact "apikey" header.
func FMPHeader(apiKey string) Authentication {
	return Authentication{mode: authFMPHeader, name: fmpHeaderName, secret: apiKey}
}

// APIKeyFromEnv reads FMP_API_KEY, trims surrounding whitespace, and
// reports false when the variable is unset, empty, or whitespace-only.
func APIKeyFromEnv() (string, bool) {
	value := strings.TrimSpace(os.Getenv(EnvAPIKey))
	return value, value != ""
}

// FMPHeaderFromEnv builds FMPHeader authentication from FMP_API_KEY using the
// normalization of APIKeyFromEnv. NewClient never reads a credential from
// the environment on its own, so callers opt in explicitly.
func FMPHeaderFromEnv() (Authentication, bool) {
	key, ok := APIKeyFromEnv()
	if !ok {
		return Authentication{}, false
	}
	return FMPHeader(key), true
}

// FMPQuery sends the FMP API key in the "apikey" query parameter.
func FMPQuery(apiKey string) Authentication {
	return Authentication{mode: authFMPQuery, name: fmpQueryName, secret: apiKey}
}

// Bearer sends an RFC 6750 "Authorization: Bearer ..." header.
func Bearer(token string) Authentication {
	return Authentication{mode: authBearer, name: bearerHeader, secret: token}
}

// CustomHeader sends a secret in a caller-selected header. It is
// CustomHeaderWithPrefix with an empty prefix.
func CustomHeader(name, secret string) Authentication {
	return CustomHeaderWithPrefix(name, "", secret)
}

// CustomHeaderWithPrefix sends a secret in a caller-selected header with
// non-secret text placed immediately before it. The header value is exactly
// prefix followed by secret; no separator is inserted, so a "Bearer " prefix
// must carry its own trailing space. It mirrors the Rust
// Authentication::custom_header(name, Some(prefix), secret) form, and an empty
// prefix is the same value as CustomHeader(name, secret) (Rust's Some("")
// differs from None in Debug and equality, not on the wire). The prefix is
// validated like a header value when the client is built. Formatting the
// value reports whether a prefix is present, never its text.
func CustomHeaderWithPrefix(name, prefix, secret string) Authentication {
	return Authentication{mode: authCustomHeader, name: name, prefix: prefix, secret: secret}
}

// CustomQuery sends a secret in a caller-selected query parameter. The name
// must be non-empty and use only ASCII letters, digits, and "-._~", the
// characters the Redactor can protect; NewClient rejects any other name with
// ConfigurationKindInvalidQueryName.
func CustomQuery(name, secret string) Authentication {
	return Authentication{mode: authCustomQuery, name: name, secret: secret}
}

// String describes the mode without the secret.
func (a Authentication) String() string {
	switch a.mode {
	case authNone:
		return "None"
	case authFMPHeader:
		return "FMPHeader([REDACTED])"
	case authFMPQuery:
		return "FMPQuery([REDACTED])"
	case authBearer:
		return "Bearer([REDACTED])"
	case authCustomHeader:
		return fmt.Sprintf("CustomHeader{name: %q, has_prefix: %t, secret: [REDACTED]}",
			a.name, a.prefix != "")
	case authCustomQuery:
		return fmt.Sprintf("CustomQuery{name: %q, secret: [REDACTED]}", a.name)
	default:
		return "Authentication(?)"
	}
}

// Format prints the String description for every verb, including %+v and %#v.
func (a Authentication) Format(f fmt.State, _ rune) {
	_, _ = fmt.Fprint(f, a.String())
}

// authMaterial is the validated, transport-ready form of an Authentication.
type authMaterial struct {
	headerName  string
	headerValue string
	queryName   string
	querySecret string
}

func buildAuthMaterial(a Authentication) (authMaterial, error) {
	switch a.mode {
	case authNone:
		return authMaterial{}, nil
	case authFMPHeader:
		return secretHeader(fmpHeaderName, "", a.secret)
	case authFMPQuery:
		if err := validateCredential(a.secret); err != nil {
			return authMaterial{}, err
		}
		return authMaterial{queryName: fmpQueryName, querySecret: a.secret}, nil
	case authBearer:
		return secretHeader(bearerHeader, bearerPrefix, a.secret)
	case authCustomHeader:
		return secretHeader(a.name, a.prefix, a.secret)
	case authCustomQuery:
		if err := validateQueryName(a.name); err != nil {
			return authMaterial{}, err
		}
		if err := validateCredential(a.secret); err != nil {
			return authMaterial{}, err
		}
		return authMaterial{queryName: a.name, querySecret: a.secret}, nil
	default:
		return authMaterial{}, configurationError(ConfigurationKindConflictingAuthentication,
			"authentication mode is not supported")
	}
}

func secretHeader(name, prefix, secret string) (authMaterial, error) {
	if err := validateCredential(secret); err != nil {
		return authMaterial{}, err
	}
	if !validHeaderName(name) {
		return authMaterial{}, configurationError(ConfigurationKindInvalidHeaderName,
			"header name is not a valid HTTP field name")
	}
	if isUnsafeTransportHeader(name) {
		return authMaterial{}, configurationError(ConfigurationKindProtectedFieldCollision,
			"authentication header is owned by HTTP framing or transport")
	}
	value := prefix + secret
	if !validHeaderValue(value) {
		return authMaterial{}, configurationError(ConfigurationKindInvalidHeaderValue,
			"header value is not a valid HTTP field value")
	}
	return authMaterial{headerName: asciiLower(name), headerValue: value}, nil
}

func validateCredential(secret string) error {
	if secret == "" {
		return configurationError(ConfigurationKindEmptyCredential,
			"authentication credential must not be empty")
	}
	return nil
}

// validateQueryName applies the Redactor's secret query name rule (ASCII
// letters, digits, and "-._~") when the client is built, so a name the
// Redactor could not protect is rejected before it is ever sent.
func validateQueryName(name string) error {
	if validateSecretName(name, isQueryNameByte) != nil {
		return configurationError(ConfigurationKindInvalidQueryName,
			"secret query name must be non-empty ASCII letters, digits, or -._~")
	}
	return nil
}

func registerAuthRedaction(redactor *Redactor, a Authentication) error {
	switch a.mode {
	case authNone:
	case authFMPHeader, authBearer:
		redactor.AddSecret(a.secret)
	case authFMPQuery:
		redactor.AddSecret(a.secret)
		if err := redactor.AddSecretQueryName(fmpQueryName); err != nil {
			return configurationError(ConfigurationKindInvalidQueryName,
				"invalid built-in secret query name")
		}
	case authCustomHeader:
		redactor.AddSecret(a.secret)
		if err := redactor.AddSecretHeaderName(a.name); err != nil {
			return configurationError(ConfigurationKindInvalidHeaderName,
				"custom secret header name is invalid")
		}
	case authCustomQuery:
		redactor.AddSecret(a.secret)
		if err := redactor.AddSecretQueryName(a.name); err != nil {
			return configurationError(ConfigurationKindInvalidQueryName,
				"custom secret query name is invalid")
		}
	}
	return nil
}
