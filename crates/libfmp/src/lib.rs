//! Rust-first access to Financial Modeling Prep data.
//!
//! # Cargo features
//!
//! - `rustls` (default): HTTPS through `reqwest` with rustls. Disabling
//!   default features leaves the built-in executor without TLS, for plain-HTTP
//!   targets such as a local proxy, or for a caller-supplied [`transport::HttpExecutor`].

pub mod client;
pub mod codecs;
pub mod config;
pub mod endpoints;
pub mod error;
pub mod query;
pub mod responses;
pub mod transport;
pub mod types;

pub use client::{Client, ClientBuilder};
pub use error::{Error, Result};

pub use bytes;
pub use chrono;
pub use http;
pub use serde_json;
pub use url;

const _: () = {
    const fn assert_send_sync_static<T: Send + Sync + 'static>() {}
    assert_send_sync_static::<Error>();
    assert_send_sync_static::<Client>();
};

/// The version of the `libfmp` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_matches_the_package_version() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }
}
