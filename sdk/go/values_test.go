package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

type numericForm int

const (
	formNumber numericForm = iota
	formNumericString
	formPercentString
)

// TestNumberOrStringPreservesWireKinds carries the literals of the Rust tests
// mixed_numeric_forms_round_trip_without_f64_coercion and
// percentages_and_boolean_families_preserve_wire_kinds_and_spelling.
func TestNumberOrStringPreservesWireKinds(t *testing.T) {
	t.Parallel()
	cases := []struct {
		wire string
		form numericForm
		text string
	}{
		{`"33644000000"`, formNumericString, "33644000000"},
		{`416161000000`, formNumber, "416161000000"},
		{`9007199254740993`, formNumber, "9007199254740993"},
		{`18446744073709551616`, formNumber, "18446744073709551616"},
		{`4874072686740`, formNumber, "4874072686740"},
		{`0.1`, formNumber, "0.1"},
		{`0.4690516410716045`, formNumber, "0.4690516410716045"},
		{`"0.10335"`, formNumericString, "0.10335"},
		{`"97.26%"`, formPercentString, "97.26%"},
		{`"2025"`, formNumericString, "2025"},
		{`-2025`, formNumber, "-2025"},
		{`"20.25"`, formNumericString, "20.25"},
		{`"-0"`, formNumericString, "-0"},
		{`"1E+5"`, formNumericString, "1E+5"},
		{`"0%"`, formPercentString, "0%"},
	}
	for _, tc := range cases {
		t.Run(tc.wire, func(t *testing.T) {
			t.Parallel()
			var value NumberOrString
			if err := json.Unmarshal([]byte(tc.wire), &value); err != nil {
				t.Fatalf("Unmarshal(%s): %v", tc.wire, err)
			}
			got := [3]bool{value.IsNumber(), value.IsNumericString(), value.IsPercentString()}
			want := [3]bool{tc.form == formNumber, tc.form == formNumericString, tc.form == formPercentString}
			if got != want || value.IsString() != (tc.form != formNumber) || value.IsZero() {
				t.Fatalf("forms = %v, want %v", got, want)
			}
			if value.Text() != tc.text || value.String() != tc.text || string(value.Raw()) != tc.wire {
				t.Fatalf("Text() = %q Raw() = %s, want %q %s", value.Text(), value.Raw(), tc.text, tc.wire)
			}
			encoded, err := json.Marshal(value)
			if err != nil || string(encoded) != tc.wire {
				t.Fatalf("Marshal = %s (%v), want %s", encoded, err, tc.wire)
			}
			direct, err := NewNumberOrString(jsontext.Value(" " + tc.wire + "\n"))
			if err != nil || direct.Text() != tc.text {
				t.Fatalf("NewNumberOrString: %v, Text() = %q", err, direct.Text())
			}
		})
	}
}

// TestNumberOrStringRejectsOtherForms mirrors the rejections of
// NumericString::new, PercentString::new, and the untagged enums.
func TestNumberOrStringRejectsOtherForms(t *testing.T) {
	t.Parallel()
	wires := []string{
		`true`, `null`, `[1]`, `{"value":1}`, `"01"`, `"+1"`, `".5"`, `"5."`, `"1e"`, `"NaN"`,
		`"Infinity"`, `" 1"`, `"1 "`, `""`, `"abc"`, `"%"`, `"25 %"`, `"25%%"`, `"%25"`, `"1_000"`,
	}
	for _, wire := range wires {
		t.Run(wire, func(t *testing.T) {
			t.Parallel()
			var value NumberOrString
			err := json.Unmarshal([]byte(wire), &value)
			if !errors.Is(err, ErrInvalidNumberOrString) {
				t.Fatalf("Unmarshal(%s) = %v, want ErrInvalidNumberOrString", wire, err)
			}
			if !value.IsZero() {
				t.Fatalf("rejected input left %v", value)
			}
			if _, err := NewNumberOrString(jsontext.Value(wire)); !errors.Is(err, ErrInvalidNumberOrString) {
				t.Fatalf("NewNumberOrString(%s) = %v", wire, err)
			}
		})
	}
	if _, err := NewNumberOrString(jsontext.Value(`"abc`)); !errors.Is(err, ErrInvalidNumberOrString) {
		t.Fatalf("malformed raw value: %v", err)
	}
}

func TestNumberOrStringNullAndZeroValueSemantics(t *testing.T) {
	t.Parallel()
	type optional struct {
		Value *NumberOrString `json:"value"`
	}
	for _, wire := range []string{`{"value":null}`, `{}`} {
		var row optional
		if err := json.Unmarshal([]byte(wire), &row); err != nil || row.Value != nil {
			t.Fatalf("%s: value = %v, err = %v", wire, row.Value, err)
		}
	}
	var row optional
	if err := json.Unmarshal([]byte(`{"value":"33644000000"}`), &row); err != nil || row.Value == nil ||
		!row.Value.IsNumericString() {
		t.Fatalf("present optional: %+v, %v", row.Value, err)
	}

	type required struct {
		Value NumberOrString `json:"value"`
	}
	if err := json.Unmarshal([]byte(`{"value":null}`), &required{}); !errors.Is(err, ErrInvalidNumberOrString) {
		t.Fatalf("required null = %v", err)
	}

	encoded, err := json.Marshal(struct {
		Zero    NumberOrString `json:"zero"`
		Omitted NumberOrString `json:"omitted,omitzero"`
	}{})
	if err != nil || string(encoded) != `{"zero":null}` {
		t.Fatalf("zero value = %s (%v)", encoded, err)
	}
	var zero NumberOrString
	if _, ok := zero.Float64(); ok || zero.Text() != "" || zero.IsNumber() || zero.IsString() || zero.Raw() != nil {
		t.Fatalf("zero value accessors: %v", zero)
	}
}

func TestNumberOrStringFloat64NeverScales(t *testing.T) {
	t.Parallel()
	cases := []struct {
		wire string
		want float64
		ok   bool
	}{
		{`0.4690516410716045`, 0.4690516410716045, true},
		{`"0.10335"`, 0.10335, true},
		{`416161000000`, 416161000000, true},
		{`"97.26%"`, 0, false},
		{`1e400`, 0, false},
	}
	for _, tc := range cases {
		var value NumberOrString
		if err := json.Unmarshal([]byte(tc.wire), &value); err != nil {
			t.Fatalf("Unmarshal(%s): %v", tc.wire, err)
		}
		if got, ok := value.Float64(); got != tc.want || ok != tc.ok {
			t.Fatalf("Float64(%s) = %v, %v; want %v, %v", tc.wire, got, ok, tc.want, tc.ok)
		}
	}
}

func TestNumberOrStringDoesNotAliasTheInputBuffer(t *testing.T) {
	t.Parallel()
	input := []byte(`[9007199254740993]`)
	var values []NumberOrString
	if err := json.Unmarshal(input, &values); err != nil {
		t.Fatal(err)
	}
	copy(input, strings.Repeat("0", len(input)))
	if values[0].Text() != "9007199254740993" || string(values[0].Raw()) != "9007199254740993" {
		t.Fatalf("value aliased the input: %v", values[0])
	}
}
