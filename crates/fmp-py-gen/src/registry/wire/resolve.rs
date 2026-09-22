//! Resolves one `Client` method to its [`WireEndpoint`] by evaluating the
//! descriptor function it calls.
//!
//! The evaluator is deliberately small: a descriptor body is `let`
//! bindings followed by one tail expression, the tail is an
//! `EndpointSpec::{get, get_binary, new, with_response}` call (possibly
//! wrapped in builder calls, of which `.with_metadata(..)` is read and the
//! rest are transparent) or a call to another free function that is
//! evaluated the same way with its parameters bound to the caller's
//! arguments. Values are string literals, `&[&str]` constants, enum
//! variants, the `&'static str` an enum's `const fn` match table returns
//! for a variant, and folded [`WireMetadata`] values. Anything else stops
//! the resolution with a reason naming the expression, never a guess.

use std::collections::BTreeMap;

use syn::{Expr, ExprMethodCall, Stmt};

use super::collect::{
    Collected, FnDef, ModulePath, path_segments, query_type, relative_item, string_list,
    string_literal, text,
};
use super::metadata::{self, WireMetadata};
use super::params::params;
use super::{Contract, HttpMethod, WireEndpoint};

/// Deepest chain of descriptor helper calls the evaluator follows.
const MAX_DEPTH: usize = 8;

/// A value the evaluator can carry between expressions.
#[derive(Debug, Clone)]
enum Value {
    Str(String),
    Strs(Vec<String>),
    /// `Type::Variant`, kept for the match tables and `HttpMethod::Get`.
    Variant(String, String),
    /// A folded `EndpointMetadata` const or builder chain.
    Metadata(WireMetadata),
    /// Anything the evaluator does not model, carried as source text, or
    /// as the reason when it looked like metadata but could not be folded.
    Opaque(String),
}

/// The literal facts of one `EndpointSpec` constructor call and the
/// metadata attached by `.with_metadata(..)` on the way out.
struct Spec {
    id: String,
    relative_path: String,
    http_method: HttpMethod,
    contract: Contract,
    metadata: Option<WireMetadata>,
}

struct Evaluator<'a> {
    collected: &'a Collected,
}

pub(super) fn resolve(collected: &Collected, method: &str) -> Result<WireEndpoint, String> {
    let client = collected
        .client_methods
        .get(method)
        .ok_or_else(|| format!("libfmp `Client` has no `pub async fn {method}`"))?;
    let evaluator = Evaluator { collected };
    let (path, args) = descriptor_call(&client.body)?;
    let key = resolve_path(&client.module, &path)?;
    let descriptor = collected.functions.get(&key).ok_or_else(|| {
        format!(
            "descriptor `{}` is not a free function under endpoints/",
            key.1
        )
    })?;
    let query = query_type(&descriptor.output)?;
    let args: Vec<Value> = args
        .iter()
        .map(|arg| evaluator.eval(arg, &BTreeMap::new(), &client.module))
        .collect();
    let spec = evaluator.eval_fn(descriptor, args, 0)?;
    let params = match &query {
        Some(query) => params(collected, query, "", 0)?,
        None => Vec::new(),
    };
    Ok(WireEndpoint {
        method: method.to_owned(),
        descriptor: format!("{}::{}", key.0.join("::"), key.1),
        file: descriptor.file.clone(),
        origin: descriptor.origin.clone(),
        id: spec.id,
        relative_path: spec.relative_path,
        http_method: spec.http_method,
        query_type: query,
        contract: spec.contract,
        params,
        metadata: spec.metadata,
    })
}

/// The descriptor call inside a client method body, which must be exactly
/// `self.execute(&descriptor(args..)).await`.
fn descriptor_call(body: &syn::Block) -> Result<(syn::Path, Vec<Expr>), String> {
    let unsupported = || {
        format!(
            "client body is not `self.execute(&f(..)).await`: {}",
            text(body)
        )
    };
    let [Stmt::Expr(Expr::Await(awaited), None)] = body.stmts.as_slice() else {
        return Err(unsupported());
    };
    let Expr::MethodCall(execute) = &*awaited.base else {
        return Err(unsupported());
    };
    if execute.method != "execute" || execute.args.len() != 1 {
        return Err(unsupported());
    }
    let Expr::Reference(reference) = &execute.args[0] else {
        return Err(unsupported());
    };
    let Expr::Call(call) = &*reference.expr else {
        return Err(unsupported());
    };
    let Expr::Path(func) = &*call.func else {
        return Err(unsupported());
    };
    Ok((func.path.clone(), call.args.iter().cloned().collect()))
}

/// Resolves a call path to the `(module, name)` key of a free function.
fn resolve_path(caller: &ModulePath, path: &syn::Path) -> Result<(ModulePath, String), String> {
    relative_item(caller, &path_segments(path))
}

