package fmp

import (
	"context"
	"fmt"
	"strings"
)

// BinaryPayload is an owned binary response with validated media metadata.
// It mirrors BinaryResponse in the Rust crate.
//
// ContentType is retained exactly as received after the endpoint contract
// validated its media type. ContentDisposition is retained when the header
// was present and valid text, and is "" otherwise. Formatting a payload with
// any fmt verb prints the byte count, the media type, and whether a
// disposition is present: never the bytes and never the disposition, which
// can carry an opaque filename or other provider-controlled text.
type BinaryPayload struct {
	Data               []byte
	ContentType        string
	ContentDisposition string
}

// MediaType returns ContentType without its parameters.
func (p BinaryPayload) MediaType() string { return mediaTypeOf(p.ContentType) }

// Format prints the payload shape without the bytes or the disposition.
func (p BinaryPayload) Format(f fmt.State, _ rune) {
	_, _ = fmt.Fprintf(f, "BinaryPayload{body_bytes: %d, media_type: %q, has_content_disposition: %t}",
		len(p.Data), p.MediaType(), p.ContentDisposition != "")
}

// getBinary issues one GET for a logical endpoint and returns the raw body
// when the response carries one of the expected media types, compared
// case-insensitively without parameters. endpointID is the registry id
// carried by errors, never a URL. The redirect, timeout, body cap, and
// no-retry rules of getJSON apply unchanged.
func (c *Client) getBinary(ctx context.Context, endpointID, relativePath string, query []queryParam,
	expectedContentTypes []string) (BinaryPayload, error) {
	if err := validateRelativePath(relativePath); err != nil {
		return BinaryPayload{}, err
	}
	target := c.buildEndpointURL(relativePath)
	rawQuery, err := encodeQuery(query, c.auth.queryName)
	if err != nil {
		return BinaryPayload{}, err
	}
	target.RawQuery = rawQuery

	ctx, cancel := context.WithTimeout(ctx, c.timeout)
	defer cancel()
	resp, err := c.executeRedirects(ctx, endpointID, target)
	if err != nil {
		return BinaryPayload{}, err
	}

	contentType := resp.header.Get("Content-Type")
	switch {
	case contentType == "":
		return BinaryPayload{}, decodeError(endpointID, resp.status, c.safeBody(resp.body),
			"successful response omitted its content type", nil)
	case !validHeaderValue(contentType):
		return BinaryPayload{}, decodeError(endpointID, resp.status, c.safeBody(resp.body),
			"successful response used an invalid content type", nil)
	case !matchesMediaType(contentType, expectedContentTypes):
		return BinaryPayload{}, decodeError(endpointID, resp.status, c.safeBody(resp.body),
			"successful response used an unexpected content type", nil)
	}
	disposition := resp.header.Get("Content-Disposition")
	if !validHeaderValue(disposition) {
		disposition = ""
	}
	return BinaryPayload{Data: resp.body, ContentType: contentType, ContentDisposition: disposition}, nil
}

func mediaTypeOf(contentType string) string {
	mediaType, _, _ := strings.Cut(contentType, ";")
	return strings.TrimSpace(mediaType)
}

func matchesMediaType(contentType string, expected []string) bool {
	mediaType := mediaTypeOf(contentType)
	for _, candidate := range expected {
		if strings.EqualFold(mediaType, candidate) {
			return true
		}
	}
	return false
}
