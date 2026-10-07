//! `app` — check whether a binary name is taken.
//!
//! Local `PATH` lookup plus HTTP queries to Repology, `crates.io`, `npm`,
//! `PyPI`, and GitHub repo search. Prints per-source verdicts and an overall
//! verdict; exits `0` when free, `1` when taken, `2` when unknown.

use app_core::{ALL_SOURCES, Availability, BinaryName, Outcome, Source, summarize};
use clap::Parser;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

/// Check whether a binary name is already taken.
#[derive(Parser, Debug)]
#[command(name = "app", version, about = "Check if a binary name is taken")]
struct Args {
    /// Candidate binary name, e.g. `rg`
    name: String,
    /// Emit JSON instead of human-readable lines
    #[arg(long)]
    json: bool,
}

#[derive(Debug, serde::Serialize)]
struct ReportEntry {
    source: Source,
    availability: Availability,
    detail: String,
    url: Option<String>,
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

    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("bin-name-checker/0.1.0")
        .build()
    {
        Ok(built) => built,
        Err(problem) => {
            eprintln!("cannot build HTTP client: {problem}");
            return ExitCode::from(2);
        }
    };

    let mut outcomes: Vec<Outcome> = Vec::new();
    outcomes.push(check_local(&name));
    for source in ALL_SOURCES {
        if source == Source::LocalPath {
            continue;
        }
        let Some(url) = source.query_url(&name) else {
            continue;
        };
        outcomes.push(check_remote(&client, source, &url));
    }

    let verdict = summarize(&outcomes);
    if args.json {
        print_json(&name, verdict, &outcomes);
    } else {
        print_human(&outcomes, verdict);
    }

    match verdict {
        Availability::Free => ExitCode::SUCCESS,
        Availability::Taken => ExitCode::FAILURE,
        Availability::Unknown => ExitCode::from(2),
    }
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
            );
        }
    }
    Outcome::new(
        Source::LocalPath,
        Availability::Free,
        "not found in PATH".to_owned(),
        None,
    )
}

/// `GET` `url` and interpret the response for `source`.
///
/// Network failures, timeouts, and unparsable bodies become
/// [`Availability::Unknown`], never a hard error.
#[must_use]
fn check_remote(client: &reqwest::blocking::Client, source: Source, url: &str) -> Outcome {
    let request = if source == Source::Github {
        client
            .get(url)
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
    } else {
        client.get(url)
    };
    let response = match request.send() {
        Ok(ok) => ok,
        Err(problem) => {
            return Outcome::new(
                source,
                Availability::Unknown,
                format!("request failed: {problem}"),
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
                Some(url.to_owned()),
            );
        }
    };
    let availability = source.interpret(status, &body);
    Outcome::new(
        source,
        availability,
        format!("HTTP {status}"),
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
fn label(availability: Availability) -> &'static str {
    match availability {
        Availability::Taken => "taken",
        Availability::Free => "free",
        Availability::Unknown => "unknown",
    }
}

fn print_human(outcomes: &[Outcome], verdict: Availability) {
    for outcome in outcomes {
        let id = outcome.source.id();
        let state = label(outcome.availability);
        println!("{id}: {state} - {}", outcome.detail);
    }
    println!("verdict: {}", label(verdict));
}

fn print_json(name: &BinaryName, verdict: Availability, outcomes: &[Outcome]) {
    let results = outcomes
        .iter()
        .map(|o| ReportEntry {
            source: o.source,
            availability: o.availability,
            detail: o.detail.clone(),
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
