# `ven sync`

Restore dependencies from **`ven.lock`** with **validation** before any install.

## What it does

1. Reads **`ven.lock`** from the project directory.
2. Verifies **`content_hash`** when present (detects corruption or hand edits).
3. Runs **structural validation**: roots exist, every edge endpoint matches locked versions, semver constraints hold on dependency/peer edges, no orphan package rows.
4. Rebuilds a graph and runs **peer / ven.toml pin** checks (`analyze_npm_graph`); reports warning count.
5. Upserts rows into SQLite **`package_cache`** and **`dependency_cache`** (under `~/.ven/intelligence.db`), and records **`lock_validations`**.
6. Runs **`npm install <root>@<version>`** for each root in the lock (unless `--dry-run`).

## Usage

```bash
ven sync
ven sync --dry-run
ven sync --json
ven sync --skip-validate   # not recommended
```

If **`ven.lock`** is missing, run **`ven lock`** first.

## See also

- [`ven lock`](lock.md)
- [Command reference: dependency intelligence](../commands-reference.md#dependency-intelligence)
