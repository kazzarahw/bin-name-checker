//! Pure logic for checking whether a binary name is taken.
//!
//! No filesystem, network, clock, or environment access. I/O lives in
//! `bin-name-cli`; this crate only validates names, builds query URLs,
//! interprets responses, and aggregates verdicts.

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
    /// Bourne-style shell builtin or reserved keyword (see [`SHELL_BUILTINS`]).
    ShellBuiltin,
    /// Repology package index (aggregates 100+ distro repos).
    Repology,
    /// `crates.io` registry.
    CratesIo,
    /// `npm` registry.
    Npm,
    /// `PyPI` registry.
    Pypi,
    /// `RubyGems` registry.
    Rubygems,
    /// Homebrew formulae.
    Homebrew,
    /// GitHub repository name search.
    Github,
}

/// Every source checked by default, in display order.
pub const ALL_SOURCES: [Source; 9] = [
    Source::LocalPath,
    Source::ShellBuiltin,
    Source::Repology,
    Source::CratesIo,
    Source::Npm,
    Source::Pypi,
    Source::Rubygems,
    Source::Homebrew,
    Source::Github,
];

/// Bourne-style shell builtins and reserved keywords (bash/POSIX overlap).
///
/// A binary with one of these names is shadowed by the shell itself, so it
/// counts as taken even when no file exists on `PATH`. The list is
/// deliberately conservative: common interactive shells only, not every
/// `fish`/`zsh` extra.
pub const SHELL_BUILTINS: &[&str] = &[
    "alias",
    "bg",
    "bind",
    "break",
    "builtin",
    "caller",
    "case",
    "cd",
    "command",
    "compgen",
    "complete",
    "compopt",
    "continue",
    "declare",
    "dirs",
    "disown",
    "echo",
    "enable",
    "eval",
    "exec",
    "exit",
    "export",
    "false",
    "fc",
    "fg",
    "for",
    "function",
    "getopts",
    "hash",
    "help",
    "history",
    "if",
    "jobs",
    "kill",
    "let",
    "local",
    "logout",
    "mapfile",
    "popd",
    "printf",
    "pushd",
    "pwd",
    "read",
    "readarray",
    "readonly",
    "return",
    "select",
    "set",
    "shift",
    "shopt",
    "source",
    "suspend",
    "test",
    "time",
    "times",
    "trap",
    "true",
    "type",
    "typeset",
    "ulimit",
    "umask",
    "unalias",
    "unset",
    "until",
    "wait",
    "while",
];

/// Whether `name` collides with a shell builtin or reserved keyword.
#[must_use]
pub fn is_shell_builtin(name: &BinaryName) -> bool {
    SHELL_BUILTINS.iter().any(|b| *b == name.as_str())
}

