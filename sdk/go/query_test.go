package fmp

import (
	"errors"
	"math"
	"testing"
	"time"
)

// assertParam checks a helper's success path: the exact pair, no error.
func assertParam(t *testing.T, got queryParam, err error, name, value string) {
	t.Helper()
	if err != nil {
		t.Fatalf("%s: unexpected error %v", name, err)
	}
	if got.Name != name || got.Value != value {
		t.Fatalf("param = %+v, want %s=%s", got, name, value)
	}
}

// assertRejected checks a helper's failure path: a Validation *Error whose
// message is "<name>: <reason>" and whose cause is the sentinel.
func assertRejected(t *testing.T, err error, name string, reason error) {
	t.Helper()
	var typed *Error
	if !errors.As(err, &typed) || typed.Category != CategoryValidation {
		t.Fatalf("error = %v (%T), want a CategoryValidation *Error", err, err)
	}
	if !errors.Is(err, reason) || typed.Message != name+": "+reason.Error() {
		t.Fatalf("error = %q, want %s: %v", typed.Message, name, reason)
	}
}

func TestStringKindsValidateLikeTheRustNewtypes(t *testing.T) {
	t.Parallel()
	got, err := stringParam("exchange", "NASDAQ")
	assertParam(t, got, err, "exchange", "NASDAQ")
	got, err = stringParam("query", "Apple, Inc")
	assertParam(t, got, err, "query", "Apple, Inc")
	_, err = stringParam("exchange", " ")
	assertRejected(t, err, "exchange", ErrEmptyValue)
	_, err = stringParam("exchange", "NAS\x00DAQ")
	assertRejected(t, err, "exchange", ErrControlCharacterValue)

	got, err = textParam("query", "")
	assertParam(t, got, err, "query", "")

	got, err = tickerListParam("symbols", []string{"AAPL", "^VIX", "000001.SZ"})
	assertParam(t, got, err, "symbols", "AAPL,^VIX,000001.SZ")
	_, err = tickerListParam("symbols", nil)
	assertRejected(t, err, "symbols", ErrEmptyTickerList)
	_, err = tickerListParam("symbols", []string{"AAPL", "MSFT,GOOG"})
	assertRejected(t, err, "symbols", ErrCommaInTicker)
	_, err = tickerListParam("symbols", []string{"AAPL", ""})
	assertRejected(t, err, "symbols", ErrEmptyValue)
}

func TestNumericKindsEncodeTheRustWireText(t *testing.T) {
	t.Parallel()
	got, err := uint32Param("limit", 0)
	assertParam(t, got, err, "limit", "0")
	got, err = uint32Param("page", math.MaxUint32)
	assertParam(t, got, err, "page", "4294967295")
	got, err = uint64Param("marketCapMoreThan", 4_874_072_686_740)
	assertParam(t, got, err, "marketCapMoreThan", "4874072686740")

	got, err = periodLengthParam("periodLength", 14)
	assertParam(t, got, err, "periodLength", "14")
	_, err = periodLengthParam("periodLength", 0)
	assertRejected(t, err, "periodLength", ErrZeroPeriodLength)

	for quarter := uint8(1); quarter <= 4; quarter++ {
		got, err = calendarQuarterParam("quarter", quarter)
		assertParam(t, got, err, "quarter", string(rune('0'+quarter)))
	}
	for _, quarter := range []uint8{0, 5} {
		_, err = calendarQuarterParam("quarter", quarter)
		assertRejected(t, err, "quarter", ErrInvalidCalendarQuarter)
	}

	cases := []struct {
		value float64
		want  string
	}{
		{1.5, "1.5"}, {-0.25, "-0.25"}, {100, "100"}, {1e21, "1000000000000000000000"}, {0.1, "0.1"},
	}
	for _, tc := range cases {
		got, err = finiteDecimalParam("beta", tc.value)
		assertParam(t, got, err, "beta", tc.want)
	}
	for _, value := range []float64{math.NaN(), math.Inf(1), math.Inf(-1)} {
		_, err = finiteDecimalParam("beta", value)
		assertRejected(t, err, "beta", ErrNonFiniteDecimal)
	}

	got, err = boolParam("short", true)
	assertParam(t, got, err, "short", "true")
	got, err = boolParam("includeSpread", false)
	assertParam(t, got, err, "includeSpread", "false")
}

func TestTemporalKindsEncodeExactWireTextAndRejectZero(t *testing.T) {
	t.Parallel()
	date, err := NewDate(2024, time.February, 29)
	if err != nil {
		t.Fatal(err)
	}
	got, err := dateParam("from", date)
	assertParam(t, got, err, "from", "2024-02-29")
	_, err = dateParam("from", Date{})
	assertRejected(t, err, "from", ErrZeroTemporalValue)

	dt, err := NewDateTime(2024, time.February, 29, 9, 30, 0)
	if err != nil {
		t.Fatal(err)
	}
	got, err = dateTimeParam("from", dt)
	assertParam(t, got, err, "from", "2024-02-29 09:30:00")
	_, err = dateTimeParam("to", DateTime{})
	assertRejected(t, err, "to", ErrZeroTemporalValue)
}

func TestEnumeratedKindsAcceptOnlyDocumentedWireValues(t *testing.T) {
	t.Parallel()
	got, err := retrievalFrequencyParam("period", RetrievalFrequencyAnnual)
	assertParam(t, got, err, "period", "annual")
	got, err = retrievalFrequencyParam("period", RetrievalFrequencyQuarterly)
	assertParam(t, got, err, "period", "quarter")
	for _, value := range []RetrievalFrequency{"", "Annual", "quarterly", "annual "} {
		_, err = retrievalFrequencyParam("period", value)
		var typed *Error
		if !errors.As(err, &typed) || typed.Category != CategoryValidation || !errors.Is(err, ErrUnknownWireValue) {
			t.Fatalf("%q: error = %v, want a CategoryValidation *Error wrapping ErrUnknownWireValue", value, err)
		}
		if want := "period: " + ErrUnknownWireValue.Error() + ", expected one of annual, quarter"; typed.Message != want {
			t.Fatalf("%q: message = %q, want %q", value, typed.Message, want)
		}
	}
}
