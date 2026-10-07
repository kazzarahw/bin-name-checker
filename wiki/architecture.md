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

## Open questions

- _None yet. Move settled rows to the table above with rationale._
