package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
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
