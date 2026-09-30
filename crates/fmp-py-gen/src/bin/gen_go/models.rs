//! Renders `sdk/go/<domain>_models.go`: one public struct per discovered
//! response struct, its pointer-field shadow, and the `UnmarshalJSONFrom`
//! that enforces required members the way serde does (ADR 0030).

use std::fmt::Write;

use fmp_py_gen::responses::StructDef;

use crate::emit::{GENERATED_HEADER, doc_comment, go_name, local_ident, lower_first, lower_lead};
use crate::types::{Codec, GoField, TypeTable};

/// The shadow and emit member that collects the members json/v2 cannot spell
/// in a struct tag (`json:",embed"` on a `map[string]jsontext.Value`).
const RAW_MEMBERS: &str = "RawMembers";

/// One Go model ready to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelPlan {
    pub(crate) name: String,
    pub(crate) doc: String,
    /// The Rust field names, parallel to `fields`, for local identifiers.
    pub(crate) rust_names: Vec<String>,
    pub(crate) fields: Vec<GoField>,
}

/// Maps every struct of one domain, failing on the first unmapped field.
pub(crate) fn plan_models(
    domain: &str,
    defs: &[&StructDef],
    table: &TypeTable<'_>,
) -> Result<Vec<ModelPlan>, String> {
    defs.iter()
        .map(|def| {
            let fields = def
                .fields
                .iter()
                .map(|field| table.go_field(def, field))
                .collect::<Result<Vec<_>, _>>()?;
            if def.custom_deserialize {
                check_custom_deserialize_shape(&def.name, &fields)?;
            }
            check_raw_key_fallback(&def.name, &fields)?;
            let name = go_name(&def.name);
            let doc = match &def.doc {
                Some(doc) => format!("{name} is {}", lower_lead(doc)),
                None => format!("{name} is a response model of the {domain} domain."),
            };
            Ok(ModelPlan {
                name,
                doc,
                rust_names: def.fields.iter().map(|f| f.name.clone()).collect(),
                fields,
            })
        })
        .collect()
}

/// A struct with a hand-written `Deserialize` is supported in one shape
/// only: named scalar members plus exactly one `DynamicObject` holding every
/// other member (the FinancialReportJson shape). Anything else needs a
/// decision, so it fails naming the struct.
fn check_custom_deserialize_shape(name: &str, fields: &[GoField]) -> Result<(), String> {
    let embedded = fields
        .iter()
        .filter(|field| field.codec == Codec::Embedded)
        .count();
    let plain = fields
        .iter()
        .all(|field| matches!(field.codec, Codec::Embedded | Codec::Plain));
    if embedded == 1 && plain {
        return Ok(());
    }
    Err(format!(
        "{name}: a hand-written Deserialize is supported by gen_go only as plain members plus \
         exactly one DynamicObject holding the remaining members; found {embedded} DynamicObject \
         member(s) and {} other codec(s)",
        if plain { "no" } else { "some" }
    ))
}

/// A raw-keyed member (a wire name json/v2 cannot spell in a struct tag)
/// rides on an embedded fallback, and json/v2 allows one fallback per
/// struct: a model whose remaining members already flow through an
/// `Embedded` member cannot also carry raw keys, and the fallback's own name
/// must stay free.
fn check_raw_key_fallback(name: &str, fields: &[GoField]) -> Result<(), String> {
    if !fields.iter().any(|f| f.raw_key) {
        return Ok(());
    }
    if let Some(rest) = fields.iter().find(|f| f.codec == Codec::Embedded) {
        return Err(format!(
            "{name}: raw-keyed members need the embedded fallback, but {} already embeds the \
             remaining members; json/v2 allows one embedded fallback per struct",
            rest.name
        ));
    }
    if fields.iter().any(|f| f.name == RAW_MEMBERS) {
        return Err(format!(
            "{name}: a field named {RAW_MEMBERS} collides with the embedded fallback that carries \
             its raw-keyed members"
        ));
    }
    Ok(())
}

/// Renders the whole models file (before `gofmt`).
pub(crate) fn render_models(domain: &str, models: &[ModelPlan]) -> String {
    let mut out = String::new();
    out.push_str(GENERATED_HEADER);
    out.push_str("\n\npackage fmp\n\n");
    let _ = writeln!(
        out,
        "// Response models of the {domain} domain, generated from the\n\
         // crates/libfmp/src/responses/{domain} module with the ADR 0030 type table.\n"
    );
    out.push_str("import (\n\t\"encoding/json/jsontext\"\n\t\"encoding/json/v2\"\n)\n");
    for model in models {
        out.push('\n');
        render_model(model, &mut out);
    }
    out
}

