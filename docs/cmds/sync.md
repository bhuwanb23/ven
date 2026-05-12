# `ven sync`

Restore (or audit) dependencies from **`ven.lock`** with **validation** before any install.

## What it does

1. Reads **`ven.lock`** from the project directory.
2. Verifies **`content_hash`** when present (detects corruption or hand edits).
3. Runs **structural validation**: roots exist, every edge endpoint matches locked versions, semver constraints hold on dependency/peer edges, no orphan package rows. v2 lockfiles also validate per-package **`integrity`** strings (`sha256|sha384|sha512-<base64>`).
4. Rebuilds a graph and runs **peer / ven.toml pin** checks (`analyze_npm_graph`); reports warning count.
5. Upserts rows into SQLite **`package_cache`** and **`dependency_cache`** (under `~/.ven/intelligence.db`), and records **`lock_validations`**.
6. Runs **`npm install <root>@<version>`** for each root in the lock (unless `--dry-run` or `--check`).

If the lock is **format v1** (no per-package `integrity`), `ven sync` prints a hint suggesting you regenerate with `ven lock` to gain SRI hashes — but the install still proceeds.

## Three modes

| Flag                | Purpose                                                                                                        | Exit code on issue |
|---------------------|----------------------------------------------------------------------------------------------------------------|---------------------|
| (none)              | Validate, then install pinned roots                                                                            | non-zero only on validation failure or install failure |
| `--dry-run`         | Validate + print install plan; do nothing else                                                                 | 0 unless validation fails |
| **`--check`**       | **CI-safe drift audit**: validate, then compare lock vs `node_modules/` (npm) or installed pip packages (Python). Reports `MISSING`, `STALE`, `OUT-OF-LOCK`, `MISMATCH`, and informational `ORPHAN` categories. | **non-zero on any drift** |

`--check` and `--dry-run` are mutually-additive intentions; `--check` always wins (no install in either case).

### Drift report categories (`--check`)

| Category         | Meaning                                                                                            |
|------------------|----------------------------------------------------------------------------------------------------|
| `MISSING`        | A package pinned in `ven.lock` is not installed in `node_modules/` (or not in `pip list`).         |
| `STALE`          | Installed at a different version than what `ven.lock` pins.                                        |
| `OUT-OF-LOCK`    | A root in `ven.toml [packages]` does not appear in `ven.lock` — run `ven lock`.                    |
| `MISMATCH`       | A root's `ven.toml` constraint cannot be satisfied by the lock pin.                                |
| `ORPHAN`         | Present in `node_modules/` but not in `ven.lock` (informational; usually a transitive dep of a root). |

`ORPHAN` does **not** flip the exit code — only the four actionable categories do.

## Usage

```bash
ven sync                    # validate + install
ven sync --dry-run          # validate + print plan; exit 0
ven sync --check            # CI mode — fail on any drift
ven sync --json             # machine-readable
ven sync --check --json     # combine: emit drift report as JSON, exit non-zero on drift
ven sync --skip-validate    # not recommended; bypass schema/constraint checks
```

If **`ven.lock`** is missing, run **`ven lock`** first.

## Python projects

When `ven.toml` declares `[runtime] python` (and no Node/Bun), `ven sync` switches to **Python mode**:

- `--check` runs `pip list --format=json` against the resolved interpreter (project `./venv` → `./.venv` → `$VIRTUAL_ENV` → `~/.ven/python/<v>/`) and compares each declared `[packages]` / `requirements.txt` entry against what's installed.
- The install path runs `pip install -r requirements.txt` and reconciles pins back into `ven.toml`.

## See also

- [`ven lock`](lock.md)
- [Command reference: dependency intelligence](../commands-reference.md#dependency-intelligence)
