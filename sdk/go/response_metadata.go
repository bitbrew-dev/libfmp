package fmp

import (
	"context"
	"net/http"
)

// ResponseMetadata describes one successful response: the registry endpoint
// id, the HTTP status, and the allowlisted response headers.
//
// Headers holds only the names accepted by IsRetainedHeaderName (Retry-After,
// X-Proxy-*, X-RateLimit-*), redacted and bounded exactly like Error.Headers,
// in canonical form; it is nil when none qualify. Read a proxy's cache status
// with meta.Headers.Get("X-Proxy-Cache").
type ResponseMetadata struct {
	EndpointID string
	StatusCode int
	Headers    http.Header
}

type responseMetadataKey struct{}

// WithResponseMetadata returns a context that asks the client to fill *meta
// after a successful call made with it. The client writes *meta only once the
// response has been decoded; on any error *meta is left untouched, and the
// *Error already carries its own allowlisted headers.
//
// Use one context and one ResponseMetadata per call: concurrent calls that
// share a meta race on it, and a later call overwrites an earlier one. A nil
// meta returns ctx unchanged.
//
//	var meta fmp.ResponseMetadata
//	rows, err := client.Quote.Full(fmp.WithResponseMetadata(ctx, &meta), fmp.NewQuoteQuery("AAPL"))
//	remaining := meta.Headers.Get("X-Proxy-Daily-Remaining")
func WithResponseMetadata(ctx context.Context, meta *ResponseMetadata) context.Context {
	if meta == nil {
		return ctx
	}
	return context.WithValue(ctx, responseMetadataKey{}, meta)
}

// recordResponseMetadata fills the caller's ResponseMetadata, if any, from a
// successful response.
func (c *Client) recordResponseMetadata(ctx context.Context, endpointID string, resp *bufferedResponse) {
	meta, ok := ctx.Value(responseMetadataKey{}).(*ResponseMetadata)
	if !ok {
		return
	}
	*meta = ResponseMetadata{
		EndpointID: endpointID,
		StatusCode: resp.status,
		Headers:    retainedHeaders(resp.header, c.redactor),
	}
}
