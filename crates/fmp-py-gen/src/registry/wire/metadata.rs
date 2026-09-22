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

use std::fmt;
use std::str::FromStr;

use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprCall, ExprMethodCall, Lit, Token};

use super::collect::{Collected, ConstDef, ModulePath, path_segments, string_literal, text};

/// Deepest chain of const-to-const derivation the evaluator follows.
const MAX_DEPTH: usize = 8;

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

/// Whether `expr`, written in `module`, is an `EndpointMetadata` value: a
/// builder chain rooted in `EndpointMetadata::new()` or in a const declared
/// with that type. Says nothing about whether [`evaluate`] will succeed.
pub(super) fn is_metadata(collected: &Collected, module: &ModulePath, expr: &Expr) -> bool {
    match chain(expr).0 {
        Expr::Path(path) => path.path.get_ident().is_some_and(|name| {
            collected
                .constant(module, &name.to_string())
                .is_some_and(|(_, def)| def.ty == "EndpointMetadata")
        }),
        Expr::Call(call) => is_constructor(call, "EndpointMetadata", "new"),
        _ => false,
    }
}

/// Folds `expr`, written in `module`, into its metadata.
pub(super) fn evaluate(
    collected: &Collected,
    module: &ModulePath,
    expr: &Expr,
) -> Result<WireMetadata, String> {
    Evaluator { collected }.metadata(module, expr, 0)
}

struct Evaluator<'a> {
    collected: &'a Collected,
}

impl Evaluator<'_> {
    fn metadata(
        &self,
        module: &ModulePath,
        expr: &Expr,
        depth: usize,
    ) -> Result<WireMetadata, String> {
        if depth > MAX_DEPTH {
            return Err(format!("metadata const chain deeper than {MAX_DEPTH}"));
        }
        let (root, calls) = chain(expr);
        let mut metadata = match root {
            Expr::Call(call) if is_constructor(call, "EndpointMetadata", "new") => {
                args_of::<0>(call, "`EndpointMetadata::new`")?;
                WireMetadata::default()
            }
            Expr::Path(_) => {
                let (home, def) = self.constant(module, root, "EndpointMetadata")?;
                self.metadata(home, &def.expr, depth + 1)?
            }
            other => {
                return Err(format!(
                    "not an `EndpointMetadata::new()` chain or a metadata const: {}",
                    text(other)
                ));
            }
        };
        for call in calls {
            let [arg] = args_of(call, &format!("`{}`", call.method))?;
            match call.method.to_string().as_str() {
                "with_geography" => {
                    metadata.geography =
                        variant(arg, "GeographicAvailability", |name| match name {
                            "Worldwide" => Some(GeographicAvailability::Worldwide),
                            "UsOnly" => Some(GeographicAvailability::UsOnly),
                            "Unspecified" => Some(GeographicAvailability::Unspecified),
                            _ => None,
                        })?;
                }
                "with_access" => metadata.access = access(arg)?,
                "with_conditional_plan" => {
                    metadata.conditional_plan = Some(conditional_plan(arg)?);
                }
                "with_realtime" => metadata.realtime = Some(self.realtime(module, arg, depth)?),
                "with_bounds" => metadata.bounds = bounds(arg)?,
                other => {
                    return Err(format!(
                        "unsupported `EndpointMetadata` builder `{other}`: {}",
                        text(call)
                    ));
                }
            }
        }
        Ok(metadata)
    }

    /// `RealtimeAccess::new(Option<MarketDataDelay>, Option<UserDeclarationRequirement>)`
    /// or a const holding one.
    fn realtime(
        &self,
        module: &ModulePath,
        expr: &Expr,
        depth: usize,
    ) -> Result<RealtimeAccess, String> {
        if depth > MAX_DEPTH {
            return Err(format!("realtime const chain deeper than {MAX_DEPTH}"));
        }
        match expr {
            Expr::Path(_) => {
                let (home, def) = self.constant(module, expr, "RealtimeAccess")?;
                self.realtime(home, &def.expr, depth + 1)
            }
            Expr::Call(call) if is_constructor(call, "RealtimeAccess", "new") => {
                let [delay, declaration] = args_of(call, "`RealtimeAccess::new`")?;
                Ok(RealtimeAccess {
                    delay: option(delay, market_data_delay)?,
                    user_declaration: option(declaration, |expr| {
                        variant(expr, "UserDeclarationRequirement", |name| match name {
                            "RequiredForRealtime" => {
                                Some(UserDeclarationRequirement::RequiredForRealtime)
                            }
                            _ => None,
                        })
                    })?,
                })
            }
            other => Err(format!(
                "realtime is not `RealtimeAccess::new(..)` or a const: {}",
                text(other)
            )),
        }
    }

    /// The const a bare path names, in `module` or through its `use`
    /// imports, checked against the type it was declared with.
    fn constant<'c>(
        &'c self,
        module: &ModulePath,
        expr: &Expr,
        expected: &str,
    ) -> Result<(&'c ModulePath, &'c ConstDef), String> {
        let Expr::Path(path) = expr else {
            return Err(format!("not a const path: {}", text(expr)));
        };
        let name = path
            .path
            .get_ident()
            .map(ToString::to_string)
            .ok_or_else(|| format!("not a bare const name: {}", text(expr)))?;
        let (home, def) = self.collected.constant(module, &name).ok_or_else(|| {
            format!(
                "`{name}` is not a const in module `{}` or its `use` imports \
                 (glob imports and re-exports are not followed)",
                module.join("::")
            )
        })?;
        if def.ty != expected {
            return Err(format!(
                "const `{name}` is declared as `{}`, not `{expected}`",
                def.ty
            ));
        }
        Ok((home, def))
    }
}

