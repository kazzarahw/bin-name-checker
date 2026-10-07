//! `bin-name-checker` — check whether a binary name is taken.
//!
//! Local `PATH` lookup, shell-builtin lookup, plus HTTP queries to Repology,
//! `crates.io`, `npm`, `PyPI`, `RubyGems`, Homebrew formulae, and GitHub repo
//! search. Prints a per-source table and an overall verdict; exits `0` when
//! free, `1` when taken, `2` when unknown.

use bin_name_core::{
    ALL_SOURCES, Availability, BinaryName, ColorMode, Outcome, Source, is_shell_builtin,
    render_table, summarize,
};
use clap::Parser;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

/// Check whether a binary name is already taken.
#[derive(Parser, Debug)]
#[command(
    name = "bin-name-checker",
    version,
    about = "Check if a binary name is taken"
)]
struct Args {
    /// Candidate binary name, e.g. `rg`
    name: String,
    /// Emit JSON instead of human-readable lines
    #[arg(long)]
    json: bool,
    /// Skip remote sources; check local `PATH` and shell builtins only
    #[arg(long)]
    offline: bool,
}

#[derive(Debug, serde::Serialize)]
struct ReportEntry {
    source: Source,
    availability: Availability,
    detail: String,
    evidence: Option<ReportEvidence>,
    url: Option<String>,
}

/// Serializable copy of [`bin_name_core::Evidence`].
#[derive(Debug, serde::Serialize)]
struct ReportEvidence {
    title: String,
    detail: String,
}

#[derive(Debug, serde::Serialize)]
struct Report<'a> {
    name: &'a str,
    verdict: Availability,
    results: Vec<ReportEntry>,
}

fn main() -> ExitCode {
    run()
}

fn run() -> ExitCode {
    let args = Args::parse();
    let name = match BinaryName::parse(&args.name) {
        Ok(valid) => valid,
        Err(problem) => {
            eprintln!("invalid name '{}': {problem}", args.name);
            return ExitCode::from(2);
        }
    };

    let mut outcomes: Vec<Outcome> = Vec::new();
    outcomes.push(check_local(&name));
    outcomes.push(check_builtin(&name));
    if !args.offline {
        let client = match build_client() {
            Ok(built) => built,
            Err(problem) => {
                eprintln!("cannot build HTTP client: {problem}");
                return ExitCode::from(2);
            }
        };
        for source in ALL_SOURCES {
            if source == Source::LocalPath || source == Source::ShellBuiltin {
                continue;
            }
            let Some(url) = source.query_url(&name) else {
                continue;
            };
            outcomes.push(check_remote(&client, source, &url));
        }
    }

    let verdict = summarize(&outcomes);
    if args.json {
        print_json(&name, verdict, &outcomes);
    } else {
        println!("{}", render_table(&name, &outcomes, verdict, color_mode()));
    }

    match verdict {
        Availability::Free => ExitCode::SUCCESS,
        Availability::Taken => ExitCode::FAILURE,
        Availability::Unknown => ExitCode::from(2),
    }
}

/// Build the shared blocking HTTP client.
fn build_client() -> Result<reqwest::blocking::Client, reqwest::Error> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(format!("bin-name-checker/{}", env!("CARGO_PKG_VERSION")))
        .build()
}

/// Search each directory on `PATH` for an executable file called `name`.
#[must_use]
fn check_local(name: &BinaryName) -> Outcome {
    let Some(paths) = std::env::var_os("PATH") else {
        return Outcome::new(
            Source::LocalPath,
            Availability::Unknown,
            "PATH is not set".to_owned(),
            None,
            None,
        );
    };
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(name.as_str());
        if is_executable(&candidate) {
            return Outcome::new(
                Source::LocalPath,
                Availability::Taken,
                format!("found at {}", candidate.display()),
                None,
                None,
            );
        }
    }
    Outcome::new(
        Source::LocalPath,
        Availability::Free,
        "not found in PATH".to_owned(),
        None,
        None,
    )
}

/// Check `name` against the static shell-builtin list.
#[must_use]
fn check_builtin(name: &BinaryName) -> Outcome {
    if is_shell_builtin(name) {
        Outcome::new(
            Source::ShellBuiltin,
            Availability::Taken,
            "shell builtin or reserved keyword".to_owned(),
            None,
            None,
        )
    } else {
        Outcome::new(
            Source::ShellBuiltin,
            Availability::Free,
            "not a shell builtin".to_owned(),
            None,
            None,
        )
    }
}

/// `GET` `url` and interpret the response for `source`.
///
/// Network failures, timeouts, and unparsable bodies become
/// [`Availability::Unknown`], never a hard error. When `GITHUB_TOKEN` is set,
/// it is sent as a bearer token on GitHub requests to raise the rate limit.
#[must_use]
fn check_remote(client: &reqwest::blocking::Client, source: Source, url: &str) -> Outcome {
    let mut request = client.get(url);
    if source == Source::Github {
        request = request.header(reqwest::header::ACCEPT, "application/vnd.github+json");
        if let Ok(token) = std::env::var("GITHUB_TOKEN")
            && !token.is_empty()
        {
            request = request.bearer_auth(token);
        }
    }
    let response = match request.send() {
        Ok(ok) => ok,
        Err(problem) => {
            return Outcome::new(
                source,
                Availability::Unknown,
                format!("request failed: {problem}"),
                None,
                Some(url.to_owned()),
            );
        }
    };
    let status = response.status().as_u16();
    let body = match response.text() {
        Ok(text) => text,
        Err(problem) => {
            return Outcome::new(
                source,
                Availability::Unknown,
                format!("unreadable body (HTTP {status}): {problem}"),
                None,
                Some(url.to_owned()),
            );
        }
    };
    let availability = source.interpret(status, &body);
    let evidence = source.evidence(status, &body);
    Outcome::new(
        source,
        availability,
        format!("HTTP {status}"),
        evidence,
        Some(url.to_owned()),
    )
}

#[cfg(unix)]
#[must_use]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    match path.metadata() {
        Ok(meta) => meta.is_file() && meta.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

#[cfg(not(unix))]
#[must_use]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[must_use]
fn color_mode() -> ColorMode {
    use std::io::IsTerminal as _;
    let dumb = std::env::var("TERM").is_ok_and(|term| term == "dumb");
    if std::io::stdout().is_terminal() && !dumb && std::env::var("NO_COLOR").is_err() {
        ColorMode::Color
    } else {
        ColorMode::Plain
    }
}

fn print_json(name: &BinaryName, verdict: Availability, outcomes: &[Outcome]) {
    let results = outcomes
        .iter()
        .map(|o| ReportEntry {
            source: o.source,
            availability: o.availability,
            detail: o.detail.clone(),
            evidence: o.evidence.as_ref().map(|e| ReportEvidence {
                title: e.title.clone(),
                detail: e.detail.clone(),
            }),
            url: o.url.clone(),
        })
        .collect::<Vec<ReportEntry>>();
    let report = Report {
        name: name.as_str(),
        verdict,
        results,
    };
    match serde_json::to_string_pretty(&report) {
        Ok(text) => println!("{text}"),
        Err(problem) => eprintln!("cannot encode JSON: {problem}"),
    }
}
