use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use fmp_py_gen::registry::wire::wire_surface;
use fmp_py_gen::registry::{ArgKind, Registry};
use fmp_py_gen::responses::{FieldAttrs, FieldDef, StructDef, discover};

use super::{Selection, generate, generated_domains, parse_args};
use crate::models::{plan_models, render_models};
use crate::types::{Codec, TypeTable, arg_kind_go};

/// Registry methods whose descriptor attaches `.with_metadata(..)`, the
/// number `registry_check` prints as `metadata ok`.
const DESCRIPTORS_WITH_METADATA: usize = 251;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Collapses the whitespace `gofmt` aligns so unformatted output compares
/// against the committed, formatted files without needing Go installed.
fn normalized(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn sdk_go() -> PathBuf {
    manifest().join("../../sdk/go")
}

/// Renders `names` as the whole generated set, unformatted.
fn generate_domains(names: &BTreeSet<String>) -> Result<Vec<(String, String)>, String> {
    let registry = Registry::load(&manifest().join("registry")).expect("registry loads");
    let wire = wire_surface(&manifest().join("../libfmp/src/endpoints")).expect("endpoints parse");
    let discovery = discover(&manifest().join("../libfmp/src/responses")).expect("responses parse");
    generate(&registry, &wire, &discovery, names, names)
}

fn domain_set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| name.to_string()).collect()
}

/// Mirrors the shell gate: every domain whose committed `<domain>.go`
/// carries the header regenerates to the committed text, at any stage of
/// the fan-out. The quote domain is always among them.
#[test]
fn committed_domains_regenerate_to_the_committed_files() {
    let registry = Registry::load(&manifest().join("registry")).expect("registry loads");
    let committed_set = generated_domains(&registry, &sdk_go());
    assert!(committed_set.contains("quote"), "{committed_set:?}");
    let files = generate_domains(&committed_set).expect("committed domains generate");
    assert_eq!(files.len(), committed_set.len() * 2 + 3);
    for (name, source) in &files {
        let committed = fs::read_to_string(sdk_go().join(name))
            .unwrap_or_else(|error| panic!("{name} is committed under sdk/go: {error}"));
        assert_eq!(
            normalized(source),
            normalized(&committed),
            "{name} differs from the committed file; run gen_go and commit the result"
        );
    }
    let file = |wanted: &str| {
        files
            .iter()
            .find(|(name, _)| name == wanted)
            .map(|(_, source)| source.as_str())
            .unwrap_or_else(|| panic!("{wanted} is rendered"))
    };
    let quote = file("quote.go");
    assert_eq!(quote.matches("func (n *QuoteNamespace) ").count(), 16);
    assert!(quote.contains("shortOnlyParams"));
    assert!(!quote.contains("type QuoteQuery struct"));
    assert!(
        file("queries.go").contains("type QuoteQuery struct"),
        "shared queries live in queries.go"
    );
    let models = file("quote_models.go");
    assert!(models.contains("OneDay float64 `json:\"1D\"`"));
    assert!(models.contains("MarketCap *float64 `json:\"marketCap\"`"));
    assert!(file("namespaces.go").contains("Quote QuoteNamespace"));
    let table = file("metadata_table.go");
    assert!(
        table.contains(
            "\t\"Quote.Full\": {Geography: GeographyWorldwide, Realtime: &RealtimeAccess{Delay: \
             &MarketDataDelay{Minutes: 15, Scope: DelayScopeNasdaq}, UserDeclaration: \
             UserDeclarationRequiredForRealtime}},\n"
        ),
        "{table}"
    );
    assert!(
        table.contains("\t\"Analyst.PriceTargetConsensus\": {Geography: GeographyUSOnly},\n"),
        "{table}"
    );
    if committed_set.len() == registry.domains.len() {
        let (metadata_entries, by_id) = table
            .split_once("var endpointMethodsByID")
            .expect("the id map follows the metadata table");
        assert_eq!(
            metadata_entries
                .lines()
                .filter(|line| line.starts_with("\t\""))
                .count(),
            DESCRIPTORS_WITH_METADATA,
            "one table entry per method whose descriptor attaches metadata"
        );
        assert!(
            table.contains("func EndpointMetadataFor(method string) (EndpointMetadata, bool)"),
            "{table}"
        );
        assert!(
            !metadata_entries.contains("\"Directory.AvailableCountries\""),
            "a method without metadata has no entry: {metadata_entries}"
        );
        assert!(
            by_id.contains(
                "\t\"quote\": {\"Commodities.Quote\", \"Crypto.Quote\", \"Forex.Quote\", \
                 \"Indexes.Quote\", \"Quote.Full\"},\n"
            ),
            "one sorted entry per endpoint id: {by_id}"
        );
        assert!(
            by_id.contains("\"Directory.AvailableCountries\""),
            "the id map lists methods without metadata too: {by_id}"
        );
    }
}

