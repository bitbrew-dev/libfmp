//! The ordered wire parameters a query type emits, read from its
//! `encode` body: `encoder.required("key", value)` and
//! `encoder.optional("key", value)` calls, where the value is a field of
//! the query (`self.f`, `&self.f`, `self.f.as_ref()`) or a literal, plus
//! `self.f.encode(encoder)` delegation into the field's own type (the DCF
//! assumptions builder). Anything else is reported, never guessed.

use quote::ToTokens;
use syn::{Expr, Member, Stmt};

use super::collect::{Collected, string_literal, text};
use super::{Presence, Source, WireParam};

/// Deepest chain of `encode` delegation the reader follows.
const MAX_DEPTH: usize = 8;

/// The wire parameters a query type's `encode` body emits, in order.
/// `prefix` is the dotted field path of a delegating parent.
pub(super) fn params(
    collected: &Collected,
    query: &str,
    prefix: &str,
    depth: usize,
) -> Result<Vec<WireParam>, String> {
    if depth > MAX_DEPTH {
        return Err(format!("encode delegation deeper than {MAX_DEPTH}"));
    }
    let body = collected
        .encoders
        .get(query)
        .ok_or_else(|| format!("no `impl QueryParameters for {query}` under endpoints/"))?;
    let mut out = Vec::new();
    for stmt in &body.stmts {
        let Stmt::Expr(Expr::MethodCall(call), Some(_)) = stmt else {
            return Err(format!(
                "unsupported encode statement in `{query}`: {}",
                text(stmt)
            ));
        };
        let method = call.method.to_string();
        let receiver = receiver_name(&call.receiver);
        match (receiver.as_deref(), method.as_str(), call.args.len()) {
            (Some("encoder"), "required" | "optional", 2) => {
                let name = string_literal(&call.args[0]).ok_or_else(|| {
                    format!("wire key in `{query}` is not a literal: {}", text(stmt))
                })?;
                let source = param_source(&call.args[1], prefix).ok_or_else(|| {
                    format!(
                        "value in `{query}` is not a field or literal: {}",
                        text(stmt)
                    )
                })?;
                out.push(WireParam {
                    name,
                    presence: if method == "required" {
                        Presence::Required
                    } else {
                        Presence::Optional
                    },
                    source,
                });
            }
            (None, "encode", 1) => {
                let Some(field) = self_field(&call.receiver) else {
                    return Err(format!(
                        "unsupported encode delegation in `{query}`: {}",
                        text(stmt)
                    ));
                };
                let target = collected
                    .struct_fields
                    .get(query)
                    .and_then(|fields| fields.get(&field))
                    .ok_or_else(|| format!("`{query}` has no field `{field}` to delegate to"))?;
                out.extend(params(
                    collected,
                    target,
                    &format!("{prefix}{field}."),
                    depth + 1,
                )?);
            }
            _ => {
                return Err(format!(
                    "unsupported encode statement in `{query}`: {}",
                    text(stmt)
                ));
            }
        }
    }
    Ok(out)
}

/// The bare identifier a method is called on, if the receiver is one.
fn receiver_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(path) => path.path.get_ident().map(ToString::to_string),
        _ => None,
    }
}

/// `self.field` as the field name.
fn self_field(expr: &Expr) -> Option<String> {
    let Expr::Field(field) = expr else {
        return None;
    };
    if !matches!(&*field.base, Expr::Path(path) if path.path.is_ident("self")) {
        return None;
    }
    match &field.member {
        Member::Named(name) => Some(name.to_string()),
        Member::Unnamed(_) => None,
    }
}

/// Where a wire value comes from: `self.f`, `&self.f`, `self.f.as_ref()`
/// (and `as_deref`, `clone`, `copied`, `as_str`) are the field `f`; a literal
/// is a constant.
fn param_source(expr: &Expr, prefix: &str) -> Option<Source> {
    match expr {
        Expr::Reference(reference) => param_source(&reference.expr, prefix),
        Expr::Paren(inner) => param_source(&inner.expr, prefix),
        Expr::Field(_) => self_field(expr).map(|field| Source::Field(format!("{prefix}{field}"))),
        Expr::MethodCall(call)
            if call.args.is_empty()
                && ["as_ref", "as_deref", "clone", "copied", "as_str"]
                    .contains(&call.method.to_string().as_str()) =>
        {
            param_source(&call.receiver, prefix)
        }
        Expr::Lit(literal) => Some(Source::Constant(match &literal.lit {
            syn::Lit::Str(value) => value.value(),
            other => other.to_token_stream().to_string(),
        })),
        _ => None,
    }
}
