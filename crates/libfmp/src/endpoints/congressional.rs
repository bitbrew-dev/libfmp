//! Congressional financial-disclosure endpoint query contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata, GeographicAvailability},
    },
    responses::congressional::{
        CongressionalMemberPosition, CongressionalMemberProfile, CongressionalTrade,
    },
    types::{CongressionalMemberId, Limit, Page, SearchTerm, Ticker},
};

/// Optional pagination for a chamber's latest financial disclosures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LatestCongressionalDisclosuresQuery {
    page: Option<Page>,
    limit: Option<Limit>,
}

impl LatestCongressionalDisclosuresQuery {
    /// Creates a query without undocumented defaults.
    pub const fn new() -> Self {
        Self {
            page: None,
            limit: None,
        }
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl QueryParameters for LatestCongressionalDisclosuresQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

/// Required ticker and optional pagination for chamber trade activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CongressionalTradesQuery {
    symbol: Ticker,
    page: Option<Page>,
    limit: Option<Limit>,
}

impl CongressionalTradesQuery {
    /// Creates a ticker query without undocumented pagination defaults.
    pub const fn new(symbol: Ticker) -> Self {
        Self {
            symbol,
            page: None,
            limit: None,
        }
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Borrows the requested ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl From<Ticker> for CongressionalTradesQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for CongressionalTradesQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for CongressionalTradesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

/// Required member-name search text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CongressionalTradesByNameQuery {
    name: SearchTerm,
}

impl CongressionalTradesByNameQuery {
    /// Creates a member-name search.
    pub const fn new(name: SearchTerm) -> Self {
        Self { name }
    }

    /// Borrows the exact search text.
    pub const fn name(&self) -> &SearchTerm {
        &self.name
    }
}

impl From<SearchTerm> for CongressionalTradesByNameQuery {
    fn from(name: SearchTerm) -> Self {
        Self::new(name)
    }
}

impl From<&SearchTerm> for CongressionalTradesByNameQuery {
    fn from(name: &SearchTerm) -> Self {
        Self::new(name.clone())
    }
}

impl QueryParameters for CongressionalTradesByNameQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("name", &self.name);
    }
}

/// Independently optional pagination and member ID for a chamber's ID lookup.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CongressionalTradesByMemberIdQuery {
    page: Option<Page>,
    limit: Option<Limit>,
    member_id: Option<CongressionalMemberId>,
}

impl CongressionalTradesByMemberIdQuery {
    /// Creates a query without undocumented defaults.
    pub const fn new() -> Self {
        Self {
            page: None,
            limit: None,
            member_id: None,
        }
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the optional congressional member identifier.
    pub fn with_member_id(mut self, member_id: CongressionalMemberId) -> Self {
        self.member_id = Some(member_id);
        self
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }

    /// Borrows the optional congressional member identifier.
    pub const fn member_id(&self) -> Option<&CongressionalMemberId> {
        self.member_id.as_ref()
    }
}

impl QueryParameters for CongressionalTradesByMemberIdQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
        encoder.optional("senateID", self.member_id.as_ref());
    }
}

/// Independently optional filters and pagination for congressional profiles.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CongressionalProfilesQuery {
    active: Option<bool>,
    member_id: Option<CongressionalMemberId>,
    latest_party: Option<String>,
    latest_position: Option<String>,
    page: Option<Page>,
    limit: Option<Limit>,
}

impl CongressionalProfilesQuery {
    /// Creates a query without undocumented defaults.
    pub const fn new() -> Self {
        Self {
            active: None,
            member_id: None,
            latest_party: None,
            latest_position: None,
            page: None,
            limit: None,
        }
    }

    /// Sets the optional active-member filter, preserving explicit false.
    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Sets the optional congressional member identifier.
    pub fn with_member_id(mut self, member_id: CongressionalMemberId) -> Self {
        self.member_id = Some(member_id);
        self
    }

    /// Sets the optional provider party filter without narrowing its vocabulary.
    pub fn with_latest_party(mut self, latest_party: impl Into<String>) -> Self {
        self.latest_party = Some(latest_party.into());
        self
    }