/// The statements domain is the first with nested `[[namespace]]` entries,
/// a hand-written Deserialize (FinancialReportJson), and a binary endpoint.
#[test]
fn nested_namespaces_embed_children_by_registry_path() {
    let files = generate_domains(&domain_set(&["statements"])).expect("statements generates");
    let file = |wanted: &str| {
        files
            .iter()
            .find(|(name, _)| name == wanted)
            .map(|(_, source)| source.as_str())
            .unwrap_or_else(|| panic!("{wanted} is rendered"))
    };
    let domain = file("statements.go");
    assert!(
        domain.contains(
            "// AsReported groups the statements.as_reported endpoints of the statements domain.\n\
             \tAsReported StatementsAsReportedNamespace"
        ),
        "{domain}"
    );
    assert!(
        domain.contains("AsReported: newStatementsAsReportedNamespace(client),"),
        "{domain}"
    );
    assert!(
        domain.contains("reached as Client.Statements.Income and is valid"),
        "{domain}"
    );
    assert!(
        domain.contains("func (n *StatementsIncomeNamespace) Statement(ctx context.Context, q IncomeStatementQuery) ([]IncomeStatement, error)"),
        "{domain}"
    );
    assert_eq!(domain.matches("Namespace struct {").count(), 11, "{domain}");
    assert_eq!(
        domain.matches("\nfunc (n *Statements").count(),
        27,
        "{domain}"
    );
    assert!(
        domain.contains(
            "return n.client.getBinary(ctx, \"financial-reports-xlsx\", \"financial-reports-xlsx\", params, \
             []string{\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet\", \"application/octet-stream\"})"
        ),
        "{domain}"
    );
    assert!(
        domain.contains("statementPeriodParam(\"period\", *q.period)")
            && domain.contains("fiscalPeriodParam(\"period\", q.period)")
            && domain.contains("segmentationStructureParam(\"structure\", *q.structure)"),
        "{domain}"
    );
    let models = file("statements_models.go");
    assert!(
        models.contains("Sections jsontext.Value `json:\",embed\"`"),
        "{models}"
    );
    assert!(
        models.contains("LinkJSON string `json:\"linkJson\"`"),
        "{models}"
    );
    assert!(
        file("namespaces.go").contains("Statements StatementsNamespace"),
        "namespaces.go"
    );
}

#[test]
fn cross_domain_models_require_their_owner_to_be_generated() {
    let error =
        generate_domains(&domain_set(&["forex"])).expect_err("forex references other models");
    assert!(error.contains("domain first"), "{error}");
    assert!(error.contains("_models.go"), "{error}");
}

fn field(name: &str, ty: &str, attrs: FieldAttrs) -> FieldDef {
    FieldDef {
        name: name.to_string(),
        ty: syn::parse_str(ty).expect("valid type"),
        attrs,
    }
}

fn row(fields: Vec<FieldDef>) -> StructDef {
    StructDef {
        name: "Row".to_string(),
        module_path: vec!["test".to_string()],
        doc: None,
        rename_all: Some("camelCase".to_string()),
        fields,
        custom_deserialize: false,
    }
}

fn with(attr: &str) -> FieldAttrs {
    FieldAttrs {
        with: Some(attr.to_string()),
        ..FieldAttrs::default()
    }
}

fn deserialize_with(attr: &str) -> FieldAttrs {
    FieldAttrs {
        deserialize_with: Some(attr.to_string()),
        ..FieldAttrs::default()
    }
}

fn skip_serializing_if(predicate: &str) -> FieldAttrs {
    FieldAttrs {
        skip_serializing_if: Some(predicate.to_string()),
        ..FieldAttrs::default()
    }
}

fn rename(wire: &str) -> FieldAttrs {
    FieldAttrs {
        rename: Some(wire.to_string()),
        ..FieldAttrs::default()
    }
}

