package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// newsRoutes is the exact path and query table of the ten news endpoints,
// keyed on path plus RawQuery so the three query shapes (no query, optional
// filters only, required symbols first) share one table. Wire text copied
// from the query tests in crates/libfmp/src/endpoints/news.rs: symbols,
// then from, to, page, limit, each optional one omitted independently.
var newsRoutes = map[string]string{
	"/router/stable/fmp-articles":                                                                 "fmp_articles.json",
	"/router/stable/fmp-articles?page=0&limit=20":                                                 "fmp_articles.json",
	"/router/stable/news/general-latest":                                                          "latest_general_news.json",
	"/router/stable/news/general-latest?from=2026-01-27&to=2026-04-28&page=0&limit=20":            "latest_general_news.json",
	"/router/stable/news/general-latest?to=2026-04-28":                                            "latest_general_news.json",
	"/router/stable/news/press-releases-latest?from=2026-01-27":                                   "latest_press_releases.json",
	"/router/stable/news/stock-latest?page=0&limit=20":                                            "latest_stock_news.json",
	"/router/stable/news/crypto-latest?limit=20":                                                  "latest_crypto_news.json",
	"/router/stable/news/forex-latest":                                                            "latest_forex_news.json",
	"/router/stable/news/press-releases?symbols=AAPL%2CMSFT":                                      "search_press_releases.json",
	"/router/stable/news/stock?symbols=AAPL%2CMSFT&from=2026-01-27&to=2026-04-28&page=0&limit=20": "search_stock_news.json",
	"/router/stable/news/stock?symbols=AAPL%2CMSFT&to=2026-04-28":                                 "search_stock_news.json",
	"/router/stable/news/crypto?symbols=BTCUSD&from=2026-01-27":                                   "search_crypto_news.json",
	"/router/stable/news/forex?symbols=EURUSD&page=1":                                             "search_forex_news.json",
}

func newsRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		key := r.URL.Path
		if r.URL.RawQuery != "" {
			key += "?" + r.URL.RawQuery
		}
		fixture, ok := newsRoutes[key]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", key)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestNewsMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, newsRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	from, to := mustParseDate(t, "2026-01-27"), mustParseDate(t, "2026-04-28")
	symbols := []string{"AAPL", "MSFT"}

	articles, err := client.News.Articles(ctx, NewArticlesQuery())
	if err != nil || len(articles) != 1 || articles[0].Author != "Andrew Wynn" {
		t.Fatalf("Articles = %+v, %v", articles, err)
	}
	if articles, err = client.News.Articles(ctx, NewArticlesQuery().WithPage(0).WithLimit(20)); err != nil ||
		len(articles) != 1 {
		t.Fatalf("Articles with page and limit = %+v, %v", articles, err)
	}
	general, err := client.News.LatestGeneralNews(ctx, NewLatestGeneralNewsQuery())
	if err != nil || len(general) != 1 || general[0].Symbol != nil || general[0].Publisher != "Seeking Alpha" {
		t.Fatalf("LatestGeneralNews = %+v, %v", general, err)
	}
	if general, err = client.News.LatestGeneralNews(ctx,
		NewLatestGeneralNewsQuery().WithFrom(from).WithTo(to).WithPage(0).WithLimit(20)); err != nil || len(general) != 1 {
		t.Fatalf("LatestGeneralNews with every filter = %+v, %v", general, err)
	}
	if general, err = client.News.LatestGeneralNews(ctx, NewLatestGeneralNewsQuery().WithTo(to)); err != nil ||
		len(general) != 1 {
		t.Fatalf("LatestGeneralNews with only to = %+v, %v", general, err)
	}
	press, err := client.News.LatestPressReleases(ctx, NewLatestPressReleasesQuery().WithFrom(from))
	if err != nil || len(press) != 1 || press[0].Symbol == nil || *press[0].Symbol != "RXT" {
		t.Fatalf("LatestPressReleases = %+v, %v", press, err)
	}
	stock, err := client.News.LatestStockNews(ctx, NewLatestStockNewsQuery().WithPage(0).WithLimit(20))
	if err != nil || len(stock) != 1 || stock[0].Symbol == nil || *stock[0].Symbol != "KO" {
		t.Fatalf("LatestStockNews = %+v, %v", stock, err)
	}
	crypto, err := client.News.LatestCryptoNews(ctx, NewLatestCryptoNewsQuery().WithLimit(20))
	if err != nil || len(crypto) != 1 || crypto[0].Publisher != "Crypto Briefing" {
		t.Fatalf("LatestCryptoNews = %+v, %v", crypto, err)
	}
	forex, err := client.News.LatestForexNews(ctx, NewLatestForexNewsQuery())
	if err != nil || len(forex) != 1 || forex[0].Publisher != "FXEmpire" {
		t.Fatalf("LatestForexNews = %+v, %v", forex, err)
	}
	searchPress, err := client.News.SearchPressReleases(ctx, NewSearchPressReleasesQuery(symbols))
	if err != nil || len(searchPress) != 1 || searchPress[0].Publisher != "Business Wire" {
		t.Fatalf("SearchPressReleases = %+v, %v", searchPress, err)
	}
	searchStock, err := client.News.SearchStockNews(ctx,
		NewSearchStockNewsQuery(symbols).WithFrom(from).WithTo(to).WithPage(0).WithLimit(20))
	if err != nil || len(searchStock) != 1 || searchStock[0].URL != "https://www.youtube.com/watch?v=ZKMD80U8dRM" {
		t.Fatalf("SearchStockNews = %+v, %v", searchStock, err)
	}
	if searchStock, err = client.News.SearchStockNews(ctx, NewSearchStockNewsQuery(symbols).WithTo(to)); err != nil ||
		len(searchStock) != 1 {
		t.Fatalf("SearchStockNews with only to = %+v, %v", searchStock, err)
	}
	searchCrypto, err := client.News.SearchCryptoNews(ctx, NewSearchCryptoNewsQuery([]string{"BTCUSD"}).WithFrom(from))
	if err != nil || len(searchCrypto) != 1 || searchCrypto[0].Publisher != "AMBCrypto" {
		t.Fatalf("SearchCryptoNews = %+v, %v", searchCrypto, err)
	}
	searchForex, err := client.News.SearchForexNews(ctx, NewSearchForexNewsQuery([]string{"EURUSD"}).WithPage(1))
	if err != nil || len(searchForex) != 1 || searchForex[0].Symbol == nil || *searchForex[0].Symbol != "EURUSD" {
		t.Fatalf("SearchForexNews = %+v, %v", searchForex, err)
	}

	requests := rec.all()
	if len(requests) != len(newsRoutes) {
		t.Fatalf("requests = %d, want exactly one per route", len(requests))
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

func TestNewsQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, newsRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	for _, tc := range []struct {
		name    string
		symbols []string
		reason  error
	}{
		{"nil list", nil, ErrEmptyTickerList},
		{"empty list", []string{}, ErrEmptyTickerList},
		{"blank ticker", []string{"AAPL", " "}, ErrEmptyValue},
		{"control character", []string{"AA\tPL"}, ErrControlCharacterValue},
		{"comma", []string{"AAPL,MSFT"}, ErrCommaInTicker},
	} {
		_, err := client.News.SearchStockNews(ctx, NewSearchStockNewsQuery(tc.symbols))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != "symbols: "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want symbols: %v", tc.name, err, tc.reason)
		}
		_, err = client.News.SearchForexNews(ctx, NewSearchForexNewsQuery(tc.symbols))
		if forex := assertQuoteError(t, err, CategoryValidation, 0, ""); forex.Message != typed.Message {
			t.Fatalf("%s: SearchForexNews message %q differs from SearchStockNews message %q", tc.name, forex.Message,
				typed.Message)
		}
	}
	_, err := client.News.LatestGeneralNews(ctx, NewLatestGeneralNewsQuery().WithFrom(Date{}))
	typed := assertQuoteError(t, err, CategoryValidation, 0, "")
	if !errors.Is(err, ErrZeroTemporalValue) || typed.Message != "from: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero from: error = %v", err)
	}
	_, err = client.News.SearchCryptoNews(ctx, NewSearchCryptoNewsQuery([]string{"BTCUSD"}).WithTo(Date{}))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "to: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero to: error = %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	symbols := []string{"AAPL", "MSFT"}
	q := NewSearchStockNewsQuery(symbols)
	symbols[0] = "MUTATED"
	if got := q.Symbols(); len(got) != 2 || got[0] != "AAPL" || got[1] != "MSFT" {
		t.Fatalf("Symbols() shares the caller's slice: %v", got)
	}
	if q.From() != nil || q.To() != nil || q.Page() != nil || q.Limit() != nil {
		t.Fatalf("new search query has optional values set: %v %v %v %v", q.From(), q.To(), q.Page(), q.Limit())
	}
	to := mustParseDate(t, "2026-04-28")
	if got := q.WithTo(to).To(); got == nil || *got != to || q.To() != nil {
		t.Fatalf("WithTo mutated the receiver or lost the value: %v %v", got, q.To())
	}
	latest := NewLatestStockNewsQuery().WithPage(0).WithLimit(20)
	if page, limit := latest.Page(), latest.Limit(); page == nil || *page != 0 || limit == nil || *limit != 20 {
		t.Fatalf("latest getters = %v %v", page, limit)
	}
	params, err := NewArticlesQuery().params()
	if err != nil || len(params) != 0 {
		t.Fatalf("NewArticlesQuery().params() = %v, %v, want no parameters", params, err)
	}
}

func TestNewsMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	fixture := string(readFixture(t, "latest_general_news.json"))
	corrupted := strings.Replace(fixture, `"symbol": null,`, ``, 1)
	if corrupted == fixture {
		t.Fatal("the symbol member was not removed from the fixture text")
	}
	server, rec := newServer(t, jsonHandler(corrupted))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.News.LatestGeneralNews(context.Background(), NewLatestGeneralNewsQuery())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "news/general-latest")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"symbol"`) {
		t.Fatalf("cause = %v, want it to name the missing member symbol", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
