//! The language-neutral intermediate representation of a discovered response
//! struct: its name, module path, and named fields with their `syn` types.

use syn::Type;

/// A single response struct discovered in libfmp.
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub module_path: Vec<String>,
    pub fields: Vec<FieldDef>,
}

/// One named field of a response struct.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub ty: Type,
}

/// The composition wrappers peeled from a field type, outermost first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrap {
    Option,
    Vec,
}