#[test]
fn unmapped_types_fail_naming_the_struct_and_field() {
    let aliases = BTreeMap::new();
    let structs = Vec::new();
    let table = TypeTable::new(&structs, &aliases);
    let cases = [
        (
            "fiscal_year",
            "FiscalYear",
            FieldAttrs::default(),
            "FiscalYear",
        ),
        (
            "link",
            "SecretString",
            FieldAttrs::default(),
            "unmapped Rust type `SecretString`",
        ),
        (
            "value",
            "Vec<Number>",
            FieldAttrs::default(),
            "Number inside Vec has no per-element number check",
        ),
        (
            "counted",
            "Number",
            deserialize_with("required_option"),
            "Option<scalar, Number, or DynamicJson>",
        ),
        (
            "twice",
            "Option<Option<f64>>",
            FieldAttrs::default(),
            "Option<Option<_>>",
        ),
        (
            "tuple",
            "(u8, u8)",
            FieldAttrs::default(),
            "unsupported type shape",
        ),
        ("odd", "Option<Date>", with("bogus"), "codec `bogus`"),
        (
            "plain",
            "f64",
            deserialize_with("required_option"),
            "Option<scalar, Number, or DynamicJson>",
        ),
        (
            "object",
            "Option<DynamicObject>",
            deserialize_with("required_option"),
            "Option<scalar, Number, or DynamicJson>",
        ),
        (
            "defaulted",
            "f64",
            FieldAttrs {
                default: true,
                ..FieldAttrs::default()
            },
            "default",
        ),
        (
            "defaulted_codec",
            "Option<f64>",
            FieldAttrs {
                default: true,
                deserialize_with: Some("required_option".to_string()),
                ..FieldAttrs::default()
            },
            "default",
        ),
        (
            "never_none",
            "f64",
            skip_serializing_if("Option::is_none"),
            "needs an Option<_> field",
        ),
        (
            "custom_skip",
            "Option<f64>",
            skip_serializing_if("Vec::is_empty"),
            "skip_serializing_if `Vec::is_empty`",
        ),
    ];
    for (name, ty, attrs, expected) in cases {
        let def = row(vec![field(name, ty, attrs)]);
        let error = table.go_field(&def, &def.fields[0]).expect_err(name);
        assert!(error.starts_with(&format!("Row.{name}: ")), "{error}");
        assert!(error.contains(expected), "{name}: {error}");
    }
}

/// `#[serde(default)]` on a plain `Option<_>` field changes nothing: serde
/// already reads a missing key as `None`, and `None` still serializes as
/// null, so the member keeps the pointer shadow without `omitzero`.
#[test]
fn serde_default_on_a_plain_option_keeps_the_pointer_shadow() {
    let aliases = BTreeMap::new();
    let structs = Vec::new();
    let table = TypeTable::new(&structs, &aliases);
    let def = row(vec![field(
        "has_financials",
        "Option<bool>",
        FieldAttrs {
            default: true,
            ..FieldAttrs::default()
        },
    )]);
    let mapped = table.go_field(&def, &def.fields[0]).expect("maps");
    assert_eq!(
        (
            mapped.name.as_str(),
            mapped.wire.as_str(),
            mapped.public_ty.as_str(),
            mapped.shadow_ty.as_str(),
            mapped.codec,
            mapped.required_key(),
            mapped.omit_none,
        ),
        (
            "HasFinancials",
            "hasFinancials",
            "*bool",
            "*bool",
            Codec::Plain,
            false,
            false
        )
    );
}