impl Source {
    /// Short stable identifier used in text and JSON output.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::LocalPath => "local-path",
            Self::ShellBuiltin => "shell-builtin",
            Self::Repology => "repology",
            Self::CratesIo => "crates-io",
            Self::Npm => "npm",
            Self::Pypi => "pypi",
            Self::Rubygems => "rubygems",
            Self::Homebrew => "homebrew",
            Self::Github => "github",
        }
    }

    /// HTTP URL to query for `name`, or `None` when the source is local.
    ///
    /// `None` for [`Source::LocalPath`] and [`Source::ShellBuiltin`]; every
    /// remote source returns `Some`. Names are pre-validated (no `/` or
    /// whitespace), so no percent-encoding is needed.
    #[must_use]
    pub fn query_url(self, name: &BinaryName) -> Option<String> {
        let n = name.as_str();
        match self {
            Self::LocalPath | Self::ShellBuiltin => None,
            Self::Repology => Some(format!("https://repology.org/api/v1/project/{n}")),
            Self::CratesIo => Some(format!("https://crates.io/api/v1/crates/{n}")),
            Self::Npm => Some(format!("https://registry.npmjs.org/{n}/latest")),
            Self::Pypi => Some(format!("https://pypi.org/pypi/{n}/json")),
            Self::Rubygems => Some(format!("https://rubygems.org/api/v1/gems/{n}.json")),
            Self::Homebrew => Some(format!("https://formulae.brew.sh/api/formula/{n}.json")),
            Self::Github => Some(format!(
                "https://api.github.com/search/repositories?q={n}+in:name&per_page=5"
            )),
        }
    }

    /// Map an HTTP `(status, body)` pair to an [`Availability`].
    ///
    /// Never fails: unexpected statuses and unparsable bodies become
    /// [`Availability::Unknown`]. [`Source::LocalPath`] and
    /// [`Source::ShellBuiltin`] have no HTTP representation and always map to
    /// `Unknown` here; the CLI fills in the real local verdicts separately.
    #[must_use]
    pub fn interpret(self, status: u16, body: &str) -> Availability {
        match self {
            Self::LocalPath | Self::ShellBuiltin => Availability::Unknown,
            Self::CratesIo | Self::Npm | Self::Pypi | Self::Rubygems | Self::Homebrew => {
                registry_status(status)
            }
            Self::Repology => interpret_repology(status, body),
            Self::Github => interpret_github(status, body),
        }
    }

    /// Extract the colliding package or repo from a `Taken` response.
    ///
    /// Returns `None` unless [`interpret`](Self::interpret) says `Taken` and
    /// the body names the colliding entry. Local sources never have evidence
    /// here; the CLI reports their hit directly. For `npm`, the `bin` field
    /// is compared against `name`, so evidence says whether the package
    /// actually ships that binary or merely occupies the registry name.
    #[must_use]
    pub fn evidence(self, name: &BinaryName, status: u16, body: &str) -> Option<Evidence> {
        if self.interpret(status, body) != Availability::Taken {
            return None;
        }
        match self {
            Self::LocalPath | Self::ShellBuiltin => None,
            Self::CratesIo => registry_evidence(body, Some("crate"), "name", "description"),
            Self::Npm => npm_evidence(name, body),
            Self::Pypi => registry_evidence(body, Some("info"), "name", "summary"),
            Self::Rubygems => registry_evidence(body, None, "name", "info"),
            Self::Homebrew => registry_evidence(body, None, "name", "desc"),
            Self::Repology => repology_evidence(body),
            Self::Github => github_evidence(body),
        }
    }
}

/// Evidence behind a `Taken` verdict: which package or repo collides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    /// Colliding package, formula, or `owner/repo` — or a count summary.
    pub title: String,
    /// Short description, or repo list; empty when unavailable.
    pub detail: String,
}

impl Evidence {
    /// Build an [`Evidence`] from its parts.
    #[must_use]
    pub fn new(title: String, detail: String) -> Self {
        Self { title, detail }
    }
}

/// Whether [`render_table`] may emit ANSI color codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Plain ASCII, no escape codes (pipes, tests, `NO_COLOR`).
    Plain,
    /// Bold headers plus red/green/yellow states.
    Color,
}

