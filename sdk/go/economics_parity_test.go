package fmp

import (
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

func TestEconomicsFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[TreasuryRate](t, "treasury_rates.json")
	assertFixtureParity[EconomicIndicatorObservation](t, "economic_indicators.json")
	assertFixtureParity[EconomicCalendarEvent](t, "economic_calendar.json")
	assertFixtureParity[MarketRiskPremium](t, "market_risk_premium.json")
}

// Exact values copied from crates/libfmp/tests/economics_responses.rs.
func TestTreasuryFixtureDecodesAll13ExactFieldsWithoutScaling(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[TreasuryRate](t, "treasury_rates.json")
	want := TreasuryRate{
		Date: mustParseDate(t, "2026-07-29"), Month1: 3.73, Month2: 3.83, Month3: 3.83, Month6: 3.97,
		Year1: 4.04, Year2: 4.22, Year3: 4.29, Year5: 4.37, Year7: 4.51, Year10: 4.67, Year20: 5.21, Year30: 5.2,
	}
	if len(rows) != 1 || rows[0] != want {
		t.Fatalf("treasury_rates = %+v, want %+v", rows, want)
	}
	if got := memberSet(t, rows[0]); len(got) != 13 {
		t.Fatalf("re-encoded members = %d, want 13", len(got))
	}
}

// Mirrors indicator_fixture_decodes_exact_three_field_contract: the name is a
// plain string on the model, so an undocumented provider name decodes too.
func TestIndicatorFixtureDecodesExactThreeFieldContract(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[EconomicIndicatorObservation](t, "economic_indicators.json")
	want := EconomicIndicatorObservation{Name: "GDP", Date: mustParseDate(t, "2025-10-01"), Value: 31_422.526}
	if len(rows) != 1 || rows[0] != want {
		t.Fatalf("economic_indicators = %+v, want %+v", rows, want)
	}
	if EconomicIndicator(rows[0].Name) != EconomicIndicatorGdp {
		t.Fatalf("name %q does not convert to the documented constant", rows[0].Name)
	}
	open := strings.Replace(string(readFixture(t, "economic_indicators.json")), `"GDP"`, `"futureProviderIndicator"`, 1)
	if err := json.Unmarshal([]byte(open), &rows); err != nil || rows[0].Name != "futureProviderIndicator" {
		t.Fatalf("open indicator name = %+v, %v", rows, err)
	}
}

func TestCalendarFixtureDecodesAll11FieldsWithRequiredNumericValues(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[EconomicCalendarEvent](t, "economic_calendar.json")
	want := EconomicCalendarEvent{
		Date: mustParseDateTime(t, "2026-07-29 03:30:00"), Country: "SG", Event: "Import Prices YoY (Jun)",
		Currency: "SGD", Previous: 18.5, Estimate: 21.0, Actual: 13.6, Change: -4.9, Impact: "Low",
		ChangePercentage: -26.486, Unit: "%",
	}
	if len(rows) != 1 || rows[0] != want {
		t.Fatalf("economic_calendar = %+v, want %+v", rows, want)
	}
	if got := memberSet(t, rows[0]); len(got) != 11 {
		t.Fatalf("re-encoded members = %d, want 11", len(got))
	}
}

// Mirrors calendar_datetime_remains_strictly_timezone_less.
func TestCalendarDateTimeRemainsStrictlyTimezoneLess(t *testing.T) {
	t.Parallel()
	for _, invalid := range []string{"2026-07-29T03:30:00Z", "2026-07-29 03:30:00+08:00"} {
		wire := strings.Replace(string(readFixture(t, "economic_calendar.json")), "2026-07-29 03:30:00", invalid, 1)
		var rows []EconomicCalendarEvent
		if err := json.Unmarshal([]byte(wire), &rows); err == nil {
			t.Fatalf("%q decoded into a DateTime member", invalid)
		}
	}
}

func TestMarketRiskFixtureUsesFullCountryNameAndRawPercentageValues(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[MarketRiskPremium](t, "market_risk_premium.json")
	want := MarketRiskPremium{Country: "Zimbabwe", Continent: "Africa", CountryRiskPremium: 11.66,
		TotalEquityRiskPremium: 15.89}
	if len(rows) != 1 || rows[0] != want {
		t.Fatalf("market_risk_premium = %+v, want %+v", rows, want)
	}
	if got := memberSet(t, rows[0]); len(got) != 4 {
		t.Fatalf("re-encoded members = %d, want 4 and no date", len(got))
	}
}

// Mirrors every_economics_row_requires_documented_fields_and_accepts_unknown_fields
// for the documented-null ambiguity of the economics calendar
// (docs/contract-ambiguities.md, ADR 0010): the Rust model keeps previous,
// estimate, and actual as required f64, so a null actual is a decode error
// here too, and the missing-required-member path is covered once for this
// domain.
func TestEconomicsRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	head := `[{"date":"2026-07-29 03:30:00","country":"SG","event":"Import Prices YoY (Jun)","currency":"SGD",` +
		`"previous":18.5,"estimate":21,`
	tail := `"change":-4.9,"impact":"Low","changePercentage":-26.486,"unit":"%"}]`
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"null actual", head + `"actual":null,` + tail, "actual"},
		{"missing actual", head + tail, "actual"},
		{"empty object", `[{}]`, "date"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []EconomicCalendarEvent
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "EconomicCalendarEvent") {
				t.Fatalf("message = %q, want it to name member %q of EconomicCalendarEvent", typed.Message, tc.member)
			}
		})
	}
	var rates []TreasuryRate
	err := json.Unmarshal([]byte(`[{"date":"2026-07-29","month1":3.73}]`), &rates)
	var typed *Error
	if !errors.As(err, &typed) || !strings.Contains(typed.Message, `"month2"`) {
		t.Fatalf("TreasuryRate error = %v, want the first missing member month2", err)
	}
	var premiums []MarketRiskPremium
	if err := json.Unmarshal([]byte(`{"rates":[]}`), &premiums); err == nil {
		t.Fatal("an object decoded into a bare-array contract")
	}
	if err := json.Unmarshal([]byte(`[]`), &premiums); err != nil || len(premiums) != 0 {
		t.Fatalf("empty array = %+v, %v", premiums, err)
	}
}
