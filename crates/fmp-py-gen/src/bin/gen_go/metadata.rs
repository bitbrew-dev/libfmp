//! Renders `sdk/go/metadata_table.go`: the advisory `EndpointMetadata` of
//! every generated method whose descriptor attaches one, keyed by the Go
//! call path (`Quote.Full`, `Statements.Growth.Income`), behind the
//! package-level `EndpointMetadataFor` lookup.
//!
//! The table is keyed per method, not per endpoint id: several methods share
//! one id through helper reuse (`quote` serves five namespaces) and carry
//! different metadata, so the id is not a key. A second map lists the call
//! paths serving each id, behind the hand-written `EndpointMetadataByID`
//! that resolves `Error.Endpoint`. The Go types the literals name live in
//! the hand-written `sdk/go/metadata.go`.

use std::collections::BTreeMap;
use std::fmt::Write;

use fmp_py_gen::registry::wire::{
    AccessRequirement, DelayScope, GeographicAvailability, PlanCondition,
    UserDeclarationRequirement, WireMetadata,
};

use crate::emit::{doc_comment, exported};
use crate::methods::DomainPlan;
use crate::render::file_prelude;

/// Renders the table over every generated domain, one entry per method with
/// metadata, sorted by key. A key rendered twice is an error, never a
/// silent overwrite.
pub(crate) fn render_metadata_table(domains: &[&DomainPlan]) -> Result<String, String> {
    let mut entries: BTreeMap<String, String> = BTreeMap::new();
    let mut by_id: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for domain in domains {
        for namespace in &domain.namespaces {
            for method in &namespace.methods {
                let mut key: Vec<String> = namespace.path.iter().map(|s| exported(s)).collect();
                key.push(method.name.clone());
                let key = key.join(".");
                by_id
                    .entry(method.endpoint_id.as_str())
                    .or_default()
                    .push(key.clone());
                let Some(metadata) = &method.metadata else {
                    continue;
                };
                if entries.insert(key.clone(), literal(metadata)).is_some() {
                    return Err(format!(
                        "{}: metadata key `{key}` is rendered twice",
                        domain.name
                    ));
                }
            }
        }
    }
    let mut out = file_prelude(
        "The advisory EndpointMetadata of every generated method whose Rust descriptor \
         attaches one, keyed by the Go call path, generated from the wire contract read \
         from crates/libfmp/src/endpoints (ADR 0030).",
    );
    out.push_str(&doc_comment(
        "EndpointMetadataFor returns the advisory metadata of one endpoint method, keyed \
         by its Go call path without the client: \"Quote.Full\", \
         \"Statements.Growth.Income\". It reports false, with the zero value, for a \
         method whose descriptor attaches no metadata and for an unknown key. The value \
         is a copy: mutating it does not change later lookups. The client never \
         validates a request against it.",
    ));
    out.push_str(
        "func EndpointMetadataFor(method string) (EndpointMetadata, bool) {\n\
         \tmetadata, ok := endpointMetadataTable[method]\n\
         \tif !ok {\n\t\treturn EndpointMetadata{}, false\n\t}\n\
         \treturn metadata.clone(), true\n}\n\n",
    );
    out.push_str(&doc_comment(&format!(
        "endpointMetadataTable holds the {} generated methods whose descriptor attaches \
         metadata. Every other generated method has none.",
        entries.len()
    )));
    out.push_str("var endpointMetadataTable = map[string]EndpointMetadata{\n");
    for (key, value) in &entries {
        let _ = writeln!(out, "\t{key:?}: {value},");
    }
    out.push_str("}\n\n");
    out.push_str(&doc_comment(&format!(
        "endpointMethodsByID lists, for each of the {} endpoint ids, the call paths of \
         the generated methods that send it, sorted. Several methods share one id \
         through helper reuse.",
        by_id.len()
    )));
    out.push_str("var endpointMethodsByID = map[string][]string{\n");
    for (id, methods) in &mut by_id {
        methods.sort();
        let quoted: Vec<String> = methods.iter().map(|method| format!("{method:?}")).collect();
        let _ = writeln!(out, "\t{id:?}: {{{}}},", quoted.join(", "));
    }
    out.push_str("}\n");
    Ok(out)
}

