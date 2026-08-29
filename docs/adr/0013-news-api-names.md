# ADR 0013: News API names and contract boundaries

## Status

Accepted for issue #22's Rust contract implementation and reserved future
Python facade.

## Decision

The news endpoints use the following Rust descriptor, client, query, and
response-row names and reserve the listed future Python names.

| FMP path | Rust method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `fmp-articles` | `fmp_articles` | `FmpArticlesQuery` | `FmpArticle` | `FmpClient.fmp_articles`; `fmp.news.FmpArticle` |
| `news/general-latest` | `latest_general_news` | `LatestGeneralNewsQuery` | `NewsArticle` | `FmpClient.latest_general_news`; `fmp.news.NewsArticle` |
| `news/press-releases-latest` | `latest_press_releases` | `LatestPressReleasesQuery` | `NewsArticle` | `FmpClient.latest_press_releases`; `fmp.news.NewsArticle` |
| `news/stock-latest` | `latest_stock_news` | `LatestStockNewsQuery` | `NewsArticle` | `FmpClient.latest_stock_news`; `fmp.news.NewsArticle` |
| `news/crypto-latest` | `latest_crypto_news` | `LatestCryptoNewsQuery` | `NewsArticle` | `FmpClient.latest_crypto_news`; `fmp.news.NewsArticle` |
| `news/forex-latest` | `latest_forex_news` | `LatestForexNewsQuery` | `NewsArticle` | `FmpClient.latest_forex_news`; `fmp.news.NewsArticle` |
| `news/press-releases` | `search_press_releases` | `SearchPressReleasesQuery` | `NewsArticle` | `FmpClient.search_press_releases`; `fmp.news.NewsArticle` |
| `news/stock` | `search_stock_news` | `SearchStockNewsQuery` | `NewsArticle` | `FmpClient.search_stock_news`; `fmp.news.NewsArticle` |
| `news/crypto` | `search_crypto_news` | `SearchCryptoNewsQuery` | `NewsArticle` | `FmpClient.search_crypto_news`; `fmp.news.NewsArticle` |
| `news/forex` | `search_forex_news` | `SearchForexNewsQuery` | `NewsArticle` | `FmpClient.search_forex_news`; `fmp.news.NewsArticle` |

Each public query remains endpoint-owned even where its wire shape is shared.
Latest-feed queries preserve independent optional `from` and `to` values and
encode `from`, `to`, `page`, then `limit`. Search queries additionally require
a non-empty `TickerList` first. The SDK does not invent date-range validation
or constructor rejection for page and limit values.

The nine provider-news endpoints return the same documented eight-field row,
so they share `NewsArticle`. Its `symbol` key is required but explicitly
nullable: a JSON `null` decodes as `None`, while an absent key is an error. FMP
editorial articles retain their distinct `FmpArticle` model. Article content,
ticker text, HTML, Unicode, image locations, and publication URLs remain opaque
exact strings rather than parsed or normalized values. Both response shapes
are bare arrays with required documented fields and unknown-field tolerance.

The latest and search provider-news sections state a maximum of 250 responses
per call and a maximum page number of 100. They do not state a maximum value
for the `limit` query parameter. These are future descriptor metadata, not
query-constructor bounds. The FMP Articles section documents neither bound.

The source explicitly marks FMP Articles and both press-release endpoints as
US-only, and General News as worldwide. It provides no geography statement for
the stock, cryptocurrency, or foreign-exchange latest/search endpoints; future
descriptors must preserve those gaps rather than infer availability. Runtime
descriptors, client methods, and Python bindings are deferred. The future
Python facade will accept ordinary method arguments and will not expose Rust
query structs as Python public classes.
