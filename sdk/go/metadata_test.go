package fmp

import (
	"fmt"
	"reflect"
	"slices"
	"testing"
)

// metadataSample is a fully populated value whose members are all pointers
// or discriminated structs, so the copy and label tests cover every branch.
func metadataSample() EndpointMetadata {
	return EndpointMetadata{
		Geography: GeographyWorldwide,
		Access:    AccessRequirement{Kind: AccessNamedAddOn, AddOn: "TipRanks"},
		ConditionalPlan: &ConditionalPlanRequirement{
			Plan:      "Enterprise",
			Condition: PlanCondition{Kind: PlanConditionHistoryOlderThanYears, Years: 3},
		},
		Realtime: &RealtimeAccess{
			Delay:           &MarketDataDelay{Minutes: 15, Scope: DelayScopeNasdaq},
			UserDeclaration: UserDeclarationRequiredForRealtime,
		},
		Bounds: EndpointBounds{
			Limit:         inclusiveMaximum(5000),
			ResponseRows:  inclusiveMaximum(1000),
			Page:          inclusiveMaximum(100),
			DateRangeDays: inclusiveMaximum(90),
		},
	}
}

func TestEndpointMetadataZeroValueIsUnspecified(t *testing.T) {
	t.Parallel()
	var zero EndpointMetadata
	if zero.Geography != GeographyUnspecified {
		t.Errorf("Geography = %v, want %v", zero.Geography, GeographyUnspecified)
	}
	if zero.Access != (AccessRequirement{}) || zero.Access.Kind != AccessUnspecified {
		t.Errorf("Access = %+v, want unspecified", zero.Access)
	}
	if zero.ConditionalPlan != nil || zero.Realtime != nil {
		t.Errorf("ConditionalPlan = %v, Realtime = %v, want both nil", zero.ConditionalPlan, zero.Realtime)
	}
	if zero.Bounds != (EndpointBounds{}) {
		t.Errorf("Bounds = %+v, want no maxima", zero.Bounds)
	}
	want := "geography unspecified; access unspecified; bounds unbounded"
	if got := zero.String(); got != want {
		t.Errorf("String() = %q, want %q", got, want)
	}
}

func TestEndpointMetadataLabelsAreShortAndStable(t *testing.T) {
	t.Parallel()
	cases := []struct {
		value fmt.Stringer
		want  string
	}{
		{GeographyWorldwide, "worldwide"},
		{GeographyUSOnly, "us-only"},
		{GeographicAvailability(9), "GeographicAvailability(9)"},
		{AccessRequirement{Kind: AccessStandard}, "standard"},
		{AccessRequirement{Kind: AccessNamedAddOn, AddOn: "TipRanks"}, "add-on TipRanks"},
		{AccessKind(9), "AccessKind(9)"},
		{PlanCondition{}, "unspecified"},
		{PlanCondition{Kind: PlanConditionKind(9)}, "PlanCondition(9)"},
		{ConditionalPlanRequirement{Plan: "Enterprise", Condition: PlanCondition{Kind: PlanConditionHistoryOlderThanYears, Years: 3}}, "Enterprise when history older than 3 years"},
		{UserDeclarationRequiredForRealtime, "required-for-realtime"},
		{UserDeclarationRequirement(9), "UserDeclarationRequirement(9)"},
		{DelayScopeNasdaq, "nasdaq"},
		{DelayScope(9), "DelayScope(9)"},
		{MarketDataDelay{Minutes: 15, Scope: DelayScopeNasdaq}, "15 minutes (nasdaq)"},
		{RealtimeAccess{}, "no caveats"},
		{RealtimeAccess{UserDeclaration: UserDeclarationRequiredForRealtime}, "declaration required-for-realtime"},
		{EndpointBounds{Page: inclusiveMaximum(100), DateRangeDays: inclusiveMaximum(90)}, "page<=100, date_range_days<=90"},
		{metadataSample(), "geography worldwide; access add-on TipRanks; plan Enterprise when history older than 3 years; " +
			"realtime delay 15 minutes (nasdaq), declaration required-for-realtime; bounds limit<=5000, response_rows<=1000, page<=100, date_range_days<=90"},
	}
	for _, tc := range cases {
		if got := tc.value.String(); got != tc.want {
			t.Errorf("%T.String() = %q, want %q", tc.value, got, tc.want)
		}
	}
}