/// The Go composite literal of one `EndpointMetadata`, naming only the
/// members that differ from the zero value.
fn literal(metadata: &WireMetadata) -> String {
    let mut fields = Vec::new();
    match metadata.geography {
        GeographicAvailability::Worldwide => {
            fields.push("Geography: GeographyWorldwide".to_string())
        }
        GeographicAvailability::UsOnly => fields.push("Geography: GeographyUSOnly".to_string()),
        GeographicAvailability::Unspecified => {}
    }
    match &metadata.access {
        AccessRequirement::Standard => {
            fields.push("Access: AccessRequirement{Kind: AccessStandard}".to_string());
        }
        AccessRequirement::NamedAddOn(name) => fields.push(format!(
            "Access: AccessRequirement{{Kind: AccessNamedAddOn, AddOn: {name:?}}}"
        )),
        AccessRequirement::Unspecified => {}
    }
    if let Some(plan) = &metadata.conditional_plan {
        let condition = match plan.condition {
            PlanCondition::HistoryOlderThanYears(years) => {
                format!("PlanCondition{{Kind: PlanConditionHistoryOlderThanYears, Years: {years}}}")
            }
        };
        fields.push(format!(
            "ConditionalPlan: &ConditionalPlanRequirement{{Plan: {:?}, Condition: {condition}}}",
            plan.plan
        ));
    }
    if let Some(realtime) = &metadata.realtime {
        let mut inner = Vec::new();
        if let Some(delay) = realtime.delay {
            let scope = match delay.scope {
                DelayScope::Nasdaq => "DelayScopeNasdaq",
            };
            inner.push(format!(
                "Delay: &MarketDataDelay{{Minutes: {}, Scope: {scope}}}",
                delay.minutes
            ));
        }
        if let Some(declaration) = realtime.user_declaration {
            let value = match declaration {
                UserDeclarationRequirement::RequiredForRealtime => {
                    "UserDeclarationRequiredForRealtime"
                }
            };
            inner.push(format!("UserDeclaration: {value}"));
        }
        fields.push(format!("Realtime: &RealtimeAccess{{{}}}", inner.join(", ")));
    }
    let bounds: Vec<String> = [
        ("Limit", metadata.bounds.limit),
        ("ResponseRows", metadata.bounds.response_rows),
        ("Page", metadata.bounds.page),
        ("DateRangeDays", metadata.bounds.date_range_days),
    ]
    .into_iter()
    .filter_map(|(name, maximum)| maximum.map(|value| format!("{name}: inclusiveMaximum({value})")))
    .collect();
    if !bounds.is_empty() {
        fields.push(format!("Bounds: EndpointBounds{{{}}}", bounds.join(", ")));
    }
    format!("{{{}}}", fields.join(", "))
}

#[cfg(test)]
mod tests {
    use fmp_py_gen::registry::wire::{
        ConditionalPlanRequirement, EndpointBounds, MarketDataDelay, RealtimeAccess,
    };

    use super::*;

    #[test]
    fn literal_names_only_the_members_set() {
        assert_eq!(literal(&WireMetadata::default()), "{}");
        let full = WireMetadata {
            geography: GeographicAvailability::Worldwide,
            access: AccessRequirement::NamedAddOn("TipRanks".to_string()),
            conditional_plan: Some(ConditionalPlanRequirement {
                plan: "Enterprise".to_string(),
                condition: PlanCondition::HistoryOlderThanYears(3),
            }),
            realtime: Some(RealtimeAccess {
                delay: Some(MarketDataDelay {
                    minutes: 15,
                    scope: DelayScope::Nasdaq,
                }),
                user_declaration: Some(UserDeclarationRequirement::RequiredForRealtime),
            }),
            bounds: EndpointBounds {
                limit: Some(5_000),
                response_rows: None,
                page: Some(100),
                date_range_days: None,
            },
        };
        assert_eq!(
            literal(&full),
            "{Geography: GeographyWorldwide, \
             Access: AccessRequirement{Kind: AccessNamedAddOn, AddOn: \"TipRanks\"}, \
             ConditionalPlan: &ConditionalPlanRequirement{Plan: \"Enterprise\", \
             Condition: PlanCondition{Kind: PlanConditionHistoryOlderThanYears, Years: 3}}, \
             Realtime: &RealtimeAccess{Delay: &MarketDataDelay{Minutes: 15, Scope: DelayScopeNasdaq}, \
             UserDeclaration: UserDeclarationRequiredForRealtime}, \
             Bounds: EndpointBounds{Limit: inclusiveMaximum(5000), Page: inclusiveMaximum(100)}}"
        );
        let bare_realtime = WireMetadata {
            geography: GeographicAvailability::UsOnly,
            access: AccessRequirement::Standard,
            realtime: Some(RealtimeAccess::default()),
            ..WireMetadata::default()
        };
        assert_eq!(
            literal(&bare_realtime),
            "{Geography: GeographyUSOnly, Access: AccessRequirement{Kind: AccessStandard}, \
             Realtime: &RealtimeAccess{}}"
        );
    }
}
