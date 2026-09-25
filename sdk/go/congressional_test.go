package fmp

import (
	"context"
	"errors"
	"fmt"
	"math"
	"net/http"
	"strings"
	"testing"
)

// congressionalRoutes is the exact request-URI table of the congressional
// domain, copied from the URL assertions in
// crates/libfmp/tests/congressional_*_endpoints.rs: every query type appears
// at least once, with its no-query and all-setters variants where the Rust
// tests have them.
var congressionalRoutes = map[string]string{
	"/router/stable/senate-latest?page=0&limit=250":                        "congress_senate_latest.json",
	"/router/stable/house-latest?page=4294967295&limit=0":                  "congress_house_latest.json",
	"/router/stable/senate-trades?symbol=BRK.B+%2F+Class+A&page=1&limit=2": "congress_senate_trades.json",
	"/router/stable/senate-trades-by-name?name=Jerry+Moran":                "congress_senate_trades_by_name.json",
	"/router/stable/senate-trades-by-id?page=3&limit=4&senateID=M001242":   "congress_senate_trades_by_id.json",
	"/router/stable/house-trades?symbol=AAPL":                              "congress_house_trades.json",
	"/router/stable/house-trades-by-name?name=James+A.":                    "congress_house_trades_by_name.json",
	"/router/stable/house-trades-by-id":                                    "congress_house_trades_by_id.json",
	"/router/stable/senate-profile":                                        "congress_senate_profile.json",
	"/router/stable/senate-profile?active=false&senateID=P000197&latestParty=Independent+%2F+Other&latestPosition=Representative+At-Large&page=0&limit=500": "congress_senate_profile.json",
	"/router/stable/senate-positions": "congress_senate_positions.json",
	"/router/stable/senate-positions?senateID=P000197&party=Republican+%2F+Other&position=Representative+At-Large&page=0&limit=300": "congress_senate_positions.json",
	"/router/stable/senate-net-worth?senateID=P000197":                       "congress_senate_net_worth.json",
	"/router/stable/senate-net-worth?senateID=P000197&page=0&limit=250":      "congress_senate_net_worth.json",
	"/router/stable/senate-net-worth-aggregated?senateID=P000197":            "congress_senate_net_worth_aggregated.json",
	"/router/stable/senate-net-worth-aggregated?senateID=P000197&totalsCol=": "congress_senate_net_worth_aggregated.json",
}

func congressionalRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := congressionalRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestCongressionalTradeMethodsUseExactPathsAndWireParameterOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, congressionalRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	senateLatest, err := client.Congressional.LatestSenateDisclosures(ctx,
		NewLatestCongressionalDisclosuresQuery().WithPage(0).WithLimit(250))
	if err != nil || len(senateLatest) != 1 || senateLatest[0].MemberId != "M001242" {
		t.Fatalf("LatestSenateDisclosures = %+v, %v", senateLatest, err)
	}
	houseLatest, err := client.Congressional.LatestHouseDisclosures(ctx,
		NewLatestCongressionalDisclosuresQuery().WithPage(math.MaxUint32).WithLimit(0))
	if err != nil || len(houseLatest) != 1 || houseLatest[0].Symbol != "META" {
		t.Fatalf("LatestHouseDisclosures = %+v, %v", houseLatest, err)
	}
	senateTrades, err := client.Congressional.SenateTrades(ctx,
		NewCongressionalTradesQuery("BRK.B / Class A").WithPage(1).WithLimit(2))
	if err != nil || len(senateTrades) != 1 || senateTrades[0].Owner != "Spouse" {
		t.Fatalf("SenateTrades = %+v, %v", senateTrades, err)
	}
	byName, err := client.Congressional.SenateTradesByName(ctx, NewCongressionalTradesByNameQuery("Jerry Moran"))
	if err != nil || len(byName) != 1 || byName[0].Symbol != "" {
		t.Fatalf("SenateTradesByName = %+v, %v", byName, err)
	}
	byID, err := client.Congressional.SenateTradesByMemberId(ctx,
		NewCongressionalTradesByMemberIdQuery().WithPage(3).WithLimit(4).WithMemberId("M001242"))
	if err != nil || len(byID) != 1 || byID[0].Symbol != "CM" {
		t.Fatalf("SenateTradesByMemberId = %+v, %v", byID, err)
	}
	houseTrades, err := client.Congressional.HouseTrades(ctx, NewCongressionalTradesQuery("AAPL"))
	if err != nil || len(houseTrades) != 1 || houseTrades[0].District != "TX02" {
		t.Fatalf("HouseTrades = %+v, %v", houseTrades, err)
	}
	houseByName, err := client.Congressional.HouseTradesByName(ctx, NewCongressionalTradesByNameQuery("James A."))
	if err != nil || len(houseByName) != 1 || houseByName[0].Symbol != "BAC" {
		t.Fatalf("HouseTradesByName = %+v, %v", houseByName, err)
	}
	houseByID, err := client.Congressional.HouseTradesByMemberId(ctx, NewCongressionalTradesByMemberIdQuery())
	if err != nil || len(houseByID) != 1 || houseByID[0].Symbol != "MNST" {
		t.Fatalf("HouseTradesByMemberId = %+v, %v", houseByID, err)
	}
	assertCongressionalRequests(t, rec, 8)
}