#[test]
fn codec_fields_map_to_the_shadow_shapes_of_the_adr() {
    let aliases: BTreeMap<String, String> = [("Price", "f64"), ("DynamicJson", "Value")]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    let structs = vec![row(vec![])];
    let table = TypeTable::new(&structs, &aliases);
    let def = row(vec![
        field(
            "stock_return",
            "Option<Price>",
            deserialize_with("required_option"),
        ),
        field(
            "declaration_date",
            "Option<Date>",
            with("empty_or_null_date"),
        ),
        field("date_of_first_sale", "Option<Date>", with("empty_date")),
        field("data", "DynamicObject", FieldAttrs::default()),
        field("extra", "Option<DynamicObject>", FieldAttrs::default()),
        field("pay", "Option<DynamicJson>", FieldAttrs::default()),
        field("raw", "DynamicJson", FieldAttrs::default()),
        field("rows", "Vec<Row>", FieldAttrs::default()),
        field("filed", "UsDate", FieldAttrs::default()),
        field("flag", "Option<YnFlag>", FieldAttrs::default()),
        field("ratio", "PercentageValue", FieldAttrs::default()),
        field("prices", "Vec<Option<Price>>", FieldAttrs::default()),
        field(
            "shares",
            "Option<DynamicJson>",
            deserialize_with("required_option"),
        ),
        field("pct_of_open_interest", "Number", FieldAttrs::default()),
        field("price_target", "Option<Number>", FieldAttrs::default()),
        field(
            "stock_return",
            "Option<Number>",
            deserialize_with("required_option"),
        ),
    ]);
    let fields: Vec<_> = def
        .fields
        .iter()
        .map(|f| table.go_field(&def, f).expect("maps"))
        .collect();
    let shape = |index: usize| {
        let f = &fields[index];
        (
            f.name.as_str(),
            f.wire.as_str(),
            f.public_ty.as_str(),
            f.shadow_ty.as_str(),
            f.codec,
            f.required_key(),
        )
    };
    assert_eq!(
        shape(0),
        (
            "StockReturn",
            "stockReturn",
            "*float64",
            "jsontext.Value",
            Codec::RequiredOption,
            true
        )
    );
    assert_eq!(
        shape(1),
        (
            "DeclarationDate",
            "declarationDate",
            "*Date",
            "jsontext.Value",
            Codec::EmptyOrNullDate,
            true
        )
    );
    assert_eq!(
        shape(2),
        (
            "DateOfFirstSale",
            "dateOfFirstSale",
            "*Date",
            "jsontext.Value",
            Codec::EmptyDate,
            true
        )
    );
    assert_eq!(
        shape(3),
        (
            "Data",
            "data",
            "jsontext.Value",
            "*jsontext.Value",
            Codec::DynamicObject,
            true
        )
    );
    assert_eq!(
        shape(4),
        (
            "Extra",
            "extra",
            "*jsontext.Value",
            "*jsontext.Value",
            Codec::DynamicObject,
            true
        )
    );
    assert_eq!(
        shape(5),
        (
            "Pay",
            "pay",
            "*jsontext.Value",
            "*jsontext.Value",
            Codec::Plain,
            false
        )
    );
    assert_eq!(
        shape(6),
        (
            "Raw",
            "raw",
            "jsontext.Value",
            "jsontext.Value",
            Codec::DynamicJson,
            true
        )
    );
    assert_eq!(
        shape(7),
        ("Rows", "rows", "[]Row", "*[]Row", Codec::Plain, true)
    );
    assert_eq!(
        shape(8),
        ("Filed", "filed", "USDate", "*USDate", Codec::Plain, true)
    );
    assert_eq!(
        shape(9),
        ("Flag", "flag", "*string", "*string", Codec::Plain, false)
    );
    assert_eq!(
        shape(10),
        (
            "Ratio",
            "ratio",
            "NumberOrString",
            "*NumberOrString",
            Codec::Plain,
            true
        )
    );
    assert_eq!(
        shape(11),
        (
            "Prices",
            "prices",
            "[]*float64",
            "*[]*float64",
            Codec::Plain,
            true
        )
    );
    assert_eq!(
        shape(12),
        (
            "Shares",
            "shares",
            "*jsontext.Value",
            "jsontext.Value",
            Codec::RequiredOption,
            true
        )
    );
    assert_eq!(
        shape(13),
        (
            "PctOfOpenInterest",
            "pctOfOpenInterest",
            "jsontext.Value",
            "*jsontext.Value",
            Codec::Number,
            true
        )
    );
    assert_eq!(
        shape(14),
        (
            "PriceTarget",
            "priceTarget",
            "*jsontext.Value",
            "*jsontext.Value",
            Codec::Number,
            false
        )
    );
    assert_eq!(
        shape(15),
        (
            "StockReturn",
            "stockReturn",
            "*jsontext.Value",
            "jsontext.Value",
            Codec::RequiredNumber,
            true
        )
    );
}

