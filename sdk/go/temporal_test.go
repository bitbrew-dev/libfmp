package fmp

import (
	"encoding/json/v2"
	"errors"
	"testing"
	"time"
)

func mustParseDate(t *testing.T, value string) Date {
	t.Helper()
	date, err := ParseDate(value)
	if err != nil {
		t.Fatalf("ParseDate(%q) error = %v", value, err)
	}
	return date
}

func mustParseDateTime(t *testing.T, value string) DateTime {
	t.Helper()
	datetime, err := ParseDateTime(value)
	if err != nil {
		t.Fatalf("ParseDateTime(%q) error = %v", value, err)
	}
	return datetime
}

func assertTemporalError(t *testing.T, err error, expected string) {
	t.Helper()
	var invalid *InvalidTemporalValueError
	if !errors.As(err, &invalid) {
		t.Fatalf("error = %v, want *InvalidTemporalValueError", err)
	}
	if invalid.Expected != expected {
		t.Fatalf("Expected = %q, want %q", invalid.Expected, expected)
	}
	if want := "value must be a valid " + expected; invalid.Error() != want {
		t.Fatalf("Error() = %q, want %q", invalid.Error(), want)
	}
}

func TestParseDateMirrorsRustAcceptedAndRejectedInputs(t *testing.T) {
	t.Parallel()
	accepted := []string{"2024-02-29", "2014-02-14", "0000-01-01", "2000-02-29", "9999-12-31"}
	for _, value := range accepted {
		if got := mustParseDate(t, value).String(); got != value {
			t.Fatalf("ParseDate(%q).String() = %q", value, got)
		}
	}
	rejected := []string{
		"2024-2-29", "24-02-29", "2024/02/29", "2023-02-29", "2024-02-29 ",
		"02-14-2014", "2014-02-30", "1900-02-29", "2024-00-10", "2024-13-01",
		"2024-04-31", "2024-01-00", "", "2024-02-29T00:00:00", "２０２４-02-29",
	}
	for _, value := range rejected {
		_, err := ParseDate(value)
		if err == nil {
			t.Fatalf("ParseDate(%q) accepted", value)
		}
		assertTemporalError(t, err, "YYYY-MM-DD date")
	}
}

func TestDateAccessorsComparisonAndConversion(t *testing.T) {
	t.Parallel()
	date := mustParseDate(t, "2024-02-29")
	if date.Year() != 2024 || date.Month() != time.February || date.Day() != 29 {
		t.Fatalf("accessors = %d %v %d", date.Year(), date.Month(), date.Day())
	}
	if date.IsZero() || !(Date{}).IsZero() {
		t.Fatal("IsZero() mismatch")
	}
	if built, err := NewDate(2024, time.February, 29); err != nil || built != date {
		t.Fatalf("NewDate() = %v, %v", built, err)
	}
	if _, err := NewDate(2023, time.February, 29); err == nil {
		t.Fatal("NewDate(2023-02-29) accepted")
	}
	later := mustParseDate(t, "2024-03-01")
	if date.Compare(later) != -1 || later.Compare(date) != 1 || date.Compare(date) != 0 {
		t.Fatal("Compare() ordering mismatch")
	}
	if got := date.Time(nil); !got.Equal(time.Date(2024, 2, 29, 0, 0, 0, 0, time.UTC)) || got.Location() != time.UTC {
		t.Fatalf("Time(nil) = %v", got)
	}
	zone := time.FixedZone("plus-five-thirty", 5*3600+1800)
	if got := date.Time(zone); got.Location() != zone || got.Hour() != 0 {
		t.Fatalf("Time(zone) = %v", got)
	}
	fromTime, err := DateFromTime(time.Date(2024, 2, 29, 23, 59, 59, 0, zone))
	if err != nil || fromTime != date {
		t.Fatalf("DateFromTime() = %v, %v", fromTime, err)
	}
	if _, err := DateFromTime(time.Date(10000, 1, 1, 0, 0, 0, 0, time.UTC)); err == nil {
		t.Fatal("DateFromTime(year 10000) accepted")
	}
}

func TestDateJSONAndTextRoundTripByteIdentical(t *testing.T) {
	t.Parallel()
	date := mustParseDate(t, "2024-02-29")
	encoded, err := json.Marshal(date)
	if err != nil || string(encoded) != `"2024-02-29"` {
		t.Fatalf("Marshal() = %s, %v", encoded, err)
	}
	var decoded Date
	if err := json.Unmarshal([]byte(`"2024-02-29"`), &decoded); err != nil || decoded != date {
		t.Fatalf("Unmarshal() = %v, %v", decoded, err)
	}
	text, err := date.MarshalText()
	if err != nil || string(text) != "2024-02-29" {
		t.Fatalf("MarshalText() = %s, %v", text, err)
	}
	var fromText Date
	if err := fromText.UnmarshalText([]byte("2024-02-29")); err != nil || fromText != date {
		t.Fatalf("UnmarshalText() = %v, %v", fromText, err)
	}
	if err := fromText.UnmarshalText([]byte("2024/02/29")); err == nil {
		t.Fatal("UnmarshalText accepted a slash date")
	}
	if _, err := json.Marshal(Date{}); !errors.Is(err, ErrZeroTemporalValue) {
		t.Fatalf("Marshal(zero) error = %v", err)
	}
	if _, err := (Date{}).MarshalText(); !errors.Is(err, ErrZeroTemporalValue) {
		t.Fatalf("MarshalText(zero) error = %v", err)
	}
}

