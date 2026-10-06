package fmp

import (
	"bytes"
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
// case-insensitively without parameters, or when the body opens with the
// signature of one of them (FMP has labeled real workbooks as JSON, #411).
// A JSON-labeled provider message keeps its provider-message error; every
// other error summarizes the body instead of echoing its bytes. endpointID is the registry id
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
	resp, err := c.executeRedirects(ctx, endpointID, target, c.maxResponseBodyBytes)
	if err != nil {
		return BinaryPayload{}, err
	}

	contentType := resp.header.Get("Content-Type")
	switch {
	case contentType == "":
		return BinaryPayload{}, decodeError(endpointID, resp.status, c.binaryErrorBody(resp.body, ""),
			"successful response omitted its content type", nil)
	case !visibleHeaderText(contentType):
		return BinaryPayload{}, decodeError(endpointID, resp.status, c.binaryErrorBody(resp.body, ""),
			"successful response used an invalid content type", nil)
	case !matchesMediaType(contentType, expectedContentTypes) && !matchesSignature(resp.body, expectedContentTypes):
		if validJSONMediaType(contentType) && isProviderMessage(resp.body) {
			return BinaryPayload{}, c.providerMessageError(endpointID, resp)
		}
		return BinaryPayload{}, decodeError(endpointID, resp.status, c.binaryErrorBody(resp.body, contentType),
			"successful response used an unexpected content type", nil)
	}
	disposition := resp.header.Get("Content-Disposition")
	if !visibleHeaderText(disposition) {
		disposition = ""
	}
	c.recordResponseMetadata(ctx, endpointID, resp)
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

var zipMagic = []byte("PK\x03\x04")

// binarySignatures maps binary media types to the magic bytes their bodies
// start with. Types without an entry are never sniffed.
var binarySignatures = map[string][]byte{
	"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet": zipMagic,
	"application/zip": zipMagic,
}

func matchesSignature(body []byte, expected []string) bool {
	for _, candidate := range expected {
		if magic, ok := binarySignatures[strings.ToLower(candidate)]; ok && bytes.HasPrefix(body, magic) {
			return true
		}
	}
	return false
}

// binaryErrorBody summarizes a rejected binary body without its bytes.
func (c *Client) binaryErrorBody(body []byte, contentType string) *SafeBody {
	summary := fmt.Sprintf("<binary body omitted: %d bytes>", len(body))
	if contentType != "" {
		summary = fmt.Sprintf("<binary body omitted: %d bytes, content-type %s>", len(body), contentType)
	}
	safe := NewSafeBody(summary, c.redactor)
	return &safe
}
