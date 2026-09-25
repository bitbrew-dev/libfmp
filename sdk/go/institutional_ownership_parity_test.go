package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"math"
	"strings"
	"testing"
)

// The eight institutional_ownership fixtures, one per response model. Every
// member is required and non-null in the Rust models, so no fixture carries
// an intentionally unknown member.
func TestInstitutionalOwnershipFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[InstitutionalOwnershipFiling](t, "latest_institutional_ownership_filings.json")
	assertFixtureParity[InstitutionalHolding](t, "institutional_ownership_extract.json")
	assertFixtureParity[Form13fFilingDate](t, "form_13f_filing_dates.json")
	assertFixtureParity[InstitutionalHolderAnalytics](t, "institutional_holder_analytics.json")
	assertFixtureParity[HolderPerformanceSummary](t, "holder_performance_summary.json")
	assertFixtureParity[HolderIndustryBreakdown](t, "holder_industry_breakdown.json")
	assertFixtureParity[InstitutionalPositionSummary](t, "institutional_positions_summary.json")
	assertFixtureParity[InstitutionalIndustrySummary](t, "institutional_industry_summary.json")
}

// Exact values copied from crates/libfmp/tests/institutional_ownership_responses.rs.
func TestDocumentedFilingExtractAndDatesFixturesDecodeExactly(t *testing.T) {
	t.Parallel()
	filings := assertFixtureParity[InstitutionalOwnershipFiling](t, "latest_institutional_ownership_filings.json")
	wantFiling := InstitutionalOwnershipFiling{
		Cik: "0001803005", Name: "WEALTH ADVISORS OF IOWA, LLC", Date: mustParseDate(t, "2026-06-30"),
		FilingDate: mustParseDateTime(t, "2026-07-30 00:00:00"), AcceptedDate: mustParseDateTime(t, "2026-07-30 13:14:23"),
		FormType:  "13F-HR",
		Link:      "https://www.sec.gov/Archives/edgar/data/1803005/000180300526000003/0001803005-26-000003-index.htm",
		FinalLink: "https://www.sec.gov/Archives/edgar/data/1803005/000180300526000003/xslForm13F_X02/primary_doc.xml",
	}
	if len(filings) != 1 || filings[0] != wantFiling {
		t.Fatalf("latest_institutional_ownership_filings = %+v, want %+v", filings, wantFiling)
	}
	if got := memberSet(t, filings[0]); len(got) != 8 {
		t.Fatalf("re-encoded filing members = %d, want 8", len(got))
	}

	holdings := assertFixtureParity[InstitutionalHolding](t, "institutional_ownership_extract.json")
	wantHolding := InstitutionalHolding{
		Date: mustParseDate(t, "2023-09-30"), FilingDate: mustParseDate(t, "2023-11-13"),
		AcceptedDate: mustParseDate(t, "2023-11-13"), Cik: "0001388838", SecurityCusip: "674215207", Symbol: "CHRD",
		NameOfIssuer: "CHORD ENERGY CORPORATION", Shares: 13_280, TitleOfClass: "COM NEW", SharesType: "SH",
		PutCallShare: "", Value: 2_152_290,
		Link:      "https://www.sec.gov/Archives/edgar/data/1388838/000117266123003760/0001172661-23-003760-index.htm",
		FinalLink: "https://www.sec.gov/Archives/edgar/data/1388838/000117266123003760/infotable.xml",
	}
	if len(holdings) != 1 || holdings[0] != wantHolding {
		t.Fatalf("institutional_ownership_extract = %+v, want %+v", holdings, wantHolding)
	}
	if got := memberSet(t, holdings[0]); len(got) != 14 {
		t.Fatalf("re-encoded holding members = %d, want 14", len(got))
	}

	dates := assertFixtureParity[Form13fFilingDate](t, "form_13f_filing_dates.json")
	wantDate := Form13fFilingDate{Date: mustParseDate(t, "2026-03-31"), Year: 2026, Quarter: 1}
	if len(dates) != 1 || dates[0] != wantDate {
		t.Fatalf("form_13f_filing_dates = %+v, want %+v", dates, wantDate)
	}
}

