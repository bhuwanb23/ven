# `ven` — Complete Feature Reference

A category-by-category roll-up of every feature in the current `ven` codebase, with the **exact** command syntax for each. Use this page as a top-down map; deep dives live in the per-language and per-command pages linked at the end of each section.

> **Conventions in this doc**
> - Commands shown like `ven add <pkg>` mean "type the literal `ven add` and replace `<pkg>` with your value".
> - Anything in `[brackets]` is optional.
> - All commands are also reachable via `ven --help` and `ven <cmd> --help`.

---

## 1. Multi-language runtime management

`ven` installs and manages **8** language runtimes from their official sources, with SHA-256 verification (where the upstream provides checksums) and post-install binary validation.

| Runtime    | Source                                | Per-language doc                              |
|------------|---------------------------------------|------------------------------------------------|
| Node.js    | `nodejs.org/dist` (SHA-256 verified)  | [`languages/node.md`](languages/node.md)       |
| Python     | `python.org/ftp` (Windows embeddable) | [`languages/python.md`](languages/python.md)   |
| Go         | `go.dev/dl`                           | [`languages/go.md`](languages/go.md)           |
| Rust       | `rustup-init` from `static.rust-lang.org` | [`languages/rust.md`](languages/rust.md)   |
| Java (JDK) | Adoptium Temurin                      | [`languages/java.md`](languages/java.md)       |
| Deno       | `github.com/denoland/deno/releases`   | [`languages/deno.md`](languages/deno.md)       |
| Bun        | `github.com/oven-sh/bun/releases`     | [`languages/bun.md`](languages/bun.md)         |
| Ruby (MRI) | RubyInstaller2 (Win) / `ruby/ruby-builder` (Unix) | [`languages/ruby.md`](languages/ruby.md) |

### Capabilities

- Install **any** version available upstream
- **Multiple versions** of the same runtime coexist side-by-side under `~/.ven/<lang>/<version>/`
- **Alias resolution**: `latest`, `lts`, `stable` (Rust), bare major (`20`), bare major.minor (`20.11`), exact (`20.11.0`)
- **SHA-256 verification** for runtimes whose upstream publishes checksums (Node.js)
- **Binary validation** after install — install fails if the expected `node` / `python` / `go` / etc. is not present in the bin path
- **Storage override** via `VEN_STORAGE_PATH` env var (defaults to `~/.ven`)

### Commands

```bash
ven install <lang> <version>     # exact: ven install node 20.11.0
ven install <lang> <alias>       # alias: ven install python 3.12, ven install rust stable
ven install <lang>               # show & pick from upstream release list
ven install                      # full interactive picker (language + version)

ven list                         # all runtimes + their installed versions
ven list <lang>                  # filter to one runtime, e.g. ven list node
ven list --verbose               # add disk usage + install date
ven list --json                  # machine-readable

# Switch the active runtime in your shell session by editing ven.toml
# (then `ven-use` or just `cd .` triggers the hook). There is no
# `ven use <lang> <version>` — pinning lives in `[runtime]`.
```

See: [`cmds/install.md`](cmds/install.md) · [`cmds/list.md`](cmds/list.md)

---

## 2. Project configuration — `ven.toml`

The project manifest. ven discovers it by **walking up the directory tree** (Git-style) from the working directory.

### Sections

| Section      | Purpose                                                                  |
|--------------|--------------------------------------------------------------------------|
| `[runtime]`  | Per-language version pins (`node`, `python`, `go`, `rust`, `java`, `deno`, `bun`, `ruby`) |
| `[packages]` | Project dependencies (typed by the primary runtime — see § 6)             |
| `[env]`      | Custom environment variables, applied after PATH/runtime vars             |
| `[venv]`     | Python virtual-environment behavior (`auto_path = true|false`)            |

### How resolution works

1. `find_ven_toml` walks **up** from `cwd` looking for the nearest `ven.toml`.
2. Each non-empty `[runtime]` field is resolved against **installed** versions under `~/.ven/<lang>/`.
3. Missing toolchains produce a clean `MissingToolchain { language, install_with }` error pointing at the right `ven install` to run.

### Full example

```toml
[runtime]
node   = "20"
python = "3.12"

[packages]
express = "^4.18.2"
fastapi = ">=0.110"

[env]
NODE_ENV = "development"
PORT     = "3000"

[venv]
auto_path = true
```

See: [`ven-toml.md`](ven-toml.md)

