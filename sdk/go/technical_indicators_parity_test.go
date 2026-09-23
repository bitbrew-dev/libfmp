package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

// technicalIndicatorBar is the seven-member row shape every indicator model
// shares (crates/libfmp/src/responses/technical_indicators.rs); Metric is the
// one member whose wire name differs per model.
type technicalIndicatorBar struct {
	Date   DateTime
	Open   float64
	High   float64
	Low    float64
	Close  float64
	Volume float64
	Metric float64
}

// technicalIndicatorsFixtures maps every fixture of the domain to its model
// and the exact values of its single row, copied from
// crates/libfmp/tests/technical_indicator_responses.rs and the fixtures.
var technicalIndicatorsFixtures = []struct {
	fixture string
	metric  string
	decode  func(t *testing.T) technicalIndicatorBar
}{
	{"technical_indicator_sma.json", "sma", func(t *testing.T) technicalIndicatorBar {
		r := assertFixtureParity[SimpleMovingAverageBar](t, "technical_indicator_sma.json")
		return technicalIndicatorBar{r[0].Date, r[0].Open, r[0].High, r[0].Low, r[0].Close, r[0].Volume, r[0].Sma}
	}},
	{"technical_indicator_ema.json", "ema", func(t *testing.T) technicalIndicatorBar {
		r := assertFixtureParity[ExponentialMovingAverageBar](t, "technical_indicator_ema.json")
		return technicalIndicatorBar{r[0].Date, r[0].Open, r[0].High, r[0].Low, r[0].Close, r[0].Volume, r[0].Ema}
	}},
	{"technical_indicator_wma.json", "wma", func(t *testing.T) technicalIndicatorBar {
		r := assertFixtureParity[WeightedMovingAverageBar](t, "technical_indicator_wma.json")
		return technicalIndicatorBar{r[0].Date, r[0].Open, r[0].High, r[0].Low, r[0].Close, r[0].Volume, r[0].Wma}
	}},
	{"technical_indicator_dema.json", "dema", func(t *testing.T) technicalIndicatorBar {
		r := assertFixtureParity[DoubleExponentialMovingAverageBar](t, "technical_indicator_dema.json")
		return technicalIndicatorBar{r[0].Date, r[0].Open, r[0].High, r[0].Low, r[0].Close, r[0].Volume, r[0].Dema}
	}},
	{"technical_indicator_tema.json", "tema", func(t *testing.T) technicalIndicatorBar {
		r := assertFixtureParity[TripleExponentialMovingAverageBar](t, "technical_indicator_tema.json")
		return technicalIndicatorBar{r[0].Date, r[0].Open, r[0].High, r[0].Low, r[0].Close, r[0].Volume, r[0].Tema}
	}},
	{"technical_indicator_rsi.json", "rsi", func(t *testing.T) technicalIndicatorBar {
		r := assertFixtureParity[RelativeStrengthIndexBar](t, "technical_indicator_rsi.json")
		return technicalIndicatorBar{r[0].Date, r[0].Open, r[0].High, r[0].Low, r[0].Close, r[0].Volume, r[0].Rsi}
	}},
	{"technical_indicator_standard_deviation.json", "standardDeviation", func(t *testing.T) technicalIndicatorBar {
		r := assertFixtureParity[StandardDeviationBar](t, "technical_indicator_standard_deviation.json")
		return technicalIndicatorBar{r[0].Date, r[0].Open, r[0].High, r[0].Low, r[0].Close, r[0].Volume, r[0].StandardDeviation}
	}},
	{"technical_indicator_williams.json", "williams", func(t *testing.T) technicalIndicatorBar {
		r := assertFixtureParity[WilliamsBar](t, "technical_indicator_williams.json")
		return technicalIndicatorBar{r[0].Date, r[0].Open, r[0].High, r[0].Low, r[0].Close, r[0].Volume, r[0].Williams}
	}},
	{"technical_indicator_adx.json", "adx", func(t *testing.T) technicalIndicatorBar {
		r := assertFixtureParity[AverageDirectionalIndexBar](t, "technical_indicator_adx.json")
		return technicalIndicatorBar{r[0].Date, r[0].Open, r[0].High, r[0].Low, r[0].Close, r[0].Volume, r[0].Adx}
	}},
}

// Metric values per fixture, copied verbatim from the fixture files; the
// Rust test source_values_preserve_second_precision_large_volume_and_negative_williams
// pins sma, williams, and adx.
var technicalIndicatorsMetrics = map[string]float64{
	"sma":               331.621,
	"ema":               331.1209325826155,
	"wma":               333.21345454545457,
	"dema":              337.7659642917977,
	"tema":              337.09671042323214,
	"rsi":               59.55175118203601,
	"standardDeviation": 5.675893674127448,
	"williams":          -48.29500396510714,
	"adx":               34.69458756515438,
}

