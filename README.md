# ven

> Any-first intelligent runtime + dependency manager.

`ven` is a Rust CLI for project-scoped runtimes and package workflows driven by `ven.toml`.  
It supports Node/Bun/Python/Go/Rust/Java/Deno/Ruby and includes a dependency intelligence layer for pre-install simulation, conflict explanation, graph inspection, and lockfile validation.

---

## Why `ven`

- Project-local runtime and package control with one CLI
- Pre-install dependency intelligence (`check-add`, `graph`, `resolve`)
- Lockfile safety flow (`ven lock` -> `ven sync` with graph validation)
- Cross-platform shell integration (`setup`, `use`, `deactivate`)
- Clean docs and machine-readable outputs (`--json` where it matters)

---

## Supported runtimes

- `node`
- `bun`
- `python`
- `go`
- `rust`
- `java`
- `deno`
- `ruby`

More runtime details: [`docs/languages.md`](docs/languages.md).

---

## Install (from source)

```bash
cargo build --release
```

Binary path:
- `target/release/ven`
- `target/release/ven-launcher`

Optional storage override:
- `VEN_STORAGE_PATH` (defaults to `~/.ven` or `%USERPROFILE%\.ven`)

---

## Quick start

```bash
# 1) Initialize project config
ven init --template

# 2) Install runtime
ven install node 20

# 3) Add packages with simulation-first checks
ven add express axios

# 4) Inspect dependency intelligence graph
ven graph

# 5) Create lockfile and restore safely
ven lock
ven sync
```

---

## Core commands

### Runtime + shell
- `ven install <runtime> [version]`
- `ven list [runtime]`
- `ven setup`
- `ven use [path]`
- `ven deactivate`

### Dependency intelligence
- `ven check-add <pkg[@version]> [--json]`
- `ven graph [--json] [--resolve]`
- `ven why <package>`
- `ven resolve`

### Package lifecycle
- `ven add <pkg...>`
- `ven upgrade <pkg...> [--all] [--apply]`
- `ven remove <pkg...> [--cleanup]`

### Lock + restore
- `ven lock` — writes `ven.lock` with graph + content hash
- `ven sync` — validates `ven.lock` graph/hash before install

Full command reference: [`docs/commands-reference.md`](docs/commands-reference.md).

---

## Dependency intelligence architecture

Main module: `src/intelligence/`

- `engine.rs` — orchestration (`DependencyIntelligenceService`)
- `adapters/` — runtime adapter contract + implementations
- `graph.rs` — normalized graph model
- `conflicts.rs` / `suggestions.rs` — conflict analysis + guidance
- `store.rs` — SQLite persistence (`~/.ven/intelligence.db`)
- `ven_lock.rs` — lockfile schema + validation

This powers `ven add`, `ven upgrade`, `ven check-add`, `ven graph`, `ven lock`, and `ven sync`.

---

## Media (recommended)

Add these files under `docs/media/` and reference them here:

- CLI demo GIF: `docs/media/ven-demo.gif`
- Graph output screenshot: `docs/media/ven-graph.png`
- Lock/sync validation screenshot: `docs/media/ven-sync.png`

Example markdown:

```md
![ven demo](docs/media/ven-demo.gif)
```

---

## Docs map

- Docs index: [`docs/README.md`](docs/README.md)
- Commands: [`docs/commands-reference.md`](docs/commands-reference.md)
- Config schema: [`docs/ven-toml.md`](docs/ven-toml.md)
- Shell integration: [`docs/shell-integration.md`](docs/shell-integration.md)
- Command pages: [`docs/cmds/`](docs/cmds)

---

## Development

```bash
cargo check
cargo test
```

After code changes, update graph metadata:

```bash
graphify update .
```

---

## License

MIT