---

## 3. Automatic shell activation

Each shell hook re-runs on every `cd` (or every prompt, depending on the shell), looks for `ven.toml`, and overlays the resolved environment **for that terminal only**. Hooks are:

| Shell                | Trigger                                                          |
|----------------------|------------------------------------------------------------------|
| **bash / zsh**       | `cd` is overridden to call `__ven_activate` after every change   |
| **fish**             | `fish_prompt` event handler                                      |
| **PowerShell** (5.1, 7) | `Set-Location` is wrapped **and** `prompt` calls `__ven_activate` (covers Cursor / VS Code hosts where the prompt event alone misses) |

> **Windows `cmd.exe` is not a hook target.** Use PowerShell, or use `ven-launcher` to spawn a fully-pre-loaded shell from anywhere.

### Per-terminal isolation

Two terminals can run different ven projects (different Node versions, different Python venvs, different `JAVA_HOME`, …) **simultaneously** with zero cross-talk. The overlay touches only that process's environment block.

### Variables exported during activation

Every activation prepends each resolved bin dir to `PATH` (overlay first, then your original `PATH`), then exports a runtime-specific tail set:

| Runtime  | Path prepend                              | Extra env vars exported                   |
|----------|-------------------------------------------|--------------------------------------------|
| Node     | `~/.ven/node/<v>/[bin]`                   | `VEN_NODE_VERSION`, `NODE_PATH`            |
| Python   | venv `bin`/`Scripts` first, then ven-managed | `VEN_PYTHON_VERSION`, `VIRTUAL_ENV`     |
| Go       | `~/.ven/go/<v>/bin`                       | `VEN_GO_VERSION`, `GOROOT`, `GOPATH`       |
| Rust     | `~/.ven/rust/<v>/bin`                     | `VEN_RUST_VERSION`, `CARGO_HOME`, `RUSTUP_HOME` |
| Java     | `~/.ven/java/<v>/bin`                     | `VEN_JAVA_VERSION`, `JAVA_HOME`            |
| Deno     | `~/.ven/deno/<v>/`                        | `VEN_DENO_VERSION`                         |
| Bun      | `~/.ven/bun/<v>/`                         | `VEN_BUN_VERSION`                          |
| Ruby     | `~/.ven/ruby/<v>/bin`                     | `VEN_RUBY_VERSION`, `GEM_HOME`, `GEM_PATH` |
| (always) |                                           | `VEN_TOML` (path that won)                 |
| (always) |                                           | every `[env]` key from `ven.toml`          |

### Commands

```bash
ven setup                        # one-time: install hook into your default shell rc/profile
ven shell install                # same idea, called explicitly (PowerShell or bash/zsh)
ven shell hook bash              # print bash hook script (for manual `eval`)
ven shell hook zsh
ven shell hook fish
ven shell hook powershell
ven shell activate <dir>         # print shell exports for <dir> (used by hooks)
ven shell deactivate             # print exports that undo the overlay
ven use [DIR]                    # alias for `ven shell activate` (default: `.`)
ven deactivate                   # alias for `ven shell deactivate`
```

After `ven setup` completes you also get two **shell helpers** defined inside the hook:

- **`ven-use`** — apply (or re-apply) the overlay for the current directory in the current shell session.
- **`ven deactivate`** — set `VEN_SKIP_PROJECT_VENV=1` to pause the auto-prepend of `./venv` until the next `ven-use`.

See: [`shell-integration.md`](shell-integration.md) · [`cmds/setup.md`](cmds/setup.md) · [`cmds/shell.md`](cmds/shell.md)

---

## 4. Project initialization

`ven init` is an interactive wizard with three modes:

| Mode                          | Flag(s)                       | What it does                                                  |
|-------------------------------|-------------------------------|---------------------------------------------------------------|
| **Plain**                     | (none)                        | Pick language, pick installed version, write minimal `ven.toml`. |
| **Template**                  | `--template`                  | Same plus a curated template (Express API, React+Vite, Next.js full-stack, Empty). Each pre-fills `[packages]`. |
| **With packages**             | `--with-packages`             | After language/version, multi-select from a curated popular-packages list. |
| **Validate after init**       | `--validate`                  | Run health checks (runtime present? venv present? packages declared?) right after writing the file. |

### Python-specific behavior

When you pick Python during `ven init`:

