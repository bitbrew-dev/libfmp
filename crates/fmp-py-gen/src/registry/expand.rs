//! Bounded `macro_rules!` expansion for the query types `libfmp` emits
//! through declarative macros.
//!
//! Only the shape the endpoint modules use is supported: one or more arms
//! whose matcher is a flat comma list of `$name:fragment` captures, invoked
//! with a flat comma list of token groups. Repetitions, nested captures, and
//! metavariables the matcher does not bind make the invocation unexpandable,
//! and the caller records it as trusted rather than verified.
//!
//! Two invocation positions are supported: a file-level invocation that
//! emits items ([`expand`]) and an invocation inside an `impl` block that
//! emits methods ([`expand_impl_items`]), the shape the screener and DCF
//! setter macros use.

use proc_macro2::{Group, Punct, Spacing, TokenStream, TokenTree};

/// Why an invocation was left unexpanded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unexpandable {
    /// The `macro_rules!` body is not a list of `(matcher) => {body}` arms.
    MalformedDefinition,
    /// No arm binds a flat capture list of the invocation's arity.
    NoMatchingArm,
    /// The body uses a metavariable the matcher does not bind.
    UnboundMetavariable(String),
    /// The substituted body is not a sequence of Rust items.
    NotItems(String),
    /// The substituted body is not a sequence of `impl` block items.
    NotImplItems(String),
}

impl std::fmt::Display for Unexpandable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedDefinition => f.write_str("macro body is not `(..) => {..}` arms"),
            Self::NoMatchingArm => f.write_str("no flat `$x:frag` arm matches the arity"),
            Self::UnboundMetavariable(name) => write!(f, "body uses unbound `${name}`"),
            Self::NotItems(error) => write!(f, "expansion is not a list of items: {error}"),
            Self::NotImplItems(error) => {
                write!(f, "expansion is not a list of impl items: {error}")
            }
        }
    }
}

struct Arm {
    captures: Vec<String>,
    body: TokenStream,
}

/// Expands `invocation` (the tokens between the invocation's delimiters)
/// against `definition` (the tokens between the `macro_rules!` braces) into
/// file-level items.
pub fn expand(
    definition: &TokenStream,
    invocation: &TokenStream,
) -> Result<syn::File, Unexpandable> {
    let expanded = substitute_invocation(definition, invocation)?;
    syn::parse2::<syn::File>(expanded).map_err(|error| Unexpandable::NotItems(error.to_string()))
}

/// Expands an invocation written inside an `impl` block into the methods
/// and associated items it emits.
pub fn expand_impl_items(
    definition: &TokenStream,
    invocation: &TokenStream,
) -> Result<Vec<syn::ImplItem>, Unexpandable> {
    let expanded = substitute_invocation(definition, invocation)?;
    let wrapped: TokenStream = quote::quote!(impl Expanded { #expanded });
    syn::parse2::<syn::ItemImpl>(wrapped)
        .map(|item| item.items)
        .map_err(|error| Unexpandable::NotImplItems(error.to_string()))
}

fn substitute_invocation(
    definition: &TokenStream,
    invocation: &TokenStream,
) -> Result<TokenStream, Unexpandable> {
    let arms = parse_arms(definition)?;
    let arguments = split_commas(invocation);
    let arm = arms
        .iter()
        .find(|arm| arm.captures.len() == arguments.len())
        .ok_or(Unexpandable::NoMatchingArm)?;
    let bindings: Vec<(&str, &TokenStream)> = arm
        .captures
        .iter()
        .map(String::as_str)
        .zip(arguments.iter())
        .collect();
    substitute(&arm.body, &bindings)
}

fn parse_arms(definition: &TokenStream) -> Result<Vec<Arm>, Unexpandable> {
    let mut arms = Vec::new();
    let mut tokens = definition.clone().into_iter().peekable();
    while let Some(matcher) = tokens.next() {
        let TokenTree::Group(matcher) = matcher else {
            return Err(Unexpandable::MalformedDefinition);
        };
        let arrow_ok = matches!(tokens.next(), Some(TokenTree::Punct(p)) if p.as_char() == '=')
            && matches!(tokens.next(), Some(TokenTree::Punct(p)) if p.as_char() == '>');
        if !arrow_ok {
            return Err(Unexpandable::MalformedDefinition);
        }
        let Some(TokenTree::Group(body)) = tokens.next() else {
            return Err(Unexpandable::MalformedDefinition);
        };
        if matches!(tokens.peek(), Some(TokenTree::Punct(p)) if p.as_char() == ';') {
            tokens.next();
        }
        if let Some(captures) = flat_captures(&matcher.stream()) {
            arms.push(Arm {
                captures,
                body: body.stream(),
            });
        }
    }
    Ok(arms)
}

/// Reads `$a:frag, $b:frag` into `["a", "b"]`, or `None` for any other shape.
fn flat_captures(matcher: &TokenStream) -> Option<Vec<String>> {
    let mut captures = Vec::new();
    for piece in split_commas(matcher) {
        let tokens: Vec<TokenTree> = piece.into_iter().collect();
        match tokens.as_slice() {
            [
                TokenTree::Punct(dollar),
                TokenTree::Ident(name),
                TokenTree::Punct(colon),
                TokenTree::Ident(_),
            ] if dollar.as_char() == '$' && colon.as_char() == ':' => {
                captures.push(name.to_string());
            }
            _ => return None,
        }
    }
    Some(captures)
}

/// Splits a token stream at top-level commas, dropping a trailing empty piece.
fn split_commas(tokens: &TokenStream) -> Vec<TokenStream> {
    let mut pieces = vec![TokenStream::new()];
    for token in tokens.clone() {
        match token {
            TokenTree::Punct(p) if p.as_char() == ',' => pieces.push(TokenStream::new()),
            other => pieces
                .last_mut()
                .expect("pieces always holds the current piece")
                .extend([other]),
        }
    }
    if pieces.last().is_some_and(TokenStream::is_empty) {
        pieces.pop();
    }
    pieces
}

fn substitute(
    body: &TokenStream,
    bindings: &[(&str, &TokenStream)],
) -> Result<TokenStream, Unexpandable> {
    let mut out = TokenStream::new();
    let mut tokens = body.clone().into_iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Punct(p) if p.as_char() == '$' => {
                let Some(TokenTree::Ident(name)) = tokens.next() else {
                    return Err(Unexpandable::UnboundMetavariable("(".to_owned()));
                };
                let name = name.to_string();
                let bound = bindings
                    .iter()
                    .find(|(capture, _)| *capture == name)
                    .ok_or_else(|| Unexpandable::UnboundMetavariable(name.clone()))?;
                out.extend(bound.1.clone());
            }
            TokenTree::Group(group) => {
                let inner = substitute(&group.stream(), bindings)?;
                out.extend([TokenTree::Group(Group::new(group.delimiter(), inner))]);
            }
            other => out.extend([other]),
        }
    }
    Ok(out)
}

