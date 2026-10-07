# bin-name-checker

Check whether a candidate binary name is already taken, so you can pick unique
names for new CLI projects.

```sh
cargo run -q -p bin-name-cli -- <name>        # human output
cargo run -q -p bin-name-cli -- <name> --json # JSON for scripting
```

Exit code is `0` when free, `1` when taken, `2` when unknown or invalid.

## Sources

Each run checks the local `PATH` and shell builtins (offline), plus these
remote indexes over HTTPS:

| Source | Signal |
| --- | --- |
| Repology | 100+ distro repos, aggregated by package name |
| crates.io, npm, PyPI, RubyGems | language registries (`200` = taken, `404` = free) |
| Homebrew formulae | `formulae.brew.sh` (`200` = taken, `404` = free) |
| GitHub repo search | advisory mindshare signal only |

Package names are not binary names (`rg` ships in the `ripgrep` package), so
the verdict is advisory, never a guarantee. See
[`wiki/research/package-sources.md`](wiki/research/package-sources.md) for the
full source survey and inclusion rule.

Set `GITHUB_TOKEN` to raise the GitHub search rate limit; pass `--offline` to
skip remote sources entirely.

## Evals

`cargo run -p xtask -- eval` runs 8 fixed names (6 taken, 2 free) against the
live sources and compares verdicts. It needs network access, so it is a manual
`xtask` gate, not part of `cargo t`. `GITHUB_TOKEN` is picked up automatically
when set.

## Development

Stable Rust (edition 2024, MSRV 1.93; `rustup` fetches the pinned toolchain).

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
