//! The argument-kind vocabulary: one variant per conversion function in
//! `crates/fmp-py/src/args/`.
//!
//! A registry entry names the kind of every Python parameter, and the
//! namespace emitter maps a kind to the `args::<name>` conversion, to the
//! Rust parameter type the `#[pymethods]` function takes, to the Python stub
//! type, and to the `libfmp` type the query constructor or setter expects.
//! Keeping the four projections in one table is what lets validation prove a
//! registry entry matches the real query signature.

use std::fmt;
use std::str::FromStr;

macro_rules! arg_kinds {
    ($($variant:ident => ($name:literal, $rust:literal, $input:literal, $python:literal)),+ $(,)?) => {
        /// One conversion in `fmp_py::args`, addressed by the registry `kind` field.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum ArgKind {
            $($variant,)+
        }

        impl ArgKind {
            /// Every kind, in declaration order.
            pub const ALL: &'static [ArgKind] = &[$(ArgKind::$variant,)+];

            /// The `fmp_py::args` function name, which is also the registry spelling.
            pub const fn name(self) -> &'static str {
                match self { $(ArgKind::$variant => $name,)+ }
            }

            /// The `libfmp` type the conversion produces, as the query constructor
            /// or setter parameter names it.
            pub const fn libfmp_type(self) -> &'static str {
                match self { $(ArgKind::$variant => $rust,)+ }
            }

            /// The Rust parameter type the generated `#[pymethods]` function takes.
            pub const fn input_type(self) -> &'static str {
                match self { $(ArgKind::$variant => $input,)+ }
            }

            /// The Python annotation the stub declares for the parameter.
            pub const fn python_type(self) -> &'static str {
                match self { $(ArgKind::$variant => $python,)+ }
            }
        }
    };
}

arg_kinds! {
    Ticker => ("ticker", "Ticker", "&str", "str"),
    TickerList => ("ticker_list", "TickerList", "SymbolsArg", "str | list[str]"),
    SearchTerm => ("search_term", "SearchTerm", "&str", "str"),
    Cik => ("cik", "Cik", "&str", "str"),
    ExchangeCode => ("exchange_code", "ExchangeCode", "&str", "str"),
    CongressionalMemberId => ("congressional_member_id", "CongressionalMemberId", "&str", "str"),
    TipranksExpertUid => ("tipranks_expert_uid", "TipRanksExpertUid", "&str", "str"),
    MarketHoursTimestamp => ("market_hours_timestamp", "MarketHoursTimestamp", "&str", "str"),
    BenchmarkYear => ("benchmark_year", "BenchmarkYear", "&str", "str"),
    TransactionTypeCode => ("transaction_type_code", "TransactionTypeCode", "&str", "str"),
    Sector => ("sector", "Sector", "&str", "str"),
    Industry => ("industry", "Industry", "&str", "str"),
    CountryCode => ("country_code", "CountryCode", "&str", "str"),
    CurrencyCode => ("currency_code", "CurrencyCode", "&str", "str"),
    BulkPart => ("bulk_part", "BulkPart", "&str", "str"),
    Cusip => ("cusip", "Cusip", "&str", "str"),
    Isin => ("isin", "Isin", "&str", "str"),
    Lei => ("lei", "Lei", "&str", "str"),
    FormType => ("form_type", "FormType", "&str", "str"),
    Limit => ("limit", "Limit", "i64", "int"),
    Page => ("page", "Page", "i64", "int"),
    Year => ("year", "Year", "i64", "int"),
    CalendarYear => ("calendar_year", "CalendarYear", "i64", "int"),
    Quarter => ("quarter", "Quarter", "i64", "int"),
    CalendarQuarter => ("calendar_quarter", "CalendarQuarter", "i64", "int"),
    PeriodLength => ("period_length", "PeriodLength", "i64", "int"),
    MarketCapitalization => ("market_capitalization", "MarketCapitalization", "i64", "int"),
    Volume => ("volume", "Volume", "i64", "int"),
    FiniteDecimal => ("finite_decimal", "FiniteDecimal", "f64", "float"),
    TrueFalseFlag => ("true_false_flag", "TrueFalseFlag", "bool", "bool"),
    Date => ("date", "Date", "DateArg", "datetime.date | str"),
    ApiDatetime => ("api_datetime", "ApiDateTime", "DateTimeArg", "datetime.datetime | str"),
    DateRange => ("date_range", "DateRange", "(DateArg, DateArg)", "tuple[datetime.date | str, datetime.date | str]"),
    FiscalPeriod => ("fiscal_period", "FiscalPeriod", "&str", "str"),
    RetrievalFrequency => ("retrieval_frequency", "RetrievalFrequency", "&str", "str"),
    StatementPeriod => ("statement_period", "StatementPeriod", "&str", "str"),
    ChartTimeframe => ("chart_timeframe", "ChartTimeframe", "&str", "str"),
    SegmentationStructure => ("segmentation_structure", "SegmentationStructure", "&str", "str"),
    EconomicIndicator => ("economic_indicator", "EconomicIndicator", "&str", "str"),
    OpenEconomicIndicator => ("open_economic_indicator", "OpenEconomicIndicator", "&str", "str"),
}

