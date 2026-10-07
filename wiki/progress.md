# progress

Living state of the project. Update this file (plus `index.md` and `log.md`)
as part of any change that moves the work forward.

## Current state

Polished past MVP and green across the full gate. `bin-name-checker <name>`
checks local `PATH` + shell builtins + Repology + crates.io + npm + PyPI +
RubyGems + Homebrew + GitHub search, prints per-source verdicts, exits `0`
free / `1` taken / `2` unknown-or-invalid. `--json` for scripting, `--offline`
for local-only, `GITHUB_TOKEN` for GitHub quota.

- Cargo workspace, resolver 3, edition 2024, MSRV 1.93.
- Three crates: `bin-name-core` (pure logic), `bin-name-cli` (the
  `bin-name-checker` binary; all I/O), `xtask` (manual eval gate).
- Strict lint table, `#[expect]`-only suppressions, `clippy.toml` test relaxations.
- Aliases: `fmt-check`, `lint`, `t`, `doc-check`; gates: `deny`, `mutants`, `typos`.
- Deps: `clap` (derive), `reqwest` (`blocking` + `native-tls`), `serde`,
  `serde_json`. `native-tls` chosen over `rustls-tls` to stay inside the
  existing `deny.toml` license allow-list (no policy edit).
- Evals: `cargo run -p xtask -- eval`, 8 fixed rows against live sources,
  currently 8/8.
- llm-wiki live: `research/package-sources.md` records the source survey;
  `architecture.md` records all decisions; template rename done.

## Done

- [x] Scaffold template from `iji` structure.
- [x] Publish as public GitHub template repo; applied `main` ruleset.
- [x] Fixed `ci-success` job: shell variable inside `${{ }}` killed the first
  CI run before any job started. Checks now explicit per job.
- [x] Polish: renamed `app-*` → `bin-name-*` (+ binary `bin-name-checker`);
  real `repository`/description/keywords; version-derived user-agent; added
  RubyGems + Homebrew + shell-builtin sources with tests; `GITHUB_TOKEN` +
  `--offline`; `xtask` eval (8/8 live); `research/package-sources.md` source
  survey; rewrote `README.md`; full gate green.

## Next

1. Optional sources only if they meet the inclusion rule (one-`GET` JSON,
   `200`/`404` semantics, deny-clean, tested): none pending.
2. Optional UX: per-source timeouts, shell completions.

## Open decisions

| # | Question | Notes | Blocks |
| --- | --- | --- | --- |
| 1 | Per-distro binary-file accuracy? | Deferred per inclusion rule; Repology signal enough | — |

## Research needed

- _Survey complete for now; see `research/package-sources.md`. Revisit only
  if a new registry meets the inclusion rule._
