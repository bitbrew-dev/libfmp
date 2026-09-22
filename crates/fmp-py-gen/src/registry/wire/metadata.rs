//! The advisory `EndpointMetadata` a descriptor attaches with
//! `.with_metadata(..)`, folded from the `const` builder chains in
//! `crates/libfmp/src/endpoints/**` into plain data.
//!
//! The Rust side is a `const fn` builder: `EndpointMetadata::new()` followed
//! by `with_geography`, `with_access`, `with_conditional_plan`,
//! `with_realtime`, and `with_bounds`, usually bound to a file-level `const`
//! and sometimes derived from another const (`US_ONLY.with_bounds(..)`) or
//! imported from a sibling module with `use`. The evaluator folds exactly
//! those shapes; anything else is an error naming the offending expression,
//! never a silent default, so a refactor on the `libfmp` side cannot drop
//! metadata unnoticed.

/// Geographic coverage, mirroring `libfmp::endpoints::metadata::GeographicAvailability`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GeographicAvailability {
    Worldwide,
    UsOnly,
    #[default]
    Unspecified,
}

/// Plan access, mirroring `libfmp::endpoints::metadata::AccessRequirement`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AccessRequirement {
    Standard,
    NamedAddOn(String),
    #[default]
    Unspecified,
}

/// The condition under which an additional plan is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanCondition {
    HistoryOlderThanYears(u16),
}

/// A named plan required only under its condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalPlanRequirement {
    pub plan: String,
    pub condition: PlanCondition,
}

/// Whether a user declaration is required for a real-time feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserDeclarationRequirement {
    RequiredForRealtime,
}

/// The market scope a documented delay applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayScope {
    Nasdaq,
}

/// A documented market-data delay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketDataDelay {
    pub minutes: u16,
    pub scope: DelayScope,
}

/// Real-time access caveats, mirroring `RealtimeAccess`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RealtimeAccess {
    pub delay: Option<MarketDataDelay>,
    pub user_declaration: Option<UserDeclarationRequirement>,
}

/// Inclusive maxima documented for one endpoint, mirroring `EndpointBounds`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EndpointBounds {
    pub limit: Option<u32>,
    pub response_rows: Option<u32>,
    pub page: Option<u32>,
    pub date_range_days: Option<u32>,
}

/// The advisory metadata of one descriptor, mirroring `EndpointMetadata`.
/// `WireMetadata::default()` is what `EndpointMetadata::new()` produces.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WireMetadata {
    pub geography: GeographicAvailability,
    pub access: AccessRequirement,
    pub conditional_plan: Option<ConditionalPlanRequirement>,
    pub realtime: Option<RealtimeAccess>,
    pub bounds: EndpointBounds,
}