#[test]
fn count_codec_decodes_through_the_raw_shadow_and_keeps_the_public_uint64() {
    let aliases: BTreeMap<String, String> = [("Count".to_string(), "u64".to_string())].into();
    let structs = Vec::new();
    let table = TypeTable::new(&structs, &aliases);
    let codec = "crate::codecs::count::deserialize";
    let def = row(vec![
        field("strong_buy", "Count", deserialize_with(codec)),
        field("hold", "Count", FieldAttrs::default()),
    ]);
    let mapped = table.go_field(&def, &def.fields[0]).expect("maps");
    assert_eq!(
        (
            mapped.public_ty.as_str(),
            mapped.shadow_ty.as_str(),
            mapped.codec,
            mapped.required_key(),
        ),
        ("uint64", "jsontext.Value", Codec::Count, true)
    );
    let models = plan_models("test", &[&def], &table).expect("plans");
    let rendered = render_models("test", &models);
    for expected in [
        "StrongBuy uint64 `json:\"strongBuy\"`",
        "StrongBuy jsontext.Value `json:\"strongBuy\"`",
        "case len(shadow.StrongBuy) == 0:",
        "strongBuy, err := decodeCount(\"Row\", \"strongBuy\", shadow.StrongBuy)",
        "StrongBuy: strongBuy,",
        "Hold *uint64 `json:\"hold\"`",
        "Hold: *shadow.Hold,",
    ] {
        assert!(
            rendered.contains(expected),
            "missing {expected:?} in\n{rendered}"
        );
    }

    for (name, ty) in [
        ("maybe", "Option<Count>"),
        ("price", "f64"),
        ("many", "Vec<Count>"),
    ] {
        let def = row(vec![field(name, ty, deserialize_with(codec))]);
        let error = table.go_field(&def, &def.fields[0]).expect_err(name);
        assert!(error.contains("bare Count"), "{name}: {error}");
    }
}