    /// Sets the optional provider position filter without narrowing its vocabulary.
    pub fn with_latest_position(mut self, latest_position: impl Into<String>) -> Self {
        self.latest_position = Some(latest_position.into());
        self
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Returns the optional active-member filter.
    pub const fn active(&self) -> Option<bool> {
        self.active
    }

    /// Borrows the optional congressional member identifier.
    pub const fn member_id(&self) -> Option<&CongressionalMemberId> {
        self.member_id.as_ref()
    }

    /// Borrows the optional provider party filter.
    pub fn latest_party(&self) -> Option<&str> {
        self.latest_party.as_deref()
    }

    /// Borrows the optional provider position filter.
    pub fn latest_position(&self) -> Option<&str> {
        self.latest_position.as_deref()
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl QueryParameters for CongressionalProfilesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("active", self.active);
        encoder.optional("senateID", self.member_id.as_ref());
        encoder.optional("latestParty", self.latest_party.as_deref());
        encoder.optional("latestPosition", self.latest_position.as_deref());
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

/// Independently optional filters and pagination for congressional positions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CongressionalPositionsQuery {
    member_id: Option<CongressionalMemberId>,
    party: Option<String>,
    position: Option<String>,
    page: Option<Page>,
    limit: Option<Limit>,
}

impl CongressionalPositionsQuery {
    /// Creates a query without undocumented defaults.
    pub const fn new() -> Self {
        Self {
            member_id: None,
            party: None,
            position: None,
            page: None,
            limit: None,
        }
    }

    /// Sets the optional congressional member identifier.
    pub fn with_member_id(mut self, member_id: CongressionalMemberId) -> Self {
        self.member_id = Some(member_id);
        self
    }

    /// Sets the optional provider party filter without narrowing its vocabulary.
    pub fn with_party(mut self, party: impl Into<String>) -> Self {
        self.party = Some(party.into());
        self
    }

    /// Sets the optional provider position filter without narrowing its vocabulary.
    pub fn with_position(mut self, position: impl Into<String>) -> Self {
        self.position = Some(position.into());
        self
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Borrows the optional congressional member identifier.
    pub const fn member_id(&self) -> Option<&CongressionalMemberId> {
        self.member_id.as_ref()
    }

    /// Borrows the optional provider party filter.
    pub fn party(&self) -> Option<&str> {
        self.party.as_deref()
    }

    /// Borrows the optional provider position filter.
    pub fn position(&self) -> Option<&str> {
        self.position.as_deref()
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl QueryParameters for CongressionalPositionsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("senateID", self.member_id.as_ref());
        encoder.optional("party", self.party.as_deref());
        encoder.optional("position", self.position.as_deref());
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

const PAGINATED_US_ONLY: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::UsOnly)
    .with_bounds(EndpointBounds::new().with_response_rows(250).with_page(100));
const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);
const CONGRESSIONAL_PROFILES_METADATA: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::UsOnly)
    .with_bounds(EndpointBounds::new().with_response_rows(500).with_page(20));
const CONGRESSIONAL_POSITIONS_METADATA: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::UsOnly)
    .with_bounds(EndpointBounds::new().with_response_rows(300).with_page(50));

macro_rules! paginated_endpoint {
    ($function:ident, $path:literal, $query:ty) => {
        #[doc = concat!("Describes `GET ", $path, "` without binding a transport.")]
        pub fn $function(query: $query) -> EndpointSpec<$query, Vec<CongressionalTrade>> {
            EndpointSpec::get($path, $path, query).with_metadata(PAGINATED_US_ONLY)
        }
    };
}

macro_rules! name_endpoint {
    ($function:ident, $path:literal) => {
        #[doc = concat!("Describes `GET ", $path, "` without binding a transport.")]
        pub fn $function(
            query: CongressionalTradesByNameQuery,
        ) -> EndpointSpec<CongressionalTradesByNameQuery, Vec<CongressionalTrade>> {
            EndpointSpec::get($path, $path, query).with_metadata(US_ONLY)
        }
    };
}

paginated_endpoint!(
    latest_senate_disclosures,
    "senate-latest",
    LatestCongressionalDisclosuresQuery
);
paginated_endpoint!(
    latest_house_disclosures,
    "house-latest",
    LatestCongressionalDisclosuresQuery
);
paginated_endpoint!(senate_trades, "senate-trades", CongressionalTradesQuery);
name_endpoint!(senate_trades_by_name, "senate-trades-by-name");
paginated_endpoint!(
    senate_trades_by_member_id,
    "senate-trades-by-id",
    CongressionalTradesByMemberIdQuery
);
paginated_endpoint!(house_trades, "house-trades", CongressionalTradesQuery);
name_endpoint!(house_trades_by_name, "house-trades-by-name");
paginated_endpoint!(
    house_trades_by_member_id,
    "house-trades-by-id",
    CongressionalTradesByMemberIdQuery
);

/// Describes `GET senate-profile` without binding a transport.
pub fn congressional_profiles(
    query: CongressionalProfilesQuery,
) -> EndpointSpec<CongressionalProfilesQuery, Vec<CongressionalMemberProfile>> {
    EndpointSpec::get("senate-profile", "senate-profile", query)
        .with_metadata(CONGRESSIONAL_PROFILES_METADATA)
}

/// Describes `GET senate-positions` without binding a transport.
pub fn congressional_positions(
    query: CongressionalPositionsQuery,
) -> EndpointSpec<CongressionalPositionsQuery, Vec<CongressionalMemberPosition>> {
    EndpointSpec::get("senate-positions", "senate-positions", query)
        .with_metadata(CONGRESSIONAL_POSITIONS_METADATA)
}

impl Client {
    /// Retrieves the latest Senate financial disclosures.
    pub async fn latest_senate_disclosures(
        &self,
        query: LatestCongressionalDisclosuresQuery,
    ) -> Result<Vec<CongressionalTrade>> {
        self.execute(&latest_senate_disclosures(query)).await
    }

