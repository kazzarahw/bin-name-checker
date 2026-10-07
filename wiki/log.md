# log

Append-only chronological record of wiki events. Never rewrite history.
Format: `## [YYYY-MM-DD] <op> | <title>`, where `<op>` is `ingest`, `query`,
`lint`, `decision`, or `progress`.

Tip: `grep "^## \[" wiki/log.md | tail -5` shows the last 5 events.

## [2026-10-06] progress | Template scaffolded

- Created from `iji` hardening: workspace lints, toolchain, aliases, CI, deny,
  lefthook, editorconfig, typos.
- Replaced `docs/` with llm-wiki: `raw/` + `wiki/` + schema in `AGENTS.md`.
- Next: rename crates, set OWNER/REPO, rewrite overview/progress for the real
  project, ingest first source.

## [2026-10-06] decision | Fixed ci-success workflow (shell var in expression)

- First CI run on `main` died before any job started ("workflow file issue").
- Root cause: `ci-success` used `${{ needs.$job.result }}` inside a shell loop.
  `needs.<job>` requires a literal job ID; a shell variable is a workflow
  parse error. Rewrote with one explicit `needs.<id>.result` check per job.
- Same bug exists in `iji`'s `ci.yml` (copied verbatim); flag it there.
- `wiki/index.md` intentionally untouched: no pages added or removed.

## [2026-10-06] decision | Solo-friendly ruleset (0 required approvals)

- The `main` ruleset (copied from `iji`) required 1 approving + code-owner
  review. Solo maintainer authoring the PR can never satisfy that: GitHub does
  not count the author's own approval, so every PR deadlocks. Same latent issue
  will hit `iji` when its ruleset is applied.
- Relaxed to `required_approving_review_count: 0`, code-owner/last-push
  approval off. Still enforced: PRs, linear history, `ci-success`, no force
  pushes, no deletion. `.github/rulesets/main.json` updated to match.

## [2026-10-07] query | Binary-name check layers filed back

- Filed `wiki/queries/binary-name-check.md`: why no single registry exists,
  the four layers (local PATH, Repology aggregator, language registries,
  forges), and the package-name != binary-name gap (`rg` vs `ripgrep`).
- Updated `wiki/index.md` to catalog it.

## [2026-10-07] progress | MVP bin-name-checker implemented
- `app-core`: `BinaryName`, `NameError`, `Availability`, `Source`
  (`LocalPath`, `Repology`, `CratesIo`, `Npm`, `Pypi`, `Github`),
  `Outcome`, `summarize`; 7 unit tests.
- `app-cli`: `PATH` scan + blocking HTTPS (Repology, crates.io, npm, PyPI,
  GitHub search), human/`--json` output, exits `0`/`1`/`2`.
- Deps: `clap`, `reqwest` (`blocking` + `native-tls`; `rustls-tls` rejected —
  its `webpki-roots` `CDLA-Permissive-2.0` license fails `cargo deny` and
  `deny.toml` is owner-protected), `serde`, `serde_json`.
- Verified live: `rg` → taken (rc 1), random name → free (rc 0), bad name →
  rc 2. Full gate green: `fmt-check`, `lint`, `t`, `test --doc`, `doc-check`,
  `deny check`, `typos`.
- Rewrote `wiki/overview.md`, recorded decisions in `wiki/architecture.md`,
  updated `wiki/progress.md` + `wiki/index.md`.

## [2026-10-07] decision | Evals live in xtask, never in tests

- `AGENTS.md` Testing section now states: evals needing live network access
  live in `crates/xtask` as manual gates, never as `#[test]`s. Co-evolves the
  schema with the new `xtask` layout entry.

## [2026-10-07] progress | Polish: rename, versions, evals, source gaps

- Renamed `app-core`/`app-cli`/`app` → `bin-name-core`/`bin-name-cli`/
  `bin-name-checker`; updated `deny.toml` `skip-tree`, `README.md`
  (rewritten for the real project), `AGENTS.md`.
- Real `repository` URL (`kazzarahw/bin-name-checker`), description, keywords;
  user-agent now derives from `CARGO_PKG_VERSION`.
- Added RubyGems + Homebrew sources (both verified live 200/404) and a
  `ShellBuiltin` source (`SHELL_BUILTINS` + `is_shell_builtin`); 9 unit tests.
- Added `GITHUB_TOKEN` bearer auth and `--offline` to the CLI.
- Added `crates/xtask` with `eval` (8 fixed rows, live): 8/8 passed.
- Filed `wiki/research/package-sources.md` (included vs deferred + rule);
  deferred: Go proxy, GitLab/Codeberg, Snap, per-distro file lists.
- `cargo mutants -p bin-name-core`: 5 missed → strengthened tests (length
  boundary, `Source::id` stability, GitHub 404), then 1 missed (`state_style`)
  → asserted per-state color codes → 56 caught, 10 unviable, 0 missed
  (17 unit tests).
- Output cleanup: ASCII table (`[x]`/`[ ]`/`[?]` + words, tty-only color via
  `anstyle`, 60-char evidence cap); per-source `Evidence` (package/repo name
  + description) in table and `--json`; npm moved to `/latest` doc;
  `README.md` sample output. 17 unit tests; eval still 8/8.
- CI triage (2026-10-07): red on `origin/main` is pre-existing — the template
  `Initial commit` fails `clippy` under newer stable (`assert!(!x.is_empty())`
  in the scaffold test). Local toolchain synced 1.93.1 → 1.99.0; full gate
  re-verified green. `README.md` gained an Install section.
- Full gate green: `fmt-check`, `lint`, `t`, `test --doc`, `doc-check`,
  `deny check`, `typos`.
