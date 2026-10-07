# overview

What this project is. Rewrite me for your project.

## Intent

<!-- TODO(template): one paragraph: what the program does and for whom. -->

A strictly verified Rust program written with an LLM. Pure logic in
`app-core`; all I/O in `app-cli`.

## Goals

1. **Compiler-enforced correctness.** Invalid states unrepresentable; errors as
   `Result` with `thiserror`; no `unwrap`/`expect`/`panic` outside tests.
2. **Functional core.** Pure functions, immutable data, effects at the edges.
3. **Compounding knowledge.** Research, decisions, and progress accumulate in
   this wiki via ingest / query / lint (see `AGENTS.md`), not scattered chat.

## Non-goals

<!-- TODO(template): what this project explicitly will not do. -->