1. ven creates **`./venv`** (preferred) using `python -m venv --copies`. If the chosen Python lacks the stdlib `venv` module (common with Windows embeddable builds), it `pip install`s `virtualenv` and uses that instead.
2. Forces `include-system-site-packages = false` in the generated `pyvenv.cfg` so the venv stays isolated.
3. Appends `venv/` and `.venv/` to `.gitignore` if missing.

### Commands

```bash
ven init                         # plain
ven init --template              # interactive template selection
ven init --with-packages         # interactive package picker
ven init --validate              # run health checks after init
ven init --node 20               # legacy back-compat: pre-fill node = "20"
```

See: [`cmds/init.md`](cmds/init.md)

---

## 5. Status & observability

`ven status` renders the resolved state of the current project tree.

### Three views

| Flag             | Use case                                                    |
|------------------|-------------------------------------------------------------|
| (default)        | One-screen summary: discovered `ven.toml`, runtimes, packages, env hint, health line |
| `--verbose` / `-v` | Adds disk usage, package compatibility (when applicable), full env-var listing, hint lines for fixes |
| `--json`         | Machine-readable; designed for CI / `jq` consumption        |
| `--fix`          | Where supported, auto-installs missing packages / venv      |

### What it reports

- The `ven.toml` path that won
- Each `[runtime]` key + the resolved installed version (or "missing — run `ven install …`")
- `[packages]` count + per-package state
- `[env]` keys that activation would inject
- Last persisted dependency-intelligence snapshot (when Node/Bun)
- Whether you're inside the activated overlay (`VEN_TOML` env presence)

### Commands

```bash
ven status                       # basic
ven status --verbose             # detailed
ven status --json                # machine-readable
ven status --fix                 # auto-fix where possible
```

See: [`cmds/status.md`](cmds/status.md)

---

## 6. Unified package management

`ven add`, `ven remove`, `ven upgrade` route to the **right native tool** based on which `[runtime]` is set, and keep both the native manifest **and** `[packages]` in `ven.toml` in sync.

### Per-runtime routing

| Primary runtime | `ven add <pkg>` does                                                         | Manifest synced               |
|-----------------|-------------------------------------------------------------------------------|--------------------------------|
| **Node.js**     | Full **dependency-intelligence simulation** → `npm install <pkg>@<resolved>` | `package.json` + `ven.toml`    |
| **Bun**         | Same intelligence layer → `bun add <pkg>`                                     | `package.json` + `ven.toml`    |
| **Python**      | `<resolved python> -m pip install <spec>`                                     | `requirements.txt` + `ven.toml` |
| **Ruby**        | `bundle add` if `Gemfile` present, else `gem install --no-document`           | `Gemfile` (if present) + `ven.toml` |
| **Rust**        | `cargo init` (if missing), then `cargo add <spec>`                            | `Cargo.toml` + `ven.toml`      |
| **Go**          | `go mod init` (if missing), then `go get <spec>`                              | `go.mod` + `ven.toml`          |
| **Java**        | Notice — use Maven / Gradle. ven still records the pin in `[packages]`.       | `ven.toml`                     |
| **Deno**        | Notice — edit `deno.json` / imports.                                          | `ven.toml`                     |

### Dependency intelligence (npm family)

For Node and Bun specifically, `ven add` runs a full **pre-install** simulation:

1. Builds the complete transitive dependency graph from `registry.npmjs.org` (with a **24-hour SQLite cache** at `~/.ven/cache/registry.db`).
2. Checks every node's `engines.node` against the active Node.
3. Walks semver constraints to detect peer / pin conflicts with what's already in `[packages]`.
4. Prints a structured conflict report with **fix options** (downgrade X, upgrade Y, install alternative version, cancel).
5. Persists a `SimulationResult` snapshot to `~/.ven/intelligence.db` keyed by canonicalized project path.
6. Only **then** runs `npm install` / `bun add`.

`ven check-add <pkg>` runs steps 1-4 only — no install. `ven graph` reads the last persisted snapshot.

### Commands

