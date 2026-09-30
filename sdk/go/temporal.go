package fmp

import (
	"cmp"
	"encoding/json/jsontext"
	"errors"
	"fmt"
	"strconv"
	"time"
)

// ErrZeroTemporalValue is returned when a zero Date or DateTime is encoded.
// The zero value has no wire representation, so encoding it would invent one.
var ErrZeroTemporalValue = errors.New("zero temporal value cannot be encoded")

// InvalidTemporalValueError reports a wire value that does not match the exact
// temporal format of its field. It mirrors InvalidTemporalValue in the Rust
// crate and never carries the rejected value.
type InvalidTemporalValueError struct {
	// Expected names the wire shape, for example "YYYY-MM-DD date".
	Expected string
}

// Error returns the same message the Rust crate produces.
func (e *InvalidTemporalValueError) Error() string {
	return "value must be a valid " + e.Expected
}

const (
	dateExpected             = "YYYY-MM-DD date"
	dateTimeExpected         = "YYYY-MM-DD HH:MM:SS datetime"
	unixSecondsExpected      = "integer Unix timestamp in seconds"
	unixMillisecondsExpected = "integer Unix timestamp in milliseconds"
)

type separator struct {
	position int
	value    byte
}

var (
	dateSeparators     = []separator{{4, '-'}, {7, '-'}}
	dateTimeSeparators = []separator{{4, '-'}, {7, '-'}, {10, ' '}, {13, ':'}, {16, ':'}}
)

// hasExactASCIIShape mirrors the Rust shape check: the value must have exactly
// the given byte length, the given separators at their positions, and ASCII
// digits everywhere else.
func hasExactASCIIShape(value string, length int, separators []separator) bool {
	if len(value) != length {
		return false
	}
	for index := 0; index < len(value); index++ {
		expected, isSeparator := byte(0), false
		for _, s := range separators {
			if s.position == index {
				expected, isSeparator = s.value, true
				break
			}
		}
		b := value[index]
		if isSeparator {
			if b != expected {
				return false
			}
		} else if b < '0' || b > '9' {
			return false
		}
	}
	return true
}

// asciiDigits converts a run of bytes already verified to be ASCII digits.
func asciiDigits(value string) int {
	n := 0
	for index := 0; index < len(value); index++ {
		n = n*10 + int(value[index]-'0')
	}
	return n
}

func validCivilDate(year int, month time.Month, day int) bool {
	if year < 0 || year > 9999 || month < time.January || month > time.December || day < 1 {
		return false
	}
	y, m, d := time.Date(year, month, day, 0, 0, 0, 0, time.UTC).Date()
	return y == year && m == month && d == day
}

func decodeTemporalString(dec *jsontext.Decoder, expected string) (string, error) {
	if dec.PeekKind() != '"' {
		if err := dec.SkipValue(); err != nil {
			return "", err
		}
		return "", &InvalidTemporalValueError{Expected: expected}
	}
	tok, err := dec.ReadToken()
	if err != nil {
		return "", err
	}
	return tok.String(), nil
}

func encodeTemporalString(enc *jsontext.Encoder, value string, zero bool) error {
	if zero {
		return ErrZeroTemporalValue
	}
	return enc.WriteToken(jsontext.String(value))
}

// Date is a civil calendar date carried on the wire as exactly "YYYY-MM-DD".
// It has no zone and no time of day. The zero value is not a valid date and
// cannot be encoded; IsZero reports it.
type Date struct {
	year  int
	month time.Month
	day   int
}

// NewDate validates a proleptic Gregorian date with a four-digit year.
func NewDate(year int, month time.Month, day int) (Date, error) {
	if !validCivilDate(year, month, day) {
		return Date{}, &InvalidTemporalValueError{Expected: dateExpected}
	}
	return Date{year: year, month: month, day: day}, nil
}

// ParseDate accepts exactly the "YYYY-MM-DD" shape the Rust crate accepts and
// rejects every other spelling, including trailing whitespace and calendar
// values that do not exist.
func ParseDate(value string) (Date, error) {
	if !hasExactASCIIShape(value, 10, dateSeparators) {
		return Date{}, &InvalidTemporalValueError{Expected: dateExpected}
	}
	return NewDate(asciiDigits(value[0:4]), time.Month(asciiDigits(value[5:7])), asciiDigits(value[8:10]))
}

// DateFromTime keeps the civil date of t in t's own location and drops the
// time of day and the zone. It fails when the year has more than four digits.
func DateFromTime(t time.Time) (Date, error) {
	return NewDate(t.Date())
}

// Year returns the four-digit year.
func (d Date) Year() int { return d.year }

// Month returns the calendar month.
func (d Date) Month() time.Month { return d.month }

// Day returns the day of the month.
func (d Date) Day() int { return d.day }

