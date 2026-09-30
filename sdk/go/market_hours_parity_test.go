package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"reflect"
	"testing"
)

func TestMarketHoursFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[ExchangeMarketHours](t, "exchange_market_hours.json")
	assertFixtureParity[ExchangeHoliday](t, "holidays_by_exchange.json")
	assertFixtureParity[ExchangeMarketHours](t, "all_exchange_market_hours.json")
}

// Exact values copied from crates/libfmp/tests/market_hours_responses.rs:
// opening hours, closing hours, and timezone identifiers are kept as the
// provider's raw strings, never parsed or converted.
func TestDocumentedMarketHoursDecodeExactValues(t *testing.T) {
	t.Parallel()
	exchange := assertFixtureParity[ExchangeMarketHours](t, "exchange_market_hours.json")
	want := ExchangeMarketHours{
		Exchange:     "NASDAQ",
		Name:         "NASDAQ",
		OpeningHour:  "09:30 AM -04:00",
		ClosingHour:  "04:00 PM -04:00",
		Timezone:     "America/New_York",
		IsMarketOpen: true,
	}
	if len(exchange) != 1 || exchange[0] != want {
		t.Fatalf("exchange_market_hours = %+v, want %+v", exchange, want)
	}
	if members := memberSet(t, exchange[0]); len(members) != 6 {
		t.Fatalf("re-encoded members = %d, want the documented 6", len(members))
	}

	all := assertFixtureParity[ExchangeMarketHours](t, "all_exchange_market_hours.json")
	want = ExchangeMarketHours{
		Exchange:     "ASX",
		Name:         "Australian Securities Exchange",
		OpeningHour:  "10:00 AM +10:00",
		ClosingHour:  "04:00 PM +10:00",
		Timezone:     "Australia/Sydney",
		IsMarketOpen: false,
	}
	if len(all) != 1 || all[0] != want {
		t.Fatalf("all_exchange_market_hours = %+v, want %+v", all, want)
	}
}

// Exact values copied from crates/libfmp/tests/market_hours_responses.rs:
// the two adjusted times are required keys whose only documented value is
// null, so they decode to nil and re-encode as null.
func TestDocumentedExchangeHolidaysDecodeExactValues(t *testing.T) {
	t.Parallel()
	holidays := assertFixtureParity[ExchangeHoliday](t, "holidays_by_exchange.json")
	if len(holidays) != 1 {
		t.Fatalf("rows = %d, want 1", len(holidays))
	}
	row := holidays[0]
	if row.Exchange != "NASDAQ" || row.Date != mustParseDate(t, "2026-07-03") || row.Date.String() != "2026-07-03" ||
		row.Name != "Independence Day" || !reflect.DeepEqual(row.IsClosed, new(true)) || row.AdjOpenTime != nil || row.AdjCloseTime != nil {
		t.Fatalf("holidays_by_exchange = %+v", row)
	}
	if members := memberSet(t, row); len(members) != 6 {
		t.Fatalf("re-encoded members = %d, want the documented 6", len(members))
	}
}

// Mirrors adjusted_times_are_required_but_nullable_and_future_dynamic in
// crates/libfmp/tests/market_hours_responses.rs: removing either adjusted
// time is rejected, while a future string or object value is kept raw and
// re-encoded unchanged.
func TestExchangeHolidayAdjustedTimesAreRequiredButNullableAndFutureDynamic(t *testing.T) {
	t.Parallel()
	const fixture = "holidays_by_exchange.json"
	for _, member := range []string{"adjOpenTime", "adjCloseTime"} {
		var rows []ExchangeHoliday
		err := json.Unmarshal(mutateFixtureMember(t, fixture, member, nil), &rows)
		var typed *Error
		want := `required member "` + member + `" of ExchangeHoliday is missing or null`
		if !errors.As(err, &typed) || typed.Category != CategoryDecode || typed.Message != want {
			t.Fatalf("missing %s: error = %v (%T), want CategoryDecode %q", member, err, err, want)
		}
	}

	future := `[{"adjCloseTime":{"local":"01:00 PM","offset":"-04:00","earlyClose":true},` +
		`"adjOpenTime":"09:30 AM -04:00","date":"2026-07-03","exchange":"NASDAQ",` +
		`"isClosed":true,"name":"Independence Day"}]`
	var decoded []ExchangeHoliday
	if err := json.Unmarshal([]byte(future), &decoded); err != nil {
		t.Fatalf("future adjusted times rejected: %v", err)
	}
	if len(decoded) != 1 || decoded[0].AdjOpenTime == nil || string(*decoded[0].AdjOpenTime) != `"09:30 AM -04:00"` ||
		decoded[0].AdjCloseTime == nil ||
		string(*decoded[0].AdjCloseTime) != `{"local":"01:00 PM","offset":"-04:00","earlyClose":true}` {
		t.Fatalf("future adjusted times = %+v, want the raw values", decoded)
	}
	encoded, err := json.Marshal(decoded)
	if err != nil {
		t.Fatal(err)
	}
	if got := memberSet(t, decoded[0]); len(got) != 6 {
		t.Fatalf("re-encoded members = %v, want 6", got)
	}
	var back []map[string]jsontext.Value
	if err := json.Unmarshal(encoded, &back); err != nil || len(back) != 1 ||
		string(back[0]["adjOpenTime"]) != `"09:30 AM -04:00"` ||
		string(back[0]["adjCloseTime"]) != `{"local":"01:00 PM","offset":"-04:00","earlyClose":true}` {
		t.Fatalf("re-encoded = %s (%v), want the future values verbatim", encoded, err)
	}
}

