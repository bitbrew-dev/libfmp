//! String-backed identifiers: `str -> <newtype>` and `str | list[str] -> TickerList`.

use libfmp::types::{
    BenchmarkYear, BulkPart, Cik, CongressionalMemberId, CountryCode, CurrencyCode, Cusip,
    ExchangeCode, FormType, Industry, Isin, Lei, MarketHoursTimestamp, SearchTerm, Sector, Ticker,
    TickerList, TipRanksExpertUid, TransactionTypeCode,
};
use pyo3::prelude::*;
use pyo3_stub_gen::impl_stub_type;

use crate::errors::validation_error;

macro_rules! string_arg {
    ($($(#[doc = $doc:literal])* $function:ident => $target:ident),+ $(,)?) => {
        $(
            $(#[doc = $doc])*
            #[doc = ""]
            #[doc = "Rejects empty or whitespace-only values and control characters."]
            pub fn $function(name: &str, value: &str) -> PyResult<$target> {
                $target::new(value).map_err(|error| validation_error(name, error))
            }
        )+
    };
}

string_arg! {
    /// Converts a Python `str` into a [`Ticker`]. Commas are rejected because
    /// they delimit ticker lists on the wire.
    ticker => Ticker,
    /// Converts a Python `str` into a [`SearchTerm`].
    search_term => SearchTerm,
    /// Converts a Python `str` into a [`Cik`].
    cik => Cik,
    /// Converts a Python `str` into an [`ExchangeCode`].
    exchange_code => ExchangeCode,
    /// Converts a Python `str` into a [`CongressionalMemberId`].
    congressional_member_id => CongressionalMemberId,
    /// Converts a Python `str` into a [`TipRanksExpertUid`].
    tipranks_expert_uid => TipRanksExpertUid,
    /// Converts a Python `str` into a [`MarketHoursTimestamp`].
    market_hours_timestamp => MarketHoursTimestamp,
    /// Converts a Python `str` into a [`BenchmarkYear`].
    benchmark_year => BenchmarkYear,
    /// Converts a Python `str` into a [`TransactionTypeCode`].
    transaction_type_code => TransactionTypeCode,
    /// Converts a Python `str` into a [`Sector`].
    sector => Sector,
    /// Converts a Python `str` into an [`Industry`].
    industry => Industry,
    /// Converts a Python `str` into a [`CountryCode`].
    country_code => CountryCode,
    /// Converts a Python `str` into a [`CurrencyCode`].
    currency_code => CurrencyCode,
    /// Converts a Python `str` into a [`BulkPart`].
    bulk_part => BulkPart,
    /// Converts a Python `str` into a [`Cusip`].
    cusip => Cusip,
    /// Converts a Python `str` into an [`Isin`].
    isin => Isin,
    /// Converts a Python `str` into a [`Lei`].
    lei => Lei,
    /// Converts a Python `str` into a [`FormType`].
    form_type => FormType,
}

/// A `str | list[str]` symbols argument.
///
/// A bare `str` is a single ticker; it is not split on commas because
/// [`Ticker`] rejects commas outright. A `list[str]` must be non-empty.
#[derive(Debug, Clone, PartialEq, Eq, FromPyObject)]
pub enum SymbolsArg {
    #[pyo3(transparent, annotation = "str")]
    One(String),
    #[pyo3(transparent, annotation = "list[str]")]
    Many(Vec<String>),
}

impl_stub_type!(SymbolsArg = String | Vec<String>);

/// Converts a `str | list[str]` argument into a [`TickerList`].
///
/// Element errors name the offending index, for example `symbols[1]: ...`.
pub fn ticker_list(name: &str, value: SymbolsArg) -> PyResult<TickerList> {
    let tickers = match value {
        SymbolsArg::One(symbol) => vec![ticker(name, &symbol)?],
        SymbolsArg::Many(symbols) => symbols
            .iter()
            .enumerate()
            .map(|(index, symbol)| ticker(&format!("{name}[{index}]"), symbol))
            .collect::<PyResult<Vec<Ticker>>>()?,
    };
    TickerList::new(tickers).map_err(|error| validation_error(name, error))
}

#[cfg(test)]
mod tests {
    use pyo3::types::{PyList, PyString};

    use super::*;
    use crate::args::testing::{validation_message, with_py};

    #[test]
    fn ticker_accepts_plain_symbol() {
        crate::args::testing::init();
        let converted = ticker("symbol", "AAPL").expect("valid ticker");
        assert_eq!(converted.as_str(), "AAPL");
    }

    #[test]
    fn ticker_rejects_comma_with_argument_name() {
        crate::args::testing::init();
        let error = ticker("symbol", "AAPL,MSFT").expect_err("comma rejected");
        assert_eq!(
            validation_message(error),
            "symbol: ticker must not contain a comma"
        );
    }

    #[test]
    fn string_newtypes_reject_empty_values() {
        crate::args::testing::init();
        let error = cik("cik", "   ").expect_err("whitespace rejected");
        assert_eq!(
            validation_message(error),
            "cik: value must not be empty or whitespace-only"
        );
        let error = search_term("query", "a\tb").expect_err("control char rejected");
        assert_eq!(
            validation_message(error),
            "query: value must not contain control characters"
        );
    }

    #[test]
    fn string_newtypes_preserve_representation() {
        crate::args::testing::init();
        assert_eq!(
            cik("cik", "0000320193").expect("valid").as_str(),
            "0000320193"
        );
        assert_eq!(
            exchange_code("exchange", "nasdaq").expect("valid").as_str(),
            "nasdaq"
        );
        assert_eq!(
            form_type("form_type", "10-K").expect("valid").as_str(),
            "10-K"
        );
        assert_eq!(
            lei("lei", "5493001KJTIIGC8Y1R12").expect("valid").as_str(),
            "5493001KJTIIGC8Y1R12"
        );
    }

    #[test]
    fn ticker_list_accepts_one_symbol() {
        crate::args::testing::init();
        let converted = ticker_list("symbols", SymbolsArg::One("AAPL".into())).expect("valid");
        assert_eq!(converted.to_string(), "AAPL");
    }

    #[test]
    fn ticker_list_accepts_many_symbols() {
        crate::args::testing::init();
        let symbols = SymbolsArg::Many(vec!["AAPL".into(), "MSFT".into()]);
        let converted = ticker_list("symbols", symbols).expect("valid");
        assert_eq!(converted.to_string(), "AAPL,MSFT");
    }

    #[test]
    fn ticker_list_rejects_empty_list() {
        crate::args::testing::init();
        let error = ticker_list("symbols", SymbolsArg::Many(vec![])).expect_err("empty");
        assert_eq!(
            validation_message(error),
            "symbols: ticker list must contain at least one ticker"
        );
    }

    #[test]
    fn ticker_list_names_offending_index() {
        crate::args::testing::init();
        let symbols = SymbolsArg::Many(vec!["AAPL".into(), "".into()]);
        let error = ticker_list("symbols", symbols).expect_err("empty element");
        assert_eq!(
            validation_message(error),
            "symbols[1]: value must not be empty or whitespace-only"
        );
    }

    #[test]
    fn symbols_arg_extracts_str_and_list() {
        crate::args::testing::init();
        with_py(|py| {
            let one: SymbolsArg = PyString::new(py, "AAPL").extract().expect("str");
            assert_eq!(one, SymbolsArg::One("AAPL".into()));
            let list = PyList::new(py, ["AAPL", "MSFT"]).expect("list");
            let many: SymbolsArg = list.extract().expect("list[str]");
            assert_eq!(many, SymbolsArg::Many(vec!["AAPL".into(), "MSFT".into()]));
            let error = py
                .None()
                .into_bound(py)
                .extract::<SymbolsArg>()
                .expect_err("None is neither");
            assert!(error.is_instance_of::<pyo3::exceptions::PyTypeError>(py));
        });
    }
}
