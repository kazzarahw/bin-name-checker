//! Core library for the `app` binary.
//!
//! Pure logic only: no filesystem, network, clock, or environment access. I/O
//! belongs in `app-cli`.

/// Returns the crate version as reported by Cargo at build time.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!version().is_empty());
    }
}
