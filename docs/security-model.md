# ven Security Model

This document describes ven's threat model, supply-chain controls, and the
exit-code contract for security tooling.

## Threat model

ven is a **client-side** tool that downloads runtime tarballs and queries
package registries. The core threats it defends against:

| Threat | Defense |
|--------|---------|
| Tampered runtime tarball during transit | SHA-256 verification against upstream sidecar / manifest checksum (Node.js, Python, Go, Rust, Java, Deno, Ruby, Bun) — see `src/core/integrity.rs`. |
| Tampered runtime tarball at rest in the local cache | Re-verified on every fresh download; cached extracts are still smoke-tested (`<bin> --version`). |
| Lockfile drift (`node_modules/` modified out-of-band) | `ven sync --check` reports `MISSING`/`STALE`/`OUT-OF-LOCK`/`MISMATCH` and returns non-zero in CI. |
| Lockfile tampering | Top-level `content_hash` (SHA-256 over canonical payload) plus per-package `integrity` (SRI from npm `dist.integrity`) — both in `ven.lock` v2. |
| Known vulnerable transitive deps | `ven check --security` queries [osv.dev](https://osv.dev) for every locked `(ecosystem, package, version)`. |
| Outdated runtime past upstream EOL | `ven check --eol` queries [endoflife.date](https://endoflife.date) per declared `[runtime]` and flags passed-EOL majors. |
| Undeclared (ghost) source dependencies | `ven scan --ghosts` walks source files (gitignore-aware) and reports anything imported but not declared. |

## What ven does **not** defend against

- Compromise of the upstream registry itself (npm/PyPI/etc.). ven trusts
  the registry's TLS chain plus the SRI integrity it publishes.
- Malicious maintainer publishing a clean-but-evil package. ven tells you
  what's in your tree; supply-chain provenance review is out of scope.
- Local privilege escalation. ven runs entirely as the invoking user.

## Caching strategy & TTLs

| Cache | Backing store | TTL | Stale-on-failure |
|-------|---------------|-----|------------------|
| npm package metadata | `~/.ven/cache/registry.db` | 24 h | n/a |
| Dependency intelligence snapshots | `~/.ven/intelligence.db` (`snapshots`, `package_cache`, `dependency_cache`) | per query | reused |
| **OSV vulnerabilities** | `intelligence.db` (`osv_cache`) | **6 h** | yes — last-known reports served when osv.dev is unreachable |
| **Runtime EOL** | `intelligence.db` (`eol_cache`) | **24 h** | yes |
| **Package docs** | `intelligence.db` (`doc_cache`) | **7 d** (docs rarely change for a fixed version) | implicit |

The `VEN_STORAGE_PATH` env var overrides `~/.ven` for all caches and
storage (used by tests and corporate setups with read-only home dirs).

## Exit-code contract

CI is the primary consumer of these commands. Every security/health command
sets a deterministic exit code so a job can fail fast on regression.

| Command | Returns `0` when… | Returns `1` when… |
|---------|-------------------|---------------------|
| `ven check`            | no HIGH/CRITICAL CVE and no passed-EOL runtime | any HIGH/CRITICAL CVE OR any passed-EOL runtime |
| `ven check --security` | (same, only counts CVEs)                       | any HIGH/CRITICAL CVE |
| `ven check --eol`      | no passed-EOL runtime                          | any passed-EOL runtime |
| `ven scan --ghosts`    | no ghosts (or `--fix` resolved them)           | ghosts found and `--fix` not passed |
| `ven sync --check`     | lock matches `node_modules/` / `pip list`      | any `MISSING`/`STALE`/`OUT-OF-LOCK`/`MISMATCH` (orphans alone do **not** fail) |
| `ven lock`             | wrote a valid lockfile                         | merge conflict (same package resolved to two versions) |

`MODERATE`/`LOW` CVEs are **always** included in the report but do **not**
flip the exit code — they're informational. To fail on them too, post-process
the JSON output (`ven check --json | jq '.security[].vulns[] | select(.severity_label == "MODERATE")'`).

## Disabling network calls

Set `OFFLINE=1` in your env (honored by `reqwest` retries through the system
proxy stack) — caches will continue to serve stale data. There is no full
"airgap" mode; if you need fully offline operation, pre-warm the caches
with `ven check --json` once per project from a connected machine.

## See also

- [`cmds/check.md`](cmds/check.md), [`cmds/scan.md`](cmds/scan.md), [`cmds/lock.md`](cmds/lock.md), [`cmds/sync.md`](cmds/sync.md)
- Source modules: `src/core/osv.rs`, `src/core/endoflife.rs`, `src/core/ghost_scanner.rs`, `src/intelligence/drift.rs`, `src/intelligence/ven_lock.rs`
