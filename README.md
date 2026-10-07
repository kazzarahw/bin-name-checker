# bin-name-checker

Check whether a candidate binary name is already taken, so you can pick unique
names for new CLI projects.

## Install

Requires stable Rust (edition 2024, MSRV 1.93; `rustup` fetches the pinned
toolchain automatically):

```sh
cargo install --path crates/bin-name-cli
bin-name-checker <name>        # human output
bin-name-checker <name> --json # JSON for scripting
```

Or run without installing:

```sh
cargo run -q -p bin-name-cli -- <name>        # human output
cargo run -q -p bin-name-cli -- <name> --json # JSON for scripting
```

Exit code is `0` when free, `1` when taken, `2` when unknown or invalid.

```text
$ bin-name-checker test
Checking 'test':
source         state        detail
-------------  -----------  ------------------------------------------------------------
local-path     [x] taken    found at /usr/sbin/test
shell-builtin  [x] taken    shell builtin or reserved keyword
repology       [?] unknown  1 package - pld
crates-io      [ ] free
npm            [x] taken    test - Node.js 18's node:test, as an npm package [ships `...
...
verdict: taken
```

Red (`[x] taken`) means a proven binary clash — an executable on `PATH`, a
shell builtin, or an `npm` package whose `bin` field ships that exact binary.
Yellow (`[?] unknown`) means the name is occupied somewhere without proof
about the binary (a registry package, a distro package, or a failed check) —
read the detail column and judge for yourself. Green (`[ ] free`) means
confirmed absent. Colors show on a terminal; pipes and `NO_COLOR` get plain
ASCII. `--json` reports the same per source (including `evidence`). See
[`wiki/research/package-sources.md`](wiki/research/package-sources.md) for the
per-source signal survey.

## Sources

Each run checks the local `PATH` and shell builtins (offline), plus these
remote indexes over HTTPS:

| Source | Signal |
| --- | --- |
| Repology | 100+ distro repos, aggregated by package name |
| crates.io, npm, PyPI, RubyGems | language registries (`200` = taken, `404` = free) |
| Homebrew formulae | `formulae.brew.sh` (`200` = taken, `404` = free) |

Package names are not binary names (`rg` ships in the `ripgrep` package), so
the verdict is advisory, never a guarantee. See
[`wiki/research/package-sources.md`](wiki/research/package-sources.md) for the
full source survey and inclusion rule.

Pass `--offline` to skip remote sources entirely.

## Evals

`cargo run -p xtask -- eval` runs 10 fixed names (6 taken, 2 unknown, 2 free)
against the live sources and compares verdicts. It needs network access, so
it is a manual `xtask` gate, not part of `cargo t`.

## Development

`rustup` fetches the pinned toolchain automatically (see above).

```sh
cargo fmt-check
cargo lint
cargo t
cargo test --doc --workspace
cargo doc-check
cargo deny check
typos
```

Aliases live in `.cargo/config.toml`. Layout and wiki workflows are defined in
[`AGENTS.md`](AGENTS.md): pure logic in `crates/bin-name-core`, all I/O in
`crates/bin-name-cli`, project tooling in `crates/xtask`, compounding
knowledge in `wiki/` (`raw/` sources are immutable).

## License

MIT. See [LICENSE](LICENSE).
