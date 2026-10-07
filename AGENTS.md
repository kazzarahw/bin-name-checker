# AGENTS.md

Project rules for working in this repo. This file is the source of truth for
conventions; where it is silent, prefer the local codebase's existing patterns.
It is also the **schema** for the repo's llm-wiki (see below): it tells the LLM
how the wiki is structured and what workflows to follow.

## What this repo is

A minimal, strictly verified Rust workspace designed for LLM-written programs:

1. **Enforce type safety and strict verification via the compiler.** Types are a
   verification mechanism here, not a style preference. Reach for the type
   system before reaching for a runtime check.
2. **Encourage functional behavior** — function composition, idempotency,
   memoization. Prefer pure functions and immutable data; keep effects at the
   edges.

## Repository layout

```text
crates/
  app-core/    pure logic: types, traits, algorithms. No I/O.
  app-cli/     the `app` binary. All I/O (fs, network, time, env) lives here.
raw/           immutable source material (articles, papers, transcripts).
wiki/          LLM-maintained knowledge base (see "llm-wiki" below).
  index.md       content catalog: every page with a link + one-line summary.
  log.md         append-only chronological record of ingests, queries, lints.
  overview.md    what the project is, goals, non-goals.
  architecture.md  settled design decisions and rationale.
  progress.md    living state: done, next, open decisions, needed research.
  research/    compiled synthesis pages, one per topic (not raw dumps).
  queries/     valuable answers filed back as pages (not chat history).
AGENTS.md      this file: project conventions + wiki schema.
```

Pure logic belongs in library crates so failures localize and the core is
testable without a filesystem or network. If custom tooling logic is ever needed,
add a small `xtask` crate rather than a shell script.

TODO(template): rename `app-core` / `app-cli` / `app` to your project names.
When you do, update `deny.toml` `skip-tree`, `README.md`, and this file.

## Commands

Cargo aliases live in `.cargo/config.toml`.

```sh
cargo fmt-check   # cargo fmt --all --check
cargo fmt         # cargo fmt --all
cargo lint        # clippy, all targets/features, -D warnings
cargo t           # nextest run --workspace --all-features
cargo doc-check   # cargo doc --workspace --no-deps, rustdoc warnings denied
cargo deny check  # dependency policy: advisories, bans, licenses, sources
cargo mutants --in-diff <file>   # mutation testing on a diff file
```

Doctests run separately: `cargo test --doc --workspace`.

`fmt`, `doc`, `deny`, and `mutants` are cargo subcommands backed by installed
binaries and are deliberately not aliased — an alias would shadow the builtin
and eventually become a hard error. `cargo fmt-check` and `cargo doc-check` are
their workspace-wide, strict-mode counterparts.

## llm-wiki

This repo follows Karpathy's llm-wiki pattern
(`https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f`):
a persistent, compounding wiki sits between raw sources and answers, instead of
re-deriving knowledge from scratch on every question. The LLM maintains the
wiki; the human curates sources, steers, and asks good questions.

There are three layers:

- **Raw sources (`raw/`)** — curated, immutable source of truth. Articles,
  papers, transcripts, specs. The LLM reads from `raw/` but never modifies it.
  One file per source: `raw/YYYY-MM-DD-slug.md`. Preserve provenance (URL, date,
  author) at the top.
- **The wiki (`wiki/`)** — LLM-generated markdown. Summaries, entity/concept
  pages, comparisons, synthesis. The LLM owns this layer: it creates pages,
  updates them when new sources arrive, maintains cross-references, and keeps
  everything consistent. Humans read it and steer it; they rarely edit it
  directly.
- **The schema (this file)** — structure, conventions, and workflows below.
  Co-evolve it as you learn what works for this domain.

### Operations

**Ingest.** When a new source lands in `raw/`:

1. Read the source. Discuss key takeaways with the user; note emphasis.
2. Write or update wiki pages: a summary plus updates to every affected
   entity/concept page. One source may touch many pages. Update
   `architecture.md` if a decision settled, `progress.md` if state moved.
3. Update `wiki/index.md` (catalog every touched page).
4. Append one entry to `wiki/log.md` (see format below).
5. Never edit `raw/` during ingest. Never leave `index.md`/`log.md` stale.

Ingest one source at a time with the user involved by default; batch only when
asked. Research synthesis goes in `wiki/research/<topic>.md` — compiled,
cross-referenced, with contradictions flagged — never a raw paste.

**Query.** When the user asks a question:

1. Read `wiki/index.md` first, then drill into relevant pages.
2. Synthesize an answer with citations (relative links to wiki pages/sources).
3. File valuable answers back: comparison, analysis, or discovered connection
   becomes a new page under `wiki/queries/` (or the relevant topic dir),
   indexed in `index.md` and logged in `log.md`. Explorations compound; they do
   not disappear into chat history.

**Lint.** On request, or when the wiki feels stale, health-check it:

- Contradictions between pages; stale claims superseded by newer sources.
- Orphan pages (no inbound links); concepts mentioned but lacking their own page.
- Missing cross-references; data gaps fillable with a targeted search.
- Suggest new questions to investigate and new sources to ingest.

Report findings; fix with user guidance; log the pass.

### Indexing and logging