```bash
# Add
ven add <pkg>                    # latest compatible
ven add <pkg>@<version>          # exact / spec
ven add <pkg> --dry-run          # preview only
ven add <pkg> --skip-check       # bypass intelligence (Node/Bun only)
ven add <pkg> --verbose          # show full dep tree

# Pre-install analysis (Node/Bun)
ven check-add <pkg>              # simulate, don't install
ven check-add <pkg>@<v> --json   # machine-readable

# Remove
ven remove <pkg>                 # safe (warns about dependents)
ven remove <pkg> --force         # skip dependent check
ven remove <pkg> --dry-run       # preview
ven remove --cleanup             # find & remove orphans
ven remove <pkg> --json          # machine-readable

# Upgrade
ven upgrade                      # preview all (no install)
ven upgrade <pkg>                # preview one
ven upgrade <pkg> --apply        # actually upgrade one
ven upgrade --all                # preview every package in ven.toml
ven upgrade --all --apply        # apply all
ven upgrade --all --apply --force  # CI mode, no prompts
ven upgrade --json               # machine-readable

# Reverse lookup (Node)
ven why <pkg>                    # who depends on this?

# Reproducibility (Node/Bun)
ven lock                         # write ven.lock v2 with SRI integrity + content_hash
ven sync                         # validate ven.lock + install pins
ven sync --dry-run               # validate only; print install plan; exit 0
ven sync --check                 # CI mode — drift report; exit non-zero on drift
ven sync --check --json          # machine-readable drift report (CI)
ven sync --skip-validate         # install without re-checking the lock

# Auto-fix
ven resolve                      # find & apply optimal version set
```

#### Lockfile (`ven.lock`) — what's in it

| Field                  | Meaning                                                                              |
|------------------------|--------------------------------------------------------------------------------------|
| `lock_format_version`  | `2` for new locks; v1 still readable (no `integrity` field)                          |
| `ecosystem`            | `npm` (only ecosystem with full intelligence today)                                  |
| `runtime_kind`         | `NpmFamily` for Node/Bun                                                             |
| `runtime_version`      | The pinned `[runtime] node` (or `bun`)                                               |
| `roots`                | Sorted list of `[packages]` keys                                                     |
| `packages[name]`       | `{ version, integrity?: "sha512-..."|"sha256-...", metadata? }`                      |
| `edges[]`              | `{ from, to, constraint, kind: Dependency|Peer|Dev }`                                |
| `content_hash`         | SHA-256 of the canonical payload (with `content_hash` field stripped)                |

#### Drift detection (`ven sync --check`)

`--check` reports five categories: `MISSING`, `STALE`, `OUT-OF-LOCK`, `MISMATCH`, `ORPHAN` — see [`cmds/sync.md`](cmds/sync.md) for the full table. Only the first four flip the exit code; `ORPHAN` is informational (transitive deps live there too).

See: [`cmds/add.md`](cmds/add.md) · [`cmds/remove.md`](cmds/remove.md) · [`cmds/upgrade.md`](cmds/upgrade.md) · [`cmds/check-add.md`](cmds/check-add.md) · [`cmds/graph.md`](cmds/graph.md) · [`cmds/lock.md`](cmds/lock.md) · [`cmds/sync.md`](cmds/sync.md) · [`cmds/resolve.md`](cmds/resolve.md)

---

## 7. Version-resolution engine

The same eight resolvers (`resolve_<lang>_version` in `src/core/config.rs`) are used by activation, `ven init`'s validation, and the verbose status report. They produce **deterministic, cross-language consistent** behavior:

| Spec       | Behavior                                                                       |
|------------|---------------------------------------------------------------------------------|
| `latest`   | Highest installed version                                                       |
| `lts`      | (Node) highest installed with even major number; (others) same as `latest`      |
| `stable`   | (Rust) same as `latest`                                                         |
| `X`        | Highest installed `X.*.*`                                                       |
| `X.Y`      | Highest installed `X.Y.*`                                                       |
| `X.Y.Z`    | Used verbatim — must exist on disk                                              |

If no installed version matches, activation returns `MissingToolchain { language, install_with }` so the shell hook can print a precise install hint instead of failing silently.

At install time (`src/cli/install/fetch.rs`), the same shapes are matched against the **upstream** release index for each language (Node.js dist index, python.org FTP listing, go.dev JSON, GitHub releases for Rust/Deno/Bun/Ruby, Adoptium API for Java).

---

## 8. Environment variable injection

Every activation produces an export script that is **non-destructive and fully reversible**:

