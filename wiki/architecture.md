# architecture

Settled design decisions and their rationale. Update this file (plus
`index.md` and `log.md`) in the same commit as any decision. Research context
lives in `research/`; living state lives in `progress.md`.

## Decisions

| Decision | Status | Rationale |
| --- | --- | --- |
| Workspace with `bin-name-core` (pure) + `bin-name-cli` (I/O) + `xtask` (manual gates) | adopted | Failures localize; core testable without fs/network; live-network evals stay out of `cargo t` |
| Strict `[workspace.lints]`, opt-in per crate | adopted | Compiler as verification; see root `Cargo.toml` |
| Suppressions via `#[expect(..., reason)]` only | adopted | Stale suppressions become compile errors |
| llm-wiki (`raw/` + `wiki/`) instead of `docs/` | adopted | Knowledge compounds; see `AGENTS.md` |
| Aggregator-first OS coverage (Repology, not per-distro HTTP) | adopted | Package-name signal covers ~90% of conflicts; per-distro `/usr/bin/*` file lists are high maintenance for little gain ([query](queries/binary-name-check.md)) |
| Blocking `reqwest` + `native-tls`, no `tokio` | adopted | Sequential blocking GETs are fast enough for 5 sources; avoids async runtime and avoids `webpki-roots` `CDLA-Permissive-2.0` license (keeps `cargo deny` green without policy change) |
| `Source::query_url` returns `Option<String>` (`None` = local) | adopted | Invalid states unrepresentable: local checks have no URL by construction |
| Verdict rollup: any-`Taken` > any-`Unknown` > `Free`; empty = `Unknown` | adopted | A single collision vetoes the name; silent `Free` on zero checks would lie |
| Exit codes `0` free / `1` taken / `2` unknown-or-invalid | adopted | Scriptable; `main` returns `ExitCode`, never calls `process::exit` (forbidden lint) |
| Shell builtins as a `Source` (`SHELL_BUILTINS` + `is_shell_builtin`) | adopted | `test`/`time`/`cd` are shadowed by the shell itself — a real collision class the `PATH` scan misses; offline, pure, unit-tested |
| New sources need one-`GET` JSON + `200`/`404` semantics | adopted | RubyGems + Homebrew verified live and added; Go proxy (bare names never resolve), GitLab/Codeberg (duplicate advisory class), Snap (custom header) deferred — see [research](research/package-sources.md) |
| `GITHUB_TOKEN` bearer auth when set; `--offline` flag | adopted | Raises GitHub search quota for evals; local-only mode for firewalled use. Token read from env at runtime, never committed |
| Evals in `crates/xtask` (`cargo run -p xtask -- eval`) | adopted | 8 fixed rows against live sources; manual gate per testing conventions (no network in `#[test]`s) |
| Evidence per source (`Source::evidence` + `Evidence`) | adopted | Table/JSON name the colliding package or repo (`serde` — framework, `a/b` — desc) instead of bare `HTTP 200`; parsed from already-fetched bodies, pure and unit-tested |
| ASCII table via `render_table` + `anstyle`, no table crate | adopted | `[x]`/`[ ]`/`[?]` glyphs plus words (color-blind and grep safe); color only on tty without `NO_COLOR`/`TERM=dumb`; evidence capped at 60 chars; hand-rolled to avoid a heavy table dep |

## Open questions

- _None. The `bin-name-*` rename is done; per-distro file accuracy stays
  deferred per the [research](research/package-sources.md) inclusion rule._
