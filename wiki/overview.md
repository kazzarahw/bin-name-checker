# overview

`bin-name-checker` tells you whether a candidate binary name is already taken,
so you can pick unique names for new CLI projects.

## Intent

A small Rust CLI (`bin-name-checker` binary): given a name like `rg`, it
checks the local `PATH` and shell builtins (offline) plus remote package
indexes over plain HTTPS — Repology (100+ distro repos), `crates.io`, `npm`,
`PyPI`, `RubyGems`, and Homebrew formulae — then prints a per-source verdict
and an overall verdict (`free`/`taken`/`unknown`). Exit code is `0`/`1`/`2`
respectively, with `--json` for scripting and `--offline` for local-only
checks.

## Goals

1. **Compiler-enforced correctness.** Invalid states unrepresentable; errors as
   `Result` with `thiserror`; no `unwrap`/`expect`/`panic` outside tests.
2. **Functional core.** Pure functions, immutable data, effects at the edges:
   name validation, URL building, response interpretation, and verdict rollup
   live in `bin-name-core` with no I/O.
3. **Compounding knowledge.** Research, decisions, and progress accumulate in
   this wiki via ingest / query / lint (see `AGENTS.md`), not scattered chat.

## Non-goals

- Exhaustive per-distro binary-file coverage (`Contents-amd64.gz`, `pacman
  -F`, `dnf repoquery -l`). Repology covers package names, not exact
  `/usr/bin/*` contents; see [architecture](architecture.md).
- Reserving or registering names anywhere. This is a read-only advisory check.
