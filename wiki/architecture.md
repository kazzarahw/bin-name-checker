# architecture

Settled design decisions and their rationale. Update this file (plus
`index.md` and `log.md`) in the same commit as any decision. Research context
lives in `research/`; living state lives in `progress.md`.

## Decisions

| Decision | Status | Rationale |
| --- | --- | --- |
| Workspace with `app-core` (pure) + `app-cli` (I/O) | adopted | Failures localize; core testable without fs/network |
| Strict `[workspace.lints]`, opt-in per crate | adopted | Compiler as verification; see root `Cargo.toml` |
| Suppressions via `#[expect(..., reason)]` only | adopted | Stale suppressions become compile errors |
| llm-wiki (`raw/` + `wiki/`) instead of `docs/` | adopted | Knowledge compounds; see `AGENTS.md` |
| Aggregator-first OS coverage (Repology, not per-distro HTTP) | adopted | Package-name signal covers ~90% of conflicts; per-distro `/usr/bin/*` file lists are high maintenance for little gain ([query](queries/binary-name-check.md)) |
| Blocking `reqwest` + `native-tls`, no `tokio` | adopted | Sequential blocking GETs are fast enough for 5 sources; avoids async runtime and avoids `webpki-roots` `CDLA-Permissive-2.0` license (keeps `cargo deny` green without policy change) |
| `Source::query_url` returns `Option<String>` (`None` = local) | adopted | Invalid states unrepresentable: local checks have no URL by construction |
| Verdict rollup: any-`Taken` > any-`Unknown` > `Free`; empty = `Unknown` | adopted | A single collision vetoes the name; silent `Free` on zero checks would lie |
| Exit codes `0` free / `1` taken / `2` unknown-or-invalid | adopted | Scriptable; `main` returns `ExitCode`, never calls `process::exit` (forbidden lint) |

## Open questions

- Rename `app-core` / `app-cli` / `app` to `bin-name-*` per template checklist?
  Deferred: churn with no behavior gain (see [progress](progress.md)).