func TestDateRejectsEveryNonStringJSONKind(t *testing.T) {
	t.Parallel()
	for _, wire := range []string{`null`, `20240229`, `true`, `{"date":"2024-02-29"}`, `["2024-02-29"]`, `"2023-02-29"`} {
		var date Date
		err := json.Unmarshal([]byte(wire), &date)
		if err == nil {
			t.Fatalf("Unmarshal(%s) accepted", wire)
		}
		assertTemporalError(t, err, "YYYY-MM-DD date")
	}
}

func TestParseDateTimeMirrorsRustAcceptedAndRejectedInputs(t *testing.T) {
	t.Parallel()
	accepted := []string{
		"2024-02-29 23:59:58", "2024-02-29 23:59:60", "2024-12-31 23:59:60",
		"0000-01-01 00:00:00", "2026-07-30 08:07:21", "2023-12-28 09:26:13",
		"2024-03-04 00:00:00", "2026-07-30 00:00:00",
	}
	for _, value := range accepted {
		if got := mustParseDateTime(t, value).String(); got != value {
			t.Fatalf("ParseDateTime(%q).String() = %q", value, got)
		}
	}
	rejected := []string{
		"2024-02-29T23:59:58", "2024-02-29 23:59", "2024-02-29 23:59:58Z",
		"2024-02-29 24:00:00", "2023-02-29 12:00:00", "2024-02-29 23:60:00",
		"2024-02-29 23:59:61", "2024-02-29", "2024-02-29 23:59:58 ", "",
	}
	for _, value := range rejected {
		_, err := ParseDateTime(value)
		if err == nil {
			t.Fatalf("ParseDateTime(%q) accepted", value)
		}
		assertTemporalError(t, err, "YYYY-MM-DD HH:MM:SS datetime")
	}
}

func TestDateTimeAccessorsComparisonAndConversion(t *testing.T) {
	t.Parallel()
	datetime := mustParseDateTime(t, "2024-02-29 23:59:58")
	if datetime.Date() != mustParseDate(t, "2024-02-29") {
		t.Fatalf("Date() = %v", datetime.Date())
	}
	if hour, minute, second := datetime.Clock(); hour != 23 || minute != 59 || second != 58 {
		t.Fatalf("Clock() = %d %d %d", hour, minute, second)
	}
	if datetime.IsZero() || !(DateTime{}).IsZero() {
		t.Fatal("IsZero() mismatch")
	}
	if built, err := NewDateTime(2024, time.February, 29, 23, 59, 58); err != nil || built != datetime {
		t.Fatalf("NewDateTime() = %v, %v", built, err)
	}
	if _, err := NewDateTime(2024, time.February, 29, 24, 0, 0); err == nil {
		t.Fatal("NewDateTime(hour 24) accepted")
	}
	later := mustParseDateTime(t, "2024-02-29 23:59:59")
	if datetime.Compare(later) != -1 || later.Compare(datetime) != 1 || datetime.Compare(datetime) != 0 {
		t.Fatal("Compare() ordering mismatch")
	}
	zone := time.FixedZone("minus-four", -4*3600)
	if got := datetime.Time(zone); got.Location() != zone || !got.Equal(time.Date(2024, 2, 29, 23, 59, 58, 0, zone)) {
		t.Fatalf("Time(zone) = %v", got)
	}
	if got := datetime.Time(nil); got.Location() != time.UTC {
		t.Fatalf("Time(nil) location = %v", got.Location())
	}
	leap := mustParseDateTime(t, "2024-02-29 23:59:60")
	if got := leap.Time(nil); !got.Equal(time.Date(2024, 3, 1, 0, 0, 0, 0, time.UTC)) {
		t.Fatalf("leap second Time() = %v", got)
	}
	fromTime, err := DateTimeFromTime(time.Date(2024, 2, 29, 23, 59, 58, 123456789, zone))
	if err != nil || fromTime != datetime {
		t.Fatalf("DateTimeFromTime() = %v, %v", fromTime, err)
	}
}