// Mirrors all_nine_exact_source_rows_round_trip_with_seven_flat_fields and
// source_values_preserve_second_precision_large_volume_and_negative_williams.
func TestTechnicalIndicatorsFixturesDecodeExactSevenFieldRows(t *testing.T) {
	t.Parallel()
	for _, tc := range technicalIndicatorsFixtures {
		t.Run(tc.metric, func(t *testing.T) {
			t.Parallel()
			got := tc.decode(t)
			want := technicalIndicatorBar{
				Date: mustParseDateTime(t, "2026-07-30 00:00:00"), Open: 333.13, High: 334.48, Low: 329.59,
				Close: 332.39, Volume: 29_207_295, Metric: technicalIndicatorsMetrics[tc.metric],
			}
			if got != want {
				t.Fatalf("%s = %+v, want %+v", tc.fixture, got, want)
			}
			var wire []map[string]jsontext.Value
			if err := json.Unmarshal(readFixture(t, tc.fixture), &wire); err != nil || len(wire[0]) != 7 {
				t.Fatalf("%s: wire members = %d, %v, want 7", tc.fixture, len(wire[0]), err)
			}
			if _, ok := wire[0][tc.metric]; !ok {
				t.Fatalf("%s: metric member %q is absent from the fixture", tc.fixture, tc.metric)
			}
		})
	}
}

// Mirrors every_documented_field_is_required_non_null_and_unknowns_are_accepted
// for one model per metric type: a missing or null member is a Decode error
// naming the member, and an unknown member is ignored.
func TestTechnicalIndicatorsMembersAreRequiredAndNonNull(t *testing.T) {
	t.Parallel()
	raw := readFixture(t, "technical_indicator_sma.json")
	for _, member := range []string{"date", "open", "high", "low", "close", "volume", "sma"} {
		t.Run(member, func(t *testing.T) {
			t.Parallel()
			var rows []SimpleMovingAverageBar
			var typed *Error
			err := json.Unmarshal(technicalIndicatorsRewrite(t, raw, member, nil), &rows)
			if !errors.As(err, &typed) || typed.Category != CategoryDecode ||
				!strings.Contains(typed.Message, `"`+member+`"`) || !strings.Contains(typed.Message, "SimpleMovingAverageBar") {
				t.Fatalf("missing %s: error = %v, want a CategoryDecode *Error naming the member", member, err)
			}
			err = json.Unmarshal(technicalIndicatorsRewrite(t, raw, member, jsontext.Value("null")), &rows)
			if !errors.As(err, &typed) || !strings.Contains(typed.Message, `"`+member+`"`) {
				t.Fatalf("null %s: error = %v, want a *Error naming the member", member, err)
			}
		})
	}

	forward := technicalIndicatorsRewrite(t, raw, "futureProviderField", jsontext.Value(`{"nested": [1, true, null]}`))
	var rows []SimpleMovingAverageBar
	if err := json.Unmarshal(forward, &rows); err != nil || len(rows) != 1 || rows[0].Sma != 331.621 {
		t.Fatalf("unknown nested member was not ignored: %+v, %v", rows, err)
	}
}

// technicalIndicatorsRewrite re-encodes a one-row fixture with member set to
// value, or removed when value is nil.
func technicalIndicatorsRewrite(t *testing.T, raw []byte, member string, value jsontext.Value) []byte {
	t.Helper()
	var wire []map[string]jsontext.Value
	if err := json.Unmarshal(raw, &wire); err != nil || len(wire) != 1 {
		t.Fatalf("fixture is not a one-row array: %v", err)
	}
	if value == nil {
		delete(wire[0], member)
	} else {
		wire[0][member] = value
	}
	encoded, err := json.Marshal(wire)
	if err != nil {
		t.Fatal(err)
	}
	return encoded
}

// Mirrors volume_preserves_values_above_u32_and_float_metrics_accept_integers_and_fractions
// and metric_keys_are_distinct_and_exactly_cased.
func TestTechnicalIndicatorsVolumeIsUint64AndMetricKeysAreExact(t *testing.T) {
	t.Parallel()
	raw := string(readFixture(t, "technical_indicator_sma.json"))
	wide := strings.Replace(strings.Replace(raw, "29207295", "18446744073709551615", 1), "331.621", "7", 1)
	var sma []SimpleMovingAverageBar
	if err := json.Unmarshal([]byte(wide), &sma); err != nil || sma[0].Volume != 18_446_744_073_709_551_615 || sma[0].Sma != 7 {
		t.Fatalf("u64 volume and integer metric = %+v, %v", sma, err)
	}
	williams := strings.Replace(string(readFixture(t, "technical_indicator_williams.json")), "-48.29500396510714", "-100", 1)
	var bars []WilliamsBar
	if err := json.Unmarshal([]byte(williams), &bars); err != nil || bars[0].Williams != -100 {
		t.Fatalf("integer williams = %+v, %v", bars, err)
	}

	if err := json.Unmarshal(readFixture(t, "technical_indicator_ema.json"), &sma); err == nil {
		t.Fatal("an ema row decoded into SimpleMovingAverageBar")
	}
	snake := strings.Replace(string(readFixture(t, "technical_indicator_standard_deviation.json")),
		`"standardDeviation"`, `"standard_deviation"`, 1)
	var deviations []StandardDeviationBar
	if err := json.Unmarshal([]byte(snake), &deviations); err == nil {
		t.Fatal("a snake_case standard_deviation member decoded into StandardDeviationBar")
	}

	for _, invalid := range []string{"2026-07-30 00:00", "2026-07-30T00:00:00Z"} {
		malformed := strings.Replace(raw, "2026-07-30 00:00:00", invalid, 1)
		if err := json.Unmarshal([]byte(malformed), &sma); err == nil {
			t.Fatalf("timestamp %q decoded into a DateTime member", invalid)
		}
	}
}
