use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use fmp_py_gen::registry::wire::wire_surface;
use fmp_py_gen::registry::{ArgKind, Registry};
use fmp_py_gen::responses::{FieldAttrs, FieldDef, StructDef, discover};

use super::{Selection, generate, generated_domains, parse_args};
use crate::models::{plan_models, render_models};
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
    assert_eq!(files.len(), committed_set.len() * 2 + 2);
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
    assert!(models.contains("MarketCap *uint64 `json:\"marketCap\"`"));
    assert!(file("namespaces.go").contains("Quote QuoteNamespace"));
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
