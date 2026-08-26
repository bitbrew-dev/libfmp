//! Rust-first access to Financial Modeling Prep data.

pub mod error;
pub mod responses;
pub mod types;

pub use error::{Error, Result};

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
