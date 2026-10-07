# progress

Living state of the project. Update this file (plus `index.md` and `log.md`)
as part of any change that moves the work forward.

## Current state

MVP implemented and green across the full gate. `app <name>` checks local
`PATH` + Repology + crates.io + npm + PyPI + GitHub search, prints per-source
verdicts, exits `0` free / `1` taken / `2` unknown-or-invalid.

- Cargo workspace, resolver 3, edition 2024, MSRV 1.93.
- Two crates: `app-core` (pure logic) and `app-cli` (all I/O).
- Strict lint table, `#[expect]`-only suppressions, `clippy.toml` test relaxations.
- Aliases: `fmt-check`, `lint`, `t`, `doc-check`; gates: `deny`, `mutants`, `typos`.
- Deps added: `clap` (derive), `reqwest` (`blocking` + `native-tls`), `serde`,
  `serde_json`. `native-tls` chosen over `rustls-tls` to stay inside the
  existing `deny.toml` license allow-list (no policy edit).
- llm-wiki live: `overview.md` rewritten, `architecture.md` records MVP
  decisions, first filed-back query filed.

## Done

- [x] Scaffold template from `iji` structure.
- [x] Publish as public GitHub template repo; applied `main` ruleset.
- [x] Fixed `ci-success` job: shell variable inside `${{ }}` killed the first
  CI run before any job started. Checks now explicit per job.
- [x] MVP `bin-name-checker`: `BinaryName`, `Source`, `Outcome`, `summarize`
  in `app-core` (7 unit tests); `PATH` + HTTPS checks, human/`--json` output,
  `0`/`1`/`2` exits in `app-cli`. Verified live (`rg` taken, random name
  free). Full gate green: `fmt-check`, `lint`, `t`, `test --doc`, `doc-check`,
  `deny check`, `typos`.

## Next

1. Decide crate/binary rename (`app-*` → `bin-name-*`; template checklist:
   `deny.toml` `skip-tree`, `README.md`, `AGENTS.md`, crate manifests).
2. Optional sources: Homebrew formulae, `rubygems`/Go proxy, GitLab/Codeberg
   advisory search. Each is one `Source` variant + `interpret` + tests.
3. Optional UX: `--offline` (local only), per-source timeouts, shell
   completions.
4. Run `cargo mutants` on the new core logic and strengthen tests.

## Open decisions

| # | Question | Notes | Blocks |
| --- | --- | --- | --- |
| 1 | Rename crates/binary to `bin-name-*`? | Deferred; churn, no behavior gain | — |
| 2 | Per-distro binary-file accuracy? | Repology package-name signal deemed enough for MVP | — |

## Research needed

- Homebrew formulae + Arch/Debian file-list endpoints suitable for exact
  `/usr/bin/*` checks, if decision 2 ever flips. Findings would go in
  `research/`.
