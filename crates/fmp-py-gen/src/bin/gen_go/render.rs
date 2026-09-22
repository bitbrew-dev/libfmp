//! Renders `sdk/go/<domain>.go`, the shared `queries.go`, and
//! `namespaces.go` from the plans in `methods.rs` (before `gofmt`).

use std::fmt::Write;

use crate::emit::{GENERATED_HEADER, doc_comment, exported};
use crate::methods::{DomainPlan, MethodPlan, NamespacePlan, ParamPlan, QueryPlan, ResponseKind};

/// Renders one domain file: namespace structs, the queries only this domain
/// uses, and one method per registry entry.
pub(crate) fn render_domain(plan: &DomainPlan) -> String {
    let mut body = String::new();
    for namespace in &plan.namespaces {
        render_namespace(namespace, &plan.name, &mut body);
    }
    let needs_slices = render_queries(&plan.queries, &mut body);
    let mut needs_jsontext = false;
    for namespace in &plan.namespaces {
        for method in &namespace.methods {
            needs_jsontext |= method.response == ResponseKind::Dynamic;
            render_method(namespace, method, &mut body);
        }
    }
    let mut out = file_prelude(&format!(
        "Endpoint methods of the {} domain, generated from crates/fmp-py-gen/registry/{}.toml \
         and the wire contract read from crates/libfmp/src/endpoints (ADR 0030).",
        plan.name, plan.name
    ));
    let mut imports = vec!["context"];
    if needs_jsontext {
        imports.push("encoding/json/jsontext");
    }
    if needs_slices {
        imports.push("slices");
    }
    render_imports(&imports, &mut out);
    out.push_str(&body);
    out
}

/// Renders the query types more than one domain uses.
pub(crate) fn render_shared_queries(queries: &[QueryPlan]) -> String {
    let mut body = String::new();
    let needs_slices = render_queries(queries, &mut body);
    let mut out = file_prelude(
        "Query types shared by more than one registry domain (the union over every \
         generated domain), so each Rust query type has exactly one Go type.",
    );
    if needs_slices {
        render_imports(&["slices"], &mut out);
    }
    out.push_str(&body);
    out
}

/// Renders the `Namespaces` struct Client embeds and its binding.
pub(crate) fn render_namespaces(domains: &[&DomainPlan]) -> String {
    let mut out = file_prelude(
        "The namespace fields promoted onto Client, one per generated registry domain.",
    );
    out.push_str(&doc_comment(
        "Namespaces groups the generated endpoint namespaces, one field per registry \
         domain. Client embeds it, so client.Quote.Full is the call shape; the fields \
         are valid only on a Client built by NewClient.",
    ));
    out.push_str("type Namespaces struct {\n");
    for domain in domains {
        let field = exported(&domain.name);
        let _ = writeln!(
            out,
            "\t// {field} holds the {} endpoints.\n\t{field} {}",
            domain.name, domain.namespaces[0].struct_name
        );
    }
    out.push_str("}\n\n// bindNamespaces points every namespace at its client.\n");
    out.push_str("func (c *Client) bindNamespaces() {\n");
    for domain in domains {
        let _ = writeln!(
            out,
            "\tc.{} = new{}(c)",
            exported(&domain.name),
            domain.namespaces[0].struct_name
        );
    }
    out.push_str("}\n");
    out
}

pub(crate) fn file_prelude(description: &str) -> String {
    let mut out = String::from(GENERATED_HEADER);
    out.push_str("\n\npackage fmp\n\n");
    out.push_str(&doc_comment(description));
    out.push('\n');
    out
}

fn render_imports(imports: &[&str], out: &mut String) {
    match imports {
        [] => {}
        [single] => {
            let _ = writeln!(out, "import {single:?}\n");
        }
        many => {
            out.push_str("import (\n");
            for import in many {
                let _ = writeln!(out, "\t{import:?}");
            }
            out.push_str(")\n\n");
        }
    }
}