fn render_model(model: &ModelPlan, out: &mut String) {
    let shadow = format!("{}Shadow", lower_first(&model.name));
    let has_raw = model.fields.iter().any(|f| f.raw_key);
    out.push_str(&doc_comment(&model.doc));
    let _ = writeln!(out, "type {} struct {{", model.name);
    for field in &model.fields {
        let options = if field.omit_none { ",omitzero" } else { "" };
        let tag = if field.raw_key {
            "-".to_string()
        } else {
            member_tag(field, options)
        };
        let _ = writeln!(out, "\t{} {} `json:\"{tag}\"`", field.name, field.public_ty);
    }
    out.push_str("}\n\n");

    let mut shadow_doc = format!(
        "{shadow} mirrors {} with a pointer or raw value for every required member so a \
         missing or null member is observable after decoding.",
        model.name
    );
    if has_raw {
        shadow_doc.push_str(
            " The embedded fallback collects the members json/v2 cannot spell in a struct tag.",
        );
    }
    out.push_str(&doc_comment(&shadow_doc));
    let _ = writeln!(out, "type {shadow} struct {{");
    for field in model.fields.iter().filter(|f| !f.raw_key) {
        let _ = writeln!(
            out,
            "\t{} {} `json:\"{}\"`",
            field.name,
            field.shadow_ty,
            member_tag(field, "")
        );
    }
    if has_raw {
        let _ = writeln!(
            out,
            "\t{RAW_MEMBERS} map[string]jsontext.Value `json:\",embed\"`"
        );
    }
    out.push_str("}\n\n");

    out.push_str(
        "// UnmarshalJSONFrom decodes one JSON object and rejects it with a Decode\n\
         // error naming the first required member that is missing or null, as the\n\
         // Rust decoder does. ",
    );
    match model.fields.iter().find(|f| f.codec == Codec::Embedded) {
        Some(rest) => {
            let _ = writeln!(
                out,
                "Every member no named field claims is kept in\n// {}.",
                rest.name
            );
        }
        None => out.push_str("Unknown members are ignored.\n"),
    }
    let _ = writeln!(
        out,
        "func (m *{}) UnmarshalJSONFrom(dec *jsontext.Decoder) error {{",
        model.name
    );
    let _ = writeln!(out, "\tvar shadow {shadow}");
    out.push_str(
        "\tif err := json.UnmarshalDecode(dec, &shadow); err != nil {\n\t\treturn err\n\t}\n",
    );
    for (field, rust_name) in model.fields.iter().zip(&model.rust_names) {
        if field.raw_key {
            let _ = writeln!(
                out,
                "\t{} := rawMember(shadow.{RAW_MEMBERS}, {:?})",
                wire_local(rust_name),
                field.wire
            );
        }
    }
    render_required_switch(model, out);
    for (field, rust_name) in model.fields.iter().zip(&model.rust_names) {
        render_codec_block(&model.name, field, &local_name(rust_name), out);
    }
    let _ = writeln!(out, "\t*m = {}{{", model.name);
    for (field, rust_name) in model.fields.iter().zip(&model.rust_names) {
        let value = match field.codec {
            _ if field.raw_key => local_name(rust_name),
            Codec::Plain | Codec::DynamicObject | Codec::Number if !field.optional => {
                format!("*shadow.{}", field.name)
            }
            Codec::Plain
            | Codec::DynamicObject
            | Codec::Number
            | Codec::DynamicJson
            | Codec::Embedded => {
                format!("shadow.{}", field.name)
            }
            Codec::RequiredOption
            | Codec::RequiredNumber
            | Codec::Count
            | Codec::EmptyOrNullString
            | Codec::NullTextString
            | Codec::EmptyDate
            | Codec::EmptyOrNullDate => local_name(rust_name),
        };
        let _ = writeln!(out, "\t\t{}: {value},", field.name);
    }
    out.push_str("\t}\n\treturn nil\n}\n");
    if has_raw {
        render_marshal(model, out);
    }
}

