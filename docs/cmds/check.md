# `ven check`

Combined **health report** for the current project: package CVEs from
[osv.dev](https://osv.dev) **plus** runtime end-of-life status from
[endoflife.date](https://endoflife.date).

## What it does

1. Loads `ven.toml` and identifies the **primary runtime** (Node/Bun → npm,
   Python → PyPI, Go → Go, Rust → crates.io, Java → Maven, Ruby → RubyGems,
   Deno → npm).
2. **Security:** collects pinned `(name, version)` pairs (preferring
   `ven.lock` when present, falling back to `[packages]`), batches them into
   POSTs to `https://api.osv.dev/v1/querybatch` (1000 per call), then
   enriches each returned vuln id from `/v1/vulns/<id>` for severity,
   summary, fixed version, and CVE aliases.
3. **EOL:** GETs `https://endoflife.date/api/<product>.json` per declared
   `[runtime]` key and matches the active major against published cycles.
4. Prints a **human report** by default (or JSON with `--json`), and exits
   non-zero on any **HIGH/CRITICAL** CVE or **passed-EOL** runtime.

Both data sources are cached locally in `~/.ven/intelligence.db`:

| Cache table | TTL | Stale-on-failure? |
|-------------|-----|--------------------|
| `osv_cache` | 6 h | yes (offline-friendly) |
| `eol_cache` | 24 h | yes |

## Usage

```bash
ven check                  # both security and EOL
ven check --security       # CVE only
ven check --eol            # EOL only
ven check --json           # CI / scripting
ven check --json --eol     # combine
```

## Ecosystem coverage

| ven runtime | OSV ecosystem | EOL slug |
|-------------|---------------|----------|
| `node`      | `npm`         | `nodejs` |
| `bun`       | `npm`         | `bun`    |
| `python`    | `PyPI`        | `python` |
| `go`        | `Go`          | `go`     |
| `rust`      | `crates.io`   | `rust`   |
| `java`      | `Maven` (`groupId:artifactId`) | `java` |
| `ruby`      | `RubyGems`    | `ruby`   |
| `deno`      | `npm` (only `npm:` specifiers; URL imports skipped) | `deno` |

## Severity ranks

| Source | Bucket |
|--------|--------|
| CVSS ≥ 9.0 | CRITICAL |
| CVSS ≥ 7.0 | HIGH |
| CVSS ≥ 4.0 | MODERATE |
| CVSS  > 0  | LOW |
| `database_specific.severity` (when no CVSS) | as-published, normalized to the same buckets |

## Exit codes

| Condition | Code |
|-----------|------|
| No actionable issues | `0` |
| Any HIGH/CRITICAL CVE OR a passed-EOL runtime | `1` |

## Cross-platform

All HTTP runs through `reqwest` (rustls TLS), all caching through SQLite
(bundled). No native deps; identical behavior on Windows / macOS / Linux.

## See also

- [`ven scan`](scan.md) — ghost dependency detection
- [security-model.md](../security-model.md) — threat model & exit-code rationale
