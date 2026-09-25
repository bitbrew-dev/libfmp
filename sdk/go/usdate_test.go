package fmp

import (
	"encoding/json/v2"
	"errors"
	"testing"
	"time"
)

func TestUSDateAcceptsExactlyTheUSWireShape(t *testing.T) {
	t.Parallel()
	parsed, err := ParseUSDate("02-29-2024")
	if err != nil {
		t.Fatal(err)
	}
	if parsed.String() != "02-29-2024" || parsed.Date().String() != "2024-02-29" ||
		parsed.Date().Month() != time.February {
		t.Fatalf("parsed = %s (%s)", parsed, parsed.Date())
	}
	for _, bad := range []string{"2024-02-29", "02-30-2024", "2-29-2024", "02-29-2024 ", "13-01-2024", ""} {
		if _, err := ParseUSDate(bad); err == nil {
			t.Fatalf("%q was accepted", bad)
		}
	}
	var invalid *InvalidTemporalValueError
	_, err = ParseUSDate("2024-02-29")
	if !errors.As(err, &invalid) || invalid.Expected != "MM-DD-YYYY date" {
		t.Fatalf("error = %v, want the MM-DD-YYYY expectation", err)
	}
}

func TestUSDateRoundTripsThroughJSONAndRejectsZero(t *testing.T) {
	t.Parallel()
	var row struct {
		Date    USDate  `json:"date"`
		Missing *USDate `json:"missing"`
	}
	if err := json.Unmarshal([]byte(`{"date":"07-04-2026","missing":null}`), &row); err != nil {
		t.Fatal(err)
	}
	if row.Date.Date().Year() != 2026 || row.Missing != nil {
		t.Fatalf("row = %+v", row)
	}
	encoded, err := json.Marshal(row)
	if err != nil {
		t.Fatal(err)
	}
	if string(encoded) != `{"date":"07-04-2026","missing":null}` {
		t.Fatalf("encoded = %s", encoded)
	}
	if err := json.Unmarshal([]byte(`{"date":null}`), &row); err == nil {
		t.Fatal("null decoded into a value-typed USDate")
	}
	if err := json.Unmarshal([]byte(`{"date":"2026-07-04"}`), &row); err == nil {
		t.Fatal("YYYY-MM-DD decoded into a USDate")
	}
	if _, err := json.Marshal(USDate{}); !errors.Is(err, ErrZeroTemporalValue) {
		t.Fatalf("zero USDate encoded: %v", err)
	}
}
