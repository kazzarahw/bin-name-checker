# package sources

Survey of candidate name-collision sources for `bin-name-checker`: what each
endpoint returns, what was verified live on 2026-10-07, and why it is included
or deferred. Live probes used `curl` with known-taken and gibberish names.

## Included

Red (`Taken`) requires a proven binary clash. Yellow (`Unknown`) means the
name is occupied somewhere without proof about the binary — or the check
itself failed. Green (`Free`) means confirmed absent.

| Source | Signal | Endpoint pattern | Taken | Free | Notes |
| --- | --- | --- | --- | --- | --- |
| Local `PATH` | red iff executable found | scan `PATH` for executable file | file found | not found | Ground truth for this machine; Unix checks the exec bit |
| Shell builtins | red iff listed | static list in `bin-name-core` (`SHELL_BUILTINS`) | list hit | list miss | `test`, `time`, `cd` are shadowed by the shell itself. Offline, pure, testable |
| npm `bin` field | red iff `bin` matches, else yellow | `GET /<name>/latest`, inspect `bin` | `bin` names the binary (`[ships …]` tag) | registry 404 | `[no …]` tag means the name is occupied but ships no such binary |
| Repology | yellow at most | `GET /api/v1/project/<name>` | listed (binary status unknowable) | `[]` or `404` | Aggregates 100+ distro repos in one call; package-name signal, not `/usr/bin/*` truth |
| crates.io | yellow at most | `GET /api/v1/crates/<name>` | listed (may be squat/library) | `404` | A same-named crate may be a squat or a library (observed: `rg` crate disclaims ripgrep) |
| npm registry | yellow at most (plus `bin` tag) | `GET /<name>/latest` | listed | `404` | Uses the small `/latest` doc, not the full metadata |
| PyPI | yellow at most | `GET /pypi/<name>/json` | listed | `404` | No console-script info in the JSON API |
| RubyGems | yellow at most | `GET /api/v1/gems/<name>.json` | listed | `404` | No executable list in the API |
| Homebrew | yellow at most | `GET /api/formula/<name>.json` on `formulae.brew.sh` | listed | `404` | Formula name usually matches its binary, but not always (`ripgrep` ships `rg`) |

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

## Dropped

| Source | Rationale |
| --- | --- |
| GitHub repo search (was included; dropped 2026-10-07) | Scanning every repo is mindshare, not package management — same reason the other forges stayed deferred |

## Standing rule

A new source earns inclusion when it is (a) a one-`GET` JSON check with
`200`/`404`-style semantics, (b) license-clean for `cargo deny`, and
(c) covered by at least one eval row or unit test. Anything needing auth
walls, custom headers per source, or bulk-database mirrors stays deferred.
