package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

func TestTipranksFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[TipRanksRatingSearchResult](t, "tipranks_ratings_search.json")
	assertFixtureParity[TipRanksPointInTimeRating](t, "tipranks_point_in_time_symbol.json")
	assertFixtureParity[TipRanksPointInTimeRating](t, "tipranks_point_in_time_analyst.json")
	assertFixtureParity[TipRanksSymbolSummary](t, "tipranks_symbol_summary.json")
	assertFixtureParity[TipRanksAnalystSummary](t, "tipranks_analyst_summary.json")
	assertFixtureParity[TipRanksFirmSummary](t, "tipranks_firm_summary.json")
	assertFixtureParity[TipRanksAnalystProfile](t, "tipranks_analysts.json")
}

// Exact values copied from crates/libfmp/tests/tipranks_search_responses.rs:
// the exact "expertUID" acronym key, the ISO timestamp kept as text, and the
// price target kept in its integer wire spelling.
func TestDocumentedTipranksRatingSearchDecodesAllThirteenFields(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[TipRanksRatingSearchResult](t, "tipranks_ratings_search.json")
	if len(rows) != 1 {
		t.Fatalf("rows = %d, want 1", len(rows))
	}
	row := rows[0]
	if row.Symbol != "RR.L" || row.Date != "2026-07-30T16:40:58.403Z" ||
		row.RecommendationDate != mustParseDate(t, "2026-07-30") ||
		row.ExpertUID != "9d6962cbd29862b8d70de0a2ddb3eb0bdfedc2b7" || row.AnalystName != "Ross Law" ||
		row.FirmName != "Morgan Stanley" || row.Recommendation != "buy" || row.AnalystAction != "maintained" ||
		row.ArticleSite != "TipRanks Contributor" || string(row.PriceTarget) != "1500" ||
		row.PriceTargetCurrency != "GBX" || !strings.HasPrefix(row.URL, "https://www.tipranks.com/news/blurbs/") {
		t.Fatalf("search_tipranks_ratings = %+v", row)
	}
	members := memberSet(t, row)
	if len(members) != 13 {
		t.Fatalf("re-encoded members = %d, want the documented 13", len(members))
	}
	for _, wrong := range []string{"expertUid", "expert_uid", "expertId"} {
		for _, member := range members {
			if member == wrong {
				t.Fatalf("emitted %q instead of the exact expertUID key", wrong)
			}
		}
	}
}

// Exact values copied from crates/libfmp/tests/tipranks_point_in_time_responses.rs:
// the symbol snapshot carries every nullable member and the analyst snapshot
// carries all four as null, which decode to nil and re-encode as null.
func TestDocumentedTipranksPointInTimeRatingsDecodeValuesAndNulls(t *testing.T) {
	t.Parallel()
	symbol := assertFixtureParity[TipRanksPointInTimeRating](t, "tipranks_point_in_time_symbol.json")
	if len(symbol) != 1 {
		t.Fatalf("rows = %d, want 1", len(symbol))
	}
	row := symbol[0]
	if row.Symbol != "AAPL" || row.Date != "2026-07-29T09:30:14.797Z" ||
		row.ExpertUID != "d970a430f313c453df61608e96e87edee44e3dce" || row.AnalystName != "Wamsi Mohan" ||
		string(row.StockSuccessRate) != "0.788" || row.FirmName != "Bank of America Securities" ||
		row.LastRecommendation != "buy" || row.LastRecommendationDate != mustParseDate(t, "2026-07-28") ||
		row.LastAnalystAction != "reiterated" || row.PriceTarget == nil || string(*row.PriceTarget) != "380" ||
		row.PriceTargetCurrency == nil || *row.PriceTargetCurrency != "USD" ||
		row.StockReturn == nil || string(*row.StockReturn) != "-0.1253" || row.BeatTarget == nil || *row.BeatTarget {
		t.Fatalf("tipranks_point_in_time_symbol = %+v", row)
	}
	if members := memberSet(t, row); len(members) != 16 {
		t.Fatalf("re-encoded members = %d, want the documented 16", len(members))
	}

	analyst := assertFixtureParity[TipRanksPointInTimeRating](t, "tipranks_point_in_time_analyst.json")[0]
	if analyst.Symbol != "0J3K.L" || string(analyst.StockSuccessRate) != "0" || analyst.LastRecommendation != "Hold" ||
		analyst.LastAnalystAction != "maintained" || analyst.PriceTarget != nil || analyst.PriceTargetCurrency != nil ||
		analyst.StockReturn != nil || analyst.BeatTarget != nil {
		t.Fatalf("tipranks_point_in_time_analyst = %+v", analyst)
	}
	encoded, err := json.Marshal(analyst)
	if err != nil {
		t.Fatal(err)
	}
	for _, member := range []string{`"priceTarget":null`, `"priceTargetCurrency":null`, `"stockReturn":null`, `"beatTarget":null`} {
		if !strings.Contains(string(encoded), member) {
			t.Fatalf("re-encoded row lost the null member %s: %s", member, encoded)
		}
	}
}

