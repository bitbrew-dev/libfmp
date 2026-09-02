//! TipRanks add-on endpoint and query contracts.
//!
//! A future Python binding reserves matching `FmpClient` methods with response
//! models under `fmp.tipranks`. This crate does not implement those bindings.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{
            AccessRequirement, ConditionalPlanRequirement, EndpointBounds, EndpointMetadata,
            PlanCondition,
        },
    },
    responses::tipranks::{TipRanksPointInTimeRating, TipRanksRatingSearchResult},
    types::{Date, Limit, Page, SearchTerm, Ticker, TipRanksExpertUid},
};

/// Optional filters and pagination for the TipRanks analyst ratings search.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TipRanksSearchQuery {
    expert_uid: Option<TipRanksExpertUid>,
    symbol: Option<Ticker>,
    from: Option<Date>,
    to: Option<Date>,
    limit: Option<Limit>,
    page: Option<Page>,
    nonadjusted: Option<bool>,
}

impl TipRanksSearchQuery {
    /// Creates a search without undocumented filters, pagination, or flag defaults.
    pub const fn new() -> Self {
        Self {
            expert_uid: None,
            symbol: None,
            from: None,
            to: None,
            limit: None,
            page: None,
            nonadjusted: None,
        }
    }

    /// Restricts results to one stable TipRanks analyst identifier.
    pub fn with_expert_uid(mut self, expert_uid: TipRanksExpertUid) -> Self {
        self.expert_uid = Some(expert_uid);
        self
    }

    /// Restricts results to one ticker.
    pub fn with_symbol(mut self, symbol: Ticker) -> Self {
        self.symbol = Some(symbol);
        self
    }

    /// Sets the optional independent recommendation start date.
    pub const fn with_from(mut self, from: Date) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the optional independent recommendation end date.
    pub const fn with_to(mut self, to: Date) -> Self {
        self.to = Some(to);
        self
    }

    /// Sets the optional provider result limit.
    ///
    /// The endpoint's documented inclusive maximum is exposed through its
    /// [`EndpointMetadata`] rather than rejected while building a request.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets native-currency price targets when true, preserving explicit false.
    pub const fn with_nonadjusted(mut self, nonadjusted: bool) -> Self {
        self.nonadjusted = Some(nonadjusted);
        self
    }

    /// Borrows the optional stable TipRanks analyst identifier.
    pub fn expert_uid(&self) -> Option<&TipRanksExpertUid> {
        self.expert_uid.as_ref()
    }

    /// Borrows the optional ticker restriction.
    pub fn symbol(&self) -> Option<&Ticker> {
        self.symbol.as_ref()
    }

    /// Returns the optional independent recommendation start date.
    pub const fn from(&self) -> Option<Date> {
        self.from
    }

    /// Returns the optional independent recommendation end date.
    pub const fn to(&self) -> Option<Date> {
        self.to
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional native-currency flag.
    pub const fn nonadjusted(&self) -> Option<bool> {
        self.nonadjusted
    }
}

impl QueryParameters for TipRanksSearchQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("expertUID", self.expert_uid.as_ref());
        encoder.optional("symbol", self.symbol.as_ref());
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
        encoder.optional("limit", self.limit);
        encoder.optional("page", self.page);
        encoder.optional("nonadjusted", self.nonadjusted);
    }
}

/// Required symbol and optional snapshot controls for point-in-time ratings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointInTimeRatingsBySymbolQuery {
    symbol: Ticker,
    date: Option<Date>,
    limit: Option<Limit>,
    page: Option<Page>,
    nonadjusted: Option<bool>,
}

impl PointInTimeRatingsBySymbolQuery {
    /// Creates a current snapshot request for one ticker without injected defaults.
    pub const fn new(symbol: Ticker) -> Self {
        Self {
            symbol,
            date: None,
            limit: None,
            page: None,
            nonadjusted: None,
        }
    }