#[test]
fn skip_serializing_if_none_marks_only_the_public_tag_omitzero() {
    let aliases = BTreeMap::new();
    let structs = Vec::new();
    let table = TypeTable::new(&structs, &aliases);
    let def = row(vec![
        field("symbol", "String", FieldAttrs::default()),
        field(
            "capital_gains",
            "Option<TitleCaseBoolFlag>",
            skip_serializing_if("Option::is_none"),
        ),
        field("comment", "Option<String>", FieldAttrs::default()),
    ]);
    let fields: Vec<_> = def
        .fields
        .iter()
        .map(|f| table.go_field(&def, f).expect("maps"))
        .collect();
    assert_eq!(
        fields.iter().map(|f| f.omit_none).collect::<Vec<_>>(),
        [false, true, false]
    );
    assert_eq!(fields[1].codec, Codec::Plain);
    assert!(!fields[1].required_key());

    let models = plan_models("test", &[&def], &table).expect("plans");
    let rendered = render_models("test", &models);
    assert!(
        rendered.contains("CapitalGains *string `json:\"capitalGains,omitzero\"`"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Comment *string `json:\"comment\"`"),
        "{rendered}"
    );
    assert_eq!(rendered.matches("omitzero").count(), 1, "{rendered}");
}

fn flatten() -> FieldAttrs {
    FieldAttrs {
        flatten: true,
        ..FieldAttrs::default()
    }
}

#[test]
fn embedded_dynamic_object_holds_the_remaining_members() {
    let aliases = BTreeMap::new();
    let structs = Vec::new();
    let table = TypeTable::new(&structs, &aliases);
    let mut def = row(vec![
        field("symbol", "Ticker", FieldAttrs::default()),
        field("sections", "DynamicObject", FieldAttrs::default()),
    ]);
    def.custom_deserialize = true;
    let fields: Vec<_> = def
        .fields
        .iter()
        .map(|f| table.go_field(&def, f).expect("maps"))
        .collect();
    assert_eq!(fields[0].codec, Codec::Plain);
    assert_eq!(fields[1].codec, Codec::Embedded);
    assert!(!fields[1].required_key());
    assert_eq!(fields[1].wire, "");
    assert_eq!(fields[1].public_ty, "jsontext.Value");

    let models = plan_models("test", &[&def], &table).expect("plans");
    let rendered = render_models("test", &models);
    assert!(
        rendered.contains("Sections jsontext.Value `json:\",embed\"`"),
        "{rendered}"
    );
    assert_eq!(
        rendered.matches("`json:\",embed\"`").count(),
        2,
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "if len(shadow.Sections) == 0 {\n\t\tshadow.Sections = jsontext.Value(\"{}\")"
        ),
        "{rendered}"
    );
    assert_eq!(
        rendered.matches("missingMemberError").count(),
        1,
        "{rendered}"
    );
    assert!(
        rendered.contains("Sections: shadow.Sections,"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Every member no named field claims is kept in\n// Sections."),
        "{rendered}"
    );
    assert!(
        !rendered.contains("Unknown members are ignored"),
        "{rendered}"
    );

    let flattened = row(vec![field("data", "DynamicObject", flatten())]);
    let mapped = table
        .go_field(&flattened, &flattened.fields[0])
        .expect("serde flatten on a map is the same shape");
    assert_eq!(mapped.codec, Codec::Embedded);
    let scalar = row(vec![field("data", "f64", flatten())]);
    let error = table
        .go_field(&scalar, &scalar.fields[0])
        .expect_err("flatten of a scalar has no Go shape");
    assert!(error.contains("only on a bare DynamicObject"), "{error}");
    let optional = row(vec![field("data", "Option<DynamicObject>", flatten())]);
    assert!(table.go_field(&optional, &optional.fields[0]).is_err());

    let mut two = row(vec![
        field("first", "DynamicObject", FieldAttrs::default()),
        field("second", "DynamicObject", FieldAttrs::default()),
    ]);
    two.custom_deserialize = true;
    let error = plan_models("test", &[&two], &table).expect_err("two rest members");
    assert!(
        error.starts_with("Row: ") && error.contains("found 2 DynamicObject"),
        "{error}"
    );
    let mut mixed = row(vec![
        field("count", "Number", FieldAttrs::default()),
        field("sections", "DynamicObject", FieldAttrs::default()),
    ]);
    mixed.custom_deserialize = true;
    let error = plan_models("test", &[&mixed], &table).expect_err("a raw codec beside the rest");
    assert!(error.contains("some other codec"), "{error}");
}

#[test]
fn unspellable_wire_names_bridge_through_the_embedded_fallback() {
    let aliases = BTreeMap::new();
    let structs = Vec::new();
    let table = TypeTable::new(&structs, &aliases);
    let def = row(vec![
        field("symbol", "String", FieldAttrs::default()),
        field("stock_price", "NumericString", rename("Stock Price")),
        field("last_updated_raw", "String", rename("lastUpdated\"")),
        field(
            "note",
            "Option<String>",
            FieldAttrs {
                rename: Some("a,b".to_string()),
                ..skip_serializing_if("Option::is_none")
            },
        ),
    ]);
    let fields: Vec<_> = def
        .fields
        .iter()
        .map(|f| table.go_field(&def, f).expect("maps"))
        .collect();
    assert_eq!(
        fields.iter().map(|f| f.raw_key).collect::<Vec<_>>(),
        [false, false, true, true]
    );
    assert!(fields[2].required_key() && !fields[3].required_key());

    let models = plan_models("test", &[&def], &table).expect("plans");
    let rendered = render_models("test", &models);
    for expected in [
        "StockPrice string `json:\"Stock Price\"`",
        "LastUpdatedRaw string `json:\"-\"`",
        "Note *string `json:\"-\"`",
        "RawMembers map[string]jsontext.Value `json:\",embed\"`",
        "lastUpdatedRawWire := rawMember(shadow.RawMembers, \"lastUpdated\\\"\")",
        "noteWire := rawMember(shadow.RawMembers, \"a,b\")",
        "case lastUpdatedRawWire == nil:\n\t\treturn missingMemberError(\"Row\", \"lastUpdated\\\"\")",
        "if err := json.Unmarshal(lastUpdatedRawWire, &lastUpdatedRaw); err != nil",
        "if noteWire != nil {\n\t\tvar value string",
        "type rowPlain Row",
        "func (m Row) MarshalJSONTo(enc *jsontext.Encoder) error",
        "rawMembers[\"lastUpdated\\\"\"] = lastUpdatedRawWire",
        "if m.Note != nil {\n\t\tnoteWire, err := json.Marshal(m.Note)",
        "rowPlain: rowPlain(m),",
    ] {
        assert!(
            rendered.contains(expected),
            "missing {expected:?} in\n{rendered}"
        );
    }
    assert!(!rendered.contains("case noteWire == nil"), "{rendered}");
    assert_eq!(rendered.matches("`json:\"-\"`").count(), 2, "{rendered}");

    let plain = row(vec![field(
        "stock_price",
        "NumericString",
        rename("Stock Price"),
    )]);
    let plain_models = plan_models("test", &[&plain], &table).expect("plans");
    let plain_rendered = render_models("test", &plain_models);
    assert!(!plain_rendered.contains("RawMembers") && !plain_rendered.contains("MarshalJSONTo"));

    let coded = row(vec![field(
        "when",
        "Option<Date>",
        FieldAttrs {
            rename: Some("when\"".to_string()),
            ..with("empty_date")
        },
    )]);
    let error = table
        .go_field(&coded, &coded.fields[0])
        .expect_err("codec attributes are not bridged");
    assert!(
        error.contains("Row.when")
            && error.contains("cannot be spelled")
            && error.contains("EmptyDate"),
        "{error}"
    );

    let collision = row(vec![
        field("raw_members", "String", FieldAttrs::default()),
        field("odd", "String", rename("odd\"")),
    ]);
    let error = plan_models("test", &[&collision], &table).expect_err("collides");
    assert!(error.contains("RawMembers"), "{error}");

    let mut two_fallbacks = row(vec![
        field("odd", "String", rename("odd\"")),
        field("rest", "DynamicObject", FieldAttrs::default()),
    ]);
    two_fallbacks.custom_deserialize = true;
    let error = plan_models("test", &[&two_fallbacks], &table).expect_err("two embedded fallbacks");
    assert!(
        error.contains("Row") && error.contains("Rest") && error.contains("one embedded fallback"),
        "{error}"
    );
}

#[test]
fn unsupported_arg_kinds_name_the_helper_to_add() {
    assert_eq!(
        arg_kind_go(ArgKind::Ticker).expect("ticker").helper,
        "tickerParam"
    );
    assert_eq!(
        arg_kind_go(ArgKind::TickerList).expect("list").go_type,
        "[]string"
    );
    assert_eq!(
        arg_kind_go(ArgKind::ExchangeCode).expect("code").helper,
        "stringParam"
    );
    let frequency = arg_kind_go(ArgKind::RetrievalFrequency).expect("wire enum");
    assert_eq!(
        (frequency.go_type, frequency.helper),
        ("RetrievalFrequency", "retrievalFrequencyParam")
    );
    let indicator = arg_kind_go(ArgKind::EconomicIndicator).expect("open enum");
    assert_eq!(
        (indicator.go_type, indicator.helper),
        ("EconomicIndicator", "economicIndicatorParam")
    );
    let error = arg_kind_go(ArgKind::OpenEconomicIndicator).expect_err("open kind is deferred");
    assert!(
        error.contains("`open_economic_indicator`") && error.contains("openEconomicIndicatorParam"),
        "{error}"
    );
    let quarter = arg_kind_go(ArgKind::Quarter).expect("wire enum");
    assert_eq!(
        (quarter.go_type, quarter.helper),
        ("Quarter", "quarterParam")
    );
    let timeframe = arg_kind_go(ArgKind::ChartTimeframe).expect("wire enum");
    assert_eq!(
        (timeframe.go_type, timeframe.helper),
        ("ChartTimeframe", "chartTimeframeParam")
    );
    let error = arg_kind_go(ArgKind::DateRange).expect_err("date_range is deferred");
    assert!(
        error.contains("`date_range`") && error.contains("dateRangeParam"),
        "{error}"
    );
    for (kind, go_type, helper) in [
        (ArgKind::FiscalPeriod, "FiscalPeriod", "fiscalPeriodParam"),
        (
            ArgKind::StatementPeriod,
            "StatementPeriod",
            "statementPeriodParam",
        ),
        (
            ArgKind::SegmentationStructure,
            "SegmentationStructure",
            "segmentationStructureParam",
        ),
    ] {
        let mapped = arg_kind_go(kind).expect("statement enum kinds are mapped");
        assert_eq!((mapped.go_type, mapped.helper), (go_type, helper), "{kind}");
    }
}

#[test]
fn cli_selects_named_generated_or_all_domains() {
    let args = |list: &[&str]| parse_args(list.iter().map(|s| s.to_string()));
    assert_eq!(
        args(&["--domain", "quote", "--domain", "forex"])
            .expect("named")
            .selection,
        Selection::Named(vec!["quote".to_string(), "forex".to_string()])
    );
    assert_eq!(args(&[]).expect("no args").selection, Selection::Generated);
    let all = args(&["--all", "--out-dir", "/tmp/x"]).expect("all");
    assert_eq!(all.selection, Selection::All);
    assert_eq!(all.out_dir.as_deref(), Some(std::path::Path::new("/tmp/x")));
    assert!(args(&["--all", "--domain", "quote"]).is_err());
    assert!(args(&["--bogus"]).is_err());
    assert!(args(&["--domain"]).is_err());
}
