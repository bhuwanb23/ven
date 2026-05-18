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
| `ven install <runtime> [version]` | Install a language/toolchain version under ven’s store; `-y` / `--dry-run`; `--verbose`; `-q`. Every install verifies a **SHA256 checksum** (sidecar / manifest / vendor API per language) and runs a **post-install binary smoke test** before declaring success. Interactive mode lists versions when version omitted (where supported). |
| `ven list [runtime]` | List installed versions (`runtime` optional filter). |
| `ven delete [runtime] [version]` | Delete an installed runtime directory under `$VEN_HOME/<runtime>/<version>/`. Complement of `ven remove` (which targets packages). With no args, opens a wizard: pick language → pick version → confirm. With `<runtime>` only, jumps straight to the version picker for that language. Refuses to delete the version currently resolved by the nearest `ven.toml` unless `--force` is passed (prevents silently breaking the next `cd` activation). Flags: `-y` / `--yes` (skip confirm), `--force` (delete active), `--json` (requires explicit args + `-y`). |
| `ven use [PATH]` | Print shell exports to apply nearest `ven.toml`; **evaluate** output (`eval "$(ven use)"`, PowerShell: parse stderr hint / use hooks). |
| `ven deactivate` | Print exports that undo `ven use` overlay for current shell session. |
| `ven add <packages…>` | Unified add. Calls the native package manager **and** updates the language-native manifest **and** `ven.toml [packages]`: `package.json` (Node/Bun), `requirements.txt` (Python), `Gemfile` (Ruby — uses `bundle add` when present, otherwise direct edit + `gem install`), `pom.xml` / `build.gradle[.kts]` (Java — accepts Maven coords `group:artifact[@version]`), `deno.json` `imports` (Deno — prefers `deno add` ≥ 1.42), `go.mod` (Go — `go get`), `Cargo.toml` (Rust — `cargo add`). |
| `ven check-add <packages…>` | **Dependency intelligence**: simulate an add (peers, pins, engines) **without** installing; `--json`. |
| `ven graph` | Show last persisted simulation graph or a manifest/`node_modules` snapshot; `--json`, `--resolve` (skip SQLite snapshot). |
| `ven lock` | Write **`ven.lock`** (merged resolved graph + `content_hash`) for npm/Node-Bun projects. |
| `ven sync` | Read **`ven.lock`** (Node/Bun), validate graph + hash, refresh SQLite package/dependency cache, then **`npm install`** each root. For Python projects (`ven.toml` declares `python` only), runs `pip install -r requirements.txt` and reconciles `[packages]`. `--dry-run`, `--json`, `--skip-validate`. |
| `ven remove [packages…]` | Unified remove. Mirrors `ven add`: native uninstall + manifest + `ven.toml` cleanup for every supported language (Python `pip uninstall` + `requirements.txt`, Ruby `bundle remove` / `gem uninstall` + `Gemfile`, Java `pom.xml` / `build.gradle[.kts]`, Deno `deno.json` `imports`, Go `go get pkg@none` + `go mod tidy`, Rust `cargo remove`). `--cleanup` removes orphans. |
| `ven upgrade [packages…]` | Unified upgrade. Mirrors `ven add`: native upgrade + manifest + `ven.toml` for every supported language (Python `pip install --upgrade` + `requirements.txt`, Ruby `bundle update` / `gem install`, Java `pom.xml` / `build.gradle[.kts]`, Deno `deno.json` `imports`, Go `go get -u` + `go mod tidy`, Rust `cargo update -p`). `--all`, `--apply`, `--dry-run`. Uses the same intelligence layer before apply. |

## Dependency intelligence

Pre-install analysis lives in `src/intelligence/`: runtime **adapters** (npm family for Node/Bun; deterministic stubs for Python, Go, Rust, Java, Deno, Ruby), a shared **graph** model, **conflict** explanations, and SQLite under **`~/.ven/intelligence.db`**. `ven status` surfaces the last snapshot when Node/Bun is configured.

**`ven.lock`** (JSON) stores merged pins, edges, and a **`content_hash`** (SHA-256 of the canonical document without the hash field). **`ven sync`** verifies the hash and structural/semver consistency before installing.

**SQLite tables** (same database): `snapshots` (per-project simulation JSON, optional `graph_hash`), `package_cache` (name, version, ecosystem, metadata JSON, `cached_at` — **1 hour** TTL convention for cache freshness), `dependency_cache` (from package/version, to package, constraint string, constraint type, ecosystem, `cached_at`), and `lock_validations` (project key, validation time, graph hash, lock content hash).

## Shell integration

| Command | Purpose |
|---------|---------|
| `ven setup` | Install/update shell hooks and optional profiles. **Supported shells**: bash, zsh, fish, PowerShell (5.1+ and 7+). Windows `cmd.exe` is **not** a supported activation target — use PowerShell or `ven-launcher` for portable invocation. |

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

## Maintenance

| Command | Purpose |
|---------|---------|
| `ven update [--check\|--version <tag>\|-y\|--force\|--json]` | Self-update `ven` + `ven-launcher` to the latest GitHub release. Verifies SHA256, swaps both binaries in place (Windows: rename-aside trick; Unix: unlink + write), and auto-elevates (UAC / `sudo`) when the install dir requires admin. Distinct from `ven upgrade`, which touches **project packages**. See [cmds/update.md](cmds/update.md). |

## Installer (cross-platform)

| Binary | Purpose |
|--------|---------|
| `ven-setup` | Single self-contained installer. `ven` and `ven-launcher` are **embedded as bytes** via `build.rs` + `include_bytes!`, extracted at install time, then `PATH` is wired up and `ven setup` runs the shell hooks. **Windows**: `--mode user` (`%USERPROFILE%\.ven\bin`, no admin) or `--mode system` (`%ProgramFiles%\ven\bin`, UAC). **Unix**: `--mode user` (`~/.ven/bin` + rc-file PATH block) or `--mode system` (`/usr/local/bin` + `/etc/profile.d/ven.sh`, requires `sudo`). Supports `--dry-run` and `--no-input`. See [cmds/ven-setup.md](cmds/ven-setup.md). |