func TestEndpointMetadataCloneSharesNoPointer(t *testing.T) {
	t.Parallel()
	original := metadataSample()
	copied := original.clone()
	if copied.String() != original.String() {
		t.Fatalf("clone() = %s, want %s", copied, original)
	}
	if copied.ConditionalPlan == original.ConditionalPlan ||
		copied.Realtime == original.Realtime ||
		copied.Realtime.Delay == original.Realtime.Delay ||
		copied.Bounds.Limit == original.Bounds.Limit ||
		copied.Bounds.ResponseRows == original.Bounds.ResponseRows ||
		copied.Bounds.Page == original.Bounds.Page ||
		copied.Bounds.DateRangeDays == original.Bounds.DateRangeDays {
		t.Fatal("clone() shares a pointer with the original")
	}
	copied.ConditionalPlan.Plan = "Other"
	copied.Realtime.Delay.Minutes = 1
	*copied.Bounds.Limit = 1
	*copied.Bounds.DateRangeDays = 1
	if original.ConditionalPlan.Plan != "Enterprise" || original.Realtime.Delay.Minutes != 15 || *original.Bounds.Limit != 5000 || *original.Bounds.DateRangeDays != 90 {
		t.Fatalf("mutating the clone changed the original: %s", original)
	}
	var zero EndpointMetadata
	if got := zero.clone(); got != (EndpointMetadata{}) {
		t.Fatalf("clone() of the zero value = %+v, want the zero value", got)
	}
}

// endpointMetadataMethods is the number of generated methods whose Rust
// descriptor attaches metadata: the count registry_check prints as
// "metadata ok". The remaining methods (271 in total) report false.
const endpointMetadataMethods = 251

// metadataNasdaqRealtime is the realtime caveat the quote consts share
// (NASDAQ_DELAYED_REALTIME in crates/libfmp/src/endpoints/quote.rs).
func metadataNasdaqRealtime() *RealtimeAccess {
	return &RealtimeAccess{
		Delay:           &MarketDataDelay{Minutes: 15, Scope: DelayScopeNasdaq},
		UserDeclaration: UserDeclarationRequiredForRealtime,
	}
}

func TestEndpointMetadataForPinsValuesFromTheRustConsts(t *testing.T) {
	t.Parallel()
	cases := []struct {
		method string
		want   EndpointMetadata
	}{
		{"Quote.Full", EndpointMetadata{Geography: GeographyWorldwide, Realtime: metadataNasdaqRealtime()}},
		{"Quote.AftermarketTrade", EndpointMetadata{Geography: GeographyUSOnly, Realtime: metadataNasdaqRealtime()}},
		{"TipRanks.SearchRatings", EndpointMetadata{
			Access: AccessRequirement{Kind: AccessNamedAddOn, AddOn: "TipRanks"},
			ConditionalPlan: &ConditionalPlanRequirement{
				Plan:      "Enterprise",
				Condition: PlanCondition{Kind: PlanConditionHistoryOlderThanYears, Years: 3},
			},
			Bounds: EndpointBounds{Limit: inclusiveMaximum(5000), ResponseRows: inclusiveMaximum(5000)},
		}},
		{"Statements.Growth.Income", EndpointMetadata{
			Geography: GeographyWorldwide,
			Bounds:    EndpointBounds{ResponseRows: inclusiveMaximum(1000)},
		}},
	}
	for _, tc := range cases {
		t.Run(tc.method, func(t *testing.T) {
			t.Parallel()
			got, ok := EndpointMetadataFor(tc.method)
			if !ok {
				t.Fatalf("EndpointMetadataFor(%q) reported no metadata", tc.method)
			}
			if !reflect.DeepEqual(got, tc.want) {
				t.Fatalf("EndpointMetadataFor(%q) = %s, want %s", tc.method, got, tc.want)
			}
		})
	}
}

func TestEndpointMetadataForReportsFalseWithTheZeroValue(t *testing.T) {
	t.Parallel()
	for _, method := range []string{
		"Directory.AvailableCountries",
		"Indexes.DowJonesConstituents",
		"quote",
		"Client.Quote.Full",
		"",
	} {
		got, ok := EndpointMetadataFor(method)
		if ok || got != (EndpointMetadata{}) {
			t.Errorf("EndpointMetadataFor(%q) = %s, %v; want the zero value and false", method, got, ok)
		}
	}
}

func TestEndpointMetadataForReturnsACopy(t *testing.T) {
	t.Parallel()
	first, _ := EndpointMetadataFor("TipRanks.SearchRatings")
	first.ConditionalPlan.Plan = "Other"
	*first.Bounds.Limit = 1
	second, _ := EndpointMetadataFor("TipRanks.SearchRatings")
	if second.ConditionalPlan.Plan != "Enterprise" || *second.Bounds.Limit != 5000 {
		t.Fatalf("a mutated lookup changed the table: %s", second)
	}
}

