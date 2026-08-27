use std::str::FromStr;

use libfmp::{
    endpoints::{
        EndpointSpec,
        metadata::{
            AccessRequirement, ConditionalPlanRequirement, DelayScope, EndpointBounds,
            EndpointMetadata, GeographicAvailability, MarketDataDelay, PlanCondition,
            RealtimeAccess, UserDeclarationRequirement,
        },
    },
    responses::quote::QuoteShort,
    types::{Date, DateRange, Limit, Page},
};

#[test]
fn metadata_is_additive_and_bounds_remain_endpoint_owned() {
    let filings_bounds = EndpointBounds::new().with_response_rows(250).with_page(100);
    let government_bounds = EndpointBounds::new()
        .with_response_rows(500)
        .with_page(20)
        .with_date_range_days(90);

    assert!(filings_bounds.accepts_response_rows(250));
    assert!(!filings_bounds.accepts_response_rows(251));
    assert!(filings_bounds.accepts_page(Page(100)));
    assert!(!filings_bounds.accepts_page(Page(101)));
    assert!(government_bounds.accepts_response_rows(500));
    assert!(!government_bounds.accepts_response_rows(501));
    assert!(government_bounds.accepts_page(Page(20)));
    assert!(!government_bounds.accepts_page(Page(21)));
    assert!(EndpointBounds::new().accepts_limit(Limit(0)));
    assert!(EndpointBounds::new().accepts_limit(Limit(u32::MAX)));

    let range_90_days = DateRange::new(
        Date::from_str("2026-04-27").unwrap(),
        Date::from_str("2026-07-26").unwrap(),
    )
    .unwrap();
    let range_91_days = DateRange::new(
        Date::from_str("2026-04-27").unwrap(),
        Date::from_str("2026-07-27").unwrap(),
    )
    .unwrap();
    assert!(government_bounds.accepts_date_range(&range_90_days));
    assert!(!government_bounds.accepts_date_range(&range_91_days));

    // Keep independent documentation facts independent. In particular, this
    // synthetic descriptor says only that its family requires the named add-on.
    let tipranks = EndpointMetadata::new()
        .with_access(AccessRequirement::NamedAddOn("TipRanks"))
        .with_conditional_plan(ConditionalPlanRequirement::new(
            "Enterprise",
            PlanCondition::HistoryOlderThanYears(3),
        ));
    let descriptor: EndpointSpec<(), Vec<QuoteShort>> =
        EndpointSpec::get("metadata-probe", "metadata-probe", ()).with_metadata(tipranks);
    assert_eq!(descriptor.metadata(), tipranks);
    let enterprise_history = descriptor.metadata().conditional_plan().unwrap();
    assert_eq!(enterprise_history.plan(), "Enterprise");
    assert_eq!(
        enterprise_history.condition(),
        PlanCondition::HistoryOlderThanYears(3)
    );
    assert_eq!(
        descriptor.metadata().access(),
        AccessRequirement::NamedAddOn("TipRanks")
    );

    let worldwide_standard = EndpointMetadata::new()
        .with_geography(GeographicAvailability::Worldwide)
        .with_access(AccessRequirement::Standard);
    assert_eq!(
        worldwide_standard.geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(worldwide_standard.access(), AccessRequirement::Standard);

    let us_only = EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);
    assert_eq!(us_only.geography(), GeographicAvailability::UsOnly);

    // The quote warnings scope the 15-minute delay to Nasdaq and separately
    // mention a user declaration for real-time access.
    let nasdaq_realtime = RealtimeAccess::new(
        Some(MarketDataDelay::new(15, DelayScope::Nasdaq)),
        Some(UserDeclarationRequirement::RequiredForRealtime),
    );
    assert_eq!(
        nasdaq_realtime.delay(),
        Some(MarketDataDelay::new(15, DelayScope::Nasdaq))
    );
    assert_eq!(
        EndpointMetadata::new().access(),
        AccessRequirement::Unspecified
    );
}
