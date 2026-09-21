//! Plans the endpoint surface of one domain: namespace structs, query types
//! (constructor, getters, `With` setters, `params()` in wire order), and one
//! method per registry entry, in the shape the hand-written quote domain
//! settled (ADR 0030 phase 0). Rendering lives in `render.rs`.

use std::collections::{BTreeMap, BTreeSet};

use fmp_py_gen::registry::wire::{Contract, Presence, Source, WireEndpoint, WireSurface};
use fmp_py_gen::registry::{Domain, Endpoint, Registry};

use crate::emit::{exported, local_ident, lower_first, lower_lead};
use crate::types::arg_kind_go;

/// One constructor argument or `With` setter of a query type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArgPlan {
    /// The registry name, also the wire key it usually feeds.
    pub(crate) name: String,
    /// The unexported Go field and local identifier.
    pub(crate) field: String,
    pub(crate) go_type: &'static str,
    pub(crate) helper: &'static str,
    /// The exported getter name.
    pub(crate) getter: String,
    /// The `With` setter name when the argument is optional.
    pub(crate) setter: Option<String>,
}

/// One query pair in the order the Rust `encode` body emits it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParamPlan {
    Constant { name: String, value: String },
    Field { name: String, arg: usize },
}

/// A Go query type, or a fixed parameter list when it has no arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueryPlan {
    /// The Rust query type name, reused as the Go type name.
    pub(crate) name: String,
    pub(crate) args: Vec<ArgPlan>,
    pub(crate) params: Vec<ParamPlan>,
}

impl QueryPlan {
    /// A query with no arguments is rendered as a `var <name>Params` slice.
    pub(crate) fn is_constant(&self) -> bool {
        self.args.is_empty()
    }

    /// `ShortOnlyQuery` -> `shortOnlyParams`.
    pub(crate) fn var_name(&self) -> String {
        format!(
            "{}Params",
            lower_first(self.name.strip_suffix("Query").unwrap_or(&self.name))
        )
    }

    pub(crate) fn ctor_args(&self) -> impl Iterator<Item = &ArgPlan> {
        self.args.iter().filter(|arg| arg.setter.is_none())
    }
}

/// What a method returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResponseKind {
    Rows(String),
    Dynamic,
    Binary(Vec<String>),
}

/// One exported method on a namespace struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MethodPlan {
    pub(crate) name: String,
    pub(crate) doc: String,
    pub(crate) endpoint_id: String,
    pub(crate) path: String,
    /// The query type the method takes, when it takes one.
    pub(crate) query: Option<String>,
    /// The `var` holding the fixed parameters of a query-less method.
    pub(crate) constant: Option<String>,
    pub(crate) wire_summary: String,
    pub(crate) response: ResponseKind,
}

/// One namespace struct: the domain root or a nested `[[namespace]]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NamespacePlan {
    pub(crate) path: Vec<String>,
    pub(crate) struct_name: String,
    /// `(field, struct name)` of every direct child, sorted by field.
    pub(crate) children: Vec<(String, String)>,
    pub(crate) methods: Vec<MethodPlan>,
}

impl NamespacePlan {
    /// `Client.Statements.Income`.
    pub(crate) fn reached_as(&self) -> String {
        let mut out = String::from("Client");
        for segment in &self.path {
            out.push('.');
            out.push_str(&exported(segment));
        }
        out
    }
}

/// Everything `<domain>.go` renders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DomainPlan {
    pub(crate) name: String,
    pub(crate) namespaces: Vec<NamespacePlan>,
    /// The query types only this domain uses, sorted by name.
    pub(crate) queries: Vec<QueryPlan>,
}

/// The inputs every domain plan reads.
pub(crate) struct Context<'a> {
    pub(crate) registry: &'a Registry,
    pub(crate) wire: &'a WireSurface,
    /// Response struct name -> the domain whose `_models.go` defines it.
    pub(crate) model_owner: &'a BTreeMap<String, String>,
    /// The domains that are (or will be) generated.
    pub(crate) generated: &'a BTreeSet<String>,
}

