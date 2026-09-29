package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"math"
	"strconv"
)

// Decode helpers the generated UnmarshalJSONFrom methods call for the codecs
// a pointer shadow member cannot express (ADR 0030, "Codec policy").

// decodeEmptyDate decodes the empty_date and empty_or_null_date codecs of the
// Rust crate: the provider sends "" for an absent date, which becomes nil.
// A JSON null is nil when allowNull is set (empty_or_null_date) and a Decode
// error otherwise (empty_date). raw is the member's value, never empty: the
// generated caller has already reported a missing member.
func decodeEmptyDate(model, member string, raw jsontext.Value, allowNull bool) (*Date, error) {
	switch raw.Kind() {
	case 'n':
		if allowNull {
			return nil, nil
		}
		return nil, invalidMemberError(model, member, "string")
	case '"':
		var text string
		if err := json.Unmarshal(raw, &text); err != nil {
			return nil, err
		}
		if text == "" {
			return nil, nil
		}
		date, err := ParseDate(text)
		if err != nil {
			return nil, err
		}
		return &date, nil
	default:
		return nil, invalidMemberError(model, member, "string")
	}
}

// decodeCount decodes the count codec of the Rust crate: a Count member
// accepts a non-negative JSON integer or a finite integral JSON number below
// 2^64, since the provider has sent 3.0 for 3. A JSON null is reported as a
// missing member, as a pointer shadow member would be; a fractional,
// negative, or out-of-range number and any other kind is a Decode error that
// never echoes the value. raw is never empty: the generated caller has
// already reported a missing member.
func decodeCount(model, member string, raw jsontext.Value) (uint64, error) {
	switch raw.Kind() {
	case 'n':
		return 0, missingMemberError(model, member)
	case '0':
	default:
		return 0, invalidMemberError(model, member, "non-negative integral number")
	}
	text := string(raw)
	if value, err := strconv.ParseUint(text, 10, 64); err == nil {
		return value, nil
	}
	value, err := strconv.ParseFloat(text, 64)
	if err != nil || value < 0 || value >= 0x1p64 || value != math.Trunc(value) {
		return 0, invalidMemberError(model, member, "non-negative integral number")
	}
	return uint64(value), nil
}

// rawMember returns the value of a member that the shadow struct's embedded
// fallback collected because encoding/json/v2 cannot spell its wire name in a
// struct tag (the name contains a comma, a backslash, or a quote). A missing
// member and a JSON null both return nil, as a nil pointer shadow member
// does, so the generated required-member switch treats them alike.
func rawMember(members map[string]jsontext.Value, name string) jsontext.Value {
	value, ok := members[name]
	if !ok || value.Kind() == 'n' {
		return nil
	}
	return value
}

// requireObjectRows enforces the Vec<DynamicObject> contract of a dynamic
// endpoint: every row must be a JSON object, as serde_json::Map requires.
func requireObjectRows(endpointID string, rows []jsontext.Value) error {
	for index, row := range rows {
		if row.Kind() != '{' {
			return decodeError(endpointID, 0, nil, "row "+strconv.Itoa(index)+" is not a JSON object", nil)
		}
	}
	return nil
}
