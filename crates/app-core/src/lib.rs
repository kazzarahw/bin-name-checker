//! Pure logic for checking whether a binary name is taken.
//!
//! No filesystem, network, clock, or environment access. I/O lives in
//! `app-cli`; this crate only validates names, builds query URLs, interprets
//! responses, and aggregates verdicts.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Validated binary name.
///
/// Guarantees a non-empty, `/`-free filename that is safe to interpolate into
/// registry URLs and to look up in `PATH`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BinaryName(String);

/// Reason a candidate name was rejected.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NameError {
    /// Name is empty or whitespace-only.
    #[error("name is empty")]
    Empty,
    /// Name exceeds 255 bytes (max portable filename).
    #[error("name exceeds 255 bytes")]
    TooLong,
    /// Name contains `/` and so is a path, not a filename.
    #[error("name contains '/'")]
    ContainsSlash,
    /// Name contains a NUL byte.
    #[error("name contains NUL")]
    ContainsNul,
    /// Name contains whitespace or control characters.
    #[error("name contains whitespace or control characters")]
    ContainsWhitespaceOrControl,
    /// Name is `.` or `..`.
    #[error("name is reserved ('.' or '..')")]
    Reserved,
}

impl BinaryName {
    /// Validate `raw` into a [`BinaryName`].
    ///
    /// # Errors
    ///
    /// Returns [`NameError`] when `raw` is empty, too long, contains `/`,
    /// NUL, whitespace/control characters, or is `.`/`..`.
    pub fn parse(raw: &str) -> Result<Self, NameError> {
        if raw.is_empty() {
            return Err(NameError::Empty);
        }
        if raw.len() > 255 {
            return Err(NameError::TooLong);
        }
        if raw == "." || raw == ".." {
            return Err(NameError::Reserved);
        }
        if raw.contains('/') {
            return Err(NameError::ContainsSlash);
        }
        if raw.contains('\0') {
            return Err(NameError::ContainsNul);
        }
        if raw.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(NameError::ContainsWhitespaceOrControl);
        }
        Ok(Self(raw.to_owned()))
    }

    /// Borrow the validated name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether a name appears taken by one source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Availability {
    /// Source has this name.
    Taken,
    /// Source does not have this name.
    Free,
    /// Could not determine (network error, unexpected status, unparsable body).
    Unknown,
}

/// One place a name can collide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// Executable already on local `PATH`.
    LocalPath,
    /// Repology package index (aggregates 100+ distro repos).
    Repology,
    /// `crates.io` registry.
    CratesIo,
    /// `npm` registry.
    Npm,
    /// `PyPI` registry.
    Pypi,
    /// GitHub repository name search.
    Github,
}

/// Every source checked by default, in display order.
pub const ALL_SOURCES: [Source; 6] = [
    Source::LocalPath,
    Source::Repology,
    Source::CratesIo,
    Source::Npm,
    Source::Pypi,
    Source::Github,
];

impl Source {
    /// Short stable identifier used in text and JSON output.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::LocalPath => "local-path",
            Self::Repology => "repology",
            Self::CratesIo => "crates-io",
            Self::Npm => "npm",
            Self::Pypi => "pypi",
            Self::Github => "github",
        }
    }

    /// HTTP URL to query for `name`, or `None` when the source is local.
    ///
    /// `None` only for [`Source::LocalPath`]; every remote source returns
    /// `Some`. Names are pre-validated (no `/` or whitespace), so no
    /// percent-encoding is needed.
    #[must_use]
    pub fn query_url(self, name: &BinaryName) -> Option<String> {
        let n = name.as_str();
        match self {
            Self::LocalPath => None,
            Self::Repology => Some(format!("https://repology.org/api/v1/project/{n}")),
            Self::CratesIo => Some(format!("https://crates.io/api/v1/crates/{n}")),
            Self::Npm => Some(format!("https://registry.npmjs.org/{n}")),
            Self::Pypi => Some(format!("https://pypi.org/pypi/{n}/json")),
            Self::Github => Some(format!(
                "https://api.github.com/search/repositories?q={n}+in:name&per_page=5"
            )),
        }
    }

    /// Map an HTTP `(status, body)` pair to an [`Availability`].
    ///
    /// Never fails: unexpected statuses and unparsable bodies become
    /// [`Availability::Unknown`]. [`Source::LocalPath`] has no HTTP
    /// representation and always maps to `Unknown` here; the CLI fills in the
    /// real local verdict separately.
    #[must_use]
    pub fn interpret(self, status: u16, body: &str) -> Availability {
        match self {
            Self::LocalPath => Availability::Unknown,
            Self::CratesIo | Self::Npm | Self::Pypi => registry_status(status),
            Self::Repology => interpret_repology(status, body),
            Self::Github => interpret_github(status, body),
        }
    }
}

/// Single source verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    /// Which source was checked.
    pub source: Source,
    /// What that source said.
    pub availability: Availability,
    /// Human-readable detail (HTTP status, PATH hit, error).
    pub detail: String,
    /// Query URL, or `None` for local checks.
    pub url: Option<String>,
}

impl Outcome {
    /// Build an [`Outcome`] from its parts.
    #[must_use]
    pub fn new(
        source: Source,
        availability: Availability,
        detail: String,
        url: Option<String>,
    ) -> Self {
        Self {
            source,
            availability,
            detail,
            url,
        }
    }
}

