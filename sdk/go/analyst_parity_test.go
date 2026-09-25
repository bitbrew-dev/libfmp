package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"math"
	"strings"
	"testing"
	"time"
)

// The exact publishers text of price_target_summary.json, copied from
// crates/libfmp/tests/analyst_responses.rs: the provider sends JSON-array text
// inside a string and the model keeps it verbatim.
const analystPublishers = `["StreetInsider","TheFly","Benzinga","Pulse 2.0","TipRanks Contributor","MarketWatch",` +
	`"Investing","Barrons","Investor's Business Daily"]`

func mustNewDate(t *testing.T, year int, month time.Month, day int) Date {
	t.Helper()
	date, err := NewDate(year, month, day)
	if err != nil {
		t.Fatal(err)
	}
	return date
}

func TestAnalystFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[FinancialEstimate](t, "financial_estimates.json")
	assertFixtureParity[RatingSnapshot](t, "ratings_snapshot.json")
	assertFixtureParity[HistoricalRating](t, "historical_ratings.json")
	assertFixtureParity[PriceTargetSummary](t, "price_target_summary.json")
	assertFixtureParity[PriceTargetConsensus](t, "price_target_consensus.json")
	assertFixtureParity[StockGrade](t, "stock_grades.json")
	assertFixtureParity[HistoricalStockGrade](t, "historical_stock_grades.json")
	assertFixtureParity[StockGradesSummary](t, "stock_grades_summary.json")
}

// Exact values copied from crates/libfmp/tests/analyst_responses.rs.
func TestDocumentedFinancialEstimateDecodesAll22Fields(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[FinancialEstimate](t, "financial_estimates.json")
	want := FinancialEstimate{
		Symbol: "AAPL", Date: mustNewDate(t, 2030, time.September, 27),
		RevenueLow: 648_228_509_004, RevenueHigh: 735_022_980_353, RevenueAvg: 679_000_000_000,
		EbitdaLow: 233_968_328_102, EbitdaHigh: 265_295_486_763, EbitdaAvg: 245_074_834_838,
		EbitLow: 217_109_092_822, EbitHigh: 246_178_886_382, EbitAvg: 227_415_289_483,
		NetIncomeLow: 191_547_261_069, NetIncomeHigh: 225_370_398_908, NetIncomeAvg: 203_538_714_818,
		SgaExpenseLow: 41_721_580_524, SgaExpenseHigh: 47_307_886_087, SgaExpenseAvg: 43_702_109_337,
		EPSAvg: 13.565, EPSHigh: 15.01999, EPSLow: 12.76582,
		NumAnalystsRevenue: 16, NumAnalystsEPS: 7,
	}
	if len(rows) != 1 || rows[0] != want {
		t.Fatalf("financial_estimates = %+v, want %+v", rows, want)
	}
	if got := memberSet(t, rows[0]); len(got) != 22 {
		t.Fatalf("re-encoded members = %d, want 22", len(got))
	}
}

func TestDocumentedRatingFixturesKeepSnapshotAndHistoricalRowsDistinct(t *testing.T) {
	t.Parallel()
	snapshots := assertFixtureParity[RatingSnapshot](t, "ratings_snapshot.json")
	if len(snapshots) != 1 || snapshots[0].Rating != "B" || snapshots[0].OverallScore != 3 ||
		snapshots[0].ReturnOnEquityScore != 5 {
		t.Fatalf("ratings_snapshot = %+v", snapshots)
	}
	historical := assertFixtureParity[HistoricalRating](t, "historical_ratings.json")
	if len(historical) != 1 || historical[0].Date != mustNewDate(t, 2026, time.July, 30) ||
		historical[0].PriceToBookScore != 1 {
		t.Fatalf("historical_ratings = %+v", historical)
	}
}

func TestDocumentedPriceTargetFixturesPreserveRawPublishersAndNumericPrices(t *testing.T) {
	t.Parallel()
	summaries := assertFixtureParity[PriceTargetSummary](t, "price_target_summary.json")
	if len(summaries) != 1 || summaries[0].Publishers != analystPublishers ||
		summaries[0].LastMonthAvgPriceTarget != 333.75 || summaries[0].AllTimeCount != 254 {
		t.Fatalf("price_target_summary = %+v", summaries)
	}
	consensus := assertFixtureParity[PriceTargetConsensus](t, "price_target_consensus.json")
	want := PriceTargetConsensus{Symbol: "AAPL", TargetHigh: 400, TargetLow: 250, TargetConsensus: 337.67,
		TargetMedian: 340}
	if len(consensus) != 1 || consensus[0] != want {
		t.Fatalf("price_target_consensus = %+v, want %+v", consensus, want)
	}
}