/// `AccessRequirement::Standard`, `::Unspecified`, or `::NamedAddOn("..")`.
fn access(expr: &Expr) -> Result<AccessRequirement, String> {
    match expr {
        Expr::Path(_) => variant(expr, "AccessRequirement", |name| match name {
            "Standard" => Some(AccessRequirement::Standard),
            "Unspecified" => Some(AccessRequirement::Unspecified),
            _ => None,
        }),
        Expr::Call(call) if is_constructor(call, "AccessRequirement", "NamedAddOn") => {
            let [arg] = args_of(call, "`AccessRequirement::NamedAddOn`")?;
            string_literal(arg)
                .map(AccessRequirement::NamedAddOn)
                .ok_or_else(|| format!("add-on name is not a string literal: {}", text(arg)))
        }
        other => Err(format!(
            "access is not an `AccessRequirement` variant: {}",
            text(other)
        )),
    }
}

/// `ConditionalPlanRequirement::new("plan", PlanCondition::HistoryOlderThanYears(n))`.
fn conditional_plan(expr: &Expr) -> Result<ConditionalPlanRequirement, String> {
    let call = constructor_call(
        expr,
        "ConditionalPlanRequirement",
        "new",
        "conditional plan",
    )?;
    let [plan, condition] = args_of(call, "`ConditionalPlanRequirement::new`")?;
    let plan = string_literal(plan)
        .ok_or_else(|| format!("plan name is not a string literal: {}", text(plan)))?;
    let years = match condition {
        Expr::Call(call) if is_constructor(call, "PlanCondition", "HistoryOlderThanYears") => {
            let [years] = args_of(call, "`PlanCondition::HistoryOlderThanYears`")?;
            years
        }
        other => {
            return Err(format!(
                "unsupported `PlanCondition` variant: {}",
                text(other)
            ));
        }
    };
    Ok(ConditionalPlanRequirement {
        plan,
        condition: PlanCondition::HistoryOlderThanYears(integer(years, "years")?),
    })
}

/// `MarketDataDelay::new(minutes, DelayScope::Nasdaq)`.
fn market_data_delay(expr: &Expr) -> Result<MarketDataDelay, String> {
    let call = constructor_call(expr, "MarketDataDelay", "new", "delay")?;
    let [minutes, scope] = args_of(call, "`MarketDataDelay::new`")?;
    Ok(MarketDataDelay {
        minutes: integer(minutes, "delay minutes")?,
        scope: variant(scope, "DelayScope", |name| match name {
            "Nasdaq" => Some(DelayScope::Nasdaq),
            _ => None,
        })?,
    })
}