// Spot values copied from crates/libfmp/tests/institutional_holder_analytics_responses.rs
// and institutional_holder_summaries_responses.rs; the member counts mirror
// the Rust field-count assertions (39, 33, 12).
func TestDocumentedHolderFixturesDecodeAnalyticsPerformanceAndIndustryRows(t *testing.T) {
	t.Parallel()
	analytics := assertFixtureParity[InstitutionalHolderAnalytics](t, "institutional_holder_analytics.json")
	if len(analytics) != 1 || analytics[0].Cik != "0000102909" || analytics[0].SecurityCusip != "037833100" ||
		analytics[0].InvestorName != "VANGUARD GROUP INC" || analytics[0].FilingDate != mustParseDate(t, "2023-12-18") ||
		analytics[0].FirstAdded != mustParseDate(t, "2005-03-31") || analytics[0].QuarterEndPrice != 171.21 ||
		analytics[0].AvgPricePaid != 20.65 || analytics[0].ChangeInPerformance != -67_750_129_670 ||
		analytics[0].ChangeInSharesNumber != -3_691_373 || analytics[0].HoldingPeriod != 75 ||
		analytics[0].IsNew || analytics[0].IsSoldOut || !analytics[0].IsCountedForPerformance {
		t.Fatalf("institutional_holder_analytics = %+v", analytics)
	}
	if got := memberSet(t, analytics[0]); len(got) != 39 {
		t.Fatalf("re-encoded analytics members = %d, want 39", len(got))
	}

	performance := assertFixtureParity[HolderPerformanceSummary](t, "holder_performance_summary.json")
	if len(performance) != 1 || performance[0].Cik != "0001067983" || performance[0].Date != mustParseDate(t, "2026-03-31") ||
		performance[0].PortfolioSize != 29 || performance[0].AverageHoldingPeriodTop10 != 32 ||
		performance[0].ChangeInPerformance != -14_398_745_159 || performance[0].Performance1Year != 28_972_527_543 ||
		performance[0].PerformancePercentage5Year != 63.1842 ||
		performance[0].PerformanceSinceInceptionRelativeToSp500Percentage != -114.003 {
		t.Fatalf("holder_performance_summary = %+v", performance)
	}
	if got := memberSet(t, performance[0]); len(got) != 33 {
		t.Fatalf("re-encoded performance members = %d, want 33", len(got))
	}

	industry := assertFixtureParity[HolderIndustryBreakdown](t, "holder_industry_breakdown.json")
	want := HolderIndustryBreakdown{
		Date: mustParseDate(t, "2023-09-30"), Cik: "0001067983", InvestorName: "BERKSHIRE HATHAWAY INC",
		IndustryTitle: "ELECTRONIC COMPUTERS", Weight: 49.7704, LastWeight: 51.0035, ChangeInWeight: -1.2332,
		ChangeInWeightPercentage: -2.4178, Performance: -20_838_154_294, PerformancePercentage: -178.2938,
		LastPerformance: 26_615_340_304, ChangeInPerformance: -47_453_494_598,
	}
	if len(industry) != 1 || industry[0] != want {
		t.Fatalf("holder_industry_breakdown = %+v, want %+v", industry, want)
	}
}

// Spot values copied from crates/libfmp/tests/institutional_position_industry_summaries_responses.rs.
func TestDocumentedPositionAndIndustrySummaryFixturesDecodeExactly(t *testing.T) {
	t.Parallel()
	positions := assertFixtureParity[InstitutionalPositionSummary](t, "institutional_positions_summary.json")
	if len(positions) != 1 || positions[0].Symbol != "AAPL" || positions[0].Cik != "0000320193" ||
		positions[0].Date != mustParseDate(t, "2023-09-30") || positions[0].InvestorsHolding != 4_863 ||
		positions[0].NumberOf13fShares != 9_139_920_744 || positions[0].NumberOf13fSharesChange != -221_018_965 ||
		positions[0].TotalInvestedChange != -245_052_087_186 || positions[0].OwnershipPercent != 58.5914 ||
		positions[0].NewPositionsChange != -29 || positions[0].PutCallRatioChange != 22.0952 {
		t.Fatalf("institutional_positions_summary = %+v", positions)
	}
	if got := memberSet(t, positions[0]); len(got) != 36 {
		t.Fatalf("re-encoded position members = %d, want 36", len(got))
	}

	industry := assertFixtureParity[InstitutionalIndustrySummary](t, "institutional_industry_summary.json")
	want := InstitutionalIndustrySummary{
		IndustryTitle: "ABRASIVE, ASBESTOS & MISC NONMETALLIC MINERAL PRODS", IndustryValue: 11_088_059_691,
		Date: mustParseDate(t, "2023-09-30"),
	}
	if len(industry) != 1 || industry[0] != want {
		t.Fatalf("institutional_industry_summary = %+v, want %+v", industry, want)
	}
}

