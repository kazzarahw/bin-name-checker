# raw/

Immutable source material for the llm-wiki. The LLM reads from here but never
modifies files in place.

## Rules

- One file per source: `YYYY-MM-DD-slug.md`.
- Preserve provenance at the top: title, URL or origin, author, date.
- Paste or clip the full content. Do not summarize here — synthesis happens in
  `wiki/` during ingest.
- Never edit a file after ingest. If the source updates, add a new dated file.
- Images go in `raw/assets/` (download locally; do not rely on remote URLs).

## Ingest flow

1. Drop the file here.
2. Ask the agent to ingest it (see `AGENTS.md` → llm-wiki → Ingest).
3. The agent updates `wiki/` pages, `wiki/index.md`, and appends to `wiki/log.md`.
