package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"testing"
)

// TestDynamicValuesRoundTripLosslessly mirrors dynamic_json_preserves_recursive_native_shape
// and dynamic_object_preserves_semantics_without_treating_member_order_as_data.
func TestDynamicValuesRoundTripLosslessly(t *testing.T) {
	t.Parallel()
	wires := []string{
		`{"documenttype":"10-K","documentannualreport":"true","documentfiscalyearfocus":2025,` +
			`"nested":[null,false,{"amount":"33644000000","exact":9007199254740993}]}`,
		`{"Issuer Defined Section":{"heterogeneousCells":[null,false,"text",184467440737095516160,-42,0.125],` +
			`"nested":{"arbitrary key":[1,{"flag":true}]}},"another section":[]}`,
		`{"cik":4874072686740,"ratio":0.1,"beyondU64":18446744073709551616}`,
		`[null,false,"text",184467440737095516160,-42,0.125]`,
	}
	for _, wire := range wires {
		var value jsontext.Value
		if err := json.Unmarshal([]byte(wire), &value); err != nil {
			t.Fatalf("Unmarshal(%s): %v", wire, err)
		}
		encoded, err := json.Marshal(value)
		if err != nil || string(encoded) != wire {
			t.Fatalf("Marshal = %s (%v), want %s", encoded, err, wire)
		}
	}

	var object map[string]jsontext.Value
	if err := json.Unmarshal([]byte(wires[1]), &object); err != nil {
		t.Fatal(err)
	}
	if len(object) != 2 || string(object["another section"]) != "[]" {
		t.Fatalf("object members = %v", object)
	}
	var section struct {
		Cells []jsontext.Value `json:"heterogeneousCells"`
	}
	if err := json.Unmarshal(object["Issuer Defined Section"], &section); err != nil {
		t.Fatal(err)
	}
	if len(section.Cells) != 6 || string(section.Cells[3]) != "184467440737095516160" ||
		string(section.Cells[5]) != "0.125" {
		t.Fatalf("cells = %v", section.Cells)
	}
}

// TestWireBoolPolicy pins the ADR rule: WireBool is a Go bool while the
// provider's string flags stay string and are never converted.
func TestWireBoolPolicy(t *testing.T) {
	t.Parallel()
	type flags struct {
		Native    bool   `json:"native"`
		Yn        string `json:"yn"`
		YesNo     string `json:"yesNo"`
		TrueFalse string `json:"trueFalse"`
		TitleCase string `json:"titleCase"`
	}
	wire := `{"native":true,"yn":"N","yesNo":"Yes","trueFalse":"false","titleCase":"False"}`
	var got flags
	if err := json.Unmarshal([]byte(wire), &got); err != nil {
		t.Fatal(err)
	}
	want := flags{Native: true, Yn: "N", YesNo: "Yes", TrueFalse: "false", TitleCase: "False"}
	if got != want {
		t.Fatalf("flags = %+v, want %+v", got, want)
	}
	if encoded, err := json.Marshal(got); err != nil || string(encoded) != wire {
		t.Fatalf("Marshal = %s (%v)", encoded, err)
	}
	for _, wire := range []string{`{"native":"true"}`, `{"native":"False"}`, `{"yn":false}`, `{"trueFalse":true}`} {
		if err := json.Unmarshal([]byte(wire), &flags{}); err == nil {
			t.Fatalf("%s: string and boolean flags were coerced", wire)
		}
	}
}