impl Context<'_> {
    /// The domains using each wire query type, over the whole registry, so
    /// a type's home does not depend on which domains are generated.
    pub(crate) fn query_users(&self) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
        let mut users: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for domain in &self.registry.domains {
            for endpoint in domain.namespaces.iter().flat_map(|ns| &ns.endpoints) {
                let wire = self.wire_for(&domain.name, endpoint)?;
                if let Some(query) = &wire.query_type {
                    users
                        .entry(query.clone())
                        .or_default()
                        .insert(domain.name.clone());
                }
            }
        }
        Ok(users)
    }

    fn wire_for(&self, domain: &str, endpoint: &Endpoint) -> Result<&WireEndpoint, String> {
        self.wire.for_endpoint(endpoint).ok_or_else(|| {
            let reason = self
                .wire
                .unresolved
                .iter()
                .find(|u| u.method == endpoint.libfmp_method)
                .map_or("no wire endpoint", |u| u.reason.as_str());
            format!(
                "{domain}.{}: libfmp method `{}` has no resolved wire contract: {reason}",
                endpoint.python_name, endpoint.libfmp_method
            )
        })
    }

    /// Plans every query type used by the generated domains, keyed by name,
    /// merging the entries that share one Rust type.
    pub(crate) fn plan_queries(&self) -> Result<BTreeMap<String, QueryPlan>, String> {
        let mut plans: BTreeMap<String, QueryPlan> = BTreeMap::new();
        for domain in &self.registry.domains {
            if !self.generated.contains(&domain.name) {
                continue;
            }
            for endpoint in domain.namespaces.iter().flat_map(|ns| &ns.endpoints) {
                let wire = self.wire_for(&domain.name, endpoint)?;
                let Some(name) = &wire.query_type else {
                    continue;
                };
                let label = format!("{}.{}", domain.name, endpoint.python_name);
                let plan = plan_query(name, endpoint, wire).map_err(|e| format!("{label}: {e}"))?;
                match plans.get(name) {
                    None => {
                        plans.insert(name.clone(), plan);
                    }
                    Some(existing) if *existing == plan => {}
                    Some(_) => {
                        return Err(format!(
                            "{label}: query `{name}` differs from another entry using it; \
                             gen_go needs one shape per Rust query type"
                        ));
                    }
                }
            }
        }
        Ok(plans)
    }

    /// Plans one domain, taking ownership of the queries only it uses.
    pub(crate) fn plan_domain(
        &self,
        domain: &Domain,
        queries: &BTreeMap<String, QueryPlan>,
        users: &BTreeMap<String, BTreeSet<String>>,
    ) -> Result<DomainPlan, String> {
        let mut nodes: BTreeMap<Vec<String>, NamespacePlan> = BTreeMap::new();
        let mut used = BTreeSet::new();
        for namespace in &domain.namespaces {
            for depth in 1..namespace.path.len() {
                let parent = namespace.path[..depth].to_vec();
                let child = &namespace.path[depth];
                let entry = nodes.entry(parent.clone()).or_insert_with(|| node(parent));
                let pair = (exported(child), struct_name(&namespace.path[..=depth]));
                if !entry.children.contains(&pair) {
                    entry.children.push(pair);
                }
            }
            let entry = nodes
                .entry(namespace.path.clone())
                .or_insert_with(|| node(namespace.path.clone()));
            for endpoint in &namespace.endpoints {
                let wire = self.wire_for(&domain.name, endpoint)?;
                let method = self
                    .plan_method(endpoint, wire, queries)
                    .map_err(|e| format!("{}.{}: {e}", domain.name, endpoint.python_name))?;
                if let Some(query) = &wire.query_type {
                    used.insert(query.clone());
                }
                entry.methods.push(method);
            }
        }
        for plan in nodes.values_mut() {
            plan.children.sort();
        }
        let owned = used
            .iter()
            .filter(|name| users.get(*name).is_some_and(|set| set.len() == 1))
            .map(|name| queries[name].clone())
            .collect();
        Ok(DomainPlan {
            name: domain.name.clone(),
            namespaces: nodes.into_values().collect(),
            queries: owned,
        })
    }

    fn plan_method(
        &self,
        endpoint: &Endpoint,
        wire: &WireEndpoint,
        queries: &BTreeMap<String, QueryPlan>,
    ) -> Result<MethodPlan, String> {
        let response = match (&endpoint.response_model, &wire.contract) {
            (_, Contract::Binary(types)) if endpoint.binary => ResponseKind::Binary(types.clone()),
            (None, Contract::Rows) if endpoint.dynamic => ResponseKind::Dynamic,
            (Some(model), Contract::Rows) => {
                let owner = self.model_owner.get(&model.name).ok_or_else(|| {
                    format!("response model `{}` is not a discovered struct", model.name)
                })?;
                if !self.generated.contains(owner) {
                    return Err(format!(
                        "response model `{}` lives in {owner}_models.go; generate the `{owner}` \
                         domain first (or in the same run)",
                        model.name
                    ));
                }
                ResponseKind::Rows(model.name.clone())
            }
            (model, contract) => {
                return Err(format!(
                    "registry response {model:?} does not match wire contract {contract:?}"
                ));
            }
        };
        let (query, constant) = match &wire.query_type {
            None => (None, None),
            Some(name) => {
                let plan = &queries[name];
                if plan.is_constant() {
                    (None, Some(plan.var_name()))
                } else {
                    (Some(name.clone()), None)
                }
            }
        };
        let mut wire_summary = format!("{} {}", wire.http_method, wire.relative_path);
        let pairs: Vec<String> = wire
            .params
            .iter()
            .map(|param| match &param.source {
                Source::Constant(value) => format!("{}={value}", param.name),
                Source::Field(_) => format!("{}=", param.name),
            })
            .collect();
        if !pairs.is_empty() {
            wire_summary.push('?');
            wire_summary.push_str(&pairs.join("&"));
        }
        let name = exported(&endpoint.python_name);
        Ok(MethodPlan {
            doc: method_doc(&name, &endpoint.doc),
            name,
            endpoint_id: wire.id.clone(),
            path: wire.relative_path.clone(),
            query,
            constant,
            wire_summary,
            response,
        })
    }
}

