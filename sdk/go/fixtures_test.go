package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"maps"
	"os"
	"path/filepath"
	"slices"
	"testing"
)

// fixturesDir is the shared Rust fixture directory, relative to the package
// directory that "go test" uses as the working directory (ADR 0030: the Rust
// and Go decoders are proven against identical bytes).
var fixturesDir = filepath.Join("..", "..", "crates", "libfmp", "tests", "fixtures")

// fixturePath resolves a shared fixture by file name and fails the test when
// the file is absent, so a renamed fixture is reported by name rather than as
// a decode failure.
func fixturePath(t *testing.T, name string) string {
	t.Helper()
	path := filepath.Join(fixturesDir, name)
	if _, err := os.Stat(path); err != nil {
		t.Fatalf("fixture %s: %v", name, err)
	}
	return path
}

func readFixture(t *testing.T, name string) []byte {
	t.Helper()
	raw, err := os.ReadFile(fixturePath(t, name))
	if err != nil {
		t.Fatalf("fixture %s: %v", name, err)
	}
	return raw
}

// assertFixtureParity decodes a bare-array fixture into []T and enforces the
// ADR 0030 parity rule: every element decodes without error, re-encodes to
// exactly the fixture element's member set, and tolerates an unknown member.
// unknown lists members the fixture carries on purpose that the model does
// not know; each must appear in at least one element so the list cannot go
// stale. The decoded rows are returned for exact-value assertions.
func assertFixtureParity[T any](t *testing.T, name string, unknown ...string) []T {
	t.Helper()
	return fixtureParity[T](t, name, nil, unknown)
}

// assertFixtureParityNullWhenOmitted is assertFixtureParity for a fixture
// that omits an optional member the model always re-encodes: serde reads a
// missing Option member as None and writes None back as null, so the Go
// model re-encodes the member as null. Every member in nullWhenOmitted must
// be absent from at least one element, where it must re-encode as null, so
// the list cannot go stale.
func assertFixtureParityNullWhenOmitted[T any](t *testing.T, name string, nullWhenOmitted []string, unknown ...string) []T {
	t.Helper()
	return fixtureParity[T](t, name, nullWhenOmitted, unknown)
}

func fixtureParity[T any](t *testing.T, name string, nullWhenOmitted, unknown []string) []T {
	t.Helper()
	raw := readFixture(t, name)

	var rows []T
	if err := json.Unmarshal(raw, &rows); err != nil {
		t.Fatalf("%s: decode into []%T: %v", name, *new(T), err)
	}
	var wire []map[string]jsontext.Value
	if err := json.Unmarshal(raw, &wire); err != nil {
		t.Fatalf("%s: fixture is not a bare array of objects: %v", name, err)
	}
	if len(rows) != len(wire) {
		t.Fatalf("%s: decoded %d rows, fixture has %d", name, len(rows), len(wire))
	}

	seen := map[string]bool{}
	omitted := map[string]bool{}
	for index, row := range rows {
		want := make([]string, 0, len(wire[index]))
		for member := range wire[index] {
			if slices.Contains(unknown, member) {
				seen[member] = true
				continue
			}
			want = append(want, member)
		}
		members := memberValues(t, row)
		for _, member := range nullWhenOmitted {
			if _, present := wire[index][member]; present {
				continue
			}
			omitted[member] = true
			if value, ok := members[member]; !ok || value.Kind() != 'n' {
				t.Fatalf("%s[%d]: omitted member %q re-encoded as %s, want null", name, index, member, value)
			}
			want = append(want, member)
		}
		slices.Sort(want)
		if got := slices.Sorted(maps.Keys(members)); !slices.Equal(got, want) {
			t.Fatalf("%s[%d]: re-encoded members %v, fixture members %v", name, index, got, want)
		}

		extended := wire[index]
		extended["fmpUnknownMemberProbe"] = jsontext.Value(`"ignored"`)
		encoded, err := json.Marshal(extended)
		if err != nil {
			t.Fatalf("%s[%d]: re-encode with an unknown member: %v", name, index, err)
		}
		var probe T
		if err := json.Unmarshal(encoded, &probe); err != nil {
			t.Fatalf("%s[%d]: unknown member was not ignored: %v", name, index, err)
		}
	}
	for _, member := range unknown {
		if !seen[member] {
			t.Fatalf("%s: expected unknown member %q is absent from every element", name, member)
		}
	}
	for _, member := range nullWhenOmitted {
		if !omitted[member] {
			t.Fatalf("%s: member %q is present in every element, so it is not omitted", name, member)
		}
	}
	return rows
}

// memberValues re-encodes one model value and returns its members.
func memberValues[T any](t *testing.T, row T) map[string]jsontext.Value {
	t.Helper()
	encoded, err := json.Marshal(row)
	if err != nil {
		t.Fatalf("re-encode %T: %v", row, err)
	}
	var members map[string]jsontext.Value
	if err := json.Unmarshal(encoded, &members); err != nil {
		t.Fatalf("re-encoded %T is not an object: %v", row, err)
	}
	return members
}

// memberSet re-encodes one model value and returns its sorted member names.
func memberSet[T any](t *testing.T, row T) []string {
	t.Helper()
	return slices.Sorted(maps.Keys(memberValues(t, row)))
}