/// Single source verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    /// Which source was checked.
    pub source: Source,
    /// What that source said.
    pub availability: Availability,
    /// Human-readable detail (PATH hit, error, HTTP status fallback).
    pub detail: String,
    /// Colliding package or repo, when the source named one.
    pub evidence: Option<Evidence>,
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
        evidence: Option<Evidence>,
        url: Option<String>,
    ) -> Self {
        Self {
            source,
            availability,
            detail,
            evidence,
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

/// Pull a normalized string field out of a JSON object.
fn text_field(value: &serde_json::Value, key: &str) -> Option<String> {
    let raw = value.get(key)?.as_str()?;
    Some(normalize(raw))
}

/// Collapse control characters and whitespace runs to single spaces.
fn normalize(raw: &str) -> String {
    raw.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Evidence from a registry doc: named entry inside an optional container
/// object (`crate` for crates.io, `info` for `PyPI`, top level otherwise).
fn registry_evidence(
    body: &str,
    container: Option<&str>,
    name_key: &str,
    desc_key: &str,
) -> Option<Evidence> {
    let doc: serde_json::Value = serde_json::from_str(body).ok()?;
    let entry = match container {
        Some(key) => doc.get(key)?,
        None => &doc,
    };
    let title = text_field(entry, name_key)?;
    if title.is_empty() {
        return None;
    }
    let detail = text_field(entry, desc_key).unwrap_or_default();
    Some(Evidence::new(title, detail))
}

/// Evidence from an `npm` doc: registry entry plus whether its `bin` field
/// actually ships the queried binary.
///
/// The `bin` field is either a path string (binary named after the package)
/// or a map of binary name to path. Either way the verdict stays `Taken` —
/// the registry name is occupied — but the tag tells the two cases apart.
fn npm_evidence(name: &BinaryName, body: &str) -> Option<Evidence> {
    let doc: serde_json::Value = serde_json::from_str(body).ok()?;
    let title = text_field(&doc, "name")?;
    if title.is_empty() {
        return None;
    }
    let queried = name.as_str();
    let ships = match doc.get("bin") {
        Some(serde_json::Value::Object(binaries)) => {
            binaries.keys().any(|key| key.as_str() == queried)
        }
        Some(serde_json::Value::String(_)) => title.as_str() == queried,
        _ => false,
    };
    let tag = if ships {
        format!("[ships `{queried}` binary]")
    } else {
        format!("[no `{queried}` binary declared]")
    };
    let description = text_field(&doc, "description").unwrap_or_default();
    let detail = if description.is_empty() {
        tag
    } else {
        format!("{description} {tag}")
    };
    Some(Evidence::new(title, detail))
}

/// Evidence from a Repology project listing: package count plus sample repos.
fn repology_evidence(body: &str) -> Option<Evidence> {
    let doc: serde_json::Value = serde_json::from_str(body).ok()?;
    let items = doc.as_array()?;
    if items.is_empty() {
        return None;
    }
    let title = if items.len() == 1 {
        "1 package".to_owned()
    } else {
        format!("{} packages", items.len())
    };
    let detail = items
        .iter()
        .take(3)
        .filter_map(|e| e.get("repo").and_then(serde_json::Value::as_str))
        .collect::<Vec<&str>>()
        .join(", ");
    Some(Evidence::new(title, detail))
}

/// Evidence from a GitHub repo search: the top hit's `owner/repo`.
fn github_evidence(body: &str) -> Option<Evidence> {
    let doc: serde_json::Value = serde_json::from_str(body).ok()?;
    let top = doc.get("items")?.as_array()?.first()?;
    let title = text_field(top, "full_name")?;
    if title.is_empty() {
        return None;
    }
    let detail = text_field(top, "description").unwrap_or_default();
    Some(Evidence::new(title, detail))
}

/// ASCII state glyph plus word: `[x] taken`, `[ ] free`, `[?] unknown`.
fn state_label(availability: Availability) -> &'static str {
    match availability {
        Availability::Taken => "[x] taken",
        Availability::Free => "[ ] free",
        Availability::Unknown => "[?] unknown",
    }
}

/// Bare verdict word: `taken`, `free`, `unknown`.
fn verdict_word(availability: Availability) -> &'static str {
    match availability {
        Availability::Taken => "taken",
        Availability::Free => "free",
        Availability::Unknown => "unknown",
    }
}

/// Style for a state cell.
fn state_style(availability: Availability) -> anstyle::Style {
    let color = match availability {
        Availability::Taken => anstyle::AnsiColor::Red,
        Availability::Free => anstyle::AnsiColor::Green,
        Availability::Unknown => anstyle::AnsiColor::Yellow,
    };
    anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(color)))
}

/// Wrap `text` in `style` when `mode` allows color.
fn paint(mode: ColorMode, style: anstyle::Style, text: &str) -> String {
    match mode {
        ColorMode::Plain => text.to_owned(),
        ColorMode::Color => format!("{style}{text}{style:#}"),
    }
}

/// Approximate display width in characters (registries use short ASCII text).
fn width(text: &str) -> usize {
    text.chars().count()
}

/// Pad `text` with trailing spaces to `size` characters.
fn pad(text: &str, size: usize) -> String {
    let missing = size.saturating_sub(width(text));
    format!("{text}{}", " ".repeat(missing))
}

/// Render one evidence cell: `title - detail`, capped at 60 characters.
fn render_evidence(evidence: Option<&Evidence>) -> String {
    const MAX: usize = 60;
    const KEEP: usize = 57;
    let Some(found) = evidence else {
        return String::new();
    };
    let joined = if found.detail.is_empty() {
        found.title.clone()
    } else {
        format!("{} - {}", found.title, found.detail)
    };
    if width(&joined) <= MAX {
        joined
    } else {
        format!("{}...", joined.chars().take(KEEP).collect::<String>())
    }
}

