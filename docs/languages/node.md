# Node.js in ven

Node.js is the **flagship runtime** for `ven` — it's where the dependency-intelligence layer is fully implemented, the npm registry cache lives, and `ven.lock` is generated.

| Aspect                | Detail |
|-----------------------|--------|
| `ven.toml` key        | `runtime.node` |
| Install dir           | `~/.ven/node/<version>/` |
| Source                | `https://nodejs.org/dist/v<X.Y.Z>/node-v<X.Y.Z>-<os>-<arch>.{zip,tar.gz}` |
| Checksum              | `SHASUMS256.txt` next to the archive (SHA-256 verified before extract) |
| `latest_version()`    | First entry in `https://nodejs.org/dist/index.json` whose `lts` field is not null |
| Package manager       | npm (full intelligence) |
| Plugin                | `src/plugins/node.rs` |
| Downloader            | `src/core/download.rs` + `src/core/extract.rs` |

## Install

```bash
ven install node 20            # latest 20.x.y
ven install node 20.11.0       # exact
ven install node lts           # latest LTS
ven install node latest        # newest LTS (see notes below)
ven install node               # interactive picker (full release list)
```

### Resolution rules

`resolve_node_version` (used at activation time, against installed versions):

| Spec      | Resolves to                                            |
|-----------|--------------------------------------------------------|
| `latest`  | Highest installed                                       |
| `lts`     | Highest installed with an **even** major number (18, 20, 22, …) |
| `20`      | Highest installed `20.x.y`                              |
| `20.11`   | Used verbatim (exact match required)                    |
| `20.11.0` | Used verbatim                                           |

At install time, `resolve_install_version` (in `src/cli/install/fetch.rs`) hits `nodejs.org/dist/index.json` for major-only specs and returns the highest release in that line.

### Layout after install

```
~/.ven/node/20.20.2/
├── (Windows) node.exe, npm.cmd, npx.cmd, node_modules/, …
└── (Unix)    bin/node, bin/npm, bin/npx, include/, lib/, share/
```

Yes — Node ships its archive **with a trailing folder** (`node-vX.Y.Z-<os>-<arch>/`); `core/extract.rs` flattens that into `~/.ven/node/<version>/` automatically.

### Checksum verification

`core/download.rs::fetch_checksum` reads `SHASUMS256.txt` from the same release directory and matches by exact filename (`node-vX.Y.Z-<os>-<arch>.{ext}`). If the digest doesn't match, the cached archive is removed so the next attempt re-downloads.

If `SHASUMS256.txt` is unreachable, ven prints a warning and continues — the archive is still kept in `~/.ven/.cache/` for re-use.

## Activation

When a project's `ven.toml` declares `runtime.node`:

```toml
[runtime]
node = "20"
```

The shell hook exports:

| Variable             | Value                                                    |
|----------------------|----------------------------------------------------------|
| `PATH` (prepended)   | Windows: `~/.ven/node/<v>/`  ·  Unix: `~/.ven/node/<v>/bin` |
| `NODE_PATH`          | Same bin dir (used by tools that respect it)              |
| `VEN_NODE_VERSION`   | Resolved version (e.g. `20.20.2`)                         |
| `VEN_TOML`           | Absolute path to the `ven.toml` that won                  |

If the resolved version isn't installed, activation aborts cleanly with a message pointing at `ven install node <spec>`.

## Packages — dependency intelligence

When Node is the primary runtime, `ven add` does **not** shell out to npm naively. It:

1. Loads `ven.toml`, picks the npm-family adapter (`NpmFamilyAdapter`).
2. Builds a full transitive dependency graph for each new package, walking the npm registry (with a 24-hour SQLite cache in `~/.ven/cache/registry.db`).
3. Checks every node's `engines.node` against the active Node version.
4. Detects semver conflicts between the new package and what's already in `[packages]`.
5. Persists a `SimulationResult` snapshot keyed by canonicalized project path (`~/.ven/intelligence.db`).
6. Only then runs `npm install <pkg>@<resolved>` and updates `[packages]`.

`ven check-add <pkg>` runs steps 1-4 only — no install. `ven graph` reads the last persisted snapshot. `ven lock` merges every per-root snapshot into a single pinned `ven.lock` with a SHA-256 content hash.

### Configuration example

```toml
[runtime]
node = "20"

[packages]
express = "^4.18.2"
lodash  = "*"
axios   = "^1.6.0"

[env]
NODE_ENV = "development"
PORT     = "3000"
```

## Common errors

| Symptom                                                | Likely cause / fix                                                   |
|--------------------------------------------------------|----------------------------------------------------------------------|
| `Node X is not installed. Run: ven install node X`     | `ven.toml` pins a version not under `~/.ven/node/`. Install it.       |
| `Checksum mismatch! Corrupted download removed.`       | Bad transfer; just re-run `ven install`.                              |
| `engines.node` warnings during `ven add`               | The package needs a different Node than the project's pin. Use `--skip-check` to override, or change the pin. |
| `Cannot find module 'X'` in scripts                    | Confirm `ven status` shows the expected Node + `NODE_PATH`. The hook may not have fired yet — open a new terminal or run `ven-use`. |
