//! The namespace tree: one node per Python namespace object, including the
//! implicit parents a nested registry path implies (`statements` for
//! `statements.income`), and the file each node is emitted to.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use fmp_py_gen::registry::{Endpoint, Registry};

/// One namespace object and the sub-namespaces it exposes as getters.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Node {
    /// The dotted path segments, `["statements", "income"]`.
    pub(crate) path: Vec<String>,
    /// The methods this namespace defines, in registry order.
    pub(crate) endpoints: Vec<Endpoint>,
    /// The last path segment of every direct child, sorted.
    pub(crate) children: BTreeSet<String>,
}

impl Node {
    /// The Rust struct name: full-path CamelCase plus `Namespace`
    /// (`QuoteNamespace`, `StatementsIncomeNamespace`).
    pub(crate) fn struct_name(&self) -> String {
        struct_name_for(&self.path)
    }

    /// The dotted Python path, `statements.income`.
    pub(crate) fn dotted(&self) -> String {
        self.path.join(".")
    }

    /// The file under `crates/fmp-py/src/namespaces/`: a leaf is
    /// `<path>.rs`, a node with children is `<path>/mod.rs`.
    pub(crate) fn file(&self, root: &Path) -> PathBuf {
        let mut file = root.to_path_buf();
        for segment in &self.path {
            file.push(segment);
        }
        if self.children.is_empty() {
            file.set_extension("rs");
        } else {
            file.push("mod.rs");
        }
        file
    }
}

/// Builds the tree for every namespace in `registry`, keyed by path.
pub(crate) fn build_tree(registry: &Registry) -> BTreeMap<Vec<String>, Node> {
    let mut nodes: BTreeMap<Vec<String>, Node> = BTreeMap::new();
    for domain in &registry.domains {
        for namespace in &domain.namespaces {
            for depth in 1..namespace.path.len() {
                let parent = namespace.path[..depth].to_vec();
                let child = namespace.path[depth].clone();
                nodes
                    .entry(parent.clone())
                    .or_insert_with(|| Node {
                        path: parent,
                        ..Node::default()
                    })
                    .children
                    .insert(child);
            }
            let node = nodes.entry(namespace.path.clone()).or_insert_with(|| Node {
                path: namespace.path.clone(),
                ..Node::default()
            });
            node.endpoints.extend(namespace.endpoints.iter().cloned());
        }
    }
    nodes
}

/// `["statements", "as_reported"]` -> `StatementsAsReportedNamespace`.
pub(crate) fn struct_name_for(path: &[String]) -> String {
    let mut name = String::new();
    for segment in path {
        for word in segment.split('_') {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                name.extend(first.to_uppercase());
                name.push_str(chars.as_str());
            }
        }
    }
    name.push_str("Namespace");
    name
}
