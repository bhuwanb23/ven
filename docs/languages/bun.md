# Bun in ven

Bun is a **single binary** (like Deno) but uses the **npm registry** for packages (like Node). That makes it the second member of ven's "npm family" — it shares Node's full dependency-intelligence layer.

| Aspect                | Detail |
|-----------------------|--------|
| `ven.toml` key        | `runtime.bun` |
| Install dir           | `~/.ven/bun/<version>/` |
| Source                | `https://github.com/oven-sh/bun/releases/download/bun-v<X.Y.Z>/bun-<target>.zip` |
| Release index         | `https://api.github.com/repos/oven-sh/bun/releases?per_page=100` (filtered to strict `X.Y.Z`) |
| Architectures         | Windows / Linux / macOS · `x64`, `aarch64` |
| Package manager       | `bun add` / `bun remove` (npm registry) |
| Plugin                | `src/plugins/bun.rs` |
| Downloader            | `src/core/bun_install.rs` |

## Install

```bash
ven install bun 1.0.20         # exact
ven install bun 1.0            # latest 1.0.x
ven install bun latest         # newest stable
ven install bun                # interactive picker
```

### Target asset names

| Target            | Asset                       |
|-------------------|-----------------------------|
| Windows x86_64    | `bun-windows-x64.zip`       |
| Windows aarch64   | `bun-windows-aarch64.zip`   |
| Linux x86_64      | `bun-linux-x64.zip`         |
| Linux aarch64     | `bun-linux-aarch64.zip`     |
| macOS x86_64      | `bun-darwin-x64.zip`        |
| macOS aarch64     | `bun-darwin-aarch64.zip`    |

The release zip nests the binary inside a folder; ven's extractor picks just the `bun` / `bun.exe` file and drops it directly into `~/.ven/bun/<version>/`, then `chmod 755`s it on Unix.

> The release listing only retains versions that look like **strict** `X.Y.Z` semver — pre-release tags (`bun-v1.0.0-canary.20231015T140000`) are filtered out so the picker stays sane.

## Activation

```toml
[runtime]
bun = "1.0"
```

When active:

| Variable           | Value                                                    |
|--------------------|----------------------------------------------------------|
| `PATH` (prepended) | `~/.ven/bun/<v>/` (binary lives directly here, not under `bin/`) |
| `VEN_BUN_VERSION`  | Resolved version                                          |

## Packages — `bun add` + dependency intelligence

Bun is treated as an **npm-family** runtime by the intelligence layer (`NpmFamilyAdapter` in `src/intelligence/adapters/npm.rs`). That means everything Node enjoys — pre-install graph simulation, `engines.node` checks, `ven.lock`, `ven check-add`, `ven graph` — also works for Bun projects.

`ven add` itself in Bun mode shells out to `bun add <spec>` rather than `npm install`, but the **analysis** before that is identical to Node:

```bash
ven add hono                    # graph simulation against npm registry, then `bun add hono`
ven add hono@4 --skip-check     # skip the analysis if you're sure
ven check-add hono              # simulate only, no install
ven lock                        # writes a single ven.lock pinned graph
ven sync                        # validates the lock and runs `bun add` per root
```

`ven upgrade <pkg> --apply` runs `bun update <pkg>` and updates `[packages]`.

### Configuration example

```toml
[runtime]
bun = "1.0"

[packages]
hono = "^4.0.0"
zod = "^3.22.0"
"@types/node" = "^20.10.0"
```

## Bun vs Node in ven — when to pick which

- Both use the npm registry, so dependency intelligence works for both.
- A project can declare **only one** of `runtime.node` and `runtime.bun` — the activation precedence in `src/intelligence/adapters/mod.rs::adapter_from_ven_config` picks the first non-empty in a specific order.
- For pure speed (bundling, test running) Bun wins; for ecosystem breadth (native modules, edge cases) Node still wins.

## Common errors

| Symptom                                                                | Cause / fix                                                              |
|------------------------------------------------------------------------|--------------------------------------------------------------------------|
| `Bun <v> is not installed. Run: ven install bun <v>`                   | Pin doesn't match `~/.ven/bun/`.                                         |
| `Unsupported platform for Bun download`                                | Host isn't in the supported asset matrix above.                          |
| `Bun <v> not found` even though it's in `oven-sh/bun` releases         | ven's filter requires strict `X.Y.Z` — release with `-canary` / `-rc` suffixes is skipped. |
| `bun` not on PATH after `cd`                                           | Open a new shell or run `ven-use`; the hook may not have re-fired.       |
