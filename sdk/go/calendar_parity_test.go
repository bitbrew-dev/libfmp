package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

// The nine calendar fixtures and the model each Rust test decodes them into
// (crates/libfmp/tests/calendar_responses.rs).
func TestCalendarFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[DividendEvent](t, "dividends.json")
	assertFixtureParity[DividendEvent](t, "dividends_calendar.json")
	assertFixtureParity[EarningsEvent](t, "earnings.json")
	assertFixtureParity[EarningsEvent](t, "earnings_calendar.json")
	assertFixtureParity[IpoCalendarEvent](t, "ipos_calendar.json")
	assertFixtureParity[IpoDisclosure](t, "ipos_disclosure.json")
	assertFixtureParity[IpoProspectus](t, "ipos_prospectus.json")
	assertFixtureParity[StockSplitEvent](t, "stock_splits.json")
	assertFixtureParity[StockSplitEvent](t, "stock_splits_calendar.json")
}

// patchedFixture re-encodes the first row of a fixture after edit changed its
// members, so one fixture can drive the null, missing, and wrong-kind cases.
func patchedFixture(t *testing.T, name string, edit func(row map[string]jsontext.Value)) []byte {
	t.Helper()
	var rows []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, name), &rows); err != nil {
		t.Fatalf("%s: %v", name, err)
	}
	edit(rows[0])
	encoded, err := json.Marshal(rows)
	if err != nil {
		t.Fatalf("%s: re-encode: %v", name, err)
	}
	return encoded
}

// Exact values copied from calendar_responses.rs. declarationDate carries the
// empty_or_null_date codec: "" and null are nil, a non-string is rejected, and
// the key itself is still required.
func TestDocumentedDividendFixturesDecodeExactValuesAndEmptyDeclarationDates(t *testing.T) {
	t.Parallel()
	company := assertFixtureParity[DividendEvent](t, "dividends.json")
	if len(company) != 1 || company[0].DeclarationDate == nil {
		t.Fatalf("dividends = %+v", company)
	}
	want := DividendEvent{
		Symbol: "AAPL", Date: mustParseDate(t, "2026-05-11"), RecordDate: mustParseDate(t, "2026-05-11"),
		PaymentDate: mustParseDate(t, "2026-05-14"), DeclarationDate: company[0].DeclarationDate,
		AdjDividend: 0.27, Dividend: 0.27, Yield: 0.3587535875358754, Frequency: "Quarterly",
	}
	if company[0] != want || *company[0].DeclarationDate != mustParseDate(t, "2026-04-30") {
		t.Fatalf("dividends = %+v, want %+v", company[0], want)
	}

	calendar := assertFixtureParity[DividendEvent](t, "dividends_calendar.json")
	if len(calendar) != 1 || calendar[0].Symbol != "5871.TW" || calendar[0].Frequency != "Annual" {
		t.Fatalf("dividends_calendar = %+v", calendar)
	}
	if calendar[0].DeclarationDate != nil {
		t.Fatalf(`declarationDate "" decoded as %v, want nil`, *calendar[0].DeclarationDate)
	}

	var rows []DividendEvent
	null := patchedFixture(t, "dividends.json", func(row map[string]jsontext.Value) {
		row["declarationDate"] = jsontext.Value("null")
	})
	if err := json.Unmarshal(null, &rows); err != nil || rows[0].DeclarationDate != nil {
		t.Fatalf("null declarationDate: rows = %+v, err = %v", rows, err)
	}
	number := patchedFixture(t, "dividends.json", func(row map[string]jsontext.Value) {
		row["declarationDate"] = jsontext.Value("20260430")
	})
	var typed *Error
	err := json.Unmarshal(number, &rows)
	if !errors.As(err, &typed) || typed.Category != CategoryDecode ||
		typed.Message != `member "declarationDate" of DividendEvent must be a JSON string` {
		t.Fatalf("numeric declarationDate: error = %v", err)
	}
	missing := patchedFixture(t, "dividends.json", func(row map[string]jsontext.Value) {
		delete(row, "declarationDate")
	})
	err = json.Unmarshal(missing, &rows)
	if !errors.As(err, &typed) || !strings.Contains(typed.Message, `"declarationDate"`) {
		t.Fatalf("missing declarationDate: error = %v", err)
	}
}

