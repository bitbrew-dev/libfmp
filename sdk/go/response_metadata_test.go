package fmp

import (
	"context"
	"fmt"
	"net/http"
	"reflect"
	"strings"
	"testing"
)

// valetSuccessServer answers every request with contentType, body, status and the
// headers a valet proxy adds, plus headers that must never be retained.
func valetSuccessServer(t *testing.T, status int, contentType, body string) *Client {
	t.Helper()
	server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", contentType)
		w.Header().Set("X-Proxy-Cache", "HIT")
		w.Header().Set("X-Proxy-Daily-Remaining", "42")
		w.Header().Set("Set-Cookie", "session=cookie-secret")
		w.Header().Set("Authorization", "Bearer header-secret")
		w.Header().Set("X-Unrelated", "unrelated-value")
		w.WriteHeader(status)
		_, _ = w.Write([]byte(body))
	})
	return newClient(t, server)
}

func assertValetMetadata(t *testing.T, meta ResponseMetadata, endpointID string) {
	t.Helper()
	want := ResponseMetadata{
		EndpointID: endpointID,
		StatusCode: http.StatusOK,
		Headers:    http.Header{"X-Proxy-Cache": {"HIT"}, "X-Proxy-Daily-Remaining": {"42"}},
	}
	if !reflect.DeepEqual(meta, want) {
		t.Fatalf("meta = %+v, want %+v", meta, want)
	}
	formatted := fmt.Sprintf("%+v", meta)
	for _, leak := range []string{"cookie-secret", "header-secret", "unrelated-value"} {
		if strings.Contains(formatted, leak) {
			t.Fatalf("meta retained %q: %s", leak, formatted)
		}
	}
}

func TestWithResponseMetadataFillsJSONSuccess(t *testing.T) {
	t.Parallel()
	client := valetSuccessServer(t, http.StatusOK, "application/json", `[{"symbol":"AAPL"}]`)
	var meta ResponseMetadata
	var rows []probeRow
	err := client.getJSON(WithResponseMetadata(context.Background(), &meta), "quote", "quote", nil, &rows)
	if err != nil {
		t.Fatal(err)
	}
	if len(rows) != 1 || rows[0].Symbol != "AAPL" {
		t.Fatalf("rows = %+v", rows)
	}
	assertValetMetadata(t, meta, "quote")
}

func TestWithResponseMetadataFillsCSVSuccess(t *testing.T) {
	t.Parallel()
	client := valetSuccessServer(t, http.StatusOK, "text/csv", "symbol\nAAPL\n")
	var meta ResponseMetadata
	rows, err := getCSV[probeRow](WithResponseMetadata(context.Background(), &meta), client, csvEndpoint, "bulk-probe", nil)
	if err != nil {
		t.Fatal(err)
	}
	if len(rows) != 1 || rows[0].Symbol != "AAPL" {
		t.Fatalf("rows = %+v", rows)
	}
	assertValetMetadata(t, meta, csvEndpoint)
}

func TestWithResponseMetadataFillsBinarySuccess(t *testing.T) {
	t.Parallel()
	client := valetSuccessServer(t, http.StatusOK, binaryMediaType, "payload")
	var meta ResponseMetadata
	ctx := WithResponseMetadata(context.Background(), &meta)
	body, err := client.getBinary(ctx, binaryEndpoint, binaryPath, nil, []string{binaryMediaType})
	if err != nil {
		t.Fatal(err)
	}
	if string(body.Data) != "payload" {
		t.Fatalf("payload = %+v", body)
	}
	assertValetMetadata(t, meta, binaryEndpoint)
}

func TestWithResponseMetadataLeavesMetaUntouchedOnError(t *testing.T) {
	t.Parallel()
	sentinel := ResponseMetadata{EndpointID: "sentinel", StatusCode: -1}
	cases := []struct {
		name   string
		status int
		body   string
	}{
		{"status error", http.StatusTooManyRequests, `{"error":"slow down"}`},
		{"decode error", http.StatusOK, `{"not":"an array"}`},
		{"provider message", http.StatusOK, `{"Error Message":"Invalid API KEY."}`},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			client := valetSuccessServer(t, tc.status, "application/json", tc.body)
			meta := sentinel
			var rows []probeRow
			err := client.getJSON(WithResponseMetadata(context.Background(), &meta), "quote", "quote", nil, &rows)
			if err == nil {
				t.Fatal("expected an error")
			}
			if !reflect.DeepEqual(meta, sentinel) {
				t.Fatalf("meta = %+v, want untouched", meta)
			}
		})
	}
}

func TestWithResponseMetadataNilAndAbsentAreNoOps(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	if got := WithResponseMetadata(ctx, nil); got != ctx {
		t.Fatal("nil meta must return ctx unchanged")
	}
	client := valetSuccessServer(t, http.StatusOK, "application/json", `[{"symbol":"AAPL"}]`)
	var rows []probeRow
	if err := client.getJSON(ctx, "quote", "quote", nil, &rows); err != nil {
		t.Fatal(err)
	}
}
