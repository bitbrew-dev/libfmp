//! Additive endpoint availability, access, latency, and bounds metadata.
//!
//! Metadata is advisory and intended for discovery and introspection. Building
//! or executing a request does not automatically validate it against the
//! documented bounds. Callers that want a preflight check can use the
//! [`EndpointBounds::accepts_limit`], [`EndpointBounds::accepts_response_rows`],
//! [`EndpointBounds::accepts_page`], and [`EndpointBounds::accepts_date_range`]
//! helpers.

use crate::types::{DateRange, Limit, Page};

/// Geographic coverage documented for an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum GeographicAvailability {
    Worldwide,
    UsOnly,
    #[default]
    Unspecified,
}

/// Plan access documented for an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum AccessRequirement {
    Standard,
    NamedAddOn(&'static str),
    #[default]
    Unspecified,
}

/// A condition under which an additional plan is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlanCondition {
    /// Requests for history older than this many years require the plan.
    HistoryOlderThanYears(u16),
}

/// A named plan required only when a documented condition applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConditionalPlanRequirement {
    plan: &'static str,
    condition: PlanCondition,
}

impl ConditionalPlanRequirement {
    pub const fn new(plan: &'static str, condition: PlanCondition) -> Self {
        Self { plan, condition }
    }

    pub const fn plan(self) -> &'static str {
        self.plan
    }

    pub const fn condition(self) -> PlanCondition {
        self.condition
    }
}

/// Whether a user declaration is required for a real-time data feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UserDeclarationRequirement {
    RequiredForRealtime,
}

/// Optional real-time access caveats documented for an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RealtimeAccess {
    delay: Option<MarketDataDelay>,
    user_declaration: Option<UserDeclarationRequirement>,
}

impl RealtimeAccess {
    pub const fn new(
        delay: Option<MarketDataDelay>,
        user_declaration: Option<UserDeclarationRequirement>,
    ) -> Self {
        Self {
            delay,
            user_declaration,
        }
    }

    pub const fn delay(self) -> Option<MarketDataDelay> {
        self.delay
    }

    pub const fn user_declaration(self) -> Option<UserDeclarationRequirement> {
        self.user_declaration
    }
}

/// The market scope to which a documented data delay applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DelayScope {
    Nasdaq,
}

/// A documented market-data delay and the scope to which it applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarketDataDelay {
    minutes: u16,
    scope: DelayScope,
}

impl MarketDataDelay {
    pub const fn new(minutes: u16, scope: DelayScope) -> Self {
        Self { minutes, scope }
    }

    pub const fn minutes(self) -> u16 {
        self.minutes
    }

    pub const fn scope(self) -> DelayScope {
        self.scope
    }
}

/// An endpoint-owned inclusive maximum.
///
/// This helper deliberately carries no implicit minimum: the documentation
/// includes zero-valued page examples and does not establish a universal
/// nonzero rule for limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InclusiveMaximum(u32);

impl InclusiveMaximum {
    pub const fn new(maximum: u32) -> Self {
        Self(maximum)
    }

    pub const fn maximum(self) -> u32 {
        self.0
    }

    pub const fn contains(self, value: u32) -> bool {
        value <= self.0
    }
}

/// Advisory bounds attached to one endpoint rather than imposed globally.
///
/// These values preserve provider documentation for introspection. Query
/// builders deliberately remain representation-preserving and do not reject
/// values outside these bounds. Use the `accepts_*` helpers when an application
/// chooses to validate a request before sending it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EndpointBounds {
    limit: Option<InclusiveMaximum>,
    response_rows: Option<InclusiveMaximum>,
    page: Option<InclusiveMaximum>,
    date_range_days: Option<InclusiveMaximum>,
}

impl EndpointBounds {
    pub const fn new() -> Self {
        Self {
            limit: None,
            response_rows: None,
            page: None,
            date_range_days: None,
        }
    }

    pub const fn with_limit(mut self, maximum: u32) -> Self {
        self.limit = Some(InclusiveMaximum::new(maximum));
        self
    }

    pub const fn with_response_rows(mut self, maximum: u32) -> Self {
        self.response_rows = Some(InclusiveMaximum::new(maximum));
        self
    }

    pub const fn with_page(mut self, maximum: u32) -> Self {
        self.page = Some(InclusiveMaximum::new(maximum));
        self
    }

    pub const fn with_date_range_days(mut self, maximum: u32) -> Self {
        self.date_range_days = Some(InclusiveMaximum::new(maximum));
        self
    }

    pub const fn limit(self) -> Option<InclusiveMaximum> {
        self.limit
    }

    pub const fn response_rows(self) -> Option<InclusiveMaximum> {
        self.response_rows
    }

    pub const fn page(self) -> Option<InclusiveMaximum> {
        self.page
    }

    pub const fn date_range_days(self) -> Option<InclusiveMaximum> {
        self.date_range_days
    }

    /// Reports whether `value` is within the documented limit maximum, if any.
    pub const fn accepts_limit(self, value: Limit) -> bool {
        match self.limit {
            Some(maximum) => maximum.contains(value.0),
            None => true,
        }
    }

    /// Reports whether `value` is within the documented response-row maximum.
    pub const fn accepts_response_rows(self, value: u32) -> bool {
        match self.response_rows {
            Some(maximum) => maximum.contains(value),
            None => true,
        }
    }

    /// Reports whether `value` is within the documented page maximum, if any.
    pub const fn accepts_page(self, value: Page) -> bool {
        match self.page {
            Some(maximum) => maximum.contains(value.0),
            None => true,
        }
    }

    /// Reports whether `value` is within the documented date-span maximum.
    pub fn accepts_date_range(self, value: &DateRange) -> bool {
        match self.date_range_days {
            Some(maximum) => maximum.contains(value.span_days() as u32),
            None => true,
        }
    }
}

/// Documentation metadata attached to a transport-independent endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EndpointMetadata {
    geography: GeographicAvailability,
    access: AccessRequirement,
    conditional_plan: Option<ConditionalPlanRequirement>,
    realtime: Option<RealtimeAccess>,
    bounds: EndpointBounds,
}

impl EndpointMetadata {
    pub const fn new() -> Self {
        Self {
            geography: GeographicAvailability::Unspecified,
            access: AccessRequirement::Unspecified,
            conditional_plan: None,
            realtime: None,
            bounds: EndpointBounds::new(),
        }
    }

    pub const fn with_geography(mut self, geography: GeographicAvailability) -> Self {
        self.geography = geography;
        self
    }

    pub const fn with_access(mut self, access: AccessRequirement) -> Self {
        self.access = access;
        self
    }

    /// Attaches a plan requirement that applies only under its stated condition.
    pub const fn with_conditional_plan(mut self, requirement: ConditionalPlanRequirement) -> Self {
        self.conditional_plan = Some(requirement);
        self
    }

    pub const fn with_realtime(mut self, realtime: RealtimeAccess) -> Self {
        self.realtime = Some(realtime);
        self
    }

    pub const fn with_bounds(mut self, bounds: EndpointBounds) -> Self {
        self.bounds = bounds;
        self
    }

    pub const fn geography(self) -> GeographicAvailability {
        self.geography
    }

    pub const fn access(self) -> AccessRequirement {
        self.access
    }

    pub const fn conditional_plan(self) -> Option<ConditionalPlanRequirement> {
        self.conditional_plan
    }

    pub const fn realtime(self) -> Option<RealtimeAccess> {
        self.realtime
    }

    pub const fn bounds(self) -> EndpointBounds {
        self.bounds
    }
}