// IsZero reports whether d is the zero value, which is not a valid date.
func (d Date) IsZero() bool { return d == Date{} }

// Compare orders two dates chronologically: -1, 0, or +1.
func (d Date) Compare(other Date) int {
	if d.year != other.year {
		return cmp.Compare(d.year, other.year)
	}
	if d.month != other.month {
		return cmp.Compare(int(d.month), int(other.month))
	}
	return cmp.Compare(d.day, other.day)
}

// Time interprets the date as midnight in loc. A nil loc means UTC. The
// result carries a zone the wire value never had.
func (d Date) Time(loc *time.Location) time.Time {
	if loc == nil {
		loc = time.UTC
	}
	return time.Date(d.year, d.month, d.day, 0, 0, 0, 0, loc)
}

// String returns the exact wire text "YYYY-MM-DD".
func (d Date) String() string {
	return fmt.Sprintf("%04d-%02d-%02d", d.year, int(d.month), d.day)
}

// MarshalText encodes the wire text; the zero value is rejected.
func (d Date) MarshalText() ([]byte, error) {
	if d.IsZero() {
		return nil, ErrZeroTemporalValue
	}
	return []byte(d.String()), nil
}

// UnmarshalText parses the exact wire text.
func (d *Date) UnmarshalText(text []byte) error {
	parsed, err := ParseDate(string(text))
	if err != nil {
		return err
	}
	*d = parsed
	return nil
}

// MarshalJSONTo writes the date as a JSON string; the zero value is rejected.
func (d Date) MarshalJSONTo(enc *jsontext.Encoder) error {
	return encodeTemporalString(enc, d.String(), d.IsZero())
}

// UnmarshalJSONFrom reads one JSON string. Any other JSON kind, including
// null, is rejected, matching the Rust decoder for a required field.
func (d *Date) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	value, err := decodeTemporalString(dec, dateExpected)
	if err != nil {
		return err
	}
	return d.UnmarshalText([]byte(value))
}

// DateTime is a zone-less provider datetime carried on the wire as exactly
// "YYYY-MM-DD HH:MM:SS". Seconds may be 60, as the Rust decoder accepts a
// leap second. The zero value is not a valid datetime and cannot be encoded.
type DateTime struct {
	date   Date
	hour   int
	minute int
	second int
}

// NewDateTime validates a civil date and a wall-clock time. The second may be
// 60 to represent a leap second.
func NewDateTime(year int, month time.Month, day, hour, minute, second int) (DateTime, error) {
	invalid := &InvalidTemporalValueError{Expected: dateTimeExpected}
	date, err := NewDate(year, month, day)
	if err != nil {
		return DateTime{}, invalid
	}
	if hour < 0 || hour > 23 || minute < 0 || minute > 59 || second < 0 || second > 60 {
		return DateTime{}, invalid
	}
	return DateTime{date: date, hour: hour, minute: minute, second: second}, nil
}

// ParseDateTime accepts exactly the "YYYY-MM-DD HH:MM:SS" shape the Rust
// crate accepts: no "T" separator, no zone suffix, no missing seconds.
func ParseDateTime(value string) (DateTime, error) {
	if !hasExactASCIIShape(value, 19, dateTimeSeparators) {
		return DateTime{}, &InvalidTemporalValueError{Expected: dateTimeExpected}
	}
	return NewDateTime(
		asciiDigits(value[0:4]), time.Month(asciiDigits(value[5:7])), asciiDigits(value[8:10]),
		asciiDigits(value[11:13]), asciiDigits(value[14:16]), asciiDigits(value[17:19]),
	)
}

// DateTimeFromTime keeps the wall-clock reading of t in t's own location and
// drops the zone and any sub-second precision.
func DateTimeFromTime(t time.Time) (DateTime, error) {
	year, month, day := t.Date()
	hour, minute, second := t.Clock()
	return NewDateTime(year, month, day, hour, minute, second)
}

// Date returns the civil date part.
func (dt DateTime) Date() Date { return dt.date }

// Clock returns the hour, minute, and second parts.
func (dt DateTime) Clock() (hour, minute, second int) {
	return dt.hour, dt.minute, dt.second
}

// IsZero reports whether dt is the zero value, which is not a valid datetime.
func (dt DateTime) IsZero() bool { return dt == DateTime{} }

// Compare orders two datetimes chronologically: -1, 0, or +1.
func (dt DateTime) Compare(other DateTime) int {
	if c := dt.date.Compare(other.date); c != 0 {
		return c
	}
	if dt.hour != other.hour {
		return cmp.Compare(dt.hour, other.hour)
	}
	if dt.minute != other.minute {
		return cmp.Compare(dt.minute, other.minute)
	}
	return cmp.Compare(dt.second, other.second)
}