/// The error for a registry `kind` value that names no conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownKind(pub String);

impl fmt::Display for UnknownKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown arg kind `{}`; expected one of: {}",
            self.0,
            ArgKind::ALL
                .iter()
                .map(|kind| kind.name())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl std::error::Error for UnknownKind {}

impl FromStr for ArgKind {
    type Err = UnknownKind;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ArgKind::ALL
            .iter()
            .copied()
            .find(|kind| kind.name() == value)
            .ok_or_else(|| UnknownKind(value.to_owned()))
    }
}

impl fmt::Display for ArgKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use super::*;

    #[test]
    fn names_round_trip_through_from_str() {
        for kind in ArgKind::ALL {
            assert_eq!(kind.name().parse::<ArgKind>(), Ok(*kind));
        }
        let error = "tickers".parse::<ArgKind>().expect_err("unknown kind");
        assert!(error.to_string().starts_with("unknown arg kind `tickers`"));
    }

    #[test]
    fn names_are_unique() {
        let names: BTreeSet<&str> = ArgKind::ALL.iter().map(|kind| kind.name()).collect();
        assert_eq!(names.len(), ArgKind::ALL.len());
    }

    /// The vocabulary must track the `pub use` list of `fmp_py::args` exactly,
    /// so a conversion added in PY2-02's layer without a kind here fails.
    #[test]
    fn vocabulary_matches_fmp_py_args_exports() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fmp-py/src/args/mod.rs");
        let source = std::fs::read_to_string(&path).expect("fmp-py args/mod.rs is readable");
        let file = syn::parse_file(&source).expect("args/mod.rs parses");
        let mut exported = BTreeSet::new();
        for item in &file.items {
            if let syn::Item::Use(item) = item {
                collect_lowercase_leaves(&item.tree, &mut exported);
            }
        }
        let known: BTreeSet<String> = ArgKind::ALL
            .iter()
            .map(|kind| kind.name().to_owned())
            .collect();
        assert_eq!(exported, known, "args exports differ from the kind table");
    }

    fn collect_lowercase_leaves(tree: &syn::UseTree, out: &mut BTreeSet<String>) {
        match tree {
            syn::UseTree::Path(path) => collect_lowercase_leaves(&path.tree, out),
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    collect_lowercase_leaves(item, out);
                }
            }
            syn::UseTree::Name(name) => {
                let ident = name.ident.to_string();
                if ident.starts_with(|c: char| c.is_ascii_lowercase()) {
                    out.insert(ident);
                }
            }
            syn::UseTree::Rename(_) | syn::UseTree::Glob(_) => {}
        }
    }
}