// Required keys are enforced as the Rust decoder enforces them: a missing or
// null member of either model is a Decode error naming that member, and both
// contracts keep their bare-array roots.
func TestMarketHoursRequiredMembersAndArrayRootsAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	const hours = "exchange_market_hours.json"
	const holidays = "holidays_by_exchange.json"
	decode := func(fixture string, wire []byte) error {
		if fixture == hours {
			var rows []ExchangeMarketHours
			return json.Unmarshal(wire, &rows)
		}
		var rows []ExchangeHoliday
		return json.Unmarshal(wire, &rows)
	}
	rejected := []struct {
		name    string
		fixture string
		member  string
		value   jsontext.Value
		message string
	}{
		{"missing exchange", hours, "exchange", nil,
			`required member "exchange" of ExchangeMarketHours is missing or null`},
		{"null exchange", hours, "exchange", jsontext.Value(`null`),
			`required member "exchange" of ExchangeMarketHours is missing or null`},
		{"missing open flag", hours, "isMarketOpen", nil,
			`required member "isMarketOpen" of ExchangeMarketHours is missing or null`},
		{"missing date", holidays, "date", nil,
			`required member "date" of ExchangeHoliday is missing or null`},
		{"missing closed flag", holidays, "isClosed", nil,
			`required member "isClosed" of ExchangeHoliday is missing or null`},
	}
	for _, tc := range rejected {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			err := decode(tc.fixture, mutateFixtureMember(t, tc.fixture, tc.member, tc.value))
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode || typed.Message != tc.message {
				t.Fatalf("error = %v (%T), want CategoryDecode %q", err, err, tc.message)
			}
		})
	}
	if err := decode(holidays, mutateFixtureMember(t, holidays, "date", jsontext.Value(`"07/03/2026"`))); err == nil {
		t.Fatal("a non-ISO holiday date decoded")
	}
	if err := decode(hours, mutateFixtureMember(t, hours, "isMarketOpen", jsontext.Value(`"true"`))); err == nil {
		t.Fatal("a JSON string decoded into the bool open flag")
	}

	for _, fixture := range []string{hours, holidays} {
		if err := decode(fixture, []byte(`[]`)); err != nil {
			t.Fatalf("%s: empty array rejected: %v", fixture, err)
		}
		if err := decode(fixture, []byte(`{}`)); err == nil {
			t.Fatalf("%s: an object root decoded as an array", fixture)
		}
	}
}

// Mirrors is_closed_is_required_but_null_decodes_to_none in
// crates/libfmp/tests/market_hours_responses.rs: members FMP sends as null
// decode to nil, while a missing member still fails.
func TestExchangeHolidayIsClosedDecodesNullAsNil(t *testing.T) {
	t.Parallel()
	const fixture = "holidays_by_exchange.json"
	cases := []struct {
		member string
		isNil  func(ExchangeHoliday) bool
	}{
		{"isClosed", func(row ExchangeHoliday) bool { return row.IsClosed == nil }},
	}
	for _, tc := range cases {
		t.Run(tc.member, func(t *testing.T) {
			t.Parallel()
			var missing []ExchangeHoliday
			if err := json.Unmarshal(mutateFixtureMember(t, fixture, tc.member, nil), &missing); err == nil {
				t.Fatalf("missing %s decoded", tc.member)
			}
			var rows []ExchangeHoliday
			if err := json.Unmarshal(mutateFixtureMember(t, fixture, tc.member, jsontext.Value(`null`)), &rows); err != nil ||
				len(rows) != 1 || !tc.isNil(rows[0]) {
				t.Fatalf("null %s = %+v, %v, want nil", tc.member, rows, err)
			}
		})
	}
}
