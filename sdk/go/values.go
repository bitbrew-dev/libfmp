package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"fmt"
	"strconv"
	"strings"
)

// Value codecs (ADR 0030, "Codec policy").
//
// The Rust crate keeps every provider value in the form it arrived. The Go
// SDK maps those codecs as follows:
//
//   - NumberOrNumericString and PercentageValue: NumberOrString. The wire
//     bytes are held as a jsontext.Value and never pass through float64.
//   - NumericString and PercentString: string, exact wire text, never scaled.
//   - DynamicJson and DynamicObject: jsontext.Value. Raw bytes are retained,
//     so a 13-digit integer or a decimal such as 0.1 re-encodes byte for
//     byte. Object member order is not semantic, as in the Rust crate.
//   - WireBool: bool. The provider's string flags (YnFlag "Y"/"N", YesNoFlag
//     "Yes"/"No", TrueFalseFlag "true"/"false", TitleCaseBoolFlag
//     "True"/"False") stay string and are never converted; encoding/json/v2
//     rejects a JSON string for a bool field and a JSON boolean for a string
//     field, so the two families cannot blend.

// ErrInvalidNumberOrString reports a JSON value that is neither a number, a
// numeric string, nor a percent string.
var ErrInvalidNumberOrString = errors.New("value must be a JSON number, numeric string, or percent string")

// NumberOrString is a JSON number, a numeric string such as "33644000000",
// or a percent string such as "97.26%", kept exactly as the provider sent it.
//
// It mirrors NumberOrNumericString and PercentageValue in the Rust crate: no
// form is scaled or coerced, and the accessors report which form arrived.
// Numbers re-encode byte for byte; strings re-encode with the same content.
// The zero value is empty: it marshals as null, reports IsZero, and is
// omitted by the omitzero struct tag option. A JSON null is rejected for a
// value-typed field and yields nil for a *NumberOrString field.
type NumberOrString struct {
	raw  jsontext.Value
	text string
}

// NewNumberOrString validates raw with the rules UnmarshalJSONFrom applies:
// a JSON number, a JSON string holding a JSON number with no surrounding
// whitespace, or such a string followed by exactly one "%".
func NewNumberOrString(raw jsontext.Value) (NumberOrString, error) {
	raw = raw.Clone()
	if err := raw.Compact(); err != nil {
		return NumberOrString{}, ErrInvalidNumberOrString
	}
	switch raw.Kind() {
	case jsontext.KindNumber:
		return NumberOrString{raw: raw, text: string(raw)}, nil
	case jsontext.KindString:
		var text string
		if err := json.Unmarshal(raw, &text); err != nil {
			return NumberOrString{}, ErrInvalidNumberOrString
		}
		if number, _ := strings.CutSuffix(text, "%"); !isNumericText(number) {
			return NumberOrString{}, ErrInvalidNumberOrString
		}
		return NumberOrString{raw: raw, text: text}, nil
	default:
		return NumberOrString{}, ErrInvalidNumberOrString
	}
}

// isNumericText mirrors NumericString::new in the Rust crate: the text must
// parse as a JSON number and carry no surrounding whitespace.
func isNumericText(text string) bool {
	value := jsontext.Value(text)
	return text == strings.TrimSpace(text) && value.IsValid() && value.Kind() == jsontext.KindNumber
}

// IsZero reports whether the value is empty. It backs the omitzero tag option.
func (v NumberOrString) IsZero() bool { return v.raw == nil }

// IsNumber reports whether the provider sent a JSON number.
func (v NumberOrString) IsNumber() bool { return v.raw.Kind() == jsontext.KindNumber }

// IsString reports whether the provider sent a JSON string of either form.
func (v NumberOrString) IsString() bool { return v.raw.Kind() == jsontext.KindString }

// IsNumericString reports whether the provider sent a JSON string holding a
// number, such as "0.10335". It mirrors as_numeric_string in the Rust crate.
func (v NumberOrString) IsNumericString() bool { return v.IsString() && !v.IsPercentString() }

// IsPercentString reports whether the provider sent a JSON string ending in
// "%", such as "97.26%". It mirrors as_percent_string in the Rust crate.
func (v NumberOrString) IsPercentString() bool {
	return v.IsString() && strings.HasSuffix(v.text, "%")
}

// Text returns the exact wire text: the number's digits, or the string's
// content without quotes. A percent string keeps its "%". The zero value
// returns "".
func (v NumberOrString) Text() string { return v.text }

// String returns Text.
func (v NumberOrString) String() string { return v.text }

// Raw returns a copy of the wire bytes, including the quotes of a string.
func (v NumberOrString) Raw() jsontext.Value { return v.raw.Clone() }

// Float64 parses a number or numeric string with 64-bit precision. It reports
// false for the zero value, for a percent string (which is never scaled; use
// Text), and when the text does not fit a float64. Digits beyond float64
// precision are rounded; Text keeps them exactly.
func (v NumberOrString) Float64() (float64, bool) {
	if v.IsZero() || v.IsPercentString() {
		return 0, false
	}
	parsed, err := strconv.ParseFloat(v.text, 64)
	if err != nil {
		return 0, false
	}
	return parsed, true
}

// MarshalJSONTo writes the retained wire value, or null for the zero value.
func (v NumberOrString) MarshalJSONTo(enc *jsontext.Encoder) error {
	if v.IsZero() {
		return enc.WriteToken(jsontext.Null)
	}
	return enc.WriteValue(v.raw)
}

// UnmarshalJSONFrom accepts a JSON number, a numeric string, or a percent
// string and rejects every other value, including null.
func (v *NumberOrString) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	raw, err := dec.ReadValue()
	if err != nil {
		return err
	}
	parsed, err := NewNumberOrString(raw)
	if err != nil {
		return fmt.Errorf("fmp: NumberOrString from JSON %v: %w", raw.Kind(), err)
	}
	*v = parsed
	return nil
}
