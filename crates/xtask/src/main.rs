//! `xtask` — project tooling, runnable via `cargo run -p xtask -- <task>`.
//!
//! Tasks are manual gates, not unit tests: `eval` hits live registries over
//! the network, so it must never run as part of `cargo t` (see `AGENTS.md`:
//! no real provider APIs in tests).

use bin_name_core::{Availability, Source};
use clap::{Parser, Subcommand};
use std::process::{Command, ExitCode};

/// Project tooling.
#[derive(Parser, Debug)]
#[command(name = "xtask", version, about = "Project tooling")]
struct Args {
    #[command(subcommand)]
    task: Task,
}

/// Available tasks.
#[derive(Subcommand, Debug)]
enum Task {
    /// Check candidate names against live sources and compare verdicts.
    Eval,
}

/// One eval row: a name plus the verdict the checker should produce.
#[derive(Debug, Clone, Copy)]
struct Case {
    name: &'static str,
    expect: Availability,
}

/// Live-verdict expectations. `Taken` rows need a proven binary clash
/// (local `PATH` or shell builtin here); `Unknown` rows have an occupied
/// registry name but no proven binary; `Free` rows need every source to
/// agree, so keep them few and gibberish.
const CASES: [Case; 10] = [
    Case {
        name: "test",
        expect: Availability::Taken,
    },
    Case {
        name: "cd",
        expect: Availability::Taken,
    },
    Case {
        name: "ls",
        expect: Availability::Taken,
    },
    Case {
        name: "sh",
        expect: Availability::Taken,
    },
    Case {
        name: "git",
        expect: Availability::Taken,
    },
    Case {
        name: "curl",
        expect: Availability::Taken,
    },
    Case {
        name: "serde",
        expect: Availability::Unknown,
    },
    Case {
        name: "requests",
        expect: Availability::Unknown,
    },
    Case {
        name: "zzqxzqx-not-a-real-binary-987654321",
        expect: Availability::Free,
    },
    Case {
        name: "wqwxvqy-nor-this-one-123456789",
        expect: Availability::Free,
    },
];

/// One `--json` report from the checker, trimmed to what the eval asserts.
#[derive(Debug, serde::Deserialize)]
struct EvalReport {
    verdict: Availability,
    results: Vec<EvalEntry>,
}

/// One per-source row inside [`EvalReport`].
#[derive(Debug, serde::Deserialize)]
struct EvalEntry {
    source: Source,
    availability: Availability,
    detail: String,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match args.task {
        Task::Eval => run_eval(),
    }
}

/// Run every [`CASES`] row against the real CLI and compare verdicts.
fn run_eval() -> ExitCode {
    let mut passed: Vec<bool> = Vec::new();
    for case in CASES {
        passed.push(run_case(&case));
    }
    let failures = passed.iter().filter(|p| !**p).count();
    let total = passed.len();
    if failures == 0 {
        println!("eval: {total}/{total} passed");
        ExitCode::SUCCESS
    } else {
        println!("eval: {failures}/{total} failed");
        ExitCode::FAILURE
    }
}

/// Run one case through `bin-name-cli --json`; return whether it passed.
fn run_case(case: &Case) -> bool {
    let output = match Command::new("cargo")
        .args(["run", "-q", "-p", "bin-name-cli", "--", case.name, "--json"])
        .output()
    {
        Ok(done) => done,
        Err(problem) => {
            println!("FAIL {}: cannot spawn checker: {problem}", case.name);
            return false;
        }
    };
    let stdout = match String::from_utf8(output.stdout) {
        Ok(text) => text,
        Err(problem) => {
            println!("FAIL {}: checker output is not UTF-8: {problem}", case.name);
            return false;
        }
    };
    let report: EvalReport = match serde_json::from_str(&stdout) {
        Ok(parsed) => parsed,
        Err(problem) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!(
                "FAIL {}: unparsable report: {problem} ({stderr})",
                case.name
            );
            return false;
        }
    };
    let want = label(case.expect);
    let got = label(report.verdict);
    if report.verdict == case.expect {
        println!("PASS {}: {got}", case.name);
        true
    } else {
        println!("FAIL {}: want {want}, got {got}", case.name);
        for entry in &report.results {
            let id = entry.source.id();
            let state = label(entry.availability);
            println!("    {id}: {state} - {}", entry.detail);
        }
        false
    }
}

/// Lowercase verdict label matching the checker's JSON vocabulary.
fn label(availability: Availability) -> &'static str {
    match availability {
        Availability::Taken => "taken",
        Availability::Free => "free",
        Availability::Unknown => "unknown",
    }
}
