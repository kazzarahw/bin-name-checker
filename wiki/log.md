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
