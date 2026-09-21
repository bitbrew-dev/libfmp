package fmp

import (
	"encoding/json/v2"
	"testing"
)

type optionalTemporalFixture struct {
	DeclarationDate *Date             `json:"declarationDate"`
	FilingDate      *DateTime         `json:"filingDate"`
	Timestamp       *UnixSeconds      `json:"timestamp"`
	Milliseconds    *UnixMilliseconds `json:"milliseconds"`
}

type requiredTemporalFixture struct {
	Date      Date     `json:"date"`
	Timestamp DateTime `json:"timestamp"`
}

type stringPolicyFixture struct {
	IsoTimestamp   string `json:"isoTimestamp"`
	DateOrDateTime string `json:"dateOrDateTime"`
	OpaqueDateText string `json:"opaqueDateText"`
}

func TestOptionalTemporalFieldsDecodeNullAndMissingToNil(t *testing.T) {
	t.Parallel()
	for _, wire := range []string{
		`{"declarationDate":null,"filingDate":null,"timestamp":null,"milliseconds":null}`,
		`{}`,
	} {
		var fixture optionalTemporalFixture
		if err := json.Unmarshal([]byte(wire), &fixture); err != nil {
			t.Fatalf("Unmarshal(%s) error = %v", wire, err)
		}
		if fixture.DeclarationDate != nil || fixture.FilingDate != nil || fixture.Timestamp != nil || fixture.Milliseconds != nil {
			t.Fatalf("Unmarshal(%s) = %+v, want all nil", wire, fixture)
		}
	}

	wire := `{"declarationDate":"2026-07-21","filingDate":"2026-07-30 13:14:23","timestamp":1785430812,"milliseconds":1785430813000}`
	var fixture optionalTemporalFixture
	if err := json.Unmarshal([]byte(wire), &fixture); err != nil {
		t.Fatalf("Unmarshal(present) error = %v", err)
	}
	if fixture.DeclarationDate == nil || fixture.DeclarationDate.String() != "2026-07-21" {
		t.Fatalf("DeclarationDate = %v", fixture.DeclarationDate)
	}
	if fixture.FilingDate == nil || fixture.FilingDate.String() != "2026-07-30 13:14:23" {
		t.Fatalf("FilingDate = %v", fixture.FilingDate)
	}
	if fixture.Timestamp == nil || *fixture.Timestamp != UnixSeconds(1_785_430_812) {
		t.Fatalf("Timestamp = %v", fixture.Timestamp)
	}
	if fixture.Milliseconds == nil || *fixture.Milliseconds != UnixMilliseconds(1_785_430_813_000) {
		t.Fatalf("Milliseconds = %v", fixture.Milliseconds)
	}
	encoded, err := json.Marshal(fixture)
	if err != nil || string(encoded) != wire {
		t.Fatalf("Marshal(present) = %s, %v", encoded, err)
	}
	encoded, err = json.Marshal(optionalTemporalFixture{})
	if err != nil || string(encoded) != `{"declarationDate":null,"filingDate":null,"timestamp":null,"milliseconds":null}` {
		t.Fatalf("Marshal(nil) = %s, %v", encoded, err)
	}
}

func TestOptionalTemporalFieldsRejectMalformedValues(t *testing.T) {
	t.Parallel()
	for _, wire := range []string{
		`{"declarationDate":"02-14-2014"}`,
		`{"declarationDate":"2014-02-30"}`,
		`{"declarationDate":20140214}`,
		`{"filingDate":"2026-07-30T13:14:23Z"}`,
		`{"timestamp":"1785430812"}`,
		`{"milliseconds":1785430813000.0}`,
	} {
		var fixture optionalTemporalFixture
		if err := json.Unmarshal([]byte(wire), &fixture); err == nil {
			t.Fatalf("Unmarshal(%s) accepted", wire)
		}
	}
}

func TestRequiredTemporalFieldsRejectNull(t *testing.T) {
	t.Parallel()
	var fixture requiredTemporalFixture
	wire := `{"date":"2026-04-29","timestamp":"2026-07-30 16:11:45"}`
	if err := json.Unmarshal([]byte(wire), &fixture); err != nil {
		t.Fatalf("Unmarshal(present) error = %v", err)
	}
	if fixture.Date.String() != "2026-04-29" || fixture.Timestamp.String() != "2026-07-30 16:11:45" {
		t.Fatalf("Unmarshal(present) = %+v", fixture)
	}
	for _, wire := range []string{`{"date":null,"timestamp":"2026-07-30 16:11:45"}`, `{"date":"2026-04-29","timestamp":null}`} {
		var rejected requiredTemporalFixture
		err := json.Unmarshal([]byte(wire), &rejected)
		if err == nil {
			t.Fatalf("Unmarshal(%s) accepted null", wire)
		}
	}
	if _, err := json.Marshal(requiredTemporalFixture{}); err == nil {
		t.Fatal("Marshal(zero required fields) invented wire text")
	}
}

// The empty-string sentinel used by the Rust field codecs empty_or_null_date
// and empty_date has no Go representation yet. A plain *Date field rejects
// "" so the gap is visible rather than silently zero-filled.
func TestEmptyStringIsNotAnOptionalDate(t *testing.T) {
	t.Parallel()
	var fixture optionalTemporalFixture
	if err := json.Unmarshal([]byte(`{"declarationDate":""}`), &fixture); err == nil {
		t.Fatal(`Unmarshal("") accepted an empty date`)
	}
}

func TestStringPolicyFieldsKeepExactWireText(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name string
		wire string
	}{
		{"rfc3339 with milliseconds and Z", `{"isoTimestamp":"2026-07-30T16:00:20.049Z","dateOrDateTime":"2023-11-13","opaqueDateText":"--09-27"}`},
		{"rfc3339 with offset and fraction", `{"isoTimestamp":"2026-07-30T16:00:20.049+05:30","dateOrDateTime":"2026-07-30 00:00:00","opaqueDateText":"September 2026"}`},
		{"rfc3339 whole seconds", `{"isoTimestamp":"2026-07-29T04:00:00.000Z","dateOrDateTime":"2026-07-30T00:00:00Z","opaqueDateText":""}`},
		{"non rfc3339 text stays text", `{"isoTimestamp":"2026-07-30 16:00:20","dateOrDateTime":"","opaqueDateText":"Q3 2026"}`},
		{"tipranks samples", `{"isoTimestamp":"2026-07-30T16:40:58.403Z","dateOrDateTime":"2026-07-30","opaqueDateText":"2026-07-29T09:30:14.797Z"}`},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var fixture stringPolicyFixture
			if err := json.Unmarshal([]byte(tc.wire), &fixture); err != nil {
				t.Fatalf("Unmarshal() error = %v", err)
			}
			encoded, err := json.Marshal(fixture)
			if err != nil {
				t.Fatalf("Marshal() error = %v", err)
			}
			if string(encoded) != tc.wire {
				t.Fatalf("round trip = %s, want %s", encoded, tc.wire)
			}
		})
	}
}