func TestDocumentedGradeFixturesPreserveBothBucketKeyFamilies(t *testing.T) {
	t.Parallel()
	grades := assertFixtureParity[StockGrade](t, "stock_grades.json")
	want := StockGrade{Symbol: "AAPL", Date: mustNewDate(t, 2026, time.July, 23), GradingCompany: "Morgan Stanley",
		PreviousGrade: "Overweight", NewGrade: "Overweight", Action: "maintain"}
	if len(grades) != 1 || grades[0] != want {
		t.Fatalf("stock_grades = %+v, want %+v", grades, want)
	}
	historical := assertFixtureParity[HistoricalStockGrade](t, "historical_stock_grades.json")
	if len(historical) != 1 || historical[0].AnalystRatingsStrongBuy != 6 ||
		historical[0].AnalystRatingsStrongSell != 2 || historical[0].AnalystRatingsBuy != 23 {
		t.Fatalf("historical_stock_grades = %+v", historical)
	}
	summary := assertFixtureParity[StockGradesSummary](t, "stock_grades_summary.json")
	if len(summary) != 1 || summary[0].StrongBuy != 1 || summary[0].StrongSell != 0 || summary[0].Consensus != "Buy" {
		t.Fatalf("stock_grades_summary = %+v", summary)
	}
}

// Mirrors estimate_amounts_are_signed_and_counts_preserve_the_full_u64_domain:
// StatementAmount members hold the whole int64 range and Count members the
// whole uint64 range without a float64 round trip.
func TestAnalystAmountsAreSignedAndCountsPreserveTheFullUint64Domain(t *testing.T) {
	t.Parallel()
	rewrite := func(fixture, member, value string) []byte {
		t.Helper()
		var rows []map[string]jsontext.Value
		if err := json.Unmarshal(readFixture(t, fixture), &rows); err != nil {
			t.Fatal(err)
		}
		rows[0][member] = jsontext.Value(value)
		encoded, err := json.Marshal(rows)
		if err != nil {
			t.Fatal(err)
		}
		return encoded
	}
	var estimates []FinancialEstimate
	if err := json.Unmarshal(rewrite("financial_estimates.json", "revenueLow", "-9223372036854775808"),
		&estimates); err != nil || estimates[0].RevenueLow != math.MinInt64 {
		t.Fatalf("revenueLow = %+v, %v", estimates, err)
	}
	if err := json.Unmarshal(rewrite("financial_estimates.json", "numAnalystsEps", "18446744073709551615"),
		&estimates); err != nil || estimates[0].NumAnalystsEPS != math.MaxUint64 {
		t.Fatalf("numAnalystsEps = %+v, %v", estimates, err)
	}
	var summary []StockGradesSummary
	if err := json.Unmarshal(rewrite("stock_grades_summary.json", "strongBuy", "18446744073709551615"),
		&summary); err != nil || summary[0].StrongBuy != math.MaxUint64 {
		t.Fatalf("strongBuy = %+v, %v", summary, err)
	}
}

// The missing-required-member path of the generated decoders, once for this
// domain: serde rejects a missing and a null member alike.
func TestAnalystRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing member", `[{"symbol":"AAPL","strongBuy":1,"buy":2,"hold":3,"sell":4,"strongSell":5}]`, "consensus"},
		{"null member", `[{"symbol":"AAPL","strongBuy":null,"buy":2,"hold":3,"sell":4,"strongSell":5,"consensus":"Buy"}]`,
			"strongBuy"},
		{"empty object", `[{}]`, "symbol"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []StockGradesSummary
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "StockGradesSummary") {
				t.Fatalf("message = %q, want it to name member %q of StockGradesSummary", typed.Message, tc.member)
			}
		})
	}
	var estimates []FinancialEstimate
	err := json.Unmarshal([]byte(`[{"symbol":"AAPL","date":"2030-09-27"}]`), &estimates)
	var typed *Error
	if !errors.As(err, &typed) || !strings.Contains(typed.Message, `"revenueLow"`) {
		t.Fatalf("FinancialEstimate error = %v, want the first missing member revenueLow", err)
	}
	if err := json.Unmarshal([]byte(`[{"symbol":"AAPL","date":"2030/09/27"}]`), &estimates); err == nil {
		t.Fatal("a malformed date decoded into a Date member")
	}
}