    /// Sets the optional historical snapshot date.
    pub const fn with_date(mut self, date: Date) -> Self {
        self.date = Some(date);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets native-currency price targets when true, preserving explicit false.
    pub const fn with_nonadjusted(mut self, nonadjusted: bool) -> Self {
        self.nonadjusted = Some(nonadjusted);
        self
    }

    /// Borrows the required ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Returns the optional historical snapshot date.
    pub const fn date(&self) -> Option<Date> {
        self.date
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional native-currency flag.
    pub const fn nonadjusted(&self) -> Option<bool> {
        self.nonadjusted
    }
}

impl QueryParameters for PointInTimeRatingsBySymbolQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("date", self.date);
        encoder.optional("limit", self.limit);
        encoder.optional("page", self.page);
        encoder.optional("nonadjusted", self.nonadjusted);
    }
}

/// Optional analyst selectors and snapshot controls for point-in-time ratings.
///
/// The provider prose calls `expertUID` a supplied selector, while its parameter
/// table marks neither selector as required. This query therefore preserves the
/// documented wire surface without imposing a local selector requirement or XOR.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PointInTimeRatingsByAnalystQuery {
    expert_uid: Option<TipRanksExpertUid>,
    analyst_name: Option<SearchTerm>,
    date: Option<Date>,
    limit: Option<Limit>,
    page: Option<Page>,
    nonadjusted: Option<bool>,
}

impl PointInTimeRatingsByAnalystQuery {
    /// Creates a request with no selector, pagination, date, or flag defaults.
    pub const fn new() -> Self {
        Self {
            expert_uid: None,
            analyst_name: None,
            date: None,
            limit: None,
            page: None,
            nonadjusted: None,
        }
    }

    /// Selects a stable TipRanks analyst identifier.
    pub fn with_expert_uid(mut self, expert_uid: TipRanksExpertUid) -> Self {
        self.expert_uid = Some(expert_uid);
        self
    }

    /// Selects an analyst by the provider's open name text.
    pub fn with_analyst_name(mut self, analyst_name: SearchTerm) -> Self {
        self.analyst_name = Some(analyst_name);
        self
    }

    /// Sets the optional historical snapshot date.
    pub const fn with_date(mut self, date: Date) -> Self {
        self.date = Some(date);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets native-currency price targets when true, preserving explicit false.
    pub const fn with_nonadjusted(mut self, nonadjusted: bool) -> Self {
        self.nonadjusted = Some(nonadjusted);
        self
    }

    /// Borrows the optional stable TipRanks analyst identifier.
    pub fn expert_uid(&self) -> Option<&TipRanksExpertUid> {
        self.expert_uid.as_ref()
    }

    /// Borrows the optional analyst name selector.
    pub fn analyst_name(&self) -> Option<&SearchTerm> {
        self.analyst_name.as_ref()
    }

    /// Returns the optional historical snapshot date.
    pub const fn date(&self) -> Option<Date> {
        self.date
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional native-currency flag.
    pub const fn nonadjusted(&self) -> Option<bool> {
        self.nonadjusted
    }
}

impl QueryParameters for PointInTimeRatingsByAnalystQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("expertUID", self.expert_uid.as_ref());
        encoder.optional("analystName", self.analyst_name.as_ref());
        encoder.optional("date", self.date);
        encoder.optional("limit", self.limit);
        encoder.optional("page", self.page);
        encoder.optional("nonadjusted", self.nonadjusted);
    }
}

const TIPRANKS_SEARCH_METADATA: EndpointMetadata = EndpointMetadata::new()
    .with_access(AccessRequirement::NamedAddOn("TipRanks"))
    .with_conditional_plan(ConditionalPlanRequirement::new(
        "Enterprise",
        PlanCondition::HistoryOlderThanYears(3),
    ))
    .with_bounds(
        EndpointBounds::new()
            .with_limit(5_000)
            .with_response_rows(5_000),
    );