/// `EndpointBounds::new()` followed by `with_limit`, `with_response_rows`,
/// `with_page`, and `with_date_range_days` with integer literals.
fn bounds(expr: &Expr) -> Result<EndpointBounds, String> {
    let (root, calls) = chain(expr);
    let mut bounds = match root {
        Expr::Call(call) if is_constructor(call, "EndpointBounds", "new") => {
            args_of::<0>(call, "`EndpointBounds::new`")?;
            EndpointBounds::default()
        }
        other => {
            return Err(format!(
                "bounds are not an `EndpointBounds::new()` chain: {}",
                text(other)
            ));
        }
    };
    for call in calls {
        let method = call.method.to_string();
        let slot = match method.as_str() {
            "with_limit" => &mut bounds.limit,
            "with_response_rows" => &mut bounds.response_rows,
            "with_page" => &mut bounds.page,
            "with_date_range_days" => &mut bounds.date_range_days,
            other => {
                return Err(format!(
                    "unsupported `EndpointBounds` builder `{other}`: {}",
                    text(call)
                ));
            }
        };
        let [value] = args_of(call, &format!("`{method}`"))?;
        *slot = Some(integer(value, &method)?);
    }
    Ok(bounds)
}

/// `None`, or `Some(inner)` folded with `parse`.
fn option<T>(expr: &Expr, parse: impl Fn(&Expr) -> Result<T, String>) -> Result<Option<T>, String> {
    match expr {
        Expr::Path(path) if path.path.is_ident("None") => Ok(None),
        Expr::Call(call) if call_segments(call).as_deref() == Some(&["Some".to_owned()][..]) => {
            let [inner] = args_of(call, "`Some`")?;
            parse(inner).map(Some)
        }
        other => Err(format!("not `Some(..)` or `None`: {}", text(other))),
    }
}

/// `Type::Variant` folded with `parse`, which rejects unknown variants.
fn variant<T>(expr: &Expr, ty: &str, parse: impl Fn(&str) -> Option<T>) -> Result<T, String> {
    let Expr::Path(path) = expr else {
        return Err(format!("not a `{ty}` variant: {}", text(expr)));
    };
    match path_segments(&path.path).as_slice() {
        [head, name] if head == ty => {
            parse(name).ok_or_else(|| format!("unsupported `{ty}::{name}`"))
        }
        _ => Err(format!("not a `{ty}` variant: {}", text(expr))),
    }
}

/// An integer literal such as `5_000`, parsed into `N`.
fn integer<N>(expr: &Expr, what: &str) -> Result<N, String>
where
    N: FromStr,
    N::Err: fmt::Display,
{
    match expr {
        Expr::Lit(literal) => match &literal.lit {
            Lit::Int(value) => value
                .base10_parse::<N>()
                .map_err(|error| format!("{what} `{}` is out of range: {error}", value.token())),
            other => Err(format!("{what} is not an integer literal: {}", text(other))),
        },
        other => Err(format!("{what} is not an integer literal: {}", text(other))),
    }
}

/// The root of a builder chain and its method calls in application order.
fn chain(expr: &Expr) -> (&Expr, Vec<&ExprMethodCall>) {
    let mut calls = Vec::new();
    let mut current = expr;
    loop {
        match current {
            Expr::MethodCall(call) => {
                calls.push(call);
                current = &call.receiver;
            }
            Expr::Paren(inner) => current = &inner.expr,
            _ => break,
        }
    }
    calls.reverse();
    (current, calls)
}

fn call_segments(call: &ExprCall) -> Option<Vec<String>> {
    let Expr::Path(func) = &*call.func else {
        return None;
    };
    Some(path_segments(&func.path))
}

fn is_constructor(call: &ExprCall, ty: &str, name: &str) -> bool {
    call_segments(call).is_some_and(|segments| segments == [ty, name])
}

/// `expr` as the call `Ty::name(..)`, or an error naming `what` it was
/// meant to be.
fn constructor_call<'e>(
    expr: &'e Expr,
    ty: &str,
    name: &str,
    what: &str,
) -> Result<&'e ExprCall, String> {
    match expr {
        Expr::Call(call) if is_constructor(call, ty, name) => Ok(call),
        other => Err(format!("{what} is not `{ty}::{name}(..)`: {}", text(other))),
    }
}

/// A call whose argument count the evaluator checks: a free call
/// (`Ty::new(a, b)`) or a builder method (`.with_x(a)`).
pub(super) trait CallArgs: ToTokens {
    fn args(&self) -> &Punctuated<Expr, Token![,]>;
}

impl CallArgs for ExprCall {
    fn args(&self) -> &Punctuated<Expr, Token![,]> {
        &self.args
    }
}