// epsActual and revenueActual carry required_option: the key must be present
// and null is a value, so the two fixtures decode to nil and to pointers.
func TestDocumentedEarningsFixturesKeepOnlyActualValuesNullable(t *testing.T) {
	t.Parallel()
	company := assertFixtureParity[EarningsEvent](t, "earnings.json")
	want := EarningsEvent{
		Symbol: "AAPL", Date: mustParseDate(t, "2026-07-30"), EPSEstimated: 1.88,
		RevenueEstimated: 109_038_900_000, LastUpdated: mustParseDate(t, "2026-07-30"),
	}
	if len(company) != 1 || company[0] != want {
		t.Fatalf("earnings = %+v, want %+v", company, want)
	}

	calendar := assertFixtureParity[EarningsEvent](t, "earnings_calendar.json")
	if len(calendar) != 1 || calendar[0].Symbol != "GRG.L" || calendar[0].EPSEstimated != 0.501 ||
		calendar[0].RevenueEstimated != 1_086_300_000 {
		t.Fatalf("earnings_calendar = %+v", calendar)
	}
	if calendar[0].EPSActual == nil || *calendar[0].EPSActual != 0.549 ||
		calendar[0].RevenueActual == nil || *calendar[0].RevenueActual != 1_101_500_000 {
		t.Fatalf("earnings_calendar actuals = %v, %v", calendar[0].EPSActual, calendar[0].RevenueActual)
	}
}

// The three dynamic members are lossless raw bytes: a u64 beyond float64
// precision, an object, and an array come back exactly as sent.
func TestDocumentedIpoCalendarFixturePreservesLiteralDaaAndRawDynamicMembers(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[IpoCalendarEvent](t, "ipos_calendar.json")
	want := IpoCalendarEvent{
		Symbol: "IMC", Date: mustParseDate(t, "2026-07-29"), Daa: "2026-07-29T04:00:00.000Z",
		Company: "IMC Rare Earths Ltd", Exchange: "NYSE", Actions: "Priced",
	}
	if len(rows) != 1 || rows[0] != want {
		t.Fatalf("ipos_calendar = %+v, want %+v", rows, want)
	}

	dynamic := patchedFixture(t, "ipos_calendar.json", func(row map[string]jsontext.Value) {
		row["shares"] = jsontext.Value("18446744073709551615")
		row["priceRange"] = jsontext.Value(`{"low":12,"high":"open"}`)
		row["marketCap"] = jsontext.Value("[1,true,null]")
	})
	if err := json.Unmarshal(dynamic, &rows); err != nil {
		t.Fatalf("dynamic members: %v", err)
	}
	if rows[0].Shares == nil || string(*rows[0].Shares) != "18446744073709551615" {
		t.Fatalf("shares = %v, want the exact digits", rows[0].Shares)
	}
	if rows[0].PriceRange == nil || rows[0].PriceRange.Kind() != '{' ||
		rows[0].MarketCap == nil || rows[0].MarketCap.Kind() != '[' {
		t.Fatalf("priceRange = %v, marketCap = %v", rows[0].PriceRange, rows[0].MarketCap)
	}
}

