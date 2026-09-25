package fmp

import (
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

// newsArticleFixtures lists the nine provider-news fixtures that share the
// NewsArticle row, with the symbol and publisher copied from
// all_nine_exact_provider_fixtures_share_the_same_eight_field_row in
// crates/libfmp/tests/news_responses.rs. An empty symbol means JSON null.
var newsArticleFixtures = []struct {
	fixture   string
	symbol    string
	publisher string
}{
	{"latest_general_news.json", "", "Seeking Alpha"},
	{"latest_press_releases.json", "RXT", "GlobeNewsWire"},
	{"latest_stock_news.json", "KO", "Zacks Investment Research"},
	{"latest_crypto_news.json", "UNIUSD", "Crypto Briefing"},
	{"latest_forex_news.json", "USDJPY", "FXEmpire"},
	{"search_press_releases.json", "AAPL", "Business Wire"},
	{"search_stock_news.json", "AAPL", "CNBC Television"},
	{"search_crypto_news.json", "BTCUSD", "AMBCrypto"},
	{"search_forex_news.json", "EURUSD", "FXEmpire"},
}

const newsPressReleaseText = "NEW YORK, July 30, 2026 (GLOBE NEWSWIRE) -- Gainey McKenna & Egleston announces that a securities class action lawsuit has been filed in the United States District Court for the Southern District of New York on behalf of all persons or entities who purchased or otherwise acquired Rackspace Technology, Inc. (“Rackspace” or the “Company”) (NASDAQ: RXT) securities between May 7, 2026 and July 8, 2026, inclusive (the “Class Period”)."

const newsAppleText = "CUPERTINO, Calif.--(BUSINESS WIRE)--Apple® today announced Apple Upgrade, a new product leasing program provided by Klarna for iPhone®, Apple Watch®, Mac®, and iPad® available on the Apple Store® online, in the Apple Store app, and at Apple Store locations in the United States.1 Apple Upgrade makes it even easier for customers to get the Apple products they love with a leasing plan that is right for them. “At Apple, we put the customer at the center of everything we do,” said Karen Rasmussen, A."

func TestNewsFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[Article](t, "fmp_articles.json")
	for _, tc := range newsArticleFixtures {
		assertFixtureParity[NewsArticle](t, tc.fixture)
	}
}

// Mirrors exact_fmp_fixture_preserves_all_eight_required_opaque_fields: the
// HTML content is an opaque string, never parsed or normalized.
func TestArticleFixturePreservesAllEightRequiredOpaqueFields(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[Article](t, "fmp_articles.json")
	if len(rows) != 1 {
		t.Fatalf("rows = %d, want 1", len(rows))
	}
	want := Article{
		Title:   "Centerra Gold (NYSE:CGAU) Drives Growth with North American Investments and Strong Financials",
		Date:    mustParseDateTime(t, "2026-07-30 16:11:45"),
		Content: rows[0].Content,
		Tickers: "NYSE:CGAU",
		Image:   "https://portal.financialmodelingprep.com/positions/6a6b7d9c2514096bb2027277.jpeg",
		Link:    "https://financialmodelingprep.com/market-news/centerra-gold-cgau-growth-north-american-investments-strong-financials",
		Author:  "Andrew Wynn",
		Site:    "Financial Modeling Prep",
	}
	if rows[0] != want {
		t.Fatalf("articles = %+v, want %+v", rows[0], want)
	}
	content := rows[0].Content
	if !strings.HasPrefix(content, "<ul>\n    <li><strong>") || !strings.Contains(content, "<strong>$450 million</strong>") ||
		!strings.HasSuffix(content, "gold ...") {
		t.Fatalf("content was normalized or parsed: %q", content)
	}
	if got := memberSet(t, rows[0]); len(got) != 8 {
		t.Fatalf("re-encoded members = %d, want 8", len(got))
	}
}

