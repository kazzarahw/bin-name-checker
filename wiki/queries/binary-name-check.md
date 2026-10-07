# binary-name-check

Filed-back answer: how to check whether a binary name is taken, and what this
repo implements.

## Why no single tool exists

Binary names have no central registry (unlike domains or `npm`/`crates.io`
package names). Collisions are local (`PATH`), per-distro (which package ships
`/usr/bin/<name>`), per-ecosystem (which `npm`/`cargo`/`pip` package installs
a binary of that name), and mindshare (a GitHub project with the same name).
No one HTTP call answers all four.

## Layers (cheapest signal first)

1. **Local `PATH`.** `command -v`, `type -a`, or scan `PATH` for an executable
   file. Catches what would actually shadow the new binary on this machine.
2. **OS aggregators.** [Repology](https://repology.org/api) tracks 100+
   repositories by _package/project_ name — best global distro signal in one
   call. Exact binary-file accuracy needs per-distro file lists
   (`Contents-amd64.gz`, `pacman -F`, `dnf repoquery -l`); deferred as high
   cost, low marginal gain.
3. **Language registries.** `crates.io`, `npm`, `PyPI` (plus `rubygems`, Go
   proxy if needed) each have a trivial `GET`-by-name JSON API:
   `200` = taken, `404` = free.
4. **Forges.** GitHub/GitLab/Codeberg repo-name search. Low collision signal
   (a repo name is not a binary name), high mindshare signal. Rate-limited;
   treat as advisory.

## Core gap: package name != binary name

Example observed live: `rg` is `free` on Repology (the package is `ripgrep`)
but `taken` on `crates.io`/`npm`/`PyPI`/GitHub and on local `PATH`. Any
checker built on package indexes can only advise, never guarantee.

## What `bin-name-checker` implements

- `bin-name-core`: `BinaryName` validation, `Source::{id,query_url,interpret}`,
  `Outcome`, `summarize` (any-`Taken` wins), `SHELL_BUILTINS` +
  `is_shell_builtin`. All pure and unit-tested.
- `bin-name-cli`: `PATH` + builtin checks, blocking HTTPS to Repology,
  `crates.io`, `npm`, `PyPI`, `RubyGems`, Homebrew, GitHub search; human and
  `--json` output; exit `0`/`1`/`2`; `--offline` and `GITHUB_TOKEN` support.
- `xtask eval`: 8 fixed rows against live sources as a manual gate.