// Time interprets the wall-clock reading in loc. A nil loc means UTC. The
// result carries a zone the wire value never had, and a leap second (second
// 60) rolls over into the next minute because time.Time cannot hold it.
func (dt DateTime) Time(loc *time.Location) time.Time {
	if loc == nil {
		loc = time.UTC
	}
	return time.Date(dt.date.year, dt.date.month, dt.date.day, dt.hour, dt.minute, dt.second, 0, loc)
}

// String returns the exact wire text "YYYY-MM-DD HH:MM:SS".
func (dt DateTime) String() string {
	return fmt.Sprintf("%s %02d:%02d:%02d", dt.date.String(), dt.hour, dt.minute, dt.second)
}

// MarshalText encodes the wire text; the zero value is rejected.
func (dt DateTime) MarshalText() ([]byte, error) {
	if dt.IsZero() {
		return nil, ErrZeroTemporalValue
	}
	return []byte(dt.String()), nil
}

// UnmarshalText parses the exact wire text.
func (dt *DateTime) UnmarshalText(text []byte) error {
	parsed, err := ParseDateTime(string(text))
	if err != nil {
		return err
	}
	*dt = parsed
	return nil
}

// MarshalJSONTo writes the datetime as a JSON string; the zero value is rejected.
func (dt DateTime) MarshalJSONTo(enc *jsontext.Encoder) error {
	return encodeTemporalString(enc, dt.String(), dt.IsZero())
}

// UnmarshalJSONFrom reads one JSON string. Any other JSON kind, including
// null, is rejected, matching the Rust decoder for a required field.
func (dt *DateTime) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	value, err := decodeTemporalString(dec, dateTimeExpected)
	if err != nil {
		return err
	}
	return dt.UnmarshalText([]byte(value))
}

// UnixSeconds is a Unix timestamp in whole seconds, carried as a JSON integer.
type UnixSeconds int64

// UnixMilliseconds is a Unix timestamp in whole milliseconds, carried as a
// JSON integer.
type UnixMilliseconds int64

// Time converts the timestamp to a UTC time.Time.
func (s UnixSeconds) Time() time.Time { return time.Unix(int64(s), 0).UTC() }

// String returns the decimal wire text.
func (s UnixSeconds) String() string { return strconv.FormatInt(int64(s), 10) }

// UnmarshalJSONFrom reads one JSON integer. Strings, floats, exponents, and
// null are rejected, matching serde_json decoding into i64.
func (s *UnixSeconds) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	n, err := decodeWireInt64(dec, unixSecondsExpected)
	if err != nil {
		return err
	}
	*s = UnixSeconds(n)
	return nil
}

// Time converts the timestamp to a UTC time.Time.
func (ms UnixMilliseconds) Time() time.Time { return time.UnixMilli(int64(ms)).UTC() }

// String returns the decimal wire text.
func (ms UnixMilliseconds) String() string { return strconv.FormatInt(int64(ms), 10) }

// UnmarshalJSONFrom reads one JSON integer. Strings, floats, exponents, and
// null are rejected, matching serde_json decoding into i64.
func (ms *UnixMilliseconds) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	n, err := decodeWireInt64(dec, unixMillisecondsExpected)
	if err != nil {
		return err
	}
	*ms = UnixMilliseconds(n)
	return nil
}

// decodeWireInt64 reads one JSON value and accepts only an integer token that
// fits in int64. The literal "-0" is a float in serde_json and is rejected too.
func decodeWireInt64(dec *jsontext.Decoder, expected string) (int64, error) {
	value, err := dec.ReadValue()
	if err != nil {
		return 0, err
	}
	invalid := &InvalidTemporalValueError{Expected: expected}
	if value.Kind() != '0' || string(value) == "-0" {
		return 0, invalid
	}
	n, err := strconv.ParseInt(string(value), 10, 64)
	if err != nil {
		return 0, invalid
	}
	return n, nil
}

const usDateExpected = "MM-DD-YYYY date"

var usDateSeparators = []separator{{2, '-'}, {5, '-'}}

// USDate is a civil calendar date carried on the wire as exactly "MM-DD-YYYY",
// the US spelling some fundraising endpoints use. It mirrors USDate in the
// Rust crate: the same date as Date, a different wire text. The zero value is
// not a valid date and cannot be encoded; IsZero reports it.
type USDate struct {
	date Date
}

// NewUSDate validates a proleptic Gregorian date with a four-digit year.
func NewUSDate(year int, month time.Month, day int) (USDate, error) {
	if !validCivilDate(year, month, day) {
		return USDate{}, &InvalidTemporalValueError{Expected: usDateExpected}
	}
	return USDate{date: Date{year: year, month: month, day: day}}, nil
}

