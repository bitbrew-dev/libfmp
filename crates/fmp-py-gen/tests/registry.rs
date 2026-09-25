//! Registry parse and validation contract, driven by the fixtures under
//! `tests/fixtures/<case>/` against the real `libfmp` endpoint sources and
//! the committed `fmp-py` models.

use std::path::{Path, PathBuf};

use fmp_py_gen::registry::scan::{Origin, Surface};
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
    let quote = registry
        .domains
        .iter()
        .find(|domain| domain.name == "quote")
        .expect("the shipped registry has a quote domain");
    assert_eq!(quote.namespaces[0].path, ["quote"]);
    let names: Vec<&str> = quote.namespaces[0]
        .endpoints
        .iter()
        .map(|endpoint| endpoint.python_name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "full",
            "short",
            "mutual_funds",
            "aftermarket_trade",
            "aftermarket_quote",
            "stock_price_change",
            "batch",
            "batch_short",
            "batch_aftermarket_trade",
            "batch_aftermarket_quote",
            "exchange",
            "etfs",
            "commodities",
            "cryptocurrencies",
            "forex",
            "indexes",
        ]
    );
    let full = &quote.namespaces[0].endpoints[0];
    assert_eq!(full.args[0].kind, ArgKind::Ticker);
    assert!(full.args[0].required);

    let report = registry
        .validate(&endpoints_root(), &models_root())
        .expect("shipped registry validates");
    assert!(report.trusted.is_empty(), "{:?}", report.trusted);
    let quote_entries: Vec<_> = report
        .verified
        .iter()
        .filter(|entry| entry.entry.starts_with("quote."))
        .collect();
    let origins: Vec<Option<Origin>> = quote_entries
        .iter()
        .map(|entry| entry.query_origin.clone())
        .collect();
    let symbol = Some(Origin::Macro("symbol_query".to_owned()));
    let symbols = Some(Origin::Macro("symbols_query".to_owned()));
    let expected_origins: Vec<Option<Origin>> = [
        symbol.clone(),
        symbol.clone(),
        None,
        symbol.clone(),
        symbol.clone(),
        symbol,
        symbols.clone(),
        symbols.clone(),
        symbols.clone(),
        symbols,
        Some(Origin::Direct),
    ]
    .into_iter()
    .chain(std::iter::repeat_n(None, 5))
    .collect();
    assert_eq!(origins, expected_origins);
    let modules: Vec<Option<Vec<String>>> = quote_entries
        .iter()
        .map(|entry| entry.query_module.clone())
        .collect();
    let expected_modules: Vec<Option<Vec<String>>> = origins
        .iter()
        .map(|origin| origin.as_ref().map(|_| vec!["quote".to_owned()]))
        .collect();
    assert_eq!(modules, expected_modules);
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
    let ctor: Vec<&str> = statement.ctor_args().iter().map(|arg| arg.name()).collect();
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
fn impl_level_macro_setters_verify_as_direct() {
    let registry = Registry::load(&fixture("impl_macros")).expect("fixture loads");
    let companies = &registry.domains[0].namespaces[0].endpoints[0];
    assert!(companies.ctor_args().is_empty());
    assert_eq!(companies.setters.len(), 20);
    let booleans: Vec<&str> = companies
        .args
        .iter()
        .filter(|arg| arg.kind == ArgKind::Boolean)
        .map(|arg| arg.name.as_str())
        .collect();
    assert_eq!(
        booleans,
        [
            "is_etf",
            "is_fund",
            "is_actively_trading",
            "include_all_share_classes"
        ]
    );

    let report = registry
        .validate(&endpoints_root(), &models_root())
        .expect("impl-level macro setters verify");
    assert!(report.trusted.is_empty(), "{:?}", report.trusted);
    assert_eq!(report.verified.len(), 1);
    assert_eq!(report.verified[0].entry, "screener.companies");
    assert_eq!(report.verified[0].query_origin, Some(Origin::Direct));
    assert_eq!(
        report.verified[0].query_module,
        Some(vec!["screener".to_owned()])
    );

    let surface = Surface::scan(&endpoints_root()).expect("endpoints scan");
    let assumptions = &surface.queries["DcfAssumptions"];
    assert_eq!(assumptions.origin, Origin::Direct);
    assert_eq!(assumptions.setters.len(), 18);
    assert_eq!(assumptions.setters["with_beta"].base_type, "FiniteDecimal");
    assert_eq!(
        assumptions.setters["with_risk_free_rate"].base_type,
        "FiniteDecimal"
    );
}