/// Roll up per-source verdicts: any `Taken` wins, else any `Unknown` wins,
/// else `Free`. Empty input maps to `Unknown` (nothing was checked).
#[must_use]
pub fn summarize(outcomes: &[Outcome]) -> Availability {
    if outcomes.is_empty() {
        return Availability::Unknown;
    }
    if outcomes
        .iter()
        .any(|o| o.availability == Availability::Taken)
    {
        return Availability::Taken;
    }
    if outcomes
        .iter()
        .any(|o| o.availability == Availability::Unknown)
    {
        return Availability::Unknown;
    }
    Availability::Free
}

/// `2xx` means taken, `404` means free, anything else is unknown.
fn registry_status(status: u16) -> Availability {
    match status {
        200..=299 => Availability::Taken,
        404 => Availability::Free,
        _ => Availability::Unknown,
    }
}

/// Repology returns `200` with a JSON array; `[]` means free.
fn interpret_repology(status: u16, body: &str) -> Availability {
    match status {
        404 => Availability::Free,
        200..=299 => {
            let trimmed = body.trim();
            if trimmed == "[]" {
                return Availability::Free;
            }
            match serde_json::from_str::<serde_json::Value>(body) {
                Ok(serde_json::Value::Array(items)) => {
                    if items.is_empty() {
                        Availability::Free
                    } else {
                        Availability::Taken
                    }
                }
                Ok(_) | Err(_) => Availability::Unknown,
            }
        }
        _ => Availability::Unknown,
    }
}

/// GitHub code search returns `200` with `{"total_count": N, ...}`.
fn interpret_github(status: u16, body: &str) -> Availability {
    match status {
        200..=299 => match serde_json::from_str::<serde_json::Value>(body) {
            Ok(value) => match value.get("total_count").and_then(serde_json::Value::as_u64) {
                Some(0) => Availability::Free,
                Some(_) => Availability::Taken,
                None => Availability::Unknown,
            },
            Err(_) => Availability::Unknown,
        },
        404 => Availability::Free,
        _ => Availability::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names_parse() {
        for raw in ["rg", "my-tool", "tool_2", "a.b", "x"] {
            assert!(BinaryName::parse(raw).is_ok(), "{raw} should parse");
        }
    }

    #[test]
    fn invalid_names_rejected() {
        assert_eq!(BinaryName::parse(""), Err(NameError::Empty));
        assert_eq!(BinaryName::parse("a/b"), Err(NameError::ContainsSlash));
        assert_eq!(BinaryName::parse("."), Err(NameError::Reserved));
        assert_eq!(BinaryName::parse(".."), Err(NameError::Reserved));
        assert_eq!(
            BinaryName::parse("has space"),
            Err(NameError::ContainsWhitespaceOrControl)
        );
    }

    #[test]
    fn query_urls_contain_name() {
        let name = BinaryName::parse("rg").unwrap();
        assert!(Source::LocalPath.query_url(&name).is_none());
        for source in [
            Source::Repology,
            Source::CratesIo,
            Source::Npm,
            Source::Pypi,
            Source::Github,
        ] {
            let url = source.query_url(&name).unwrap();
            assert!(url.contains("rg"), "{source:?} url should contain name");
        }
    }

    #[test]
    fn registry_status_mapping() {
        assert_eq!(Source::CratesIo.interpret(200, ""), Availability::Taken);
        assert_eq!(Source::CratesIo.interpret(404, ""), Availability::Free);
        assert_eq!(Source::Npm.interpret(500, ""), Availability::Unknown);
    }

    #[test]
    fn repology_empty_array_is_free() {
        assert_eq!(Source::Repology.interpret(200, "[]"), Availability::Free);
        assert_eq!(
            Source::Repology.interpret(200, r#"[{"repo":"nix"}]"#),
            Availability::Taken
        );
        assert_eq!(Source::Repology.interpret(404, ""), Availability::Free);
        assert_eq!(
            Source::Repology.interpret(200, "not-json"),
            Availability::Unknown
        );
    }

    #[test]
    fn github_total_count_mapping() {
        assert_eq!(
            Source::Github.interpret(200, r#"{"total_count":0}"#),
            Availability::Free
        );
        assert_eq!(
            Source::Github.interpret(200, r#"{"total_count":3}"#),
            Availability::Taken
        );
        assert_eq!(Source::Github.interpret(200, "{}"), Availability::Unknown);
        assert_eq!(Source::Github.interpret(403, "{}"), Availability::Unknown);
    }

    #[test]
    fn summarize_prefers_taken_then_unknown() {
        let url: Option<String> = None;
        let taken = Outcome::new(Source::Npm, Availability::Taken, String::new(), url.clone());
        let free = Outcome::new(Source::Pypi, Availability::Free, String::new(), url.clone());
        let unknown = Outcome::new(
            Source::Github,
            Availability::Unknown,
            String::new(),
            url.clone(),
        );
        assert_eq!(
            summarize(&[taken.clone(), free.clone()]),
            Availability::Taken
        );
        assert_eq!(
            summarize(&[free.clone(), unknown.clone()]),
            Availability::Unknown
        );
        assert_eq!(summarize(&[free.clone(), free.clone()]), Availability::Free);
        let empty: Vec<Outcome> = Vec::new();
        assert_eq!(summarize(&empty), Availability::Unknown);
    }
}