/// The `MarshalJSONTo` of a model with raw-keyed members: the tagged members
/// marshal through a method-less copy of the struct, and the raw-keyed ones
/// through an embedded fallback next to it, each under its exact wire name.
fn render_marshal(model: &ModelPlan, out: &mut String) {
    let plain = format!("{}Plain", lower_first(&model.name));
    let emit = format!("{}Emit", lower_first(&model.name));
    let raw: Vec<(&GoField, &String)> = model
        .fields
        .iter()
        .zip(&model.rust_names)
        .filter(|(f, _)| f.raw_key)
        .collect();
    out.push_str(&doc_comment(&format!(
        "{plain} is {} without its JSON methods, so its tagged members marshal through the \
         ordinary struct rules.",
        model.name
    )));
    let _ = writeln!(out, "type {plain} {}\n", model.name);
    out.push_str(&doc_comment(&format!(
        "{emit} carries the members json/v2 cannot spell in a struct tag through an embedded \
         fallback, next to the promoted tagged members of {plain}."
    )));
    let _ = writeln!(
        out,
        "type {emit} struct {{\n\t{plain}\n\t{RAW_MEMBERS} map[string]jsontext.Value \
         `json:\",embed\"`\n}}\n"
    );
    out.push_str(
        "// MarshalJSONTo encodes the object with every member under its exact wire\n\
         // name, including the names json/v2 cannot spell in a struct tag. The value\n\
         // receiver keeps non-addressable values, such as slice elements, on this path.\n",
    );
    let _ = writeln!(
        out,
        "func (m {}) MarshalJSONTo(enc *jsontext.Encoder) error {{\n\
         \trawMembers := make(map[string]jsontext.Value, {})",
        model.name,
        raw.len()
    );
    for (field, rust_name) in raw {
        let local = wire_local(rust_name);
        let indent = if field.omit_none { "\t\t" } else { "\t" };
        if field.omit_none {
            let _ = writeln!(out, "\tif m.{} != nil {{", field.name);
        }
        let _ = writeln!(
            out,
            "{indent}{local}, err := json.Marshal(m.{})\n{indent}if err != nil {{\n\
             {indent}\treturn err\n{indent}}}\n{indent}rawMembers[{:?}] = {local}",
            field.name, field.wire
        );
        if field.omit_none {
            out.push_str("\t}\n");
        }
    }
    let _ = writeln!(
        out,
        "\treturn json.MarshalEncode(enc, {emit}{{\n\t\t{plain}: {plain}(m),\n\
         \t\t{RAW_MEMBERS}: rawMembers,\n\t}})\n}}"
    );
}

/// The `switch` that reports the first missing required member, in field order.
fn render_required_switch(model: &ModelPlan, out: &mut String) {
    let required: Vec<(&GoField, &String)> = model
        .fields
        .iter()
        .zip(&model.rust_names)
        .filter(|(f, _)| f.required_key())
        .collect();
    if required.is_empty() {
        return;
    }
    out.push_str("\tswitch {\n");
    for (field, rust_name) in required {
        let test = match field.codec {
            _ if field.raw_key => format!("{} == nil", wire_local(rust_name)),
            Codec::RequiredOption
            | Codec::RequiredNumber
            | Codec::Count
            | Codec::EmptyOrNullString
            | Codec::NullTextString
            | Codec::EmptyDate
            | Codec::EmptyOrNullDate
            | Codec::DynamicJson => format!("len(shadow.{}) == 0", field.name),
            Codec::Plain | Codec::DynamicObject | Codec::Number => {
                format!("shadow.{} == nil", field.name)
            }
            Codec::Embedded => unreachable!("embedded members are never required"),
        };
        let _ = writeln!(
            out,
            "\tcase {test}:\n\t\treturn missingMemberError({:?}, {:?})",
            model.name, field.wire
        );
    }
    out.push_str("\t}\n");
}