/// Render the full human report: header, ASCII table, verdict line.
///
/// Column widths adapt to the content; evidence is capped at 60 characters.
/// `ColorMode::Plain` emits no escape codes, so piped output stays clean.
#[must_use]
pub fn render_table(
    name: &BinaryName,
    outcomes: &[Outcome],
    verdict: Availability,
    mode: ColorMode,
) -> String {
    let bold = anstyle::Style::new().bold();
    let rows = outcomes
        .iter()
        .map(|o| {
            let detail = render_evidence(o.evidence.as_ref());
            let detail = if detail.is_empty() && o.availability == Availability::Taken {
                o.detail.clone()
            } else {
                detail
            };
            (o.source.id(), o.availability, detail)
        })
        .collect::<Vec<(&str, Availability, String)>>();
    let source_width = rows
        .iter()
        .map(|r| width(r.0))
        .chain([width("source")])
        .max()
        .unwrap_or_default();
    let state_width = rows
        .iter()
        .map(|r| width(state_label(r.1)))
        .chain([width("state")])
        .max()
        .unwrap_or_default();
    let detail_width = rows
        .iter()
        .map(|r| width(&r.2))
        .chain([width("detail")])
        .max()
        .unwrap_or_default();
    let mut lines = vec![format!("Checking '{}':", name.as_str())];
    lines.push(format!(
        "{}  {}  detail",
        paint(mode, bold, &pad("source", source_width)),
        paint(mode, bold, &pad("state", state_width)),
    ));
    lines.push(format!(
        "{}  {}  {}",
        "-".repeat(source_width),
        "-".repeat(state_width),
        "-".repeat(detail_width)
    ));
    for (source, availability, detail) in &rows {
        let line = format!(
            "{}  {}  {}",
            paint(mode, state_style(*availability), &pad(source, source_width)),
            paint(
                mode,
                state_style(*availability),
                &pad(state_label(*availability), state_width)
            ),
            pad(detail, detail_width),
        );
        lines.push(line.trim_end().to_owned());
    }
    lines.push(format!(
        "verdict: {}",
        paint(mode, state_style(verdict).bold(), verdict_word(verdict))
    ));
    lines.join("\n")
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
    fn name_length_boundary() {
        let at_limit = "a".repeat(255);
        assert!(BinaryName::parse(&at_limit).is_ok());
        let over_limit = "a".repeat(256);
        assert_eq!(BinaryName::parse(&over_limit), Err(NameError::TooLong));
    }

    #[test]
    fn source_ids_are_stable() {
        let ids = [
            (Source::LocalPath, "local-path"),
            (Source::ShellBuiltin, "shell-builtin"),
            (Source::Repology, "repology"),
            (Source::CratesIo, "crates-io"),
            (Source::Npm, "npm"),
            (Source::Pypi, "pypi"),
            (Source::Rubygems, "rubygems"),
            (Source::Homebrew, "homebrew"),
            (Source::Github, "github"),
        ];
        assert_eq!(ids.len(), ALL_SOURCES.len());
        for (source, want) in ids {
            assert_eq!(source.id(), want);
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
        assert!(Source::ShellBuiltin.query_url(&name).is_none());
        for source in [
            Source::Repology,
            Source::CratesIo,
            Source::Npm,
            Source::Pypi,
            Source::Rubygems,
            Source::Homebrew,
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
        assert_eq!(Source::Rubygems.interpret(200, ""), Availability::Taken);
        assert_eq!(Source::Rubygems.interpret(404, ""), Availability::Free);
        assert_eq!(Source::Homebrew.interpret(200, ""), Availability::Taken);
        assert_eq!(Source::Homebrew.interpret(404, ""), Availability::Free);
        assert_eq!(Source::Homebrew.interpret(429, ""), Availability::Unknown);
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
        assert_eq!(Source::Github.interpret(404, ""), Availability::Free);
    }

    #[test]
    fn shell_builtins_detected() {
        for raw in ["test", "cd", "echo", "time", "kill", "while"] {
            let name = BinaryName::parse(raw).unwrap();
            assert!(is_shell_builtin(&name), "{raw} should be a builtin");
        }
        for raw in ["rg", "ls", "git", "curl"] {
            let name = BinaryName::parse(raw).unwrap();
            assert!(!is_shell_builtin(&name), "{raw} should not be a builtin");
        }
    }

    #[test]
    fn local_sources_have_no_http_mapping() {
        assert_eq!(Source::LocalPath.interpret(200, ""), Availability::Unknown);
        assert_eq!(
            Source::ShellBuiltin.interpret(200, ""),
            Availability::Unknown
        );
    }

    #[test]
    fn evidence_names_colliding_entries() {
        let name = BinaryName::parse("serde").unwrap();
        let crates_body = r#"{"crate":{"name":"serde","description":"A framework"}}"#;
        assert_eq!(
            Source::CratesIo.evidence(&name, 200, crates_body),
            Some(Evidence::new("serde".to_owned(), "A framework".to_owned()))
        );
        let name = BinaryName::parse("react").unwrap();
        let npm_body = r#"{"name":"react","description":"UI library"}"#;
        assert_eq!(
            Source::Npm.evidence(&name, 200, npm_body),
            Some(Evidence::new(
                "react".to_owned(),
                "UI library [no `react` binary declared]".to_owned()
            ))
        );
        let name = BinaryName::parse("requests").unwrap();
        let pypi_body = r#"{"info":{"name":"requests","summary":"HTTP for Humans."}}"#;
        assert_eq!(
            Source::Pypi.evidence(&name, 200, pypi_body),
            Some(Evidence::new(
                "requests".to_owned(),
                "HTTP for Humans.".to_owned()
            ))
        );
        let name = BinaryName::parse("rails").unwrap();
        let gems_body = r#"{"name":"rails","info":"Full-stack framework"}"#;
        assert_eq!(
            Source::Rubygems.evidence(&name, 200, gems_body),
            Some(Evidence::new(
                "rails".to_owned(),
                "Full-stack framework".to_owned()
            ))
        );
        let name = BinaryName::parse("wget").unwrap();
        let brew_body = r#"{"name":"wget","desc":"Internet file retriever"}"#;
        assert_eq!(
            Source::Homebrew.evidence(&name, 200, brew_body),
            Some(Evidence::new(
                "wget".to_owned(),
                "Internet file retriever".to_owned()
            ))
        );
        let name = BinaryName::parse("curl").unwrap();
        let repo_body = r#"[{"repo":"debian_12","srcname":"curl"},{"repo":"nix"}]"#;
        assert_eq!(
            Source::Repology.evidence(&name, 200, repo_body),
            Some(Evidence::new(
                "2 packages".to_owned(),
                "debian_12, nix".to_owned()
            ))
        );
        let name = BinaryName::parse("ripgrep").unwrap();
        let hub_body =
            r#"{"total_count":2,"items":[{"full_name":"a/b","description":"Does things"}]}"#;
        assert_eq!(
            Source::Github.evidence(&name, 200, hub_body),
            Some(Evidence::new("a/b".to_owned(), "Does things".to_owned()))
        );
    }

    #[test]
    fn npm_evidence_marks_shipped_binaries() {
        let name = BinaryName::parse("rg").unwrap();
        let map_body = r#"{"name":"rg","description":"Docs","bin":{"rg":"./index.js"}}"#;
        assert_eq!(
            Source::Npm.evidence(&name, 200, map_body),
            Some(Evidence::new(
                "rg".to_owned(),
                "Docs [ships `rg` binary]".to_owned()
            ))
        );
        let string_body = r#"{"name":"rg","bin":"./cli.js"}"#;
        assert_eq!(
            Source::Npm.evidence(&name, 200, string_body),
            Some(Evidence::new(
                "rg".to_owned(),
                "[ships `rg` binary]".to_owned()
            ))
        );
        let other_body = r#"{"name":"rg","description":"Docs","bin":{"other":"./x.js"}}"#;
        assert_eq!(
            Source::Npm.evidence(&name, 200, other_body),
            Some(Evidence::new(
                "rg".to_owned(),
                "Docs [no `rg` binary declared]".to_owned()
            ))
        );
        let absent_body = r#"{"name":"rg","description":"Docs"}"#;
        assert_eq!(
            Source::Npm.evidence(&name, 200, absent_body),
            Some(Evidence::new(
                "rg".to_owned(),
                "Docs [no `rg` binary declared]".to_owned()
            ))
        );
    }

    #[test]
    fn evidence_absent_without_taken_name() {
        let name = BinaryName::parse("rg").unwrap();
        assert_eq!(Source::Npm.evidence(&name, 404, ""), None);
        assert_eq!(Source::Npm.evidence(&name, 200, "not-json"), None);
        assert_eq!(Source::Repology.evidence(&name, 200, "[]"), None);
        assert_eq!(
            Source::Github.evidence(&name, 200, r#"{"total_count":0}"#),
            None
        );
        assert_eq!(Source::LocalPath.evidence(&name, 200, ""), None);
    }

    #[test]
    fn evidence_tolerates_missing_descriptions() {
        let name = BinaryName::parse("x").unwrap();
        assert_eq!(
            Source::Npm.evidence(&name, 200, r#"{"name":"x"}"#),
            Some(Evidence::new(
                "x".to_owned(),
                "[no `x` binary declared]".to_owned()
            ))
        );
        let name = BinaryName::parse("a").unwrap();
        assert_eq!(
            Source::Github.evidence(
                &name,
                200,
                r#"{"total_count":1,"items":[{"full_name":"a/b","description":null}]}"#
            ),
            Some(Evidence::new("a/b".to_owned(), String::new()))
        );
    }

    #[test]
    fn plain_table_has_no_escape_codes() {
        let name = BinaryName::parse("rg").unwrap();
        let outcomes = vec![
            Outcome::new(
                Source::LocalPath,
                Availability::Taken,
                "found at /usr/bin/rg".to_owned(),
                None,
                None,
            ),
            Outcome::new(
                Source::Npm,
                Availability::Free,
                "HTTP 404".to_owned(),
                None,
                Some("https://registry.npmjs.org/rg/latest".to_owned()),
            ),
        ];
        let table = render_table(&name, &outcomes, Availability::Taken, ColorMode::Plain);
        assert!(table.contains("Checking 'rg':"));
        assert!(table.contains("local-path"));
        assert!(table.contains("[x] taken"));
        assert!(table.contains("[ ] free"));
        assert!(table.contains("found at /usr/bin/rg"));
        assert!(table.contains("verdict: taken"));
        assert!(!table.contains('\x1b'));
        for line in table.lines() {
            assert!(line.is_ascii(), "{line} should be ASCII-only");
            assert!(
                line.chars().next_back().is_none_or(|c| !c.is_whitespace()),
                "{line:?} should not trail whitespace"
            );
        }
    }

    #[test]
    fn color_table_marks_states() {
        let name = BinaryName::parse("rg").unwrap();
        let outcomes = vec![
            Outcome::new(
                Source::Npm,
                Availability::Taken,
                "HTTP 200".to_owned(),
                Some(Evidence::new("rg".to_owned(), "Grep".to_owned())),
                None,
            ),
            Outcome::new(
                Source::Pypi,
                Availability::Free,
                "HTTP 404".to_owned(),
                None,
                None,
            ),
        ];
        let table = render_table(&name, &outcomes, Availability::Taken, ColorMode::Color);
        assert!(table.contains('\x1b'));
        assert!(table.contains("rg - Grep"));
        assert!(table.contains("[31m"), "taken rows should be red");
        assert!(table.contains("[32m"), "free rows should be green");
    }

    #[test]
    fn long_evidence_is_capped() {
        let name = BinaryName::parse("rg").unwrap();
        let long = "w".repeat(100);
        let outcomes = vec![Outcome::new(
            Source::Npm,
            Availability::Taken,
            String::new(),
            Some(Evidence::new("rg".to_owned(), long)),
            None,
        )];
        let table = render_table(&name, &outcomes, Availability::Taken, ColorMode::Plain);
        let row = table
            .lines()
            .find(|l| l.contains("local") || l.contains("npm"));
        let row = row.unwrap();
        assert!(row.ends_with("..."));
        assert!(row.is_ascii());
    }

    #[test]
    fn summarize_prefers_taken_then_unknown() {
        let url: Option<String> = None;
        let taken = Outcome::new(
            Source::Npm,
            Availability::Taken,
            String::new(),
            None,
            url.clone(),
        );
        let free = Outcome::new(
            Source::Pypi,
            Availability::Free,
            String::new(),
            None,
            url.clone(),
        );
        let unknown = Outcome::new(
            Source::Github,
            Availability::Unknown,
            String::new(),
            None,
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
