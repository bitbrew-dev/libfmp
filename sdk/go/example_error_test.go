package fmp_test

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"

	fmp "github.com/bitbrew-dev/libfmp/sdk/go"
)

// ExampleError matches a client failure with errors.As and reads its
// category, endpoint id, and provider status. The server rejects the request
// with a 401 whose body echoes the credential it received; the SDK redacts
// the retained body, so the error text never contains the key.
func ExampleError() {
	server := httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusUnauthorized)
		_, _ = fmt.Fprintf(w, `{"Error Message":"Invalid API KEY: %s"}`, r.Header.Get("apikey"))
	}))
	defer server.Close()

	client, err := fmp.NewClient(
		fmp.WithBaseURL(server.URL),
		fmp.WithHTTPClient(server.Client()),
		fmp.WithAuthentication(fmp.FmpHeader(exampleKey)),
	)
	if err != nil {
		fmt.Println("configuration rejected:", err)
		return
	}

	_, err = client.Quote.Short(context.Background(), fmp.NewQuoteShortQuery("AAPL"))
	var fmpErr *fmp.Error
	if !errors.As(err, &fmpErr) {
		fmt.Println("not an *fmp.Error:", err)
		return
	}
	if strings.Contains(fmpErr.Error(), exampleKey) {
		fmt.Println("the error text leaked the credential")
		return
	}

	fmt.Println("category:", fmpErr.Category)
	fmt.Println("endpoint:", fmpErr.Endpoint)
	fmt.Println("status:", fmpErr.Status)
	fmt.Println("error:", fmpErr)
	// Output:
	// category: status
	// endpoint: quote-short
	// status: 401
	// error: provider returned HTTP status 401 (endpoint: quote-short): {"Error Message":"Invalid API KEY: [REDACTED]"}
}