// Exact values copied from crates/libfmp/tests/tipranks_summary_responses.rs:
// the three summary shapes share the nested count objects and keep the
// returns in their wire spelling, including the integer -1.
func TestDocumentedTipranksSummariesDecodeExactValues(t *testing.T) {
	t.Parallel()
	symbols := assertFixtureParity[TipRanksSymbolSummary](t, "tipranks_symbol_summary.json")
	if len(symbols) != 1 {
		t.Fatalf("rows = %d, want 1", len(symbols))
	}
	symbol := symbols[0]
	wantRecommendations := TipRanksRecommendationCounts{Buy: 277, Hold: 122, Sell: 26}
	wantActions := TipRanksAnalystActionCounts{Initiated: 3, Maintained: 309, Upgraded: 16, Downgraded: 5, Reiterated: 92, Resumed: 0}
	if symbol.Symbol != "AAPL" || symbol.From != mustParseDate(t, "2025-07-30") || symbol.To != mustParseDate(t, "2026-07-30") ||
		symbol.TotalRecommendations != 425 || symbol.DistinctSymbols != 1 || symbol.DistinctAnalysts != 45 ||
		symbol.ValidPriceTargets != 379 || symbol.Recommendations != wantRecommendations || symbol.AnalystAction != wantActions ||
		symbol.ComparedPriceTargets != 379 || symbol.Beats != 323 || symbol.Misses != 56 ||
		string(symbol.AverageReturn) != "0.1697" || string(symbol.TopReturn) != "0.8466" || string(symbol.WorstReturn) != "-0.169" {
		t.Fatalf("tipranks_symbol_summary = %+v", symbol)
	}
	if members := memberSet(t, symbol); len(members) != 15 {
		t.Fatalf("re-encoded members = %d, want the documented 15", len(members))
	}

	analyst := assertFixtureParity[TipRanksAnalystSummary](t, "tipranks_analyst_summary.json")[0]
	if analyst.ExpertUID != "3c6eb8cf1347a4e5757e628fccb684a93abee587" || analyst.TotalRecommendations != 22 ||
		analyst.Recommendations.Sell != 0 || analyst.Beats != 2 || analyst.Misses != 11 ||
		string(analyst.AverageReturn) != "-0.1206" {
		t.Fatalf("tipranks_analyst_summary = %+v", analyst)
	}

	firm := assertFixtureParity[TipRanksFirmSummary](t, "tipranks_firm_summary.json")[0]
	if firm.FirmName != "Morgan Stanley" || firm.TotalRecommendations != 14_182 || firm.DistinctSymbols != 4_146 ||
		firm.DistinctAnalysts != 258 || firm.AnalystAction.Reiterated != 1_583 ||
		string(firm.TopReturn) != "17.0036" || string(firm.WorstReturn) != "-1" {
		t.Fatalf("tipranks_firm_summary = %+v", firm)
	}
	encoded, err := json.Marshal(firm)
	if err != nil {
		t.Fatal(err)
	}
	if text := string(encoded); !strings.Contains(text, `"worstReturn":-1`) || !strings.Contains(text, `"analystAction":{`) ||
		strings.Contains(text, "analystActions") {
		t.Fatalf("re-encoded firm summary rewrote a return or the analystAction key: %s", text)
	}
}

// Exact values copied from crates/libfmp/tests/tipranks_directory_responses.rs.
func TestDocumentedTipranksAnalystProfilesDecodeAllNineFields(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[TipRanksAnalystProfile](t, "tipranks_analysts.json")
	if len(rows) != 1 {
		t.Fatalf("rows = %d, want 1", len(rows))
	}
	row := rows[0]
	if row.ExpertUID != "0458d251af4db6d595c17e02da3bc6ae4bb093b0" || row.AnalystName != "Sujeeva De Silva" ||
		row.FirmName != "Roth MKM" || string(row.SuccessRate) != "0.617" || string(row.ExcessReturn) != "0.563" ||
		row.TotalRecommendations != 496 || row.GoodRecommendations != 306 || row.AnalystRank != 24 || row.NumOfStars != 5 {
		t.Fatalf("tipranks_analysts = %+v", row)
	}
	if members := memberSet(t, row); len(members) != 9 {
		t.Fatalf("re-encoded members = %d, want the documented 9", len(members))
	}
}

