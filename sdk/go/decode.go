package fmp

import (
	"bytes"
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
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
			return nil, memberDecodeError(model, member, raw, err)
		}
		if text == "" {
			return nil, nil
		}
		date, err := ParseDate(text)
		if err != nil {
			return nil, invalidMemberValueError(model, member, "string holding a "+dateExpected)
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
		return 0, invalidMemberValueError(model, member, "non-negative integral number")
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
			rowErr := decodeError(endpointID, 0, nil, "row "+strconv.Itoa(index)+" is not a JSON object", nil)
			rowErr.Path, rowErr.DecodeKind = "/"+strconv.Itoa(index), DecodeKindWrongType
			return rowErr
		}
	}
	return nil
}

// decodeLocation turns a json.Unmarshal failure into the JSON pointer of the
// offending member and a coarse DecodeKind. It reads only the structured
// fields of the json/v2 error, never its text, so no member value survives.
// A generated model reports a missing or null required member against the
// enclosing object; the member name is appended and body is consulted to
// tell a JSON null from an absent member.
func decodeLocation(body []byte, err error) (string, DecodeKind) {
	var syntactic *jsontext.SyntacticError
	if errors.As(err, &syntactic) {
		return string(syntactic.JSONPointer), DecodeKindSyntax
	}
	var semantic *json.SemanticError
	if !errors.As(err, &semantic) {
		return "", DecodeKindInvalidValue
	}
	pointer := semantic.JSONPointer
	var member *Error
	if errors.As(semantic.Err, &member) && member.member != "" {
		pointer = pointer.AppendToken(member.member) + member.memberPath
		value, found := valueAt(body, pointer)
		switch {
		case !found:
			return string(pointer), DecodeKindMissingMember
		case value.Kind() == 'n':
			return string(pointer), DecodeKindNull
		case member.memberKind != DecodeKindNone:
			return string(pointer), member.memberKind
		default:
			return string(pointer), DecodeKindWrongType
		}
	}
	switch {
	case semantic.JSONKind == 'n':
		return string(pointer), DecodeKindNull
	case semantic.Err == nil:
		return string(pointer), DecodeKindWrongType
	default:
		return string(pointer), DecodeKindInvalidValue
	}
}

const (
	maxPlainTextMessageBytes   = 256
	maxErrorMessageObjectBytes = 1024
	errorMessageMember         = "Error Message"
)

// isProviderMessage recognizes the two error shapes FMP sends with HTTP 200,
// mirroring the Rust crate: a short plain-text line such as "Invalid name"
// that is not JSON at all, or a JSON object whose only member is
// "Error Message". Both checks are bounded by size so ordinary payloads are
// not parsed twice.
func isProviderMessage(body []byte) bool {
	body = bytes.Trim(body, " \t\n\f\r")
	if len(body) == 0 {
		return false
	}
	switch body[0] {
	case '{':
		if len(body) > maxErrorMessageObjectBytes {
			return false
		}
		var members map[string]jsontext.Value
		if json.Unmarshal(body, &members) != nil {
			return false
		}
		_, ok := members[errorMessageMember]
		return ok && len(members) == 1
	case '[':
		return false
	default:
		if len(body) > maxPlainTextMessageBytes {
			return false
		}
		for _, b := range body {
			if b < ' ' || b > '~' {
				return false
			}
		}
		return !jsontext.Value(body).IsValid()
	}
}

// valueAt walks body along pointer one level at a time and reports the value
// it names, or false when a token does not resolve.
func valueAt(body []byte, pointer jsontext.Pointer) (jsontext.Value, bool) {
	value := jsontext.Value(body)
	for token := range pointer.Tokens() {
		switch value.Kind() {
		case '[':
			var items []jsontext.Value
			index, err := strconv.Atoi(token)
			if json.Unmarshal(value, &items) != nil || err != nil || index < 0 || index >= len(items) {
				return nil, false
			}
			value = items[index]
		case '{':
			var members map[string]jsontext.Value
			if json.Unmarshal(value, &members) != nil {
				return nil, false
			}
			member, ok := members[token]
			if !ok {
				return nil, false
			}
			value = member
		default:
			return nil, false
		}
	}
	return value, true
}