impl CallArgs for ExprMethodCall {
    fn args(&self) -> &Punctuated<Expr, Token![,]> {
        &self.args
    }
}

/// Exactly `N` arguments of `call`, or an error naming `what` was called.
pub(super) fn args_of<'e, const N: usize>(
    call: &'e impl CallArgs,
    what: &str,
) -> Result<[&'e Expr; N], String> {
    <[&Expr; N]>::try_from(call.args().iter().collect::<Vec<_>>()).map_err(|_| {
        let count = match N {
            0 => "no arguments".to_owned(),
            1 => "one argument".to_owned(),
            2 => "two arguments".to_owned(),
            n => format!("{n} arguments"),
        };
        format!("{what} takes {count}: {}", text(call))
    })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    /// Folds the const `name` of a snippet placed at `endpoints/m.rs`.
    fn folded(source: &str, name: &str) -> Result<WireMetadata, String> {
        let root = Path::new("/virtual/endpoints");
        let files: Vec<(PathBuf, syn::File)> = vec![(
            root.join("m.rs"),
            syn::parse_file(source).expect("snippet parses"),
        )];
        let collected = Collected::from_files(root, &files);
        let (home, def) = collected
            .constant(&vec!["m".to_owned()], name)
            .expect("const is collected");
        evaluate(&collected, home, &def.expr)
    }

    fn geography(source: &str) -> Result<GeographicAvailability, String> {
        folded(source, "M").map(|metadata| metadata.geography)
    }

    #[test]
    fn new_alone_is_the_default() {
        assert_eq!(
            folded("const M: EndpointMetadata = EndpointMetadata::new();", "M"),
            Ok(WireMetadata::default())
        );
    }

    #[test]
    fn with_geography_reads_every_variant() {
        for (variant, expected) in [
            ("Worldwide", GeographicAvailability::Worldwide),
            ("UsOnly", GeographicAvailability::UsOnly),
            ("Unspecified", GeographicAvailability::Unspecified),
        ] {
            let source = format!(
                "const M: EndpointMetadata = EndpointMetadata::new()
                    .with_geography(GeographicAvailability::{variant});"
            );
            assert_eq!(geography(&source), Ok(expected), "{variant}");
        }
        let unknown = geography(
            "const M: EndpointMetadata =
                EndpointMetadata::new().with_geography(GeographicAvailability::Mars);",
        );
        assert_eq!(
            unknown,
            Err("unsupported `GeographicAvailability::Mars`".to_owned())
        );
    }

    #[test]
    fn with_access_reads_variants_and_named_add_on() {
        let named = folded(
            r#"const M: EndpointMetadata =
                EndpointMetadata::new().with_access(AccessRequirement::NamedAddOn("TipRanks"));"#,
            "M",
        );
        assert_eq!(
            named.map(|m| m.access),
            Ok(AccessRequirement::NamedAddOn("TipRanks".to_owned()))
        );
        let standard = folded(
            "const M: EndpointMetadata =
                EndpointMetadata::new().with_access(AccessRequirement::Standard);",
            "M",
        );
        assert_eq!(standard.map(|m| m.access), Ok(AccessRequirement::Standard));
        let dynamic = folded(
            "const M: EndpointMetadata =
                EndpointMetadata::new().with_access(AccessRequirement::NamedAddOn(NAME));",
            "M",
        );
        assert_eq!(
            dynamic,
            Err("add-on name is not a string literal: NAME".to_owned())
        );
    }

    #[test]
    fn with_conditional_plan_reads_plan_and_condition() {
        let plan = folded(
            r#"const M: EndpointMetadata = EndpointMetadata::new()
                .with_conditional_plan(ConditionalPlanRequirement::new(
                    "Enterprise",
                    PlanCondition::HistoryOlderThanYears(3),
                ));"#,
            "M",
        );
        assert_eq!(
            plan.map(|m| m.conditional_plan),
            Ok(Some(ConditionalPlanRequirement {
                plan: "Enterprise".to_owned(),
                condition: PlanCondition::HistoryOlderThanYears(3),
            }))
        );
        let other = folded(
            r#"const M: EndpointMetadata = EndpointMetadata::new()
                .with_conditional_plan(ConditionalPlanRequirement::new("E", PlanCondition::Always));"#,
            "M",
        );
        assert_eq!(
            other,
            Err("unsupported `PlanCondition` variant: PlanCondition :: Always".to_owned())
        );
    }

    #[test]
    fn with_realtime_reads_inline_and_const_forms() {
        let source = "
            const DELAYED: RealtimeAccess = RealtimeAccess::new(
                Some(MarketDataDelay::new(15, DelayScope::Nasdaq)),
                Some(UserDeclarationRequirement::RequiredForRealtime),
            );
            const M: EndpointMetadata = EndpointMetadata::new().with_realtime(DELAYED);
            const N: EndpointMetadata =
                EndpointMetadata::new().with_realtime(RealtimeAccess::new(None, None));
        ";
        assert_eq!(
            folded(source, "M").map(|m| m.realtime),
            Ok(Some(RealtimeAccess {
                delay: Some(MarketDataDelay {
                    minutes: 15,
                    scope: DelayScope::Nasdaq,
                }),
                user_declaration: Some(UserDeclarationRequirement::RequiredForRealtime),
            }))
        );
        assert_eq!(
            folded(source, "N").map(|m| m.realtime),
            Ok(Some(RealtimeAccess::default()))
        );
        let bare = folded(
            "const M: EndpointMetadata = EndpointMetadata::new().with_realtime(RealtimeAccess::new(Some(15), None));",
            "M",
        );
        assert_eq!(
            bare,
            Err("delay is not `MarketDataDelay::new(..)`: 15".to_owned())
        );
    }

    #[test]
    fn with_bounds_reads_every_maximum() {
        let bounds = folded(
            "const M: EndpointMetadata = EndpointMetadata::new().with_bounds(
                EndpointBounds::new()
                    .with_limit(5_000)
                    .with_response_rows(250)
                    .with_page(100)
                    .with_date_range_days(90),
            );",
            "M",
        );
        assert_eq!(
            bounds.map(|m| m.bounds),
            Ok(EndpointBounds {
                limit: Some(5_000),
                response_rows: Some(250),
                page: Some(100),
                date_range_days: Some(90),
            })
        );
        let computed = folded(
            "const M: EndpointMetadata = EndpointMetadata::new()
                .with_bounds(EndpointBounds::new().with_page(u32::MAX));",
            "M",
        );
        assert_eq!(
            computed,
            Err("with_page is not an integer literal: u32 :: MAX".to_owned())
        );
        let unrooted = folded(
            "const M: EndpointMetadata = EndpointMetadata::new().with_bounds(BOUNDS);",
            "M",
        );
        assert_eq!(
            unrooted,
            Err("bounds are not an `EndpointBounds::new()` chain: BOUNDS".to_owned())
        );
    }

    #[test]
    fn consts_derive_from_consts_until_the_depth_limit() {
        let source = "
            const US_ONLY: EndpointMetadata =
                EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);
            const M: EndpointMetadata =
                US_ONLY.with_bounds(EndpointBounds::new().with_response_rows(100));
            const LOOP: EndpointMetadata = LOOP.with_geography(GeographicAvailability::UsOnly);
        ";
        assert_eq!(
            folded(source, "M"),
            Ok(WireMetadata {
                geography: GeographicAvailability::UsOnly,
                bounds: EndpointBounds {
                    response_rows: Some(100),
                    ..EndpointBounds::default()
                },
                ..WireMetadata::default()
            })
        );
        assert_eq!(
            folded(source, "LOOP"),
            Err(format!("metadata const chain deeper than {MAX_DEPTH}"))
        );
    }

    #[test]
    fn unsupported_builders_and_roots_are_errors() {
        let builder = folded(
            "const M: EndpointMetadata = EndpointMetadata::new().with_latency(1);",
            "M",
        );
        assert_eq!(
            builder,
            Err("unsupported `EndpointMetadata` builder `with_latency`: \
                 EndpointMetadata :: new () . with_latency (1)"
                .to_owned())
        );
        let root = folded("const M: EndpointMetadata = metadata();", "M");
        assert_eq!(
            root,
            Err(
                "not an `EndpointMetadata::new()` chain or a metadata const: metadata ()"
                    .to_owned()
            )
        );
        let missing = folded("const M: EndpointMetadata = OTHER;", "M");
        assert_eq!(
            missing,
            Err("`OTHER` is not a const in module `m` or its `use` imports \
                 (glob imports and re-exports are not followed)"
                .to_owned())
        );
    }
}
