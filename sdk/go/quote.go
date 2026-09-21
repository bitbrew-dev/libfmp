package fmp

// Hand-written template for the quote domain namespace (phase 0c, issue
// #279). Every line here is one the gen_go emitter reproduces mechanically
// from crates/fmp-py-gen/registry/quote.toml and
// crates/libfmp/src/endpoints/quote.rs. The generator regenerates this file
// with all sixteen quote endpoints and deletes the hand-written copy.

import "context"

// QuoteNamespace groups the quote endpoints. It is reached as Client.Quote
// and is valid only when obtained from a Client built by NewClient.
type QuoteNamespace struct {
	client *Client
}

// QuoteQuery holds the required query parameters for the detailed
// stock-quote endpoint.
type QuoteQuery struct {
	symbol string
}

// NewQuoteQuery creates a query for one provider ticker. The ticker is
// validated when the request is built.
func NewQuoteQuery(symbol string) QuoteQuery {
	return QuoteQuery{symbol: symbol}
}

// Symbol returns the requested ticker.
func (q QuoteQuery) Symbol() string {
	return q.symbol
}

func (q QuoteQuery) params() ([]queryParam, error) {
	symbol, err := tickerParam("symbol", q.symbol)
	if err != nil {
		return nil, err
	}
	return []queryParam{symbol}, nil
}

// QuoteShortQuery holds the required query parameters for the compact
// stock-quote endpoint.
type QuoteShortQuery struct {
	symbol string
}

// NewQuoteShortQuery creates a query for one provider ticker. The ticker is
// validated when the request is built.
func NewQuoteShortQuery(symbol string) QuoteShortQuery {
	return QuoteShortQuery{symbol: symbol}
}

// Symbol returns the requested ticker.
func (q QuoteShortQuery) Symbol() string {
	return q.symbol
}

func (q QuoteShortQuery) params() ([]queryParam, error) {
	symbol, err := tickerParam("symbol", q.symbol)
	if err != nil {
		return nil, err
	}
	return []queryParam{symbol}, nil
}

// shortOnlyParams is the closed compact-query contract of the whole-asset
// quote endpoints: they always send short=true and expose no boolean choice,
// mirroring ShortOnlyQuery in the Rust crate.
var shortOnlyParams = []queryParam{{Name: "short", Value: "true"}}

// Full retrieves detailed worldwide stock quotes for one ticker.
//
// GET quote?symbol=
func (n *QuoteNamespace) Full(ctx context.Context, q QuoteQuery) ([]Quote, error) {
	params, err := q.params()
	if err != nil {
		return nil, err
	}
	var out []Quote
	if err := n.client.getJSON(ctx, "quote", "quote", params, &out); err != nil {
		return nil, err
	}
	return out, nil
}

// Short retrieves compact worldwide stock quotes for one ticker.
//
// GET quote-short?symbol=
func (n *QuoteNamespace) Short(ctx context.Context, q QuoteShortQuery) ([]QuoteShort, error) {
	params, err := q.params()
	if err != nil {
		return nil, err
	}
	var out []QuoteShort
	if err := n.client.getJSON(ctx, "quote-short", "quote-short", params, &out); err != nil {
		return nil, err
	}
	return out, nil
}

// Etfs retrieves compact quotes for the documented ETF universe.
//
// GET batch-etf-quotes?short=true
func (n *QuoteNamespace) Etfs(ctx context.Context) ([]QuoteShort, error) {
	var out []QuoteShort
	if err := n.client.getJSON(ctx, "batch-etf-quotes", "batch-etf-quotes", shortOnlyParams, &out); err != nil {
		return nil, err
	}
	return out, nil
}