    /// Retrieves the latest House financial disclosures.
    pub async fn latest_house_disclosures(
        &self,
        query: LatestCongressionalDisclosuresQuery,
    ) -> Result<Vec<CongressionalTrade>> {
        self.execute(&latest_house_disclosures(query)).await
    }

    /// Retrieves Senate trades for one ticker.
    pub async fn senate_trades(
        &self,
        query: impl Into<CongressionalTradesQuery>,
    ) -> Result<Vec<CongressionalTrade>> {
        self.execute(&senate_trades(query.into())).await
    }

    /// Searches Senate trades by member name.
    pub async fn senate_trades_by_name(
        &self,
        query: impl Into<CongressionalTradesByNameQuery>,
    ) -> Result<Vec<CongressionalTrade>> {
        self.execute(&senate_trades_by_name(query.into())).await
    }

    /// Looks up Senate trades by an optional member ID and pagination.
    pub async fn senate_trades_by_member_id(
        &self,
        query: CongressionalTradesByMemberIdQuery,
    ) -> Result<Vec<CongressionalTrade>> {
        self.execute(&senate_trades_by_member_id(query)).await
    }

    /// Retrieves House trades for one ticker.
    pub async fn house_trades(
        &self,
        query: impl Into<CongressionalTradesQuery>,
    ) -> Result<Vec<CongressionalTrade>> {
        self.execute(&house_trades(query.into())).await
    }

    /// Searches House trades by member name.
    pub async fn house_trades_by_name(
        &self,
        query: impl Into<CongressionalTradesByNameQuery>,
    ) -> Result<Vec<CongressionalTrade>> {
        self.execute(&house_trades_by_name(query.into())).await
    }

    /// Looks up House trades by an optional member ID and pagination.
    pub async fn house_trades_by_member_id(
        &self,
        query: CongressionalTradesByMemberIdQuery,
    ) -> Result<Vec<CongressionalTrade>> {
        self.execute(&house_trades_by_member_id(query)).await
    }

    /// Retrieves congressional profiles using optional provider filters.
    pub async fn congressional_profiles(
        &self,
        query: CongressionalProfilesQuery,
    ) -> Result<Vec<CongressionalMemberProfile>> {
        self.execute(&congressional_profiles(query)).await
    }

