//! Endpoint namespaces reached from the top-level `FmpClient`.
//!
//! Each namespace is a lightweight frozen pyclass holding a shared
//! `ClientBuilder`, exposing the endpoints of one libfmp domain as methods.

pub(crate) mod quote;
