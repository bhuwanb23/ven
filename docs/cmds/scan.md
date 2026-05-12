# `ven scan`

Source-tree scanners. Today: **ghost dependency detection** — packages
imported in source files but **not declared** in any project manifest.

## What it does

1. Loads `ven.toml` and identifies the **primary runtime**.
2. Walks the project source tree honoring `.gitignore` / `.ignore` (via the
   `ignore` crate). Hard-skips `node_modules`, `target`, `dist`, `build`,
   `.venv`, `venv`, `__pycache__`, `vendor`, `bower_components`, `.git`,
   `.idea`, `.vscode`.
3. For each file matching the runtime's source-file globs, runs a
   per-language extractor.
4. Cross-references each candidate against `ven.toml [packages]` **and**
   the runtime's native manifest(s).
5. Filters out the runtime stdlib, applies a small rename table where
   applicable (e.g. Python `cv2` → `opencv-python`).

## Per-language extractors

| Runtime | Source globs | Pattern(s) | Manifest(s) cross-checked |
|---------|--------------|------------|---------------------------|
| **Node / Bun** | `*.js`, `*.mjs`, `*.cjs`, `*.jsx`, `*.ts`, `*.tsx` | `require('x')`, `import 'x'`, `import {…} from 'x'`, `await import('x')` (scope/subpath aware) | `package.json` (deps + devDeps + peer + optional) |
| **Python**     | `*.py`        | `^\s*(?:from\|import)\s+([\w.]+)` (top-level module → pip name via rename table) | `requirements.txt`, `pyproject.toml [project] dependencies` |
| **Go**         | `*.go`        | `import "spec"` and `import (...)` blocks; first three path segments | `go.mod` |
| **Rust**       | `*.rs`        | `use crate_name::`, `extern crate crate_name` | `Cargo.toml` (dependencies + dev + build) |
| **Java**       | `*.java`      | `import (?:static\s+)?<fqn>;`; skips `java.*`, `javax.*`, `jdk.*`, `sun.*`, `com.sun.*` | `pom.xml` artifactIds, `build.gradle[.kts]` `'group:artifact:version'` |
| **Ruby**       | `*.rb`        | `require 'gem'` (skips `require_relative`) | `Gemfile` |
| **Deno**       | `*.ts`, `*.tsx`, `*.js`, `*.mjs` | `from 'spec'`, `import('spec')` — captures `npm:`, `jsr:`, and URL imports | `deno.json[c]` `imports` map |

## `--fix`

When `--fix` is passed, every detected ghost is added to `ven.toml [packages]`
with the spec `"latest"` via the same `update_ven_toml_packages` helper that
`ven add` uses. From there, the next `ven add`/`ven sync` will resolve and
install them through the normal intelligence pipeline.

## Usage

```bash
ven scan --ghosts                    # report
ven scan --ghosts --fix              # report + add to ven.toml
ven scan --ghosts --json             # CI / scripting
ven scan                             # alias for --ghosts
```

## Exit codes

| Condition | Code |
|-----------|------|
| No ghosts found | `0` |
| Ghosts found, `--fix` not passed | `1` |
| Ghosts found, `--fix` passed (auto-resolved) | `0` |

## Cross-platform

Pure Rust — `ignore` crate plus per-language `regex` extractors. No
shell-out; identical on Windows / macOS / Linux.

## See also

- [`ven check`](check.md) — security + EOL audit
- [security-model.md](../security-model.md)