fn render_namespace(namespace: &NamespacePlan, domain: &str, out: &mut String) {
    let dotted = namespace.path.join(".");
    out.push_str(&doc_comment(&format!(
        "{} groups the {dotted} endpoints. It is reached as {} and is valid only when \
         obtained from a Client built by NewClient.",
        namespace.struct_name,
        namespace.reached_as()
    )));
    let _ = writeln!(
        out,
        "type {} struct {{\n\tclient *Client",
        namespace.struct_name
    );
    for child in &namespace.children {
        let _ = writeln!(
            out,
            "\t// {0} groups the {dotted}.{1} endpoints of the {domain} domain.\n\t{0} {2}",
            child.field, child.segment, child.struct_name
        );
    }
    let _ = writeln!(
        out,
        "}}\n\nfunc new{0}(client *Client) {0} {{\n\treturn {0}{{",
        namespace.struct_name
    );
    out.push_str("\t\tclient: client,\n");
    for child in &namespace.children {
        let _ = writeln!(
            out,
            "\t\t{}: new{}(client),",
            child.field, child.struct_name
        );
    }
    out.push_str("\t}\n}\n\n");
}

/// Renders every query type, then every constant parameter list; reports
/// whether `slices` is used.
fn render_queries(queries: &[QueryPlan], out: &mut String) -> bool {
    let mut needs_slices = false;
    for query in queries.iter().filter(|query| !query.is_constant()) {
        needs_slices |= render_query(query, out);
    }
    for query in queries.iter().filter(|query| query.is_constant()) {
        render_constant(query, out);
    }
    needs_slices
}

fn render_constant(query: &QueryPlan, out: &mut String) {
    let pairs: Vec<String> = query
        .params
        .iter()
        .map(|param| match param {
            ParamPlan::Constant { name, value } => {
                format!("{{Name: {name:?}, Value: {value:?}}}")
            }
            ParamPlan::Field { .. } => unreachable!("constant queries have no fields"),
        })
        .collect();
    out.push_str(&doc_comment(&format!(
        "{} is the fixed query of the endpoints that encode {} in the Rust crate; it \
         exposes no choice.",
        query.var_name(),
        query.name
    )));
    let _ = writeln!(
        out,
        "var {} = []queryParam{{{}}}\n",
        query.var_name(),
        pairs.join(", ")
    );
}

fn render_query(query: &QueryPlan, out: &mut String) -> bool {
    let name = &query.name;
    let mut needs_slices = false;
    out.push_str(&doc_comment(&format!(
        "{name} holds the query parameters of the endpoints that take it: New{name} takes \
         the required arguments and each With method sets an optional one. Values are \
         validated when the request is built."
    )));
    let _ = writeln!(out, "type {name} struct {{");
    for arg in &query.args {
        let star = if arg.setter.is_some() { "*" } else { "" };
        let _ = writeln!(out, "\t{} {star}{}", arg.field, arg.go_type);
    }
    out.push_str("}\n\n");

    let ctor: Vec<String> = query
        .ctor_args()
        .map(|arg| format!("{} {}", arg.field, arg.go_type))
        .collect();
    out.push_str(&doc_comment(&format!(
        "New{name} creates the query from its required arguments."
    )));
    let _ = writeln!(out, "func New{name}({}) {name} {{", ctor.join(", "));
    let inits: Vec<String> = query
        .ctor_args()
        .map(|arg| {
            if arg.go_type.starts_with("[]") {
                needs_slices = true;
                format!("{}: slices.Clone({})", arg.field, arg.field)
            } else {
                format!("{}: {}", arg.field, arg.field)
            }
        })
        .collect();
    let _ = writeln!(out, "\treturn {name}{{{}}}\n}}\n", inits.join(", "));

    for arg in &query.args {
        match &arg.setter {
            None => {
                out.push_str(&doc_comment(&format!(
                    "{} returns the {} argument as given.",
                    arg.getter, arg.name
                )));
                let _ = writeln!(
                    out,
                    "func (q {name}) {}() {} {{\n\treturn q.{}\n}}\n",
                    arg.getter, arg.go_type, arg.field
                );
            }
            Some(setter) => {
                out.push_str(&doc_comment(&format!(
                    "{setter} sets the optional {} parameter and returns the updated query.",
                    arg.name
                )));
                let _ = writeln!(
                    out,
                    "func (q {name}) {setter}({} {}) {name} {{\n\tq.{} = &{}\n\treturn q\n}}\n",
                    arg.field, arg.go_type, arg.field, arg.field
                );
                out.push_str(&doc_comment(&format!(
                    "{} returns the optional {} parameter, or nil when it is unset.",
                    arg.getter, arg.name
                )));
                let _ = writeln!(
                    out,
                    "func (q {name}) {}() *{} {{\n\treturn q.{}\n}}\n",
                    arg.getter, arg.go_type, arg.field
                );
            }
        }
    }
    render_params(query, out);
    needs_slices
}

