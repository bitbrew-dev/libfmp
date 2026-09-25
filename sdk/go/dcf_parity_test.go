package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"fmt"
	"slices"
	"strings"
	"testing"
)

func TestDcfFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[DCFValuation](t, "discounted_cash_flow.json")
	assertFixtureParity[DCFValuation](t, "levered_discounted_cash_flow.json")
	assertFixtureParity[CustomDCFValuation](t, "custom_discounted_cash_flow.json")
	assertFixtureParity[CustomLeveredDCFValuation](t, "custom_levered_discounted_cash_flow.json")
}

// Exact values copied from crates/libfmp/tests/dcf_responses.rs.
func TestDcfOuterFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	standard := assertFixtureParity[DCFValuation](t, "discounted_cash_flow.json")
	want := DCFValuation{Symbol: "AAPL", Date: mustParseDate(t, "2026-07-30"), DCF: 147.10881272667325,
		StockPrice: 338.19}
	if len(standard) != 1 || standard[0] != want {
		t.Fatalf("discounted_cash_flow = %+v, want %+v", standard, want)
	}
	levered := assertFixtureParity[DCFValuation](t, "levered_discounted_cash_flow.json")
	if len(levered) != 1 || levered[0].DCF != 140.6429495133426 || levered[0].StockPrice != 338.19 {
		t.Fatalf("levered_discounted_cash_flow = %+v", levered)
	}
	if members := memberSet(t, standard[0]); !slices.Contains(members, "Stock Price") ||
		slices.Contains(members, "stockPrice") || slices.Contains(members, "stock_price") {
		t.Fatalf("re-encoded members %v, want the exact wire key \"Stock Price\"", members)
	}
}

// Exact values copied from crates/libfmp/tests/dcf_custom_responses.rs.
func TestDcfCustomFixturesDecodeExactValuesIntoDistinctModels(t *testing.T) {
	t.Parallel()
	custom := assertFixtureParity[CustomDCFValuation](t, "custom_discounted_cash_flow.json")
	if len(custom) != 1 || custom[0].Year != "2030" || custom[0].Symbol != "AAPL" ||
		custom[0].CapitalExpenditure != -14_907_445_037 || custom[0].DilutedSharesOutstanding != 15_004_697_000 ||
		custom[0].EquityValuePerShare != 147.18 || custom[0].CostOfDebt != 4.37 {
		t.Fatalf("custom_discounted_cash_flow = %+v", custom)
	}
	levered := assertFixtureParity[CustomLeveredDCFValuation](t, "custom_levered_discounted_cash_flow.json")
	if len(levered) != 1 || levered[0].OperatingCashFlow != 153_867_620_418 || levered[0].PvLfcf != 88_605_139_549 ||
		levered[0].EquityValuePerShare != 140.71 || levered[0].CostOfDebt != 4.37 {
		t.Fatalf("custom_levered_discounted_cash_flow = %+v", levered)
	}
	for _, members := range [][]string{memberSet(t, custom[0]), memberSet(t, levered[0])} {
		if !slices.Contains(members, "costofDebt") || slices.Contains(members, "costOfDebt") ||
			slices.Contains(members, "cost_of_debt") {
			t.Fatalf("re-encoded members %v, want the exact wire key costofDebt", members)
		}
	}
	if custom, levered := memberSet(t, custom[0]), memberSet(t, levered[0]); len(custom) != 47 || len(levered) != 34 {
		t.Fatalf("member counts = %d and %d, want 47 and 34", len(custom), len(levered))
	}
}

// Mirrors the required-field loops of the Rust tests: removing or nulling
// any documented member of the first row is rejected with a Decode error
// naming that member, and a future member is accepted.
func TestDcfEveryDocumentedMemberIsRequiredNonNull(t *testing.T) {
	t.Parallel()
	dcfAssertEveryMemberRequired[DCFValuation](t, "discounted_cash_flow.json")
	dcfAssertEveryMemberRequired[CustomDCFValuation](t, "custom_discounted_cash_flow.json")
	dcfAssertEveryMemberRequired[CustomLeveredDCFValuation](t, "custom_levered_discounted_cash_flow.json")

	var rows []DCFValuation
	if err := json.Unmarshal([]byte("[]"), &rows); err != nil || len(rows) != 0 {
		t.Fatalf("bare empty array = %v, %v", rows, err)
	}
	if err := json.Unmarshal([]byte("{}"), &rows); err == nil {
		t.Fatal("an object root decoded into a bare-array contract")
	}
}

func dcfAssertEveryMemberRequired[T any](t *testing.T, name string) {
	t.Helper()
	var wire []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, name), &wire); err != nil || len(wire) == 0 {
		t.Fatalf("%s: fixture rows = %d, %v", name, len(wire), err)
	}
	model := fmt.Sprintf("%T", *new(T))[len("fmp."):]
	for member := range wire[0] {
		for _, variant := range []string{"missing", "null"} {
			row := make(map[string]jsontext.Value, len(wire[0]))
			for key, value := range wire[0] {
				row[key] = value
			}
			if variant == "missing" {
				delete(row, member)
			} else {
				row[member] = jsontext.Value("null")
			}
			encoded, err := json.Marshal(row)
			if err != nil {
				t.Fatalf("%s: re-encode without %q: %v", name, member, err)
			}
			var decoded T
			err = json.Unmarshal(encoded, &decoded)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("%s: %s %q accepted: %v", name, variant, member, err)
			}
			if !strings.Contains(typed.Message, `"`+member+`"`) || !strings.Contains(typed.Message, model) {
				t.Fatalf("%s: %s %q: message %q does not name the member and %s", name, variant, member,
					typed.Message, model)
			}
		}
	}
}
