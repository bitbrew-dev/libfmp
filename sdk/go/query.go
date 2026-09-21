package fmp

import (
	"errors"
	"math"
	"strconv"
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

// Reasons a non-string query argument is rejected. They carry the same
// messages as the corresponding constructors in the Rust crate.
var (
	ErrEmptyTickerList        = errors.New("ticker list must contain at least one ticker")
	ErrNonFiniteDecimal       = errors.New("decimal value must be finite")
	ErrZeroPeriodLength       = errors.New("period length must be a positive integer")
	ErrInvalidCalendarQuarter = errors.New("calendar quarter must be an integer from 1 through 4")
)

// stringParam validates one registry string argument that is not a ticker
// (exchange codes, CIKs, search terms, and so on): non-empty and free of
// control characters, commas allowed, as the Rust string newtypes do.
func stringParam(name, value string) (queryParam, error) {
	if err := validateStringValue(value, false); err != nil {
		return queryParam{}, validationError(name, err)
	}
	return queryParam{Name: name, Value: value}, nil
}

// textParam encodes a registry "text" argument, which the Rust crate takes as
// a plain String without validation.
func textParam(name, value string) (queryParam, error) {
	return queryParam{Name: name, Value: value}, nil
}

// tickerListParam validates a registry "ticker_list" argument the way
// TickerList::new does: at least one ticker, each validated as a ticker, and
// encoded comma-separated in the given order.
func tickerListParam(name string, values []string) (queryParam, error) {
	if len(values) == 0 {
		return queryParam{}, validationError(name, ErrEmptyTickerList)
	}
	for _, value := range values {
		if err := validateStringValue(value, true); err != nil {
			return queryParam{}, validationError(name, err)
		}
	}
	return queryParam{Name: name, Value: strings.Join(values, ",")}, nil
}

// uint32Param encodes the unvalidated u32 newtypes (Limit, Page, Year,
// CalendarYear) in decimal.
func uint32Param(name string, value uint32) (queryParam, error) {
	return queryParam{Name: name, Value: strconv.FormatUint(uint64(value), 10)}, nil
}

// uint64Param encodes the u64 aliases (MarketCapitalization, Volume) in decimal.
func uint64Param(name string, value uint64) (queryParam, error) {
	return queryParam{Name: name, Value: strconv.FormatUint(value, 10)}, nil
}

// periodLengthParam mirrors PeriodLength::new: the value must be positive.
func periodLengthParam(name string, value uint32) (queryParam, error) {
	if value == 0 {
		return queryParam{}, validationError(name, ErrZeroPeriodLength)
	}
	return uint32Param(name, value)
}

// calendarQuarterParam mirrors CalendarQuarter::new: 1 through 4.
func calendarQuarterParam(name string, value uint8) (queryParam, error) {
	if value < 1 || value > 4 {
		return queryParam{}, validationError(name, ErrInvalidCalendarQuarter)
	}
	return queryParam{Name: name, Value: strconv.FormatUint(uint64(value), 10)}, nil
}

// finiteDecimalParam mirrors FiniteDecimal::new: NaN and infinities are
// rejected; the shortest round-trip decimal text is sent, as Rust's Display
// for f64 does.
func finiteDecimalParam(name string, value float64) (queryParam, error) {
	if math.IsNaN(value) || math.IsInf(value, 0) {
		return queryParam{}, validationError(name, ErrNonFiniteDecimal)
	}
	return queryParam{Name: name, Value: strconv.FormatFloat(value, 'f', -1, 64)}, nil
}

// boolParam encodes a bool or TrueFalseFlag argument as "true" or "false".
func boolParam(name string, value bool) (queryParam, error) {
	return queryParam{Name: name, Value: strconv.FormatBool(value)}, nil
}

// dateParam encodes a Date as its exact "YYYY-MM-DD" wire text. The zero
// value has no wire form and is rejected.
func dateParam(name string, value Date) (queryParam, error) {
	if value.IsZero() {
		return queryParam{}, validationError(name, ErrZeroTemporalValue)
	}
	return queryParam{Name: name, Value: value.String()}, nil
}

// dateTimeParam encodes a DateTime as its exact "YYYY-MM-DD HH:MM:SS" wire
// text. The zero value has no wire form and is rejected.
func dateTimeParam(name string, value DateTime) (queryParam, error) {
	if value.IsZero() {
		return queryParam{}, validationError(name, ErrZeroTemporalValue)
	}
	return queryParam{Name: name, Value: value.String()}, nil
}