func TestDocumentedIpoFilingAndStockSplitFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	disclosure := assertFixtureParity[IpoDisclosure](t, "ipos_disclosure.json")
	wantDisclosure := IpoDisclosure{
		Symbol: "QTJA", FilingDate: mustParseDate(t, "2026-07-30"), AcceptedDate: mustParseDate(t, "2026-07-30"),
		EffectivenessDate: mustParseDate(t, "2026-07-30"), CIK: "0001415726", Form: "CERT",
		URL: "https://www.sec.gov/Archives/edgar/data/1415726/000141783526000235/8A_Cert_DDTG_DDFG.pdf",
	}
	if len(disclosure) != 1 || disclosure[0] != wantDisclosure {
		t.Fatalf("ipos_disclosure = %+v", disclosure)
	}

	prospectus := assertFixtureParity[IpoProspectus](t, "ipos_prospectus.json")
	wantProspectus := IpoProspectus{
		Symbol: "FTW-WT", AcceptedDate: mustParseDate(t, "2026-07-29"), FilingDate: mustParseDate(t, "2026-07-30"),
		IpoDate: mustParseDate(t, "2026-07-28"), CIK: "0002083125", PricePublicPerShare: 1, PricePublicTotal: 434,
		DiscountsAndCommissionsPerShare: 0, DiscountsAndCommissionsTotal: 82_251, ProceedsBeforeExpensesPerShare: 1,
		ProceedsBeforeExpensesTotal: 82_251, Form: "S-1",
		URL: "https://www.sec.gov/Archives/edgar/data/2083125/000121390026082963/ea0298363-s1_presidio.htm",
	}
	if len(prospectus) != 1 || prospectus[0] != wantProspectus {
		t.Fatalf("ipos_prospectus = %+v", prospectus)
	}

	company := assertFixtureParity[StockSplitEvent](t, "stock_splits.json")
	wantSplit := StockSplitEvent{Symbol: "AAPL", Date: mustParseDate(t, "2020-08-31"), Numerator: 4, Denominator: 1,
		SplitType: "stock-split"}
	if len(company) != 1 || company[0] != wantSplit {
		t.Fatalf("stock_splits = %+v", company)
	}
	calendar := assertFixtureParity[StockSplitEvent](t, "stock_splits_calendar.json")
	if len(calendar) != 1 || calendar[0].Symbol != "WHLR" || calendar[0].Numerator != 1 || calendar[0].Denominator != 5 {
		t.Fatalf("stock_splits_calendar = %+v", calendar)
	}
}

// assertMemberContract mirrors assert_contract in calendar_responses.rs:
// removing any member of the first row is a Decode error naming it, and null
// is accepted only for the members listed as nullable.
func assertMemberContract[T any](t *testing.T, name string, nullable ...string) {
	t.Helper()
	var wire []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, name), &wire); err != nil {
		t.Fatalf("%s: %v", name, err)
	}
	for member := range wire[0] {
		var rows []T
		missing := patchedFixture(t, name, func(row map[string]jsontext.Value) { delete(row, member) })
		var typed *Error
		err := json.Unmarshal(missing, &rows)
		if !errors.As(err, &typed) || typed.Category != CategoryDecode || !strings.Contains(typed.Message, `"`+member+`"`) {
			t.Fatalf("%s: missing %q: error = %v", name, member, err)
		}
		null := patchedFixture(t, name, func(row map[string]jsontext.Value) { row[member] = jsontext.Value("null") })
		err = json.Unmarshal(null, &rows)
		wantOK := false
		for _, candidate := range nullable {
			wantOK = wantOK || candidate == member
		}
		if (err == nil) != wantOK {
			t.Fatalf("%s: null %q: error = %v, want accepted = %v", name, member, err, wantOK)
		}
	}
}

func TestCalendarRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	assertMemberContract[DividendEvent](t, "dividends.json", "declarationDate")
	assertMemberContract[DividendEvent](t, "dividends_calendar.json", "declarationDate")
	assertMemberContract[EarningsEvent](t, "earnings.json", "epsActual", "revenueActual")
	assertMemberContract[EarningsEvent](t, "earnings_calendar.json", "epsActual", "revenueActual")
	assertMemberContract[IpoCalendarEvent](t, "ipos_calendar.json", "shares", "priceRange", "marketCap")
	assertMemberContract[IpoDisclosure](t, "ipos_disclosure.json")
	assertMemberContract[IpoProspectus](t, "ipos_prospectus.json")
	assertMemberContract[StockSplitEvent](t, "stock_splits.json")
	assertMemberContract[StockSplitEvent](t, "stock_splits_calendar.json")

	for _, wire := range []string{"[]", "[ ]"} {
		var rows []DividendEvent
		if err := json.Unmarshal([]byte(wire), &rows); err != nil || rows == nil || len(rows) != 0 {
			t.Fatalf("%q: rows = %#v, err = %v, want a non-nil empty slice", wire, rows, err)
		}
	}
	var rows []DividendEvent
	if err := json.Unmarshal([]byte(`{"events":[]}`), &rows); err == nil {
		t.Fatal("an object wrapper decoded as a bare array")
	}
}