// Required keys, the required-present nullable members (a number, a string,
// and a bool), the number-kind check, and the nested count objects are
// enforced as the Rust decoder enforces them, once for this domain.
func TestTipranksRequiredMembersAndCodecsAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	const search = "tipranks_ratings_search.json"
	const pit = "tipranks_point_in_time_symbol.json"
	const summary = "tipranks_symbol_summary.json"
	decode := func(t *testing.T, fixture string, wire []byte) error {
		t.Helper()
		switch fixture {
		case search:
			var rows []TipRanksRatingSearchResult
			return json.Unmarshal(wire, &rows)
		case pit:
			var rows []TipRanksPointInTimeRating
			return json.Unmarshal(wire, &rows)
		default:
			var rows []TipRanksSymbolSummary
			return json.Unmarshal(wire, &rows)
		}
	}
	rejected := []struct {
		name    string
		fixture string
		member  string
		value   jsontext.Value
		message string
	}{
		{"missing symbol", search, "symbol", nil,
			`required member "symbol" of TipRanksRatingSearchResult is missing or null`},
		{"null symbol", search, "symbol", jsontext.Value(`null`),
			`required member "symbol" of TipRanksRatingSearchResult is missing or null`},
		{"null search price target", search, "priceTarget", jsontext.Value(`null`),
			`required member "priceTarget" of TipRanksRatingSearchResult is missing or null`},
		{"string search price target", search, "priceTarget", jsontext.Value(`"1500"`),
			`member "priceTarget" of TipRanksRatingSearchResult must be a JSON number`},
		{"missing nullable price target", pit, "priceTarget", nil,
			`required member "priceTarget" of TipRanksPointInTimeRating is missing or null`},
		{"missing nullable currency", pit, "priceTargetCurrency", nil,
			`required member "priceTargetCurrency" of TipRanksPointInTimeRating is missing or null`},
		{"missing nullable beat target", pit, "beatTarget", nil,
			`required member "beatTarget" of TipRanksPointInTimeRating is missing or null`},
		{"string nullable price target", pit, "priceTarget", jsontext.Value(`"380"`),
			`member "priceTarget" of TipRanksPointInTimeRating must be a JSON number`},
		{"string success rate", pit, "stockSuccessRate", jsontext.Value(`"0.5"`),
			`member "stockSuccessRate" of TipRanksPointInTimeRating must be a JSON number`},
		{"missing recommendations", summary, "recommendations", nil,
			`required member "recommendations" of TipRanksSymbolSummary is missing or null`},
		{"missing nested buy", summary, "recommendations.buy", nil,
			`required member "buy" of TipRanksRecommendationCounts is missing or null`},
		{"null nested initiated", summary, "analystAction.initiated", jsontext.Value(`null`),
			`required member "initiated" of TipRanksAnalystActionCounts is missing or null`},
		{"string average return", summary, "averageReturn", jsontext.Value(`"-1"`),
			`member "averageReturn" of TipRanksSymbolSummary must be a JSON number`},
	}
	for _, tc := range rejected {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			err := decode(t, tc.fixture, mutateFixtureMember(t, tc.fixture, tc.member, tc.value))
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode || typed.Message != tc.message {
				t.Fatalf("error = %v (%T), want CategoryDecode %q", err, err, tc.message)
			}
		})
	}
	accepted := []struct {
		name    string
		fixture string
		member  string
		value   jsontext.Value
	}{
		{"null price target", pit, "priceTarget", jsontext.Value(`null`)},
		{"null currency", pit, "priceTargetCurrency", jsontext.Value(`null`)},
		{"null stock return", pit, "stockReturn", jsontext.Value(`null`)},
		{"null beat target", pit, "beatTarget", jsontext.Value(`null`)},
		{"integer stock return", pit, "stockReturn", jsontext.Value(`-2`)},
		{"integer average return", summary, "averageReturn", jsontext.Value(`-1`)},
		{"unknown nested member", summary, "recommendations.futureRating", jsontext.Value(`9`)},
	}
	for _, tc := range accepted {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			if err := decode(t, tc.fixture, mutateFixtureMember(t, tc.fixture, tc.member, tc.value)); err != nil {
				t.Fatalf("rejected: %v", err)
			}
		})
	}
	wrongKinds := []struct {
		name    string
		fixture string
		member  string
		value   jsontext.Value
	}{
		{"numeric beat target", pit, "beatTarget", jsontext.Value(`0`)},
		{"string count", summary, "totalRecommendations", jsontext.Value(`"1"`)},
		{"negative count", summary, "beats", jsontext.Value(`-1`)},
		{"fractional nested count", summary, "recommendations.hold", jsontext.Value(`1.5`)},
		{"datetime as date", summary, "from", jsontext.Value(`"2025-07-30T00:00:00Z"`)},
	}
	for _, tc := range wrongKinds {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			if err := decode(t, tc.fixture, mutateFixtureMember(t, tc.fixture, tc.member, tc.value)); err == nil {
				t.Fatalf("%s decoded into %s", tc.value, tc.member)
			}
		})
	}
}