fn node(path: Vec<String>) -> NamespacePlan {
    NamespacePlan {
        struct_name: struct_name(&path),
        path,
        children: Vec::new(),
        methods: Vec::new(),
    }
}

/// `["statements", "income"]` -> `StatementsIncomeNamespace`.
pub(crate) fn struct_name(path: &[String]) -> String {
    let mut name: String = path.iter().map(|segment| exported(segment)).collect();
    name.push_str("Namespace");
    name
}

/// `Full` + "The full quote for a symbol." -> "Full returns the full quote
/// for a symbol."; a doc that opens with a verb keeps it.
fn method_doc(name: &str, doc: &str) -> String {
    let doc = doc.split_whitespace().collect::<Vec<_>>().join(" ");
    let first = doc.split(' ').next().unwrap_or_default();
    let verb_like = first.len() > 2
        && first.ends_with('s')
        && first.starts_with(|c: char| c.is_ascii_uppercase())
        && first[1..].chars().all(|c| c.is_ascii_lowercase());
    if verb_like {
        format!("{name} {}", lower_lead(&doc))
    } else {
        format!("{name} returns {}", lower_lead(&doc))
    }
}

/// Builds the query plan of one registry entry from its wire contract.
fn plan_query(name: &str, endpoint: &Endpoint, wire: &WireEndpoint) -> Result<QueryPlan, String> {
    if let Some(registry_query) = &endpoint.query_type
        && registry_query != name
    {
        return Err(format!(
            "registry query `{registry_query}` differs from the encoded `{name}`"
        ));
    }
    let mut args = Vec::new();
    for arg in &endpoint.args {
        let setter = endpoint
            .setters
            .iter()
            .chain(endpoint.nested.iter().flat_map(|b| &b.setters))
            .find(|setter| setter.arg == arg.name)
            .map(|setter| exported(&setter.method));
        if setter.is_none() && !arg.required {
            return Err(format!(
                "optional arg `{}` is passed to the constructor; gen_go only models optional \
                 arguments as setters",
                arg.name
            ));
        }
        let go = arg_kind_go(arg.kind).map_err(|e| format!("arg `{}`: {e}", arg.name))?;
        args.push(ArgPlan {
            name: arg.name.clone(),
            field: local_ident(&arg.name),
            go_type: go.go_type,
            helper: go.helper,
            getter: exported(&arg.name),
            setter,
        });
    }
    let mut params = Vec::new();
    for param in &wire.params {
        params.push(match &param.source {
            Source::Constant(value) => ParamPlan::Constant {
                name: param.name.clone(),
                value: value.clone(),
            },
            Source::Field(path) => {
                let field = path.rsplit('.').next().unwrap_or(path);
                let index = args
                    .iter()
                    .position(|arg| arg.name == field)
                    .ok_or_else(|| {
                        format!(
                            "wire parameter `{}` reads field `{path}` which no registry arg names",
                            param.name
                        )
                    })?;
                let optional = args[index].setter.is_some();
                if optional != (param.presence == Presence::Optional) {
                    return Err(format!(
                        "wire parameter `{}` is {:?} but registry arg `{field}` is {}",
                        param.name,
                        param.presence,
                        if optional { "a setter" } else { "required" }
                    ));
                }
                ParamPlan::Field {
                    name: param.name.clone(),
                    arg: index,
                }
            }
        });
    }
    Ok(QueryPlan {
        name: name.to_string(),
        args,
        params,
    })
}