func TestEndpointMetadataTableCoversEveryDescriptorWithMetadata(t *testing.T) {
	t.Parallel()
	if got := len(endpointMetadataTable); got != endpointMetadataMethods {
		t.Fatalf("table holds %d methods, want %d (registry_check: metadata ok)", got, endpointMetadataMethods)
	}
	for method := range endpointMetadataTable {
		if _, ok := EndpointMetadataFor(method); !ok {
			t.Errorf("EndpointMetadataFor(%q) reported false for a table entry", method)
		}
	}
}

// endpointIDsWithConflictingMetadata are the ids whose methods carry more
// than one distinct metadata value: the per-asset chart and quote helpers
// the quote, chart, commodities, crypto, forex, and indexes domains share.
var endpointIDsWithConflictingMetadata = []string{
	"historical-chart/1hour",
	"historical-chart/1min",
	"historical-chart/5min",
	"historical-price-eod/full",
	"historical-price-eod/light",
	"quote",
	"quote-short",
}

func TestEndpointMetadataByIDReturnsEveryMethodSendingTheID(t *testing.T) {
	t.Parallel()
	got := EndpointMetadataByID("analyst-estimates")
	want := []EndpointMethodMetadata{{
		Method: "Analyst.FinancialEstimates",
		Metadata: EndpointMetadata{
			Geography: GeographyWorldwide,
			Bounds:    EndpointBounds{ResponseRows: inclusiveMaximum(1000)},
		},
	}}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("EndpointMetadataByID(analyst-estimates) = %+v, want %+v", got, want)
	}
	quote := EndpointMetadataByID("quote")
	methods := make([]string, 0, len(quote))
	for _, entry := range quote {
		methods = append(methods, entry.Method)
		if want, _ := EndpointMetadataFor(entry.Method); !reflect.DeepEqual(entry.Metadata, want) {
			t.Errorf("entry %s = %s, want %s", entry.Method, entry.Metadata, want)
		}
	}
	if !slices.IsSorted(methods) || !slices.Contains(methods, "Quote.Full") || len(methods) != 4 {
		t.Fatalf("EndpointMetadataByID(quote) methods = %v, want 4 sorted entries including Quote.Full", methods)
	}
}

func TestEndpointMetadataByIDKeepsConflictingIDsUnmerged(t *testing.T) {
	t.Parallel()
	var conflicting []string
	for id := range endpointMethodsByID {
		distinct := map[string]struct{}{}
		for _, entry := range EndpointMetadataByID(id) {
			distinct[entry.Metadata.String()] = struct{}{}
		}
		if len(distinct) > 1 {
			conflicting = append(conflicting, id)
		}
	}
	slices.Sort(conflicting)
	if !slices.Equal(conflicting, endpointIDsWithConflictingMetadata) {
		t.Fatalf("ids with conflicting metadata = %v, want %v", conflicting, endpointIDsWithConflictingMetadata)
	}
}

func TestEndpointMetadataByIDReportsNilWithoutMetadata(t *testing.T) {
	t.Parallel()
	for _, id := range []string{"available-countries", "Quote.Full", "unknown-id", ""} {
		if got := EndpointMetadataByID(id); got != nil {
			t.Errorf("EndpointMetadataByID(%q) = %+v, want nil", id, got)
		}
	}
}

func TestEndpointMethodsByIDCoversEveryTableEntry(t *testing.T) {
	t.Parallel()
	listed := map[string]int{}
	for id, methods := range endpointMethodsByID {
		if !slices.IsSorted(methods) {
			t.Errorf("methods of %q are not sorted: %v", id, methods)
		}
		for _, method := range methods {
			listed[method]++
		}
	}
	for method := range endpointMetadataTable {
		if listed[method] != 1 {
			t.Errorf("%s is listed under %d ids, want exactly 1", method, listed[method])
		}
	}
}

func TestEndpointMetadataByIDReturnsACopy(t *testing.T) {
	t.Parallel()
	first := EndpointMetadataByID("analyst-estimates")
	*first[0].Metadata.Bounds.ResponseRows = 1
	second := EndpointMetadataByID("analyst-estimates")
	if *second[0].Metadata.Bounds.ResponseRows != 1000 {
		t.Fatalf("a mutated lookup changed the table: %s", second[0].Metadata)
	}
}
