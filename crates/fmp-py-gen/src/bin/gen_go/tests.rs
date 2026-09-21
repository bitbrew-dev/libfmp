use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use fmp_py_gen::registry::wire::wire_surface;
use fmp_py_gen::registry::{ArgKind, Registry};
use fmp_py_gen::responses::{FieldAttrs, FieldDef, StructDef, discover};

use super::{Selection, generate, parse_args};
use crate::types::{Codec, TypeTable, arg_kind_go};

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

fn generate_domains(names: &[&str]) -> Result<Vec<(String, String)>, String> {
    let registry = Registry::load(&manifest().join("registry")).expect("registry loads");
    let wire = wire_surface(&manifest().join("../libfmp/src/endpoints")).expect("endpoints parse");
    let discovery = discover(&manifest().join("../libfmp/src/responses")).expect("responses parse");
    let generated: BTreeSet<String> = names.iter().map(|name| name.to_string()).collect();
    generate(&registry, &wire, &discovery, &generated, &generated)
}

#[test]
fn quote_domain_regenerates_the_committed_files() {
    let files = generate_domains(&["quote"]).expect("quote generates");
    let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        ["quote_models.go", "quote.go", "queries.go", "namespaces.go"]
    );
    for (name, source) in &files {
        let committed = fs::read_to_string(manifest().join("../../sdk/go").join(name))
            .unwrap_or_else(|error| panic!("{name} is committed under sdk/go: {error}"));
        assert_eq!(
            normalized(source),
            normalized(&committed),
            "{name} differs from the committed file; run gen_go --domain quote"
        );
    }
    let (_, quote) = &files[1];
    assert_eq!(quote.matches("func (n *QuoteNamespace) ").count(), 16);
    assert!(quote.contains("shortOnlyParams"));
    let (_, queries) = &files[2];
    assert!(
        queries.contains("type QuoteQuery struct"),
        "shared queries live in queries.go"
    );
    assert!(!quote.contains("type QuoteQuery struct"));
    let (_, models) = &files[0];
    assert!(models.contains("OneDay float64 `json:\"1D\"`"));
    assert!(models.contains("MarketCap *uint64 `json:\"marketCap\"`"));
}

#[test]
fn cross_domain_models_require_their_owner_to_be_generated() {
    let error = generate_domains(&["forex"]).expect_err("forex references quote models");
    assert!(
        error.contains("generate the `quote` domain first"),
        "{error}"
    );
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
            "SecretUrl",
            FieldAttrs::default(),
            "unmapped Rust type `SecretUrl`",
        ),
        (
            "value",
            "Option<Number>",
            FieldAttrs::default(),
            "unmapped Rust type `Number`",
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
            "Option<scalar or DynamicJson>",
        ),
        (
            "object",
            "Option<DynamicObject>",
            deserialize_with("required_option"),
            "Option<scalar or DynamicJson>",
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
    ];
    for (name, ty, attrs, expected) in cases {
        let def = row(vec![field(name, ty, attrs)]);
        let error = table.go_field(&def, &def.fields[0]).expect_err(name);
        assert!(error.starts_with(&format!("Row.{name}: ")), "{error}");
        assert!(error.contains(expected), "{name}: {error}");
    }
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
        ("Filed", "filed", "UsDate", "*UsDate", Codec::Plain, true)
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
    let error = arg_kind_go(ArgKind::DateRange).expect_err("date_range is deferred");
    assert!(
        error.contains("`date_range`") && error.contains("dateRangeParam"),
        "{error}"
    );
    let error = arg_kind_go(ArgKind::FiscalPeriod).expect_err("enum kinds are deferred");
    assert!(
        error.contains("fiscalPeriodParam") && error.contains("FiscalPeriod"),
        "{error}"
    );
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
