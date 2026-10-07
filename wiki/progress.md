# progress

Living state of the project. Update this file (plus `index.md` and `log.md`)
as part of any change that moves the work forward.

## Current state

Template scaffolded and (to verify) hardened. No application logic yet.

- Cargo workspace, resolver 3, edition 2024, MSRV 1.93.
- Two crates: `app-core` (pure logic) and `app-cli` (all I/O).
- Strict lint table, `#[expect]`-only suppressions, `clippy.toml` test relaxations.
- Aliases: `fmt-check`, `lint`, `t`, `doc-check`; gates: `deny`, `mutants`, `typos`.
- llm-wiki skeleton: `raw/`, `wiki/index.md`, `wiki/log.md`, this file.

## Done

- [x] Scaffold template from `iji` structure.
- [x] Publish as public GitHub template repo; applied `main` ruleset.
- [x] Fixed `ci-success` job: shell variable inside `${{ }}` killed the first
  CI run before any job started. Checks now explicit per job.

## Next

1. Merge `fix/ci-success-job` (blocked on ruleset approval — see Open decisions).
2. Rename crates and set OWNER/REPO (see `README.md` checklist).
2. Rewrite `overview.md` for the real project.
3. Ingest the first real source into `raw/`.
4. Define core types/traits as stubs (`#[expect(clippy::todo)]`), `cargo check`.

## Open decisions

| # | Question | Notes | Blocks |
| --- | --- | --- | --- |
| 1 | _TBD_ | | |

## Research needed

- _List concrete research questions here; compiled answers go in `research/`._
