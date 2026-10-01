package fmp

import (
	"bytes"
	"context"
	"encoding/csv"
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"io"
	"reflect"
	"slices"
	"strconv"
	"strings"
)

// textCSV is the media type FMP answers on every bulk route (ADR 0035).
const textCSV = "text/csv"

// cellKind says how one CSV cell is written into the JSON member its model
// field decodes from.
type cellKind uint8

const (
	cellString cellKind = iota
	cellPlainString
	cellNumber
	cellBool
)

// csvColumn is the decode plan of one model member: its cell kind and
// whether the field is a pointer, which makes an empty cell absent.
type csvColumn struct {
	kind     cellKind
	optional bool
}

// getCSV issues one GET for a bulk endpoint and decodes the headed CSV body
// into rows of T, mirroring the Rust crate's CSV contract. Each record is
// written as a JSON object keyed by the header names and decoded with the
// model's own JSON rules, so a failure reports "/<row>/<member>". Every
// json-tagged member, pointer or not, must be a header column; the first
// record checks it. An empty
// or header-only body yields no rows, and the provider-message check of
// getJSON does not apply. The redirect, timeout, and no-retry rules of
// getJSON are unchanged; the body cap is the bulk limit.
func getCSV[T any](ctx context.Context, c *Client, endpointID, relativePath string, query []queryParam) ([]T, error) {
	if err := validateRelativePath(relativePath); err != nil {
		return nil, err
	}
	target := c.buildEndpointURL(relativePath)
	rawQuery, err := encodeQuery(query, c.auth.queryName)
	if err != nil {
		return nil, err
	}
	target.RawQuery = rawQuery

	ctx, cancel := context.WithTimeout(ctx, c.timeout)
	defer cancel()
	resp, err := c.executeRedirects(ctx, endpointID, target, c.bulkMaxResponseBodyBytes)
	if err != nil {
		return nil, err
	}

	contentType := resp.header.Get("Content-Type")
	switch {
	case contentType == "":
		return nil, decodeError(endpointID, resp.status, c.safeBody(resp.body),
			"successful response omitted its content type", nil)
	case !strings.EqualFold(mediaTypeOf(contentType), textCSV):
		return nil, decodeError(endpointID, resp.status, c.safeBody(resp.body),
			"successful response used an unexpected content type", nil)
	}
	rows, path, kind, cause := decodeCSVRows[T](resp.body)
	if kind != DecodeKindNone {
		decoded := decodeError(endpointID, resp.status, c.safeBody(resp.body),
			"successful response could not be decoded", c.redactCause(cause))
		decoded.Path, decoded.DecodeKind = c.redactor.Redact(path), kind
		return nil, decoded
	}
	return rows, nil
}

// decodeCSVRows decodes body into rows of T. On failure it returns the JSON
// pointer of the row (and member), a DecodeKind, and the cause.
func decodeCSVRows[T any](body []byte) ([]T, string, DecodeKind, error) {
	reader := csv.NewReader(bytes.NewReader(body))
	reader.ReuseRecord = true
	header, err := reader.Read()
	if errors.Is(err, io.EOF) {
		return []T{}, "", DecodeKindNone, nil
	}
	if err != nil {
		return nil, "", DecodeKindSyntax, err
	}
	header = slices.Clone(header)
	plan := csvPlan(reflect.TypeFor[T]())
	rows := []T{}
	var object []byte
	for {
		record, err := reader.Read()
		if errors.Is(err, io.EOF) {
			return rows, "", DecodeKindNone, nil
		}
		row := "/" + strconv.Itoa(len(rows))
		if err != nil {
			return nil, row, DecodeKindSyntax, err
		}
		if len(rows) == 0 {
			if missing, ok := missingColumn(header, reflect.TypeFor[T]()); ok {
				return nil, string(jsontext.Pointer(row).AppendToken(missing)), DecodeKindMissingMember,
					errors.New("a model member is not a header column")
			}
		}
		object = appendCSVObject(object[:0], header, record, plan)
		var value T
		if err := json.Unmarshal(object, &value); err != nil {
			path, kind := decodeLocation(object, err)
			return nil, row + path, kind, err
		}
		rows = append(rows, value)
	}
}

// appendCSVObject writes one record as a JSON object. An empty cell is null
// unless the member is a required plain string, where it stays "".
func appendCSVObject(dst []byte, header, record []string, plan map[string]csvColumn) []byte {
	dst = append(dst, '{')
	for index, name := range header {
		if index > 0 {
			dst = append(dst, ',')
		}
		dst, _ = jsontext.AppendQuote(dst, name)
		dst = append(dst, ':')
		cell, column := record[index], plan[name]
		switch {
		case cell == "" && (column.optional || column.kind != cellPlainString):
			dst = append(dst, "null"...)
		case column.kind == cellNumber && isJSONNumber(cell),
			column.kind == cellBool && (cell == "true" || cell == "false"):
			dst = append(dst, cell...)
		default:
			dst, _ = jsontext.AppendQuote(dst, cell)
		}
	}
	return append(dst, '}')
}

// missingColumn reports the first member of model, in declaration order as
// serde reports it, that the header does not carry.
func missingColumn(header []string, model reflect.Type) (string, bool) {
	if model.Kind() != reflect.Struct {
		return "", false
	}
	for field := range model.Fields() {
		name, _, _ := strings.Cut(field.Tag.Get("json"), ",")
		if name != "" && name != "-" && !slices.Contains(header, name) {
			return name, true
		}
	}
	return "", false
}

func isJSONNumber(cell string) bool {
	value := jsontext.Value(cell)
	return value.IsValid() && value.Kind() == '0'
}

// csvPlan maps every json-tagged field of the struct type to its column plan.
func csvPlan(model reflect.Type) map[string]csvColumn {
	plan := map[string]csvColumn{}
	if model.Kind() != reflect.Struct {
		return plan
	}
	for field := range model.Fields() {
		name, _, _ := strings.Cut(field.Tag.Get("json"), ",")
		if name == "" || name == "-" {
			continue
		}
		fieldType, optional := field.Type, false
		if fieldType.Kind() == reflect.Pointer {
			fieldType, optional = fieldType.Elem(), true
		}
		plan[name] = csvColumn{kind: cellKindOf(fieldType), optional: optional}
	}
	return plan
}

func cellKindOf(fieldType reflect.Type) cellKind {
	switch fieldType.Kind() {
	case reflect.Bool:
		return cellBool
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64,
		reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64,
		reflect.Float32, reflect.Float64:
		return cellNumber
	case reflect.String:
		if fieldType == reflect.TypeFor[string]() {
			return cellPlainString
		}
	}
	return cellString
}
