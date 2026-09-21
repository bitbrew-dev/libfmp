package fmp

import (
	"encoding/json/jsontext"
	"errors"
	"strings"
	"testing"
)

func TestDecodeEmptyDateMirrorsTheRustSentinelCodecs(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name      string
		raw       string
		allowNull bool
		want      string
		wantNil   bool
		wantErr   string
	}{
		{"empty string is absent", `""`, false, "", true, ""},
		{"empty string is absent with null allowed", `""`, true, "", true, ""},
		{"null is absent for empty_or_null_date", `null`, true, "", true, ""},
		{"null is rejected for empty_date", `null`, false, "", false, `member "declarationDate" of Row must be a JSON string`},
		{"a date decodes", `"2024-02-29"`, false, "2024-02-29", false, ""},
		{"a malformed date is rejected", `"2024-2-29"`, true, "", false, "YYYY-MM-DD"},
		{"a number is rejected", `20240229`, true, "", false, "must be a JSON string"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			got, err := decodeEmptyDate("Row", "declarationDate", jsontext.Value(tc.raw), tc.allowNull)
			if tc.wantErr != "" {
				if err == nil || !strings.Contains(err.Error(), tc.wantErr) {
					t.Fatalf("error = %v, want it to contain %q", err, tc.wantErr)
				}
				return
			}
			if err != nil {
				t.Fatalf("unexpected error %v", err)
			}
			if tc.wantNil != (got == nil) {
				t.Fatalf("got = %v, want nil=%t", got, tc.wantNil)
			}
			if got != nil && got.String() != tc.want {
				t.Fatalf("got = %s, want %s", got, tc.want)
			}
		})
	}
}

func TestRawMemberTreatsMissingAndNullAlike(t *testing.T) {
	t.Parallel()
	members := map[string]jsontext.Value{
		"lastUpdated\"": jsontext.Value(`"2024-09-06\""`),
		"nulled":        jsontext.Value(`null`),
	}
	if got := rawMember(members, "lastUpdated\""); string(got) != `"2024-09-06\""` {
		t.Fatalf("present member = %s, want its raw value", got)
	}
	if got := rawMember(members, "nulled"); got != nil {
		t.Fatalf("null member = %s, want nil", got)
	}
	if got := rawMember(members, "absent"); got != nil {
		t.Fatalf("absent member = %s, want nil", got)
	}
	if got := rawMember(nil, "lastUpdated\""); got != nil {
		t.Fatalf("nil map = %s, want nil", got)
	}
}

func TestRequireObjectRowsRejectsNonObjectRows(t *testing.T) {
	t.Parallel()
	rows := []jsontext.Value{jsontext.Value(`{"a":1}`), jsontext.Value(`{}`)}
	if err := requireObjectRows("dynamic-id", rows); err != nil {
		t.Fatalf("object rows rejected: %v", err)
	}
	if err := requireObjectRows("dynamic-id", nil); err != nil {
		t.Fatalf("no rows rejected: %v", err)
	}
	err := requireObjectRows("dynamic-id", []jsontext.Value{jsontext.Value(`{}`), jsontext.Value(`[1]`)})
	var typed *Error
	if !errors.As(err, &typed) || typed.Category != CategoryDecode || typed.Endpoint != "dynamic-id" ||
		!strings.Contains(typed.Message, "row 1") {
		t.Fatalf("error = %+v, want a Decode error naming row 1 and the endpoint", typed)
	}
}