func TestDateTimeJSONAndTextRoundTripByteIdentical(t *testing.T) {
	t.Parallel()
	for _, wire := range []string{"2024-02-29 23:59:58", "2024-02-29 23:59:60"} {
		datetime := mustParseDateTime(t, wire)
		encoded, err := json.Marshal(datetime)
		if err != nil || string(encoded) != `"`+wire+`"` {
			t.Fatalf("Marshal(%q) = %s, %v", wire, encoded, err)
		}
		var decoded DateTime
		if err := json.Unmarshal(encoded, &decoded); err != nil || decoded != datetime {
			t.Fatalf("Unmarshal(%s) = %v, %v", encoded, decoded, err)
		}
		text, err := datetime.MarshalText()
		if err != nil || string(text) != wire {
			t.Fatalf("MarshalText() = %s, %v", text, err)
		}
		var fromText DateTime
		if err := fromText.UnmarshalText(text); err != nil || fromText != datetime {
			t.Fatalf("UnmarshalText() = %v, %v", fromText, err)
		}
	}
	if _, err := json.Marshal(DateTime{}); !errors.Is(err, ErrZeroTemporalValue) {
		t.Fatalf("Marshal(zero) error = %v", err)
	}
	for _, wire := range []string{`null`, `1700000000`, `"2024-02-29T23:59:58"`, `{}`} {
		var datetime DateTime
		err := json.Unmarshal([]byte(wire), &datetime)
		if err == nil {
			t.Fatalf("Unmarshal(%s) accepted", wire)
		}
		assertTemporalError(t, err, "YYYY-MM-DD HH:MM:SS datetime")
	}
}

func TestUnixTimestampsKeepSecondsAndMillisecondsDistinct(t *testing.T) {
	t.Parallel()
	var seconds UnixSeconds
	if err := json.Unmarshal([]byte("1700000000"), &seconds); err != nil || seconds != UnixSeconds(1_700_000_000) {
		t.Fatalf("UnixSeconds = %d, %v", seconds, err)
	}
	var milliseconds UnixMilliseconds
	if err := json.Unmarshal([]byte("1700000000000"), &milliseconds); err != nil || milliseconds != UnixMilliseconds(1_700_000_000_000) {
		t.Fatalf("UnixMilliseconds = %d, %v", milliseconds, err)
	}
	if seconds.String() != "1700000000" || milliseconds.String() != "1700000000000" {
		t.Fatalf("String() = %s, %s", seconds, milliseconds)
	}
	if encoded, err := json.Marshal(seconds); err != nil || string(encoded) != "1700000000" {
		t.Fatalf("Marshal(seconds) = %s, %v", encoded, err)
	}
	if encoded, err := json.Marshal(milliseconds); err != nil || string(encoded) != "1700000000000" {
		t.Fatalf("Marshal(milliseconds) = %s, %v", encoded, err)
	}
	want := time.Date(2023, 11, 14, 22, 13, 20, 0, time.UTC)
	if got := seconds.Time(); !got.Equal(want) || got.Location() != time.UTC {
		t.Fatalf("UnixSeconds.Time() = %v", got)
	}
	if got := milliseconds.Time(); !got.Equal(want) || got.Location() != time.UTC {
		t.Fatalf("UnixMilliseconds.Time() = %v", got)
	}
	if got := UnixMilliseconds(1_785_430_813_000).Time(); got.Nanosecond() != 0 || got.Unix() != 1_785_430_813 {
		t.Fatalf("UnixMilliseconds(1785430813000).Time() = %v", got)
	}
}

func TestUnixTimestampsAcceptEveryInt64AndRejectOtherTokens(t *testing.T) {
	t.Parallel()
	for _, wire := range []string{"9223372036854775807", "-9223372036854775808", "-1", "0"} {
		var seconds UnixSeconds
		var milliseconds UnixMilliseconds
		if err := json.Unmarshal([]byte(wire), &seconds); err != nil || seconds.String() != wire {
			t.Fatalf("UnixSeconds(%s) = %s, %v", wire, seconds, err)
		}
		if err := json.Unmarshal([]byte(wire), &milliseconds); err != nil || milliseconds.String() != wire {
			t.Fatalf("UnixMilliseconds(%s) = %s, %v", wire, milliseconds, err)
		}
		_ = seconds.Time()
		_ = milliseconds.Time()
	}
	rejected := []string{`"1"`, `1.0`, `1e3`, `-0`, `null`, `true`, `9223372036854775808`, `[1]`, `{}`}
	for _, wire := range rejected {
		var seconds UnixSeconds
		err := json.Unmarshal([]byte(wire), &seconds)
		if err == nil {
			t.Fatalf("UnixSeconds accepted %s", wire)
		}
		assertTemporalError(t, err, "integer Unix timestamp in seconds")
		var milliseconds UnixMilliseconds
		err = json.Unmarshal([]byte(wire), &milliseconds)
		if err == nil {
			t.Fatalf("UnixMilliseconds accepted %s", wire)
		}
		assertTemporalError(t, err, "integer Unix timestamp in milliseconds")
	}
}