/// `params()` in wire order: the compact literal form when every pair is a
/// required field, the append form otherwise.
fn render_params(query: &QueryPlan, out: &mut String) {
    let name = &query.name;
    let local = |arg: &crate::methods::ArgPlan| {
        if matches!(arg.field.as_str(), "params" | "err" | "q") {
            format!("{}Param", arg.field)
        } else {
            arg.field.clone()
        }
    };
    let compact = query.params.iter().all(
        |param| matches!(param, ParamPlan::Field { arg, .. } if query.args[*arg].setter.is_none()),
    );
    let _ = writeln!(out, "func (q {name}) params() ([]queryParam, error) {{");
    if !compact {
        let _ = writeln!(
            out,
            "\tparams := make([]queryParam, 0, {})",
            query.params.len()
        );
    }
    let mut locals = Vec::new();
    for param in &query.params {
        match param {
            ParamPlan::Constant { name, value } => {
                let _ = writeln!(
                    out,
                    "\tparams = append(params, queryParam{{Name: {name:?}, Value: {value:?}}})"
                );
            }
            ParamPlan::Field { name, arg } => {
                let arg = &query.args[*arg];
                let local = local(arg);
                if arg.setter.is_some() {
                    let _ = writeln!(
                        out,
                        "\tif q.{0} != nil {{\n\t\t{local}, err := {1}({name:?}, *q.{0})\n\
                         \t\tif err != nil {{\n\t\t\treturn nil, err\n\t\t}}\n\
                         \t\tparams = append(params, {local})\n\t}}",
                        arg.field, arg.helper
                    );
                } else {
                    let _ = writeln!(
                        out,
                        "\t{local}, err := {}({name:?}, q.{})\n\tif err != nil {{\n\
                         \t\treturn nil, err\n\t}}",
                        arg.helper, arg.field
                    );
                    if compact {
                        locals.push(local);
                    } else {
                        let _ = writeln!(out, "\tparams = append(params, {local})");
                    }
                }
            }
        }
    }
    if compact {
        let _ = writeln!(
            out,
            "\treturn []queryParam{{{}}}, nil\n}}\n",
            locals.join(", ")
        );
    } else {
        out.push_str("\treturn params, nil\n}\n\n");
    }
}

fn render_method(namespace: &NamespacePlan, method: &MethodPlan, out: &mut String) {
    out.push_str(&doc_comment(&method.doc));
    let _ = writeln!(out, "//\n// {}", method.wire_summary);
    let query_param = method
        .query
        .as_ref()
        .map_or(String::new(), |query| format!(", q {query}"));
    let (result, zero) = match &method.response {
        ResponseKind::Rows(model) => (format!("[]{model}"), "nil"),
        ResponseKind::Dynamic => ("[]jsontext.Value".to_string(), "nil"),
        ResponseKind::Binary(_) => ("BinaryPayload".to_string(), "BinaryPayload{}"),
    };
    let _ = writeln!(
        out,
        "func (n *{}) {}(ctx context.Context{query_param}) ({result}, error) {{",
        namespace.struct_name, method.name
    );
    let params = match (&method.query, &method.constant) {
        (Some(_), _) => {
            let _ = writeln!(
                out,
                "\tparams, err := q.params()\n\tif err != nil {{\n\t\treturn {zero}, err\n\t}}"
            );
            "params"
        }
        (None, Some(constant)) => constant.as_str(),
        (None, None) => "nil",
    };
    let id = &method.endpoint_id;
    let path = &method.path;
    match &method.response {
        ResponseKind::Binary(types) => {
            let types: Vec<String> = types.iter().map(|t| format!("{t:?}")).collect();
            let _ = writeln!(
                out,
                "\treturn n.client.getBinary(ctx, {id:?}, {path:?}, {params}, []string{{{}}})\n}}\n",
                types.join(", ")
            );
        }
        ResponseKind::Rows(_) | ResponseKind::Dynamic => {
            let _ = writeln!(
                out,
                "\tvar out {result}\n\tif err := n.client.getJSON(ctx, {id:?}, {path:?}, {params}, &out); err != nil {{\n\
                 \t\treturn nil, err\n\t}}"
            );
            if method.response == ResponseKind::Dynamic {
                let _ = writeln!(
                    out,
                    "\tif err := requireObjectRows({id:?}, out); err != nil {{\n\t\treturn nil, err\n\t}}"
                );
            }
            out.push_str("\treturn out, nil\n}\n\n");
        }
    }
}