/// The per-member decode a raw shadow value needs after the presence check.
fn render_codec_block(model: &str, field: &GoField, local: &str, out: &mut String) {
    if field.raw_key {
        render_raw_member(field, local, out);
        return;
    }
    match field.codec {
        Codec::Plain | Codec::DynamicJson => {}
        Codec::Embedded => {
            let _ = writeln!(
                out,
                "\tif len(shadow.{0}) == 0 {{\n\t\tshadow.{0} = jsontext.Value(\"{{}}\")\n\t}}",
                field.name
            );
        }
        Codec::RequiredOption => {
            let inner = field.public_ty.trim_start_matches('*');
            let _ = writeln!(
                out,
                "\tvar {local} {}\n\tif shadow.{}.Kind() != 'n' {{\n\t\tvar value {inner}\n\
                 \t\tif err := json.Unmarshal(shadow.{}, &value); err != nil {{\n\
                 \t\t\treturn err\n\t\t}}\n\t\t{local} = &value\n\t}}",
                field.public_ty, field.name, field.name
            );
        }
        Codec::EmptyOrNullString | Codec::NullTextString => {
            let inner = field.public_ty.trim_start_matches('*');
            let sentinel = if field.codec == Codec::NullTextString {
                "NULL"
            } else {
                ""
            };
            let _ = writeln!(
                out,
                "\tvar {local} {}\n\tif shadow.{}.Kind() != 'n' {{\n\t\tvar value {inner}\n\
                 \t\tif err := json.Unmarshal(shadow.{}, &value); err != nil {{\n\
                 \t\t\treturn err\n\t\t}}\n\t\tif value != {sentinel:?} {{\n\t\t\t{local} = &value\n\t\t}}\n\t}}",
                field.public_ty, field.name, field.name
            );
        }
        Codec::EmptyDate | Codec::EmptyOrNullDate => {
            let allow_null = field.codec == Codec::EmptyOrNullDate;
            let _ = writeln!(
                out,
                "\t{local}, err := decodeEmptyDate({model:?}, {:?}, shadow.{}, {allow_null})\n\
                 \tif err != nil {{\n\t\treturn err\n\t}}",
                field.wire, field.name
            );
        }
        Codec::Count => {
            let _ = writeln!(
                out,
                "\t{local}, err := decodeCount({model:?}, {:?}, shadow.{})\n\
                 \tif err != nil {{\n\t\treturn err\n\t}}",
                field.wire, field.name
            );
        }
        Codec::DynamicObject => render_kind_guard(model, field, '{', "object", out),
        Codec::Number => render_kind_guard(model, field, '0', "number", out),
        Codec::RequiredNumber => {
            let _ = writeln!(
                out,
                "\tvar {local} *jsontext.Value\n\tif shadow.{}.Kind() != 'n' {{\n\
                 \t\tif shadow.{}.Kind() != '0' {{\n\
                 \t\t\treturn invalidMemberError({model:?}, {:?}, \"number\")\n\t\t}}\n\
                 \t\t{local} = &shadow.{}\n\t}}",
                field.name, field.name, field.wire, field.name
            );
        }
    }
}

/// The kind check of a pointer-shadow raw member: a present value whose
/// starting token is not `kind` is rejected the way serde rejects it.
fn render_kind_guard(model: &str, field: &GoField, kind: char, expected: &str, out: &mut String) {
    let guard = if field.optional {
        format!(
            "shadow.{} != nil && shadow.{}.Kind() != '{kind}'",
            field.name, field.name
        )
    } else {
        format!("shadow.{}.Kind() != '{kind}'", field.name)
    };
    let _ = writeln!(
        out,
        "\tif {guard} {{\n\t\treturn invalidMemberError({model:?}, {:?}, {expected:?})\n\t}}",
        field.wire
    );
}

/// The `json` tag body of one member: the wire name plus options, or the
/// `embed` option alone for the member that holds the remaining members.
fn member_tag(field: &GoField, options: &str) -> String {
    if field.codec == Codec::Embedded {
        ",embed".to_string()
    } else {
        format!("{}{options}", field.wire)
    }
}

/// The decode of a raw-keyed plain member from the value `rawMember` found:
/// missing and null were already reported for a required member and mean nil
/// for an optional one.
fn render_raw_member(field: &GoField, local: &str, out: &mut String) {
    let wire = format!("{local}Wire");
    if field.optional {
        let inner = field.public_ty.trim_start_matches('*');
        let _ = writeln!(
            out,
            "\tvar {local} {}\n\tif {wire} != nil {{\n\t\tvar value {inner}\n\
             \t\tif err := json.Unmarshal({wire}, &value); err != nil {{\n\
             \t\t\treturn err\n\t\t}}\n\t\t{local} = &value\n\t}}",
            field.public_ty
        );
    } else {
        let _ = writeln!(
            out,
            "\tvar {local} {}\n\tif err := json.Unmarshal({wire}, &{local}); err != nil {{\n\
             \t\treturn err\n\t}}",
            field.public_ty
        );
    }
}

/// The local that holds the raw wire value of a raw-keyed member.
fn wire_local(rust_name: &str) -> String {
    format!("{}Wire", local_name(rust_name))
}

/// A local variable name for a raw-decoded member that cannot collide with
/// the identifiers `UnmarshalJSONFrom` already uses.
fn local_name(rust_name: &str) -> String {
    let ident = local_ident(rust_name);
    if matches!(ident.as_str(), "m" | "dec" | "shadow" | "err" | "value") {
        format!("{ident}Member")
    } else {
        ident
    }
}