#[test]
fn nested_builder_flattens_setters_and_verifies_against_the_builder_type() {
    let registry = Registry::load(&fixture("nested_builder")).expect("fixture loads");
    let custom = &registry.domains[0].namespaces[0].endpoints[0];
    let names: Vec<&str> = custom.args.iter().map(|arg| arg.name.as_str()).collect();
    assert_eq!(
        names,
        ["symbol", "revenue_growth_pct", "beta", "risk_free_rate"]
    );
    assert!(custom.args[1..].iter().all(|arg| !arg.required));
    assert_eq!(custom.nested.len(), 1);
    assert_eq!(custom.nested[0].type_name, "DcfAssumptions");
    assert_eq!(custom.nested[0].position, 1);
    assert_eq!(custom.nested[0].setters[2].method, "with_risk_free_rate");
    let ctor: Vec<&str> = custom.ctor_args().iter().map(|arg| arg.name()).collect();
    assert_eq!(ctor, ["symbol", "assumptions"]);
    assert_eq!(custom.ctor_args()[1].libfmp_type(), "DcfAssumptions");

    let report = registry
        .validate(&endpoints_root(), &models_root())
        .expect("nested builder fixture validates");
    assert!(report.trusted.is_empty(), "{:?}", report.trusted);
    assert_eq!(report.verified.len(), 1);
    assert_eq!(report.verified[0].query_origin, Some(Origin::Direct));
    assert_eq!(
        report.verified[0].nested_modules["DcfAssumptions"],
        vec!["dcf".to_owned()]
    );

    let errors = validate(&fixture("nested_builder_mismatch"));
    let file = "nested_builder_mismatch/dcf.toml";
    let custom = "dcf.custom_discounted_cash_flow";
    assert_names(
        &errors,
        file,
        custom,
        "setter arg `beta` has kind `limit` (a `Limit`) but `DcfAssumptions::with_beta` takes `FiniteDecimal`",
    );
    assert_names(
        &errors,
        file,
        custom,
        "`DcfAssumptions` has no single-parameter `pub fn with_betas` for arg `betas`",
    );
    let levered = "dcf.custom_levered_discounted_cash_flow";
    assert_names(
        &errors,
        file,
        levered,
        "ctor arg `assumptions` is the nested builder `DcfAssumptions` but `CustomDcfQuery::new` parameter `symbol` is `Ticker`",
    );
    assert_names(
        &errors,
        file,
        levered,
        "ctor arg `symbol` has kind `ticker` (a `Ticker`) but `CustomDcfQuery::new` parameter `assumptions` is `DcfAssumptions`",
    );
    assert_eq!(errors.len(), 4);
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
fn dynamic_response_verifies_only_against_dynamic_object_rows() {
    let registry = Registry::load(&fixture("dynamic_rows")).expect("fixture loads");
    let search = &registry.domains[0].namespaces[0].endpoints[0];
    assert!(search.dynamic);
    assert!(!search.binary);
    assert_eq!(search.response_model, None);
    let report = registry
        .validate(&endpoints_root(), &models_root())
        .expect("dynamic fixture validates");
    assert!(report.trusted.is_empty(), "{:?}", report.trusted);
    assert_eq!(report.verified.len(), 1);
    assert_eq!(
        report.verified[0].entry,
        "sec_filings.search_industry_classifications"
    );
    assert_eq!(report.verified[0].query_origin, Some(Origin::Direct));

    let errors = validate(&fixture("dynamic_mismatch"));
    assert_names(
        &errors,
        "dynamic_mismatch/sec_filings.toml",
        "sec_filings.search_industry_classifications",
        "returns `Vec<DynamicObject>`; set `response = \"dynamic\"`",
    );
    assert_names(
        &errors,
        "dynamic_mismatch/sec_filings.toml",
        "sec_filings.industry_classifications",
        "returns `Vec<SicClassification>`, not `Vec<DynamicObject>`; name a `response` model instead of `dynamic`",
    );
    assert_eq!(errors.len(), 2);
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
fn aliased_row_verifies_against_the_target_struct_model() {
    let registry = Registry::load(&fixture("aliased_row")).expect("fixture loads");
    let report = registry
        .validate(&endpoints_root(), &models_root())
        .expect("aliased fixture validates");
    assert!(report.trusted.is_empty(), "{:?}", report.trusted);
    assert_eq!(report.verified.len(), 1);
    assert_eq!(report.verified[0].entry, "funds.fund_disclosure_dates");

    let errors = validate(&fixture("aliased_row_mismatch"));
    assert_names(
        &errors,
        "aliased_row_mismatch/funds.toml",
        "funds.fund_disclosure_dates",
        "returns `Vec<FundDisclosureDate>` but the response model is `funds::FundDisclosure`",
    );
    assert_eq!(errors.len(), 1);
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
args = [
    { name = "symbol", kind = "ticker" },
    { name = "symbol", kind = "ticker" },
    { name = "bare" },
    { name = "inputs", required = false, nested = { type = "DcfAssumptions", setters = [{ arg = "symbol", kind = "finite_decimal" }] } },
]
setters = [{ arg = "missing" }, { arg = "symbol" }]

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
        "quote.full",
        "arg `bare` needs exactly one of `kind` or `nested`",
    );
    assert_names(
        &errors,
        "quote.toml",
        "quote.full",
        "setter arg `symbol` must be `required = false`",
    );
    assert_names(
        &errors,
        "quote.toml",
        "quote.full",
        "nested builder `inputs` is always passed to the constructor; drop `required = false`",
    );
    assert_names(
        &errors,
        "quote.toml",
        "other.sub",
        "namespace path must be `quote`",
    );
}

#[test]
fn query_setter_on_a_nested_builder_arg_fails_at_load() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("registry-tests/nested-setter");
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(
        dir.join("dcf.toml"),
        r#"
[[endpoint]]
name = "custom"
method = "custom_discounted_cash_flow"
query = "CustomDcfQuery"
response = "dcf::CustomDcfValuation"
doc = "A query setter naming a flattened builder arg."
args = [
    { name = "symbol", kind = "ticker" },
    { name = "assumptions", nested = { type = "DcfAssumptions", setters = [{ arg = "beta", kind = "finite_decimal" }] } },
]
setters = [{ arg = "beta" }]
"#,
    )
    .expect("write");
    let error = Registry::load(&dir).expect_err("nested setter clash");
    let RegistryError::Invalid(errors) = error else {
        panic!("expected RegistryError::Invalid, got {error}");
    };
    assert_names(
        &errors,
        "dcf.toml",
        "dcf.custom",
        "setter arg `beta` is already applied by a nested builder",
    );
}