    /// Retrieves congressional position history using optional provider filters.
    pub async fn congressional_positions(
        &self,
        query: CongressionalPositionsQuery,
    ) -> Result<Vec<CongressionalMemberPosition>> {
        self.execute(&congressional_positions(query)).await
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
    fn queries_encode_exact_documented_keys_and_order() {
        assert!(encoded(&LatestCongressionalDisclosuresQuery::new()).is_empty());
        assert_eq!(
            encoded(
                &LatestCongressionalDisclosuresQuery::new()
                    .with_page(Page(0))
                    .with_limit(Limit(u32::MAX))
            ),
            [
                ("page".into(), "0".into()),
                ("limit".into(), u32::MAX.to_string())
            ]
        );
        assert_eq!(
            encoded(
                &CongressionalTradesQuery::new(Ticker::new("BRK.B").unwrap())
                    .with_page(Page(7))
                    .with_limit(Limit(250))
            ),
            [
                ("symbol".into(), "BRK.B".into()),
                ("page".into(), "7".into()),
                ("limit".into(), "250".into()),
            ]
        );
        assert_eq!(
            encoded(&CongressionalTradesByNameQuery::new(
                SearchTerm::new("James A.").unwrap()
            )),
            [("name".into(), "James A.".into())]
        );
        assert_eq!(
            encoded(
                &CongressionalTradesByMemberIdQuery::new()
                    .with_page(Page(0))
                    .with_limit(Limit(100))
                    .with_member_id(CongressionalMemberId::new("P000197").unwrap())
            ),
            [
                ("page".into(), "0".into()),
                ("limit".into(), "100".into()),
                ("senateID".into(), "P000197".into()),
            ]
        );
        assert_eq!(
            encoded(
                &CongressionalProfilesQuery::new()
                    .with_active(false)
                    .with_member_id(CongressionalMemberId::new("P000197").unwrap())
                    .with_latest_party("Republican")
                    .with_latest_position("Representative")
                    .with_page(Page(0))
                    .with_limit(Limit(500))
            ),
            [
                ("active".into(), "false".into()),
                ("senateID".into(), "P000197".into()),
                ("latestParty".into(), "Republican".into()),
                ("latestPosition".into(), "Representative".into()),
                ("page".into(), "0".into()),
                ("limit".into(), "500".into()),
            ]
        );
        assert_eq!(
            encoded(
                &CongressionalPositionsQuery::new()
                    .with_member_id(CongressionalMemberId::new("P000197").unwrap())
                    .with_party("Republican")
                    .with_position("Representative")
                    .with_page(Page(0))
                    .with_limit(Limit(300))
            ),
            [
                ("senateID".into(), "P000197".into()),
                ("party".into(), "Republican".into()),
                ("position".into(), "Representative".into()),
                ("page".into(), "0".into()),
                ("limit".into(), "300".into()),
            ]
        );
        assert!(encoded(&CongressionalProfilesQuery::new()).is_empty());
        assert!(encoded(&CongressionalPositionsQuery::new()).is_empty());
    }

    #[test]
    fn profile_and_position_filters_are_independently_optional() {
        let member_id = || CongressionalMemberId::new("P000197").unwrap();
        assert_eq!(
            encoded(&CongressionalProfilesQuery::new().with_active(true)),
            [("active".into(), "true".into())]
        );
        assert_eq!(
            encoded(&CongressionalProfilesQuery::new().with_member_id(member_id())),
            [("senateID".into(), "P000197".into())]
        );
        assert_eq!(
            encoded(&CongressionalProfilesQuery::new().with_latest_party("Independent")),
            [("latestParty".into(), "Independent".into())]
        );
        assert_eq!(
            encoded(&CongressionalProfilesQuery::new().with_latest_position("Senator")),
            [("latestPosition".into(), "Senator".into())]
        );
        assert_eq!(
            encoded(&CongressionalProfilesQuery::new().with_page(Page(20))),
            [("page".into(), "20".into())]
        );
        assert_eq!(
            encoded(&CongressionalProfilesQuery::new().with_limit(Limit(500))),
            [("limit".into(), "500".into())]
        );

        assert_eq!(
            encoded(&CongressionalPositionsQuery::new().with_member_id(member_id())),
            [("senateID".into(), "P000197".into())]
        );
        assert_eq!(
            encoded(&CongressionalPositionsQuery::new().with_party("Independent")),
            [("party".into(), "Independent".into())]
        );
        assert_eq!(
            encoded(&CongressionalPositionsQuery::new().with_position("Senator")),
            [("position".into(), "Senator".into())]
        );
        assert_eq!(
            encoded(&CongressionalPositionsQuery::new().with_page(Page(50))),
            [("page".into(), "50".into())]
        );
        assert_eq!(
            encoded(&CongressionalPositionsQuery::new().with_limit(Limit(300))),
            [("limit".into(), "300".into())]
        );
    }
}