/// Describes `GET tipranks-search` without binding a transport.
pub fn tipranks_ratings_search(
    query: TipRanksSearchQuery,
) -> EndpointSpec<TipRanksSearchQuery, Vec<TipRanksRatingSearchResult>> {
    EndpointSpec::get("tipranks-search", "tipranks-search", query)
        .with_metadata(TIPRANKS_SEARCH_METADATA)
}

/// Describes `GET tipranks-pit-symbol` without binding a transport.
pub fn tipranks_point_in_time_ratings_by_symbol(
    query: PointInTimeRatingsBySymbolQuery,
) -> EndpointSpec<PointInTimeRatingsBySymbolQuery, Vec<TipRanksPointInTimeRating>> {
    EndpointSpec::get("tipranks-pit-symbol", "tipranks-pit-symbol", query)
        .with_metadata(TIPRANKS_SEARCH_METADATA)
}

/// Describes `GET tipranks-pit-analyst` without binding a transport.
pub fn tipranks_point_in_time_ratings_by_analyst(
    query: PointInTimeRatingsByAnalystQuery,
) -> EndpointSpec<PointInTimeRatingsByAnalystQuery, Vec<TipRanksPointInTimeRating>> {
    EndpointSpec::get("tipranks-pit-analyst", "tipranks-pit-analyst", query)
        .with_metadata(TIPRANKS_SEARCH_METADATA)
}

impl Client {
    /// Retrieves individual analyst ratings from the TipRanks add-on.
    pub async fn tipranks_ratings_search(
        &self,
        query: TipRanksSearchQuery,
    ) -> Result<Vec<TipRanksRatingSearchResult>> {
        self.execute(&tipranks_ratings_search(query)).await
    }

    /// Retrieves a ticker's analyst ratings as of an optional snapshot date.
    pub async fn tipranks_point_in_time_ratings_by_symbol(
        &self,
        query: PointInTimeRatingsBySymbolQuery,
    ) -> Result<Vec<TipRanksPointInTimeRating>> {
        self.execute(&tipranks_point_in_time_ratings_by_symbol(query))
            .await
    }

    /// Retrieves an analyst's active coverage as of an optional snapshot date.
    pub async fn tipranks_point_in_time_ratings_by_analyst(
        &self,
        query: PointInTimeRatingsByAnalystQuery,
    ) -> Result<Vec<TipRanksPointInTimeRating>> {
        self.execute(&tipranks_point_in_time_ratings_by_analyst(query))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        let mut visitor = |name: &str, value: &str| {
            pairs.push((name.to_owned(), value.to_owned()));
        };
        query.encode(&mut QueryEncoder::new(&mut visitor));
        pairs
    }

    #[test]
    fn query_omits_every_absent_value_and_preserves_exact_full_order() {
        assert!(encoded(&TipRanksSearchQuery::new()).is_empty());

        let from = Date::parse("2025-06-10").unwrap();
        let to = Date::parse("2026-06-10").unwrap();
        let query = TipRanksSearchQuery::new()
            .with_expert_uid(TipRanksExpertUid::new("expert / one").unwrap())
            .with_symbol(Ticker::new("AAPL").unwrap())
            .with_from(from)
            .with_to(to)
            .with_limit(Limit(5_000))
            .with_page(Page(0))
            .with_nonadjusted(false);

        assert_eq!(query.expert_uid().unwrap().as_str(), "expert / one");
        assert_eq!(query.symbol().unwrap().as_str(), "AAPL");
        assert_eq!(query.from(), Some(from));
        assert_eq!(query.to(), Some(to));
        assert_eq!(query.limit(), Some(Limit(5_000)));
        assert_eq!(query.page(), Some(Page(0)));
        assert_eq!(query.nonadjusted(), Some(false));
        assert_eq!(
            encoded(&query),
            [
                ("expertUID".into(), "expert / one".into()),
                ("symbol".into(), "AAPL".into()),
                ("from".into(), "2025-06-10".into()),
                ("to".into(), "2026-06-10".into()),
                ("limit".into(), "5000".into()),
                ("page".into(), "0".into()),
                ("nonadjusted".into(), "false".into()),
            ]
        );
    }

