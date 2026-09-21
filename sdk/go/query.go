package fmp

import (
	"errors"
	"strings"
)

// Reasons a string-like query argument is rejected. They carry the same
// messages as StringValueError in the Rust crate.
var (
	ErrEmptyValue            = errors.New("value must not be empty or whitespace-only")
	ErrControlCharacterValue = errors.New("value must not contain control characters")
	ErrCommaInTicker         = errors.New("ticker must not contain a comma")
)

// validateStringValue mirrors validate_string_value in the Rust crate: the
// original text is preserved and never normalized.
func validateStringValue(value string, rejectComma bool) error {
	if strings.TrimSpace(value) == "" {
		return ErrEmptyValue
	}
	if hasControlCharacter(value) {
		return ErrControlCharacterValue
	}
	if rejectComma && strings.Contains(value, ",") {
		return ErrCommaInTicker
	}
	return nil
}

// tickerParam validates one registry "ticker" argument the way Ticker::new
// does and returns its query pair. The Rust crate validates when the Ticker
// is constructed; the Go SDK validates when the request is built, so the
// failure surfaces as a Validation error from the endpoint method.
func tickerParam(name, value string) (queryParam, error) {
	if err := validateStringValue(value, true); err != nil {
		return queryParam{}, validationError(name, err)
	}
	return queryParam{Name: name, Value: value}, nil
}