1. Captures your original `PATH` once into `__VEN_ORIGINAL_PATH` (POSIX) / `$global:VEN_ORIGINAL_PATH` (PowerShell) at first hook load.
2. Computes the runtime overlay **fresh on each `cd`** (cached by `(dir, ven.toml mtime)` signature so unchanged projects don't re-run resolution).
3. Prepends overlay to `PATH`, exports per-runtime markers (see § 3 table), then your `[env]` keys.
4. On `ven deactivate`: restores `__VEN_ORIGINAL_PATH`, unsets every `VEN_*_VERSION`, `JAVA_HOME`, `GOROOT`, `GOPATH`, `CARGO_HOME`, `RUSTUP_HOME`, `GEM_HOME`, `GEM_PATH`, `VIRTUAL_ENV`, `NODE_PATH`, `VEN_TOML`. Sets `VEN_SKIP_PROJECT_VENV=1` so the next `cd` doesn't auto-re-prepend `./venv`.

### Session-scoped only

ven **never** writes to your shell rc, your machine's `Path` registry, or any system file outside its own profile-line installation in `ven setup`. Every overlay lives in the current process environment block.

### `[env]` overrides

`[env]` keys in `ven.toml` are applied **after** the runtime overlay, so they can reference values ven just set (e.g. `RAILS_ENV`, `DATABASE_URL`). The activation layer **skips** the literal key `PATH` if you put it in `[env]` so you can't accidentally clobber the overlay.

---

## 9. Standalone launcher — `ven-launcher`

A separate binary (`src/bin/launcher.rs`) for **no-touch** project shells.

### What it does

- Walks up from the project path (or `cwd`) to find `ven.toml`.
- Resolves the runtime exactly like activation would.
- Spawns a **new terminal window** whose process environment already contains the overlay — no shell rc, no eval step, no PATH mutation in the parent.

### Why it exists

| Constraint                                                | `ven-launcher` answer                                 |
|-----------------------------------------------------------|--------------------------------------------------------|
| Corporate machine — no admin rights                       | Doesn't need any                                       |
| Locked-down PATH                                          | Doesn't modify it                                       |
| Restricted shell config                                   | Doesn't write to your rc                                |
| Need to ship a desktop / IDE shortcut into a project      | Point the shortcut at `ven-launcher \path\to\project`   |
| Run from a USB stick                                      | The binary is portable; everything lives in `~/.ven/`   |

### Commands

```bash
ven-launcher                     # use cwd as project root
ven-launcher <PATH>              # spawn with that project active
ven-launcher --show-env          # print resolved env instead of spawning
ven-launcher --show-env <PATH>   # same, for a specific project
```

### Per-shell behavior

| Host                     | What `ven-launcher` does                                                                    |
|--------------------------|---------------------------------------------------------------------------------------------|
| Windows + PowerShell     | Spawns `powershell.exe` (or `pwsh`) with `-NoExit -NoLogo -Command <greeting + Set-Location>` |
| Windows + (anything else)| Spawns `cmd.exe /K <generated .cmd>` with the activation env applied                         |
| Unix + bash              | Spawns `bash --init-file <generated init.bash> -i` with the activation env applied           |
| Unix + zsh               | Spawns `zsh --init-file <generated init.zsh> -i` with the activation env applied             |
| Unix + custom `$SHELL`   | Spawns that program with `-i` if it's executable, otherwise prints a clear hint              |

See: [`ven-launcher.md`](ven-launcher.md)

---

## 10. Cross-platform support

| OS              | Activation shells                          | Install paths                                                                 |
|-----------------|--------------------------------------------|-------------------------------------------------------------------------------|
| **Windows**     | PowerShell 5.1, PowerShell 7 (`pwsh`)      | `~/.ven/<lang>/<v>/`; bin dirs vary per runtime (Node = root, Go/Rust/Java/Ruby = `bin/`, Deno/Bun = root) |
| **macOS**       | bash, zsh                                  | Same layout. Ruby uses prebuilt `ruby/ruby-builder` tarballs (`darwin-x64`, `darwin-arm64`)               |
| **Linux**       | bash, zsh, fish                            | Same layout. Ruby builder: `ubuntu-22.04` and `ubuntu-24.04` x64/arm64 only                                |

### Platform-specific binary paths

Where the binary lives inside an install dir varies per language:

| Language          | Windows             | Unix                          |
|-------------------|---------------------|-------------------------------|
| Node              | `<root>\node.exe`   | `<root>/bin/node`             |
| Python (embed)    | `<root>\python.exe` | _(install path Windows-only)_ |
| Go                | `<root>\bin\go.exe` | `<root>/bin/go`               |
| Rust              | `<root>\bin\cargo.exe` | `<root>/bin/cargo`         |
| Java              | `<root>\bin\java.exe`  | `<root>/bin/java`          |
| Deno              | `<root>\deno.exe`   | `<root>/deno`                 |
| Bun               | `<root>\bun.exe`    | `<root>/bun`                  |
| Ruby              | `<root>\bin\ruby.exe` | `<root>/bin/ruby`           |

### Platform-specific shell hooks

Each hook is generated specifically for its target shell — see `src/shell/mod.rs`:

- **bash/zsh hook** wraps `cd` so every directory change re-runs activation.
- **fish hook** uses the `fish_prompt` event (no `cd` override needed).
- **PowerShell hook** wraps `Set-Location` **and** chains into `prompt`, because Cursor / VS Code / IntelliJ-style hosts trigger one or the other but not always both.

### Storage location override

`VEN_STORAGE_PATH` env var overrides `~/.ven` for everything (cache, installs, intelligence DB). Useful for portable USB / network-share setups.

---

## 11. Security & health monitoring

A unified report that combines **package CVEs** (osv.dev) and **runtime end-of-life alerts** (endoflife.date), plus a source-tree **ghost dependency** scanner. All three are CI-safe (deterministic exit codes), pure Rust (cross-platform), and locally cached.

### Sources

| Signal | Source | Endpoint | Cache TTL | Ecosystems |
|--------|--------|----------|-----------|------------|
| CVEs (per package@version) | [osv.dev](https://osv.dev) | `POST /v1/querybatch`, then `GET /v1/vulns/<id>` for severity + summary | 6 h (stale-on-failure) | npm, PyPI, Go, crates.io, Maven, RubyGems, Deno (`npm:` only) |
| Runtime EOL | [endoflife.date](https://endoflife.date) | `GET /api/<product>.json` | 24 h (stale-on-failure) | nodejs, bun, python, go, rust, java, ruby, deno |
| Ghost imports (no network) | local source walk via `ignore` crate | n/a | n/a | All 8 runtimes (per-language extractors) |

### Severity buckets

| CVSS | Bucket | Counts toward CI failure |
|------|--------|--------------------------|
| ≥ 9.0 | CRITICAL | yes |
| ≥ 7.0 | HIGH | yes |
| ≥ 4.0 | MODERATE | no (informational) |
| > 0   | LOW | no |
| (no CVSS, only `database_specific.severity`) | bucketed by label | as above |

EOL: a runtime whose matched cycle has `eol` in the past triggers `[EOL]` and **fails** `ven check`. `[SUPPORT-ENDED]` (active support over but not yet EOL) is **informational**.

### Version pinning for security scans

`ven check --security` always prefers the **lockfile** when present:

1. `ven.lock` (full transitive set + integrity hashes — most accurate)
2. `ven.toml [packages]` roots only (strips `^`/`~`/`=` prefixes)
3. Skips entries pinned to `*` / `latest` / unpinned (no version → no scan)

### Ghost detection

`ven scan --ghosts` walks source files honoring `.gitignore` and a hard skip list (`node_modules`, `target`, `dist`, `build`, `.venv`, `venv`, `__pycache__`, `vendor`, `bower_components`, `.git`, `.idea`, `.vscode`). Per-language extractors and stdlib whitelists live in [`src/core/ghost_scanner.rs`](../src/core/ghost_scanner.rs); see [`cmds/scan.md`](cmds/scan.md) for the full pattern table.

`--fix` writes detected ghosts straight into `ven.toml [packages]` (using `latest` as the spec) so the next `ven add`/`ven sync` can resolve them through the normal intelligence pipeline.

### Commands

```bash
# Combined CVE + EOL report
ven check                           # both
ven check --security                # CVE only
ven check --eol                     # EOL only
ven check --json                    # CI / scripting

# Source-tree scanners
ven scan --ghosts                   # report
ven scan --ghosts --fix             # report + write to ven.toml
ven scan --ghosts --json            # CI / scripting
```

### Exit codes

| Command | Code 0 | Code 1 |
|---------|--------|---------|
| `ven check` (any flag) | no HIGH/CRITICAL CVE and no passed-EOL runtime | otherwise |
| `ven scan --ghosts` | no ghosts (or `--fix` passed) | ghosts found and not fixed |

### Cross-platform

`reqwest` (rustls TLS), `rusqlite` (bundled), `ignore`, `regex` — all
pure-Rust deps. Identical behavior on Windows / macOS / Linux. No shell-out
to `npm audit`, `pip-audit`, etc.

See: [`cmds/check.md`](cmds/check.md) · [`cmds/scan.md`](cmds/scan.md) · [`security-model.md`](security-model.md)

---

## 12. Version-pinned documentation

`ven docs <pkg>` resolves the version pin from `ven.lock` → `ven.toml` → installed manifest, then either renders the package's README/description in your terminal (markdown via [`termimad`](https://crates.io/crates/termimad)), or opens the canonical URL in your default browser.

### Per-ecosystem source

| Ecosystem | Body source | Renderable in terminal? | Canonical URL (`--browser`) |
|-----------|-------------|-------------------------|------------------------------|
| Node / Bun | `registry.npmjs.org/<pkg>/<version>` `.readme` | yes (markdown) | `npmjs.com/package/<pkg>/v/<v>` |
| Python | `pypi.org/pypi/<pkg>/<v>/json` `info.description` | yes (markdown / RST best-effort) | `pypi.org/project/<pkg>/<v>/` |
| Rust | docs.rs HTML | URL only (HTML is too rich) | `docs.rs/<pkg>/<v>/<pkg>/` |
| Go | pkg.go.dev HTML | URL only | `pkg.go.dev/<module>@<v>` |
| Java | javadoc.io | URL only | `javadoc.io/doc/<group>/<artifact>/<v>` |
| Ruby | `rubygems.org/api/v1/gems/<pkg>.json` `info` | short; URL is richer | gem version page |
| Deno | URL passthrough (`npm:`, `jsr:`, deno.land/x) | URL only | inferred from import spec |

Bodies are cached in `intelligence.db` (`doc_cache` table) for 7 days — docs rarely change for a fixed version.

### Commands

```bash
ven docs <pkg>                      # render in terminal
ven docs <pkg> --browser            # open canonical URL (xdg-open / open / cmd /c start via `webbrowser` crate)
ven docs <pkg> --diff V1 V2         # unified line diff between two versions' READMEs
ven docs <pkg> --json               # machine-readable
```

### Renderer behavior

- **TTY:** `termimad::MadSkin::default().term_text(body)` — markdown rendered with terminal width auto-detection.
- **Non-TTY (CI, pipes):** raw markdown text passes through unchanged so `ven docs … | grep` works.

### `--diff`

Uses [`similar::TextDiff::from_lines`](https://docs.rs/similar) to produce a unified `+/-` diff of the two READMEs. (Per-API surface diff — function/class signatures — is a future stretch goal that would need per-ecosystem AST parsers.)

### Cross-platform

`webbrowser` (`0.8`) opens URLs uniformly via `cmd /c start` on Windows, `open` on macOS, `xdg-open` on Linux. Set `VEN_BROWSER_DRY_RUN=1` to print the URL instead of spawning (used by tests).

See: [`cmds/docs.md`](cmds/docs.md)

---

## Quick command index (everything in one place)

| Category | Commands |
|----------|----------|
| **Runtime mgmt** | `ven install <lang> [version]` · `ven list [lang] [--verbose|--json]` |
| **Project init** | `ven init [--template] [--with-packages] [--validate]` |
| **Status**       | `ven status [--verbose|--json|--fix]` |
| **Packages**     | `ven add <pkg>[@v] [--dry-run|--skip-check|--verbose]` · `ven remove [<pkg>] [--force|--dry-run|--cleanup|--json|--verbose]` · `ven upgrade [<pkg>] [--apply|--all|--dry-run|--force|--json|--verbose]` · `ven check-add <pkg>` · `ven why <pkg>` · `ven graph [--json|--resolve]` |
| **Lockfile**     | `ven lock` · `ven sync [--dry-run|--check|--json|--skip-validate]` · `ven resolve` |
| **Health**       | `ven check [--security|--eol] [--json]` · `ven scan [--ghosts] [--fix] [--json]` |
| **Docs**         | `ven docs <pkg> [--browser|--diff V1 V2|--json]` |
| **Activation**   | `ven use [DIR]` · `ven deactivate` · `ven setup` · `ven shell hook <shell>` · `ven shell install` · `ven shell activate <DIR>` · `ven shell deactivate` |
| **Spawn**        | `ven-launcher [PATH] [--show-env]` |

For granular flags and per-command examples, see [`cmds/INDEX.md`](cmds/INDEX.md).
