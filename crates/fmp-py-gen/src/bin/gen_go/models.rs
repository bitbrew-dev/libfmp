//! Renders `sdk/go/<domain>_models.go`: one public struct per discovered
//! response struct, its pointer-field shadow, and the `UnmarshalJSONFrom`
//! that enforces required members the way serde does (ADR 0030).

use std::fmt::Write;

use fmp_py_gen::responses::StructDef;

use crate::emit::{GENERATED_HEADER, doc_comment, local_ident, lower_first, lower_lead};
use crate::types::{Codec, GoField, TypeTable};

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
            let doc = match &def.doc {
                Some(doc) => format!("{} is {}", def.name, lower_lead(doc)),
                None => format!("{} is a response model of the {domain} domain.", def.name),
            };
            Ok(ModelPlan {
                name: def.name.clone(),
                doc,
                rust_names: def.fields.iter().map(|f| f.name.clone()).collect(),
                fields,
            })
        })
        .collect()
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
    out.push_str(&doc_comment(&model.doc));
    let _ = writeln!(out, "type {} struct {{", model.name);
    for field in &model.fields {
        let _ = writeln!(
            out,
            "\t{} {} `json:\"{}\"`",
            field.name, field.public_ty, field.wire
        );
    }
    out.push_str("}\n\n");

    out.push_str(&doc_comment(&format!(
        "{shadow} mirrors {} with a pointer or raw value for every required member so a \
         missing or null member is observable after decoding.",
        model.name
    )));
    let _ = writeln!(out, "type {shadow} struct {{");
    for field in &model.fields {
        let _ = writeln!(
            out,
            "\t{} {} `json:\"{}\"`",
            field.name, field.shadow_ty, field.wire
        );
    }
    out.push_str("}\n\n");

    out.push_str(
        "// UnmarshalJSONFrom decodes one JSON object and rejects it with a Decode\n\
         // error naming the first required member that is missing or null, as the\n\
         // Rust decoder does. Unknown members are ignored.\n",
    );
    let _ = writeln!(
        out,
        "func (m *{}) UnmarshalJSONFrom(dec *jsontext.Decoder) error {{",
        model.name
    );
    let _ = writeln!(out, "\tvar shadow {shadow}");
    out.push_str(
        "\tif err := json.UnmarshalDecode(dec, &shadow); err != nil {\n\t\treturn err\n\t}\n",
    );
    render_required_switch(model, out);
    for (field, rust_name) in model.fields.iter().zip(&model.rust_names) {
        render_codec_block(&model.name, field, &local_name(rust_name), out);
    }
    let _ = writeln!(out, "\t*m = {}{{", model.name);
    for (field, rust_name) in model.fields.iter().zip(&model.rust_names) {
        let value = match field.codec {
            Codec::Plain | Codec::DynamicObject | Codec::Number if !field.optional => {
                format!("*shadow.{}", field.name)
            }
            Codec::Plain | Codec::DynamicObject | Codec::Number | Codec::DynamicJson => {
                format!("shadow.{}", field.name)
            }
            Codec::RequiredOption
            | Codec::RequiredNumber
            | Codec::EmptyDate
            | Codec::EmptyOrNullDate => local_name(rust_name),
        };
        let _ = writeln!(out, "\t\t{}: {value},", field.name);
    }
    out.push_str("\t}\n\treturn nil\n}\n");
}

/// The `switch` that reports the first missing required member, in field order.
fn render_required_switch(model: &ModelPlan, out: &mut String) {
    let required: Vec<&GoField> = model.fields.iter().filter(|f| f.required_key()).collect();
    if required.is_empty() {
        return;
    }
    out.push_str("\tswitch {\n");
    for field in required {
        let test = match field.codec {
            Codec::RequiredOption
            | Codec::RequiredNumber
            | Codec::EmptyDate
            | Codec::EmptyOrNullDate
            | Codec::DynamicJson => format!("len(shadow.{}) == 0", field.name),
            Codec::Plain | Codec::DynamicObject | Codec::Number => {
                format!("shadow.{} == nil", field.name)
            }
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
    match field.codec {
        Codec::Plain | Codec::DynamicJson => {}
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
        Codec::EmptyDate | Codec::EmptyOrNullDate => {
            let allow_null = field.codec == Codec::EmptyOrNullDate;
            let _ = writeln!(
                out,
                "\t{local}, err := decodeEmptyDate({model:?}, {:?}, shadow.{}, {allow_null})\n\
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
