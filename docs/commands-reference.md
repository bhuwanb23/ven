# Command reference

Unless noted, paths default to the **current working directory**; many commands walk **up** the tree to find `ven.toml`.

## Global

| Command | Purpose |
|---------|---------|
| `ven --help` | Short overview and examples (`after_help` lists common flows). |
| `ven -V` / `ven --version` | Binary version. |

## Project lifecycle

| Command | Purpose |
|---------|---------|
| `ven init` | Create `ven.toml` (interactive with `--template`). |
| `ven status` | Show resolved runtime, packages/env summary; `--verbose`, `--json`, `--fix`. |
| `ven install <runtime> [version]` | Install a language/toolchain version under ven’s store; `-y` / `--dry-run`; `--verbose`; `-q`. Interactive mode lists versions when version omitted (where supported). |
| `ven list [runtime]` | List installed versions (`runtime` optional filter). |
| `ven use [PATH]` | Print shell exports to apply nearest `ven.toml`; **evaluate** output (`eval "$(ven use)"`, PowerShell: parse stderr hint / use hooks). |
| `ven deactivate` | Print exports that undo `ven use` overlay for current shell session. |
| `ven add <packages…>` | Add npm/PyPI/etc. packages per `[packages]` / runtime rules; updates `ven.toml`. (Rubygems / Bundler: use **`gem`** / **`bundle`** in the activated shell.) |
| `ven check-add <packages…>` | **Dependency intelligence**: simulate an add (peers, pins, engines) **without** installing; `--json`. |
| `ven graph` | Show last persisted simulation graph or a manifest/`node_modules` snapshot; `--json`, `--resolve` (skip SQLite snapshot). |
| `ven lock` | Write **`ven.lock`** (merged resolved graph + `content_hash`) for npm/Node-Bun projects. |
| `ven sync` | Read **`ven.lock`**, validate graph + hash, refresh SQLite package/dependency cache, then **`npm install`** each root; `--dry-run`, `--json`, `--skip-validate`. |
| `ven remove [packages…]` | Remove packages; `--cleanup` removes orphans. |
| `ven upgrade [packages…]` | Upgrade pins; `--all`, `--apply`, `--dry-run`. Uses the same intelligence layer before apply. |

## Dependency intelligence

Pre-install analysis lives in `src/intelligence/`: runtime **adapters** (npm family for Node/Bun; deterministic stubs for Python, Go, Rust, Java, Deno, Ruby), a shared **graph** model, **conflict** explanations, and SQLite under **`~/.ven/intelligence.db`**. `ven status` surfaces the last snapshot when Node/Bun is configured.

**`ven.lock`** (JSON) stores merged pins, edges, and a **`content_hash`** (SHA-256 of the canonical document without the hash field). **`ven sync`** verifies the hash and structural/semver consistency before installing.

**SQLite tables** (same database): `snapshots` (per-project simulation JSON, optional `graph_hash`), `package_cache` (name, version, ecosystem, metadata JSON, `cached_at` — **1 hour** TTL convention for cache freshness), `dependency_cache` (from package/version, to package, constraint string, constraint type, ecosystem, `cached_at`), and `lock_validations` (project key, validation time, graph hash, lock content hash).

## Shell integration

| Command | Purpose |
|---------|---------|
| `ven setup` | Install/update shell hooks and optional profiles (bash/zsh/fish/PowerShell). |

Hidden / advanced:

| Command | Purpose |
|---------|---------|
| `ven shell activate` | Same core behavior as `ven use` (machinery for hooks). |
| `ven shell deactivate` | Same as `ven deactivate`. |
| `ven shell hook …` | Internal hook fragments used by `setup`. |

See [shell-integration.md](shell-integration.md).

## Platform spawn

| Binary | Purpose |
|--------|---------|
| `ven-launcher [PROJECT]` | Open a **new** terminal with env for nearest `ven.toml`; `--show-env` prints resolved env instead. See [ven-launcher.md](ven-launcher.md). |