// Mirrors shares_and_value_decode_large_fractional_and_negative_numbers_but_reject_text
// and amounts_and_shares_decode_large_and_negative_values_and_holding_period_stays_u64.
func TestInstitutionalOwnershipAmountsAndSharesDecodeAsFloat64(t *testing.T) {
	t.Parallel()
	rewrite := func(fixture string, members map[string]string) []byte {
		t.Helper()
		var rows []map[string]jsontext.Value
		if err := json.Unmarshal(readFixture(t, fixture), &rows); err != nil {
			t.Fatal(err)
		}
		for member, value := range members {
			rows[0][member] = jsontext.Value(value)
		}
		encoded, err := json.Marshal(rows)
		if err != nil {
			t.Fatal(err)
		}
		return encoded
	}
	var holdings []InstitutionalHolding
	if err := json.Unmarshal(rewrite("institutional_ownership_extract.json",
		map[string]string{"shares": "-1", "value": "1.5"}), &holdings); err != nil ||
		holdings[0].Shares != -1 || holdings[0].Value != 1.5 {
		t.Fatalf("shares and value = %+v, %v", holdings, err)
	}
	if err := json.Unmarshal(rewrite("institutional_ownership_extract.json",
		map[string]string{"shares": `"13280"`}), &holdings); err == nil {
		t.Fatal("string shares decoded into a float64 member")
	}
	var analytics []InstitutionalHolderAnalytics
	if err := json.Unmarshal(rewrite("institutional_holder_analytics.json", map[string]string{
		"lastMarketValue": "18446744073709551615", "marketValue": "9007199254740993",
		"changeInMarketValue": "-9223372036854775808", "changeInSharesNumber": "-4294967297",
		"holdingPeriod": "18446744073709551615",
	}), &analytics); err != nil || analytics[0].LastMarketValue != 18_446_744_073_709_551_616 ||
		analytics[0].MarketValue != 9_007_199_254_740_992 || analytics[0].ChangeInMarketValue != math.MinInt64 ||
		analytics[0].ChangeInSharesNumber != -4_294_967_297 || analytics[0].HoldingPeriod != math.MaxUint64 {
		t.Fatalf("analytics amounts = %+v, %v", analytics, err)
	}
}

// The missing-required-member path of the generated decoders, once for this
// domain: serde rejects a missing and a null member alike. The temporal cases
// mirror date_and_datetime_fields_reject_each_others_wire_kinds.
func TestInstitutionalOwnershipRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing member", `[{"industryTitle":"ELECTRONIC COMPUTERS","date":"2023-09-30"}]`, "industryValue"},
		{"null member", `[{"industryTitle":"ELECTRONIC COMPUTERS","industryValue":null,"date":"2023-09-30"}]`, "industryValue"},
		{"empty object", `[{}]`, "industryTitle"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []InstitutionalIndustrySummary
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) ||
				!strings.Contains(typed.Message, "InstitutionalIndustrySummary") {
				t.Fatalf("message = %q, want it to name member %q of InstitutionalIndustrySummary", typed.Message, tc.member)
			}
		})
	}
	var filings []InstitutionalOwnershipFiling
	rewritten := strings.Replace(string(readFixture(t, "latest_institutional_ownership_filings.json")),
		`"filingDate": "2026-07-30 00:00:00"`, `"filingDate": "2026-07-30"`, 1)
	if err := json.Unmarshal([]byte(rewritten), &filings); err == nil {
		t.Fatal("a date-only value decoded into the DateTime member filingDate")
	}
	rewritten = strings.Replace(string(readFixture(t, "latest_institutional_ownership_filings.json")),
		`"date": "2026-06-30"`, `"date": "2026-06-30 00:00:00"`, 1)
	if err := json.Unmarshal([]byte(rewritten), &filings); err == nil {
		t.Fatal("a timestamp decoded into the Date member date")
	}
}