// Mirrors all_nine_exact_provider_fixtures_share_the_same_eight_field_row.
func TestAllNineProviderNewsFixturesShareTheSameEightFieldRow(t *testing.T) {
	t.Parallel()
	for _, tc := range newsArticleFixtures {
		rows := assertFixtureParity[NewsArticle](t, tc.fixture)
		if len(rows) != 1 {
			t.Fatalf("%s: rows = %d, want 1", tc.fixture, len(rows))
		}
		row := rows[0]
		switch {
		case tc.symbol == "" && row.Symbol != nil:
			t.Fatalf("%s: symbol = %q, want nil for JSON null", tc.fixture, *row.Symbol)
		case tc.symbol != "" && (row.Symbol == nil || *row.Symbol != tc.symbol):
			t.Fatalf("%s: symbol = %v, want %q", tc.fixture, row.Symbol, tc.symbol)
		}
		if row.Publisher != tc.publisher {
			t.Fatalf("%s: publisher = %q, want %q", tc.fixture, row.Publisher, tc.publisher)
		}
		if !strings.HasPrefix(row.Image, "https://") || !strings.HasPrefix(row.URL, "https://") {
			t.Fatalf("%s: image %q or url %q is not https", tc.fixture, row.Image, row.URL)
		}
		if got := memberSet(t, row); len(got) != 8 {
			t.Fatalf("%s: re-encoded members = %d, want 8", tc.fixture, len(got))
		}
	}
}

// Mirrors exact_unicode_html_and_url_strings_are_not_normalized_or_parsed.
func TestNewsUnicodeHtmlAndUrlStringsAreNotNormalizedOrParsed(t *testing.T) {
	t.Parallel()
	latestPress := assertFixtureParity[NewsArticle](t, "latest_press_releases.json")
	if latestPress[0].Text != newsPressReleaseText || !strings.Contains(latestPress[0].Text, "“Rackspace”") {
		t.Fatalf("latest_press_releases text = %q", latestPress[0].Text)
	}
	if latestPress[0].PublishedDate != mustParseDateTime(t, "2026-07-30 13:08:00") {
		t.Fatalf("latest_press_releases publishedDate = %v", latestPress[0].PublishedDate)
	}
	searchPress := assertFixtureParity[NewsArticle](t, "search_press_releases.json")
	if searchPress[0].Text != newsAppleText || strings.Count(searchPress[0].Text, "®") != 6 ||
		!strings.Contains(searchPress[0].Text, "“At Apple") {
		t.Fatalf("search_press_releases text = %q", searchPress[0].Text)
	}
	stock := assertFixtureParity[NewsArticle](t, "latest_stock_news.json")
	if stock[0].URL != "https://www.zacks.com/stock/news/2964897/coca-cola-s-momentum-builds-after-strong-q2-earnings-etfs-to-consider?cid=CS-STOCKNEWSAPI-FT-etf_news_and_commentary-2964897" {
		t.Fatalf("latest_stock_news url = %q", stock[0].URL)
	}
	searched := assertFixtureParity[NewsArticle](t, "search_stock_news.json")
	if searched[0].URL != "https://www.youtube.com/watch?v=ZKMD80U8dRM" {
		t.Fatalf("search_stock_news url = %q", searched[0].URL)
	}
}

// Mirrors symbol_key_is_required_but_explicit_null_is_preserved and covers
// the missing-required-member path once for this domain: the symbol key must
// be present (required_option), null is a value, and a removed key is a
// CategoryDecode error naming the member.
func TestNewsSymbolKeyIsRequiredButExplicitNullIsPreserved(t *testing.T) {
	t.Parallel()
	general := assertFixtureParity[NewsArticle](t, "latest_general_news.json")
	if general[0].Symbol != nil {
		t.Fatalf("latest_general_news symbol = %q, want nil", *general[0].Symbol)
	}
	encoded, err := json.Marshal(general[0])
	if err != nil || !strings.Contains(string(encoded), `"symbol":null`) {
		t.Fatalf("re-encoded row = %s, %v: want an explicit null symbol", encoded, err)
	}

	fixture := string(readFixture(t, "latest_general_news.json"))
	missing := strings.Replace(fixture, `"symbol": null,`, ``, 1)
	if missing == fixture {
		t.Fatal("the symbol member was not removed from the fixture text")
	}
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing symbol", missing, "symbol"},
		{"missing publisher", strings.Replace(fixture, `"publisher": "Seeking Alpha",`, ``, 1), "publisher"},
		{"empty object", `[{}]`, "symbol"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []NewsArticle
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "NewsArticle") {
				t.Fatalf("message = %q, want it to name member %q of NewsArticle", typed.Message, tc.member)
			}
		})
	}
	var articles []Article
	err = json.Unmarshal([]byte(`[{"title":"t","date":"2026-07-30 16:11:45"}]`), &articles)
	var typed *Error
	if !errors.As(err, &typed) || !strings.Contains(typed.Message, `"content"`) {
		t.Fatalf("Article error = %v, want the first missing member content", err)
	}
}

