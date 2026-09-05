//! Registry parse and validation contract, driven by the fixtures under
//! `tests/fixtures/<case>/` against the real `libfmp` endpoint sources and
//! the committed `fmp-py` models.

use std::path::{Path, PathBuf};

use fmp_py_gen::registry::scan::Origin;
use fmp_py_gen::registry::{ArgKind, Registry, RegistryError, ValidationError};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn endpoints_root() -> PathBuf {
    manifest().join("../libfmp/src/endpoints")
}

fn models_root() -> PathBuf {
    manifest().join("../fmp-py/src/models")
}

fn fixture(case: &str) -> PathBuf {
    manifest().join("tests/fixtures").join(case)
}

fn validate(dir: &Path) -> Vec<ValidationError> {
    let registry = Registry::load(dir).expect("fixture loads");
    registry
        .validate(&endpoints_root(), &models_root())
        .expect_err("fixture must fail validation")
}

fn assert_names(errors: &[ValidationError], file: &str, entry: &str, needle: &str) {
    let hit = errors.iter().find(|error| {
        error.file.ends_with(file) && error.entry == entry && error.message.contains(needle)
    });
    assert!(
        hit.is_some(),
        "expected an error for `{entry}` in {file} containing {needle:?}, got:\n{}",
        errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn shipped_quote_registry_is_fully_verified() {
    let registry = Registry::load(&manifest().join("registry")).expect("registry loads");
    let quote = &registry.domains[0];
    assert_eq!(quote.name, "quote");
    assert_eq!(quote.namespaces[0].path, ["quote"]);
    let names: Vec<&str> = quote.namespaces[0]
        .endpoints
        .iter()
        .map(|endpoint| endpoint.python_name.as_str())
        .collect();
    assert_eq!(names, ["full", "short", "mutual_funds"]);
    let full = &quote.namespaces[0].endpoints[0];
    assert_eq!(full.args[0].kind, ArgKind::Ticker);
    assert!(full.args[0].required);

    let report = registry
        .validate(&endpoints_root(), &models_root())
        .expect("shipped registry validates");
    assert!(report.trusted.is_empty(), "{:?}", report.trusted);
    let origins: Vec<Option<Origin>> = report
        .verified
        .iter()
        .map(|entry| entry.query_origin.clone())
        .collect();
    assert_eq!(
        origins,
        [
            Some(Origin::Macro("symbol_query".to_owned())),
            Some(Origin::Macro("symbol_query".to_owned())),
            None,
        ]
    );
    let modules: Vec<Option<Vec<String>>> = report
        .verified
        .iter()
        .map(|entry| entry.query_module.clone())
        .collect();
    assert_eq!(
        modules,
        [
            Some(vec!["quote".to_owned()]),
            Some(vec!["quote".to_owned()]),
            None
        ]
    );
}

#[test]
fn nested_namespace_with_setters_and_binary_verifies() {
    let registry = Registry::load(&fixture("nested")).expect("fixture loads");
    let statements = &registry.domains[0];
    let paths: Vec<String> = statements
        .namespaces
        .iter()
        .map(|namespace| namespace.dotted())
        .collect();
    assert_eq!(paths, ["statements.income", "statements.reports"]);
    let statement = &statements.namespaces[0].endpoints[0];
    let ctor: Vec<&str> = statement.ctor_args().map(|arg| arg.name.as_str()).collect();
    assert_eq!(ctor, ["symbol"]);
    assert_eq!(statement.setters[1].method, "with_period");
    let xlsx = &statements.namespaces[1].endpoints[0];
    assert!(xlsx.binary);
    assert_eq!(xlsx.response_model, None);

    let report = registry
        .validate(&endpoints_root(), &models_root())
        .expect("nested fixture validates");
    assert_eq!(report.verified.len(), 2);
    assert_eq!(
        report.verified[0].query_origin,
        Some(Origin::Macro("statement_query".to_owned()))
    );
    assert_eq!(
        report.verified[1].query_origin,
        Some(Origin::Macro("financial_report_query".to_owned()))
    );
    assert_eq!(
        report.verified[0].query_module,
        Some(vec!["statements".to_owned()])
    );
    assert_eq!(
        report.verified[1].query_module,
        Some(vec!["statements".to_owned(), "reports".to_owned()])
    );
}

#[test]
fn keyword_arg_names_get_python_safe_spellings() {
    let registry = Registry::load(&fixture("keyword_args")).expect("fixture loads");
    let endpoint = &registry.domains[0].namespaces[0].endpoints[0];
    let names: Vec<String> = endpoint.args.iter().map(|arg| arg.python_name()).collect();
    assert_eq!(names, ["symbol", "from_", "to"]);
    assert_eq!(endpoint.setters[0].method, "with_from");
    let report = registry
        .validate(&endpoints_root(), &models_root())
        .expect("keyword fixture validates");
    assert_eq!(report.verified.len(), 1);
    assert_eq!(
        report.verified[0].query_module,
        Some(vec!["chart".to_owned()])
    );
}

#[test]
fn unknown_method_is_a_hard_error() {
    let errors = validate(&fixture("unknown_method"));
    assert_names(
        &errors,
        "unknown_method/quote.toml",
        "quote.full",
        "no `pub async fn quote_full`",
    );
}

#[test]
fn wrong_query_type_is_a_hard_error() {
    let errors = validate(&fixture("wrong_query"));
    assert_names(
        &errors,
        "wrong_query/quote.toml",
        "quote.full",
        "query type is `QuoteShortQuery` but `Client::quote` takes `QuoteQuery`",
    );
}

#[test]
fn unknown_arg_kind_fails_at_load() {
    let error = Registry::load(&fixture("unknown_kind")).expect_err("unknown kind rejected");
    let RegistryError::Invalid(errors) = error else {
        panic!("expected RegistryError::Invalid, got {error}");
    };
    assert_names(
        &errors,
        "unknown_kind/quote.toml",
        "quote.full",
        "unknown arg kind `tickers`",
    );
}

#[test]
fn missing_model_is_a_hard_error() {
    let errors = validate(&fixture("missing_model"));
    assert_names(
        &errors,
        "missing_model/quote.toml",
        "quote.full",
        "model file `crypto.rs` has no `pub(crate) struct Quote`",
    );
}

#[test]
fn setter_drift_reports_every_problem_at_once() {
    let errors = validate(&fixture("bad_setter"));
    assert_names(
        &errors,
        "bad_setter/statements.toml",
        "statements.income.statement",
        "setter arg `limit` has kind `page` (a `Page`) but `IncomeStatementQuery::with_limit` takes `Limit`",
    );
    assert_names(
        &errors,
        "bad_setter/statements.toml",
        "statements.income.statement",
        "no single-parameter `pub fn with_periods`",
    );
    assert_eq!(errors.len(), 2);
}

#[test]
fn malformed_toml_names_the_file() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("registry-tests/malformed");
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(dir.join("quote.toml"), "[[endpoint]]\nname = \n").expect("write");
    let error = Registry::load(&dir).expect_err("parse failure");
    assert!(matches!(error, RegistryError::Parse { .. }), "{error}");
    assert!(error.to_string().contains("quote.toml"), "{error}");
}

#[test]
fn structural_problems_are_collected_per_entry() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("registry-tests/structural");
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(
        dir.join("quote.toml"),
        r#"
[[endpoint]]
name = "full"
method = "quote"
query = "QuoteQuery"
response = "quote::Quote"
binary = true
doc = "Binary and response together."
args = [{ name = "symbol", kind = "ticker" }, { name = "symbol", kind = "ticker" }]
setters = [{ arg = "missing" }]

[[namespace]]
path = "other.sub"
"#,
    )
    .expect("write");
    let error = Registry::load(&dir).expect_err("structural failure");
    let RegistryError::Invalid(errors) = error else {
        panic!("expected RegistryError::Invalid, got {error}");
    };
    assert_names(&errors, "quote.toml", "quote.full", "`binary = true`");
    assert_names(
        &errors,
        "quote.toml",
        "quote.full",
        "duplicate arg `symbol`",
    );
    assert_names(
        &errors,
        "quote.toml",
        "quote.full",
        "setter `missing` names no arg",
    );
    assert_names(
        &errors,
        "quote.toml",
        "other.sub",
        "namespace path must be `quote`",
    );
}