- **`wiki/index.md`** is content-oriented: every page listed with a relative
  link, a one-line summary, and optional metadata. Organized by section
  (overview, architecture, progress, research, queries). Updated on every ingest,
  query-filed-back, and lint fix.
- **`wiki/log.md`** is chronological and append-only. Never rewrite history.
  One heading per event, parseable prefix:
  `## [YYYY-MM-DD] <op> | <title>`, where `<op>` is
  `ingest`, `query`, `lint`, `decision`, or `progress`.
  `grep "^## \[" wiki/log.md | tail -5` shows the last 5 events.

### Wiki conventions

- One concept per page. Cross-link generously with relative links.
- Note contradictions explicitly; do not silently overwrite. Newer sources
  supersede older ones; record both and why the new one wins.
- Keep `overview.md` (goals), `architecture.md` (settled decisions +
  rationale), and `progress.md` (done / next / open decisions / needed
  research) current. A change that alters behavior or settles a decision
  updates the relevant page in the same commit, plus `index.md` and `log.md`.
- At moderate scale (~100 sources, hundreds of pages) `index.md` is the search
  engine. Add `qmd` or similar only when the index stops being enough.

## Code conventions

### Rust

- MSRV is 1.93; edition 2024. Both are pinned in `Cargo.toml` and
  `rust-toolchain.toml`.
- Lints live in `[workspace.lints]` in the root `Cargo.toml`; every crate opts in
  with `[lints] workspace = true`. Do not redefine lints per crate.
- `unsafe_code` is forbidden, as are `unwrap`, `expect`, `panic`, `todo`,
  `unimplemented`, indexing, `as` conversions, `dbg!`, `mem::forget`, `exit`,
  and arithmetic side effects outside tests.
- Suppress with `#[expect(lint, reason = "...")]`, never `#[allow]`. An
  `expect` that stops being needed becomes a compile error, so suppressions
  cannot linger.
- Stubs carry `#[expect(clippy::todo, reason = "stub: ...")]` on the **enclosing
  function or item**, not on the `todo!()` statement: clippy's `todo` lint is
  reported against the enclosing body, so an attribute on the statement itself
  does not suppress it. Implementing the body makes the expectation unfulfilled
  and the build fails until the attribute is removed, so stubs cannot silently
  become real code.
- Pedantic lints are warnings. If one is noisy, report it rather than silencing
  it.
- Public items get doc comments stating contracts, not restatements of the
  signature.

### Style: simple functional Rust

- Pure functions: owned or borrowed in, owned out, no hidden state, no globals,
  no interior mutability without a justifying comment. Pass clock, RNG, and I/O
  handles as parameters; side effects live in `app-cli`.
- Make invalid states unrepresentable: newtypes for IDs and units, enums over
  bool or string flags, typestate for lifecycle stages.
- Errors are `Result` with `thiserror` enums in libraries. Propagate with `?`.
- Stay in the simple subset by default: explicit lifetimes, no `unsafe`, no
  macros. Use `async` where the task genuinely requires it (streaming provider
  responses and subprocess execution both do) rather than as a default reflex.
- Clone freely rather than fight the borrow checker.

### Testing

- Test behavior, not implementation. When behavior changes, change the tests
  that describe the old behavior.
- Business logic must be testable without a network or a terminal. Use mocks
  and fakes at the `app-cli` boundary.
- No real provider APIs or paid tokens in tests. Record and replay instead.
- Any function claimed idempotent, round-trippable, commutative, or
  order-independent gets a `proptest` asserting that property.
- Check surviving mutants from CI and strengthen the tests that missed them.

## Git

- Commit messages follow Conventional Commits 1.0.0: `<type>[optional scope]:
  <description>`. Common types: `feat`, `fix`, `docs`, `chore`, `refactor`,
  `test`.
- Write descriptions in the imperative mood ("add tool dispatch", not "added").
- Do **not** add `Co-Authored-By` or other attribution trailers.
- **Never push.** The user pushes. Do not run `git push` under any circumstances.
- Stage explicit paths. Avoid `git add -A` / `git add .` so unrelated work in
  progress is never swept into a commit.
- Never rewrite published history. No force pushes, no `git reset --hard` on
  shared branches.
- Update `wiki/progress.md`, `wiki/index.md`, and `wiki/log.md` in the same
  commit as any change that advances the project state. Never edit `raw/`
  after ingest.

## Workflow

Before declaring a task done, run all of these and report the actual results:

```sh
cargo fmt-check
cargo lint
cargo t
cargo test --doc --workspace
cargo doc-check
cargo deny check
typos
```

Never report success while any of them fail.

- Fix root causes. Never relax a lint, weaken or delete a test, or edit a
  CODEOWNERS-protected file to get to green. If a check looks wrong, stop and
  ask.
- When orchestrating larger work: define types, traits, and signatures first as
  stubs (per the `#[expect]` rule above), run `cargo check`, then delegate.
  Each subagent owns one crate or module and must not edit others. Run the full
  gate after merging.

## Working agreements

- Ask before removing functionality or code that looks intentional.
- Ask before making a significant architectural change that forecloses other
  options.
- Prefer maintained dependencies over hand-rolling when they genuinely delete
  owned code and tests.
- Never commit credentials. Secrets belong in environment variables, and `.env`
  files stay untracked.