// Mirrors response_string_and_datetime_kinds_are_strict: numbers never
// decode into string or DateTime members, and DateTime stays naive with
// the exact "YYYY-MM-DD HH:MM:SS" spelling.
func TestNewsResponseStringAndDateTimeKindsAreStrict(t *testing.T) {
	t.Parallel()
	article := string(readFixture(t, "fmp_articles.json"))
	var articles []Article
	if err := json.Unmarshal([]byte(strings.Replace(article, `"2026-07-30 16:11:45"`, `20260730161145`, 1)),
		&articles); err == nil {
		t.Fatal("a numeric date decoded into a DateTime member")
	}
	stock := string(readFixture(t, "latest_stock_news.json"))
	for _, replacement := range []string{"0", "1.5"} {
		var rows []NewsArticle
		wire := strings.Replace(stock, `"2026-07-30 13:15:49"`, replacement, 1)
		if err := json.Unmarshal([]byte(wire), &rows); err == nil {
			t.Fatalf("publishedDate %s decoded into a DateTime member", replacement)
		}
		wire = strings.Replace(stock, `"Coca-Cola's Momentum Builds After Strong Q2 Earnings: ETFs to Consider"`, replacement, 1)
		if wire == stock {
			t.Fatal("the title was not replaced in the fixture text")
		}
		if err := json.Unmarshal([]byte(wire), &rows); err == nil {
			t.Fatalf("title %s decoded into a string member", replacement)
		}
	}
	for _, invalid := range []string{"2026-07-30T13:15:49", "2026-07-30 13:15:49Z", "2026-07-30 13:15"} {
		var rows []NewsArticle
		wire := strings.Replace(stock, "2026-07-30 13:15:49", invalid, 1)
		if err := json.Unmarshal([]byte(wire), &rows); err == nil {
			t.Fatalf("%q decoded into a DateTime member", invalid)
		}
	}
}

// Mirrors both_rows_are_bare_arrays_preserving_empty_multiple_and_large_opaque_strings.
func TestNewsRowsAreBareArraysPreservingEmptyMultipleAndLargeOpaqueStrings(t *testing.T) {
	t.Parallel()
	var articles []Article
	if err := json.Unmarshal([]byte(`[]`), &articles); err != nil || articles == nil || len(articles) != 0 {
		t.Fatalf("empty Article array = %#v, %v", articles, err)
	}
	var rows []NewsArticle
	if err := json.Unmarshal([]byte(`[]`), &rows); err != nil || rows == nil || len(rows) != 0 {
		t.Fatalf("empty NewsArticle array = %#v, %v", rows, err)
	}
	if err := json.Unmarshal([]byte(`{"articles":[]}`), &rows); err == nil {
		t.Fatal("an object decoded into a bare-array contract")
	}

	stock := strings.TrimSpace(string(readFixture(t, "latest_stock_news.json")))
	multiple := stock[:len(stock)-1] + "," + stock[1:]
	if err := json.Unmarshal([]byte(multiple), &rows); err != nil || len(rows) != 2 || rows[1].Title != rows[0].Title ||
		rows[1].Symbol == nil || *rows[1].Symbol != "KO" {
		t.Fatalf("duplicated NewsArticle rows = %+v, %v", rows, err)
	}

	payload := strings.Repeat("<p>café 中文 📈 &amp; exact</p>", 40_000)
	var wire []map[string]any
	if err := json.Unmarshal(readFixture(t, "fmp_articles.json"), &wire); err != nil {
		t.Fatalf("fixture: %v", err)
	}
	wire[0]["content"] = payload
	large, err := json.Marshal(wire)
	if err != nil {
		t.Fatalf("re-encode fixture: %v", err)
	}
	if err := json.Unmarshal(large, &articles); err != nil || len(articles) != 1 || articles[0].Content != payload {
		t.Fatalf("large content: len %d, %v", len(articles), err)
	}
}