impl Evaluator<'_> {
    fn eval_fn(&self, def: &FnDef, args: Vec<Value>, depth: usize) -> Result<Spec, String> {
        if depth > MAX_DEPTH {
            return Err(format!("descriptor helper chain deeper than {MAX_DEPTH}"));
        }
        if def.params.len() != args.len() {
            return Err(format!(
                "descriptor takes {} parameter(s) but is called with {}",
                def.params.len(),
                args.len()
            ));
        }
        let mut env: BTreeMap<String, Value> = def.params.iter().cloned().zip(args).collect();
        let (tail, bindings) = def
            .body
            .stmts
            .split_last()
            .ok_or_else(|| "descriptor body is empty".to_owned())?;
        for stmt in bindings {
            let Stmt::Local(local) = stmt else {
                return Err(format!("unsupported descriptor statement: {}", text(stmt)));
            };
            let (syn::Pat::Ident(name), Some(init)) = (&local.pat, &local.init) else {
                return Err(format!("unsupported `let` binding: {}", text(stmt)));
            };
            let value = self.eval(&init.expr, &env, &def.module);
            env.insert(name.ident.to_string(), value);
        }
        let Stmt::Expr(tail, None) = tail else {
            return Err(format!(
                "descriptor does not end in an expression: {}",
                text(tail)
            ));
        };
        self.eval_spec(tail, &env, def, depth)
    }

    /// Evaluates the tail expression of a descriptor: builder method calls
    /// are peeled until the constructor or helper call underneath, and a
    /// `.with_metadata(..)` on the way back out attaches its folded
    /// argument (the outermost call wins, as in Rust).
    fn eval_spec(
        &self,
        expr: &Expr,
        env: &BTreeMap<String, Value>,
        def: &FnDef,
        depth: usize,
    ) -> Result<Spec, String> {
        match expr {
            Expr::MethodCall(builder) => {
                let mut spec = self.eval_spec(&builder.receiver, env, def, depth)?;
                if builder.method == "with_metadata" {
                    spec.metadata = Some(self.metadata_arg(builder, env, def)?);
                }
                Ok(spec)
            }
            Expr::Paren(inner) => self.eval_spec(&inner.expr, env, def, depth),
            Expr::Call(call) => {
                let Expr::Path(func) = &*call.func else {
                    return Err(format!("unsupported descriptor call: {}", text(expr)));
                };
                let args: Vec<Value> = call
                    .args
                    .iter()
                    .map(|arg| self.eval(arg, env, &def.module))
                    .collect();
                match path_segments(&func.path).as_slice() {
                    [head, constructor] if head == "EndpointSpec" => {
                        self.constructor(constructor, &args, &call.args, &def.module)
                    }
                    _ => {
                        let key = resolve_path(&def.module, &func.path)?;
                        let helper = self.collected.functions.get(&key).ok_or_else(|| {
                            format!("helper `{}` is not a free function under endpoints/", key.1)
                        })?;
                        self.eval_fn(helper, args, depth + 1)
                    }
                }
            }
            other => Err(format!(
                "unsupported descriptor expression: {}",
                text(other)
            )),
        }
    }

    /// The folded argument of one `.with_metadata(..)` call in `def`: a
    /// parameter bound by the caller, or an expression folded in place.
    /// Any failure is located by descriptor and file.
    fn metadata_arg(
        &self,
        call: &ExprMethodCall,
        env: &BTreeMap<String, Value>,
        def: &FnDef,
    ) -> Result<WireMetadata, String> {
        let located = |reason: String| {
            format!(
                "metadata of `{}::{}` in {}: {reason}",
                def.module.join("::"),
                def.name,
                def.file.display()
            )
        };
        let [arg] = metadata::args_of(call, "`with_metadata`").map_err(located)?;
        let bound = match arg {
            Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| env.get(&name.to_string())),
            _ => None,
        };
        match bound {
            Some(Value::Metadata(metadata)) => Ok(metadata.clone()),
            Some(Value::Opaque(reason)) => Err(located(format!(
                "argument is not a supported `EndpointMetadata` expression: {reason}"
            ))),
            Some(other) => Err(located(format!(
                "argument is not `EndpointMetadata`: {other:?}"
            ))),
            None => metadata::evaluate(self.collected, &def.module, arg).map_err(located),
        }
    }

    fn constructor(
        &self,
        name: &str,
        args: &[Value],
        exprs: &syn::punctuated::Punctuated<Expr, syn::Token![,]>,
        module: &ModulePath,
    ) -> Result<Spec, String> {
        let string = |index: usize, what: &str| match args.get(index) {
            Some(Value::Str(value)) => Ok(value.clone()),
            Some(Value::Opaque(source)) => Err(format!(
                "{what} of `EndpointSpec::{name}` is not a literal: {source}"
            )),
            Some(other) => Err(format!(
                "{what} of `EndpointSpec::{name}` is not a string: {other:?}"
            )),
            None => Err(format!("`EndpointSpec::{name}` is missing its {what}")),
        };
        let strings = |index: usize, what: &str| match args.get(index) {
            Some(Value::Strs(values)) => Ok(values.clone()),
            Some(other) => Err(format!(
                "{what} of `EndpointSpec::{name}` is not a `&[&str]` literal or constant: {other:?}"
            )),
            None => Err(format!("`EndpointSpec::{name}` is missing its {what}")),
        };
        let http_method = |index: usize| match args.get(index) {
            Some(Value::Variant(ty, variant)) if ty == "HttpMethod" => HttpMethod::parse(variant),
            other => Err(format!(
                "method of `EndpointSpec::{name}` is not an `HttpMethod` variant: {other:?}"
            )),
        };
        match name {
            "get" => Ok(Spec {
                id: string(0, "id")?,
                relative_path: string(1, "path")?,
                http_method: HttpMethod::Get,
                contract: Contract::Rows,
                metadata: None,
            }),
            "get_binary" => Ok(Spec {
                id: string(0, "id")?,
                relative_path: string(1, "path")?,
                http_method: HttpMethod::Get,
                contract: Contract::Binary(strings(3, "content types")?),
                metadata: None,
            }),
            "new" => Ok(Spec {
                http_method: http_method(0)?,
                id: string(1, "id")?,
                relative_path: string(2, "path")?,
                contract: Contract::Rows,
                metadata: None,
            }),
            "with_response" => Ok(Spec {
                http_method: http_method(0)?,
                id: string(1, "id")?,
                relative_path: string(2, "path")?,
                contract: self.response_contract(exprs.iter().nth(4), module)?,
                metadata: None,
            }),
            other => Err(format!("unsupported constructor `EndpointSpec::{other}`")),
        }
    }

    /// Reads `ResponseContract::json()` or `ResponseContract::binary(list)`.
    fn response_contract(
        &self,
        expr: Option<&Expr>,
        module: &ModulePath,
    ) -> Result<Contract, String> {
        let unsupported = |expr: Option<&Expr>| {
            format!(
                "response contract is not `ResponseContract::json()` or `::binary(..)`: {}",
                expr.map(text).unwrap_or_default()
            )
        };
        let Some(Expr::Call(call)) = expr else {
            return Err(unsupported(expr));
        };
        let Expr::Path(func) = &*call.func else {
            return Err(unsupported(expr));
        };
        let segments: Vec<String> = func
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        match (segments.as_slice(), call.args.first()) {
            ([head, kind], None) if head == "ResponseContract" && kind == "json" => {
                Ok(Contract::Rows)
            }
            ([head, kind], Some(list)) if head == "ResponseContract" && kind == "binary" => {
                match self.eval(list, &BTreeMap::new(), module) {
                    Value::Strs(values) => Ok(Contract::Binary(values)),
                    other => Err(format!(
                        "binary content types are not a `&[&str]` literal or constant: {other:?}"
                    )),
                }
            }
            _ => Err(unsupported(expr)),
        }
    }

    /// Evaluates a value expression written in `module`, which scopes the
    /// const lookups behind metadata paths.
    fn eval(&self, expr: &Expr, env: &BTreeMap<String, Value>, module: &ModulePath) -> Value {
        match expr {
            Expr::Lit(_) => {
                string_literal(expr).map_or_else(|| Value::Opaque(text(expr)), Value::Str)
            }
            Expr::Reference(reference) => self.eval(&reference.expr, env, module),
            Expr::Paren(inner) => self.eval(&inner.expr, env, module),
            Expr::Group(inner) => self.eval(&inner.expr, env, module),
            Expr::Array(array) => array
                .elems
                .iter()
                .map(string_literal)
                .collect::<Option<Vec<_>>>()
                .map_or_else(|| Value::Opaque(text(expr)), Value::Strs),
            Expr::Path(path) => match path_segments(&path.path).as_slice() {
                [name] => env
                    .get(name)
                    .cloned()
                    .or_else(|| {
                        let (_, def) = self.collected.constant(module, name)?;
                        string_list(&def.expr).map(Value::Strs)
                    })
                    .unwrap_or_else(|| self.opaque(expr, module)),
                [ty, variant] => Value::Variant(ty.clone(), variant.clone()),
                _ => Value::Opaque(text(expr)),
            },
            Expr::MethodCall(call) if call.args.is_empty() => {
                match self.eval(&call.receiver, env, module) {
                    Value::Variant(ty, variant) => self
                        .collected
                        .literal_methods
                        .get(&(ty, call.method.to_string()))
                        .and_then(|table| table.get(&variant))
                        .cloned()
                        .map_or_else(|| Value::Opaque(text(expr)), Value::Str),
                    _ => Value::Opaque(text(expr)),
                }
            }
            _ => self.opaque(expr, module),
        }
    }

    /// The value of an expression no other rule matched: folded metadata
    /// when it is an `EndpointMetadata` value (so a metadata const can be
    /// passed on to a helper's `.with_metadata(..)`), else opaque source
    /// text.
    fn opaque(&self, expr: &Expr, module: &ModulePath) -> Value {
        if !metadata::is_metadata(self.collected, module, expr) {
            return Value::Opaque(text(expr));
        }
        match metadata::evaluate(self.collected, module, expr) {
            Ok(metadata) => Value::Metadata(metadata),
            Err(reason) => Value::Opaque(reason),
        }
    }
}