    #[test]
    fn dates_are_independent_and_flags_preserve_both_boolean_states() {
        let from = Date::parse("2025-06-10").unwrap();
        let to = Date::parse("2026-06-10").unwrap();

        assert_eq!(
            encoded(&TipRanksSearchQuery::new().with_from(from)),
            [("from".into(), "2025-06-10".into())]
        );
        assert_eq!(
            encoded(
                &TipRanksSearchQuery::new()
                    .with_to(to)
                    .with_limit(Limit(0))
                    .with_page(Page(0))
                    .with_nonadjusted(true)
            ),
            [
                ("to".into(), "2026-06-10".into()),
                ("limit".into(), "0".into()),
                ("page".into(), "0".into()),
                ("nonadjusted".into(), "true".into())
            ]
        );
    }

    #[test]
    fn point_in_time_symbol_query_preserves_required_first_and_optional_order() {
        let query = PointInTimeRatingsBySymbolQuery::new(Ticker::new("BRK.B").unwrap())
            .with_date(Date::parse("2026-06-10").unwrap())
            .with_limit(Limit(5_000))
            .with_page(Page(0))
            .with_nonadjusted(false);

        assert_eq!(query.symbol().as_str(), "BRK.B");
        assert_eq!(query.date(), Some(Date::parse("2026-06-10").unwrap()));
        assert_eq!(query.limit(), Some(Limit(5_000)));
        assert_eq!(query.page(), Some(Page(0)));
        assert_eq!(query.nonadjusted(), Some(false));
        assert_eq!(
            encoded(&query),
            [
                ("symbol".into(), "BRK.B".into()),
                ("date".into(), "2026-06-10".into()),
                ("limit".into(), "5000".into()),
                ("page".into(), "0".into()),
                ("nonadjusted".into(), "false".into()),
            ]
        );
        assert_eq!(
            encoded(&PointInTimeRatingsBySymbolQuery::new(
                Ticker::new("AAPL").unwrap()
            )),
            [("symbol".into(), "AAPL".into())]
        );
    }

    #[test]
    fn point_in_time_analyst_query_omits_defaults_and_allows_both_selectors() {
        assert!(encoded(&PointInTimeRatingsByAnalystQuery::new()).is_empty());
        assert!(encoded(&PointInTimeRatingsByAnalystQuery::default()).is_empty());

        let query = PointInTimeRatingsByAnalystQuery::new()
            .with_expert_uid(TipRanksExpertUid::new("0001").unwrap())
            .with_analyst_name(SearchTerm::new("Analyst / Name").unwrap())
            .with_date(Date::parse("2026-06-10").unwrap())
            .with_limit(Limit(5_000))
            .with_page(Page(0))
            .with_nonadjusted(false);

        assert_eq!(query.expert_uid().unwrap().as_str(), "0001");
        assert_eq!(query.analyst_name().unwrap().as_str(), "Analyst / Name");
        assert_eq!(query.date(), Some(Date::parse("2026-06-10").unwrap()));
        assert_eq!(query.limit(), Some(Limit(5_000)));
        assert_eq!(query.page(), Some(Page(0)));
        assert_eq!(query.nonadjusted(), Some(false));
        assert_eq!(
            encoded(&query),
            [
                ("expertUID".into(), "0001".into()),
                ("analystName".into(), "Analyst / Name".into()),
                ("date".into(), "2026-06-10".into()),
                ("limit".into(), "5000".into()),
                ("page".into(), "0".into()),
                ("nonadjusted".into(), "false".into()),
            ]
        );
    }
}