/// Builds the token stream of a `(a, b)` invocation body from string pieces,
/// for tests and tooling.
pub fn invocation(pieces: &[&str]) -> Result<TokenStream, syn::Error> {
    let mut out = TokenStream::new();
    for (index, piece) in pieces.iter().enumerate() {
        if index > 0 {
            out.extend([TokenTree::Punct(Punct::new(',', Spacing::Alone))]);
        }
        out.extend(piece.parse::<TokenStream>()?);
    }
    Ok(out)
}

/// The name and `{ ... }` body of a `macro_rules!` item, or `None` when the
/// item is an invocation rather than a definition.
pub fn definition(item: &syn::ItemMacro) -> Option<(String, TokenStream)> {
    if !item.mac.path.is_ident("macro_rules") {
        return None;
    }
    let name = item.ident.as_ref()?.to_string();
    Some((name, item.mac.tokens.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFINITION: &str = r#"
        ($docs:literal, $query:ident) => {
            #[doc = $docs]
            pub struct $query { symbol: Ticker }
            impl $query {
                pub fn new(symbol: Ticker) -> Self { Self { symbol } }
                pub const fn with_limit(mut self, limit: Limit) -> Self { self }
            }
        };
    "#;

    fn definition_tokens() -> TokenStream {
        DEFINITION.parse().expect("definition tokens")
    }

    #[test]
    fn expands_flat_capture_arm() {
        let call = invocation(&["\"docs\"", "QuoteQuery"]).expect("invocation");
        let file = expand(&definition_tokens(), &call).expect("expands");
        let names: Vec<String> = file
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Struct(s) => Some(s.ident.to_string()),
                syn::Item::Impl(i) => Some(quote_ident(&i.self_ty)),
                _ => None,
            })
            .collect();
        assert_eq!(names, ["QuoteQuery", "QuoteQuery"]);
    }

    #[test]
    fn expands_impl_level_setter_macro() {
        let definition: TokenStream = r#"
            ($with:ident, $get:ident, $field:ident, $type:ty, $docs:literal) => {
                #[doc = $docs]
                pub fn $with(mut self, value: $type) -> Self {
                    self.$field = Some(value);
                    self
                }

                #[doc = $docs]
                pub const fn $get(&self) -> Option<$type> {
                    self.$field
                }
            };
        "#
        .parse()
        .expect("definition tokens");
        let call = invocation(&["with_is_etf", "is_etf", "is_etf", "bool", "\"docs\""])
            .expect("invocation");
        let items = expand_impl_items(&definition, &call).expect("expands");
        let names: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(method) => Some(method.sig.ident.to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(names, ["with_is_etf", "is_etf"]);
    }

    #[test]
    fn rejects_arity_mismatch() {
        let call = invocation(&["QuoteQuery"]).expect("invocation");
        assert_eq!(
            expand(&definition_tokens(), &call).expect_err("arity"),
            Unexpandable::NoMatchingArm
        );
    }

    #[test]
    fn rejects_repetition_matchers() {
        let definition: TokenStream = "($($x:ident),+) => { $(pub struct $x;)+ };"
            .parse()
            .expect("tokens");
        let call = invocation(&["A", "B"]).expect("invocation");
        assert_eq!(
            expand(&definition, &call).expect_err("repetition"),
            Unexpandable::NoMatchingArm
        );
    }

    #[test]
    fn rejects_unbound_metavariable() {
        let definition: TokenStream = "($x:ident) => { pub struct $x { field: $y } };"
            .parse()
            .expect("tokens");
        let call = invocation(&["A"]).expect("invocation");
        assert_eq!(
            expand(&definition, &call).expect_err("unbound"),
            Unexpandable::UnboundMetavariable("y".to_owned())
        );
    }

    fn quote_ident(ty: &syn::Type) -> String {
        match ty {
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default(),
            _ => String::new(),
        }
    }
}