// ParseUSDate accepts exactly the "MM-DD-YYYY" shape the Rust crate accepts
// and rejects every other spelling, including "YYYY-MM-DD".
func ParseUSDate(value string) (USDate, error) {
	if !hasExactASCIIShape(value, 10, usDateSeparators) {
		return USDate{}, &InvalidTemporalValueError{Expected: usDateExpected}
	}
	return NewUSDate(asciiDigits(value[6:10]), time.Month(asciiDigits(value[0:2])), asciiDigits(value[3:5]))
}

// Date returns the same calendar date as a Date, which is the type every
// other date member uses.
func (d USDate) Date() Date { return d.date }

// IsZero reports whether d is the zero value, which is not a valid date.
func (d USDate) IsZero() bool { return d == USDate{} }

// String returns the exact wire text "MM-DD-YYYY".
func (d USDate) String() string {
	return fmt.Sprintf("%02d-%02d-%04d", int(d.date.month), d.date.day, d.date.year)
}

// MarshalText encodes the wire text; the zero value is rejected.
func (d USDate) MarshalText() ([]byte, error) {
	if d.IsZero() {
		return nil, ErrZeroTemporalValue
	}
	return []byte(d.String()), nil
}

// UnmarshalText parses the exact wire text.
func (d *USDate) UnmarshalText(text []byte) error {
	parsed, err := ParseUSDate(string(text))
	if err != nil {
		return err
	}
	*d = parsed
	return nil
}

// MarshalJSONTo writes the date as a JSON string; the zero value is rejected.
func (d USDate) MarshalJSONTo(enc *jsontext.Encoder) error {
	return encodeTemporalString(enc, d.String(), d.IsZero())
}

// UnmarshalJSONFrom reads one JSON string. Any other JSON kind, including
// null, is rejected, matching the Rust decoder for a required field.
func (d *USDate) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	value, err := decodeTemporalString(dec, usDateExpected)
	if err != nil {
		return err
	}
	return d.UnmarshalText([]byte(value))
}

const dateOrYearExpected = "YYYY-MM-DD date or YYYY year"

// DateOrYear is a member documented as a "YYYY-MM-DD" date that the provider
// sometimes sends as a bare "YYYY" year. It mirrors DateOrYear in the Rust
// crate: a year stays a year and is never widened to a date. The zero value
// is neither and cannot be encoded; IsZero reports it.
type DateOrYear struct {
	date     Date
	year     int
	yearOnly bool
}

// ParseDateOrYear accepts exactly a "YYYY-MM-DD" date or four ASCII digits,
// the same two shapes the Rust crate accepts.
func ParseDateOrYear(value string) (DateOrYear, error) {
	switch len(value) {
	case 10:
		date, err := ParseDate(value)
		if err != nil {
			return DateOrYear{}, err
		}
		return DateOrYear{date: date}, nil
	case 4:
		if hasExactASCIIShape(value, 4, nil) {
			return DateOrYear{year: asciiDigits(value), yearOnly: true}, nil
		}
	}
	return DateOrYear{}, &InvalidTemporalValueError{Expected: dateOrYearExpected}
}

// Date returns the full date and true, or false when only a year arrived.
func (v DateOrYear) Date() (Date, bool) {
	return v.date, !v.yearOnly && !v.date.IsZero()
}

// Year returns the bare year and true, or false when a full date arrived.
func (v DateOrYear) Year() (int, bool) {
	return v.year, v.yearOnly
}

// IsZero reports whether v is the zero value, which holds neither form.
func (v DateOrYear) IsZero() bool { return v == DateOrYear{} }

// String returns the exact wire text, "YYYY-MM-DD" or "YYYY".
func (v DateOrYear) String() string {
	if v.yearOnly {
		return fmt.Sprintf("%04d", v.year)
	}
	return v.date.String()
}

// MarshalText encodes the wire text; the zero value is rejected.
func (v DateOrYear) MarshalText() ([]byte, error) {
	if v.IsZero() {
		return nil, ErrZeroTemporalValue
	}
	return []byte(v.String()), nil
}

// UnmarshalText parses the exact wire text.
func (v *DateOrYear) UnmarshalText(text []byte) error {
	parsed, err := ParseDateOrYear(string(text))
	if err != nil {
		return err
	}
	*v = parsed
	return nil
}

// MarshalJSONTo writes the value as a JSON string; the zero value is rejected.
func (v DateOrYear) MarshalJSONTo(enc *jsontext.Encoder) error {
	return encodeTemporalString(enc, v.String(), v.IsZero())
}

// UnmarshalJSONFrom reads one JSON string. Any other JSON kind, including
// null, is rejected, matching the Rust decoder for a required field.
func (v *DateOrYear) UnmarshalJSONFrom(dec *jsontext.Decoder) error {
	value, err := decodeTemporalString(dec, dateOrYearExpected)
	if err != nil {
		return err
	}
	return v.UnmarshalText([]byte(value))
}