func TestCongressionalMemberAndNetWorthMethodsUseExactPathsAndWireParameterOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, congressionalRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	profiles, err := client.Congressional.Profiles(ctx, NewCongressionalProfilesQuery())
	if err != nil || len(profiles) != 1 || profiles[0].MemberId != "L000397" || !profiles[0].Active {
		t.Fatalf("Profiles = %+v, %v", profiles, err)
	}
	filtered, err := client.Congressional.Profiles(ctx, NewCongressionalProfilesQuery().WithActive(false).
		WithMemberId("P000197").WithLatestParty("Independent / Other").WithLatestPosition("Representative At-Large").
		WithPage(0).WithLimit(500))
	if err != nil || len(filtered) != 1 {
		t.Fatalf("Profiles filtered = %+v, %v", filtered, err)
	}
	positions, err := client.Congressional.Positions(ctx, NewCongressionalPositionsQuery())
	if err != nil || len(positions) != 1 || positions[0].MemberId != "Z000018" || positions[0].EndDate != nil {
		t.Fatalf("Positions = %+v, %v", positions, err)
	}
	held, err := client.Congressional.Positions(ctx, NewCongressionalPositionsQuery().WithMemberId("P000197").
		WithParty("Republican / Other").WithPosition("Representative At-Large").WithPage(0).WithLimit(300))
	if err != nil || len(held) != 1 {
		t.Fatalf("Positions filtered = %+v, %v", held, err)
	}

	netWorth, err := client.Congressional.NetWorth(ctx, NewCongressionalNetWorthQuery("P000197"))
	if err != nil || len(netWorth) != 1 || netWorth[0].DebtDetails == nil ||
		netWorth[0].DebtDetails.DateIncurred != "September 2007" || netWorth[0].Income != nil {
		t.Fatalf("NetWorth = %+v, %v", netWorth, err)
	}
	paged, err := client.Congressional.NetWorth(ctx, NewCongressionalNetWorthQuery("P000197").WithPage(0).WithLimit(250))
	if err != nil || len(paged) != 1 {
		t.Fatalf("NetWorth paged = %+v, %v", paged, err)
	}
	totals, err := client.Congressional.NetWorthAggregated(ctx, NewCongressionalNetWorthAggregatedQuery("P000197"))
	if err != nil || len(totals) != 1 || totals[0].Total != 225_219_551 {
		t.Fatalf("NetWorthAggregated = %+v, %v", totals, err)
	}
	columns, err := client.Congressional.NetWorthAggregated(ctx,
		NewCongressionalNetWorthAggregatedQuery("P000197").WithTotalsCol(""))
	if err != nil || len(columns) != 1 {
		t.Fatalf("NetWorthAggregated totalsCol = %+v, %v", columns, err)
	}
	assertCongressionalRequests(t, rec, 8)
}

