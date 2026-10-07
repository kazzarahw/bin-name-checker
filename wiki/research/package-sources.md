# package sources

Survey of candidate name-collision sources for `bin-name-checker`: what each
endpoint returns, what was verified live on 2026-10-07, and why it is included
or deferred. Live probes used `curl` with known-taken and gibberish names.

## Included

| Source | Endpoint pattern | Taken | Free | Notes |
| --- | --- | --- | --- | --- |
| Local `PATH` | scan `PATH` for executable file | file found | not found | Ground truth for this machine; Unix checks the exec bit |
| Shell builtins | static list in `bin-name-core` (`SHELL_BUILTINS`) | list hit | list miss | New gap closed 2026-10-07: `test`, `time`, `cd` are shadowed by the shell itself. Offline, pure, testable |
| Repology | `GET /api/v1/project/<name>` | non-empty JSON array | `[]` or `404` | Aggregates 100+ distro repos in one call; package-name signal, not `/usr/bin/*` truth |
| crates.io | `GET /api/v1/crates/<name>` | `2xx` | `404` | |
| npm | `GET /registry.npmjs.org/<name>` | `2xx` | `404` | |
| PyPI | `GET /pypi/<name>/json` | `2xx` | `404` | |
| RubyGems | `GET /api/v1/gems/<name>.json` | `200` | `404` | Verified live (`rails` 200, gibberish 404); same trivial pattern as the others |
| Homebrew | `GET /api/formula/<name>.json` on `formulae.brew.sh` | `200` | `404` | Verified live (`wget` 200, gibberish 404) |
| GitHub | `GET /search/repositories?q=<name>+in:name` | `total_count > 0` | `total_count == 0` | Advisory mindshare signal only; a repo name is not a binary. Unauthenticated: 10 req/min — `GITHUB_TOKEN` raises it |

## Deferred

| Candidate | Finding | Rationale |
| --- | --- | --- |
| Go module proxy | `proxy.golang.org/<name>/@latest` 404s for bare names | Go modules are paths (`github.com/…`); `BinaryName` rejects `/`, so there is almost never a signal |
| GitLab project search | `GET /api/v4/projects?search=` works unauthenticated (200, JSON array) | Same advisory repo-name class as GitHub with its own rate limit; one forge representative is enough (KISS) |
| Codeberg search | `GET /api/v1/repos/search?q=` works unauthenticated (`{"ok":true,"data":[…]}`) | Same as GitLab: duplicate signal class, extra failure mode |
| Snap Store | `GET /v2/snaps/info/<name>` works with `Snap-Device-Series: 16` header | Snap-name != binary-name; custom header per source adds special cases |
| Per-distro file lists | Debian `Contents-*`, `pacman -F`, `dnf repoquery -l` | True `/usr/bin/*` accuracy, but one mirror format per distro family — high maintenance for marginal gain over Repology |
| pkgs.org / command-not-found.com | No free JSON API for file-level lookup | Manual-useful, not machine-checkable |
| Man pages / shell completions | Weak signal, no clean index | Mindshare at best; skip |

## Standing rule

A new source earns inclusion when it is (a) a one-`GET` JSON check with
`200`/`404`-style semantics, (b) license-clean for `cargo deny`, and
(c) covered by at least one eval row or unit test. Anything needing auth
walls, custom headers per source, or bulk-database mirrors stays deferred.