// assertCongressionalRequests checks that exactly count GET requests were
// sent and that none carried the credential in its query.
func assertCongressionalRequests(t *testing.T, rec *recorder, count int) {
	t.Helper()
	requests := rec.all()
	if len(requests) != count {
		t.Fatalf("requests = %d, want %d", len(requests), count)
	}
	for _, req := range requests {
		if req.Method != http.MethodGet {
			t.Fatalf("%s used %s, want GET", req.URL.Path, req.Method)
		}
		if strings.Contains(req.URL.RawQuery, "apikey") {
			t.Fatalf("%s carried the credential in the query: %s", req.URL.Path, req.URL.RawQuery)
		}
	}
}

func TestCongressionalQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, congressionalRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	cases := []struct {
		name     string
		call     func() error
		argument string
		reason   error
	}{
		{"empty member id", func() error {
			_, err := client.Congressional.NetWorth(ctx, NewCongressionalNetWorthQuery(""))
			return err
		}, "senateID", ErrEmptyValue},
		{"whitespace optional member id", func() error {
			_, err := client.Congressional.Profiles(ctx, NewCongressionalProfilesQuery().WithMemberId(" \t"))
			return err
		}, "senateID", ErrEmptyValue},
		{"control character member id", func() error {
			_, err := client.Congressional.HouseTradesByMemberId(ctx,
				NewCongressionalTradesByMemberIdQuery().WithMemberId("M00\n1242"))
			return err
		}, "senateID", ErrControlCharacterValue},
		{"empty search name", func() error {
			_, err := client.Congressional.SenateTradesByName(ctx, NewCongressionalTradesByNameQuery(""))
			return err
		}, "name", ErrEmptyValue},
		{"comma in ticker", func() error {
			_, err := client.Congressional.SenateTrades(ctx, NewCongressionalTradesQuery("AAPL,MSFT"))
			return err
		}, "symbol", ErrCommaInTicker},
		{"empty ticker", func() error {
			_, err := client.Congressional.HouseTrades(ctx, NewCongressionalTradesQuery(""))
			return err
		}, "symbol", ErrEmptyValue},
	}
	for _, tc := range cases {
		err := tc.call()
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != tc.argument+": "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want %s: %v", tc.name, err, tc.argument, tc.reason)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	base := NewCongressionalTradesByMemberIdQuery()
	withID := base.WithMemberId("P000197")
	if base.MemberId() != nil || withID.MemberId() == nil || *withID.MemberId() != "P000197" || withID.Page() != nil {
		t.Fatalf("WithMemberId mutated the receiver or dropped the value: base=%+v withID=%+v", base, withID)
	}
	if q := NewCongressionalTradesByNameQuery("  James A.  "); q.Name() != "  James A.  " {
		t.Fatalf("Name() normalized the value: %q", q.Name())
	}
	if q := NewCongressionalProfilesQuery().WithLatestParty(""); q.LatestParty() == nil || *q.LatestParty() != "" {
		t.Fatalf("WithLatestParty dropped the empty text: %+v", q)
	}
}

func TestCongressionalMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(string(readFixture(t, "congress_senate_profile.json"))))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Congressional.Positions(context.Background(), NewCongressionalPositionsQuery())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "senate-positions")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"congressNumber"`) {
		t.Fatalf("cause = %v, want it to name the missing member congressNumber", cause)
	}

	server, _ = newServer(t, jsonHandler(`{}`))
	client = newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	_, err = client.Congressional.NetWorthAggregated(context.Background(), NewCongressionalNetWorthAggregatedQuery("P000197"))
	if typed = assertQuoteError(t, err, CategoryDecode, http.StatusOK, "senate-net-worth-aggregated"); typed.Body == nil {
		t.Fatalf("non-array response kept no safe body: %+v", typed)
	}
}
