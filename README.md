<div align="center">

<img src="assets/Ven_logo.png" alt="Ven Logo" width="120" />

# ven

### The Intelligent Version & Dependency Manager

*One tool. Every language. Zero configuration overhead.*

[![Build Status](https://img.shields.io/github/actions/workflow/status/bhuwanb23/ven/ci.yml?style=flat-square&logo=github)](https://github.com/bhuwanb23/ven/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/github/v/release/bhuwanb23/ven?style=flat-square)](https://github.com/bhuwanb23/ven/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)]()
[![Stars](https://img.shields.io/github/stars/bhuwanb23/ven?style=flat-square)](https://github.com/bhuwanb23/ven/stargazers)

---

**ven** answers the question no other tool can answer — before installing a single byte:

> *"I have Node 20, express 4.18, and lodash 4.17 installed.*
> *Which version of axios works with all of them?"*

[**Get Started**](#-quick-start) •
[**Features**](#-features) •
[**Documentation**](#-documentation) •
[**Languages**](#-supported-languages) •
[**Contributing**](#-contributing)

---

![ven demo](assets/ven-demo.gif)

</div>

---

## The Problem

Every developer knows this pain:

```bash
npm install axios        # works
npm install lodash       # works  
npm install express      # 💥 conflict
# now spend 2 hours debugging
````

Every existing tool — npm, pip, nvm, pyenv, mise — is **reactive**.
They install first. They break second. They never explain why.

**ven is predictive.** It analyzes your entire dependency graph before touching your environment. It tells you what will break, why it will break, and exactly how to fix it.

---

## What Makes ven Different

|                                  | npm / pip | nvm / pyenv | mise / asdf | **ven** |
| -------------------------------- | :-------: | :---------: | :---------: | :-----: |
| Runtime version management       |     ❌     |      ✅      |      ✅      |    ✅    |
| Package management               |     ✅     |      ❌      |      ❌      |    ✅    |
| Auto-switch on `cd`              |     ❌     |      ❌      |      ✅      |    ✅    |
| Pre-install compatibility check  |     ❌     |      ❌      |      ❌      |    ✅    |
| Dependency graph analysis        |     ❌     |      ❌      |      ❌      |    ✅    |
| Lock-file with content hash      |     ✅     |      ❌      |      ❌      |    ✅    |
| Per-terminal isolation           |     ❌     |      ➖      |      ➖      |    ✅    |
| Standalone portable launcher     |     ❌     |      ❌      |      ❌      |    ✅    |
| No admin rights required         |     ➖     |      ➖      |      ➖      |    ✅    |
| Multi-language unified interface |     ❌     |      ❌      |      ✅      |    ✅    |

---

## ✨ Features

### 🔁 Automatic Environment Switching

Walk into a project. The right runtime activates. Walk out. It deactivates.
No commands. No activation scripts. No mental overhead.

```bash
cd ~/projects/frontend      # → Node 20.20.2 activates
cd ~/projects/backend       # → Node 22.11.0 + Python 3.11 activates  
cd ~/projects/systems       # → Rust 1.75.0 activates
cd ~                        # → back to system defaults
```

Every terminal session is independent. Run Node 20 in one tab, Node 22 in
another. Simultaneously. No conflicts.

---

### 📦 8 Languages. One Interface.

```bash
ven install node 20          ven install python 3.11
ven install go 1.21          ven install rust 1.75
ven install java 17          ven install ruby 3.2
ven install deno 1.40        ven install bun 1.0
```

Same command. Every language. Official sources. SHA256 verified.

---

### 🧠 Dependency Graph Intelligence

The feature no other tool has.

ven builds a complete dependency graph of your environment and simulates
every change before making it. No more surprise conflicts.

```bash
$ ven add lodash

→ Building dependency graph...
→ Simulating lodash@4.17 against current stack...

✗ Conflict detected:

  lodash@4.17 requires express@~1.2.0
  └── you have express@1.3.0
      └── constraint fails: ~1.2.0 ≠ 1.3.0

Options:
  [1] Install lodash@4.16  (supports express@1.3)
  [2] Downgrade express to 1.2  (enables lodash@4.17)
  [3] Cancel

Choose [1/2/3]:
```

---

### 🔍 Pre-Install Compatibility Resolver

Know which version works before you install it.

```bash
$ ven check-add axios

  Stack: Node 20.20.2 | express@1.3.0 | lodash@4.16.0

  COMPATIBLE
  ✓ axios@1.6.8   ← recommended
  ✓ axios@1.6.7
  ✓ axios@1.6.6

  INCOMPATIBLE
  ✗ axios@1.7.0   Node >=21 required
  ✗ axios@1.7.1   Node >=21 required

  Install: ven add axios@1.6.8
```

---

### 👥 Team Sync & Reproducibility

One command. Exact environment. Every machine.

```bash
# Developer creates lock file
ven lock

# Teammate clones repo
git clone https://github.com/org/project
cd project
ven sync

  Reading ven.lock...
  Node 20.20.2 — downloading...  ✓
  Installing 34 packages...      ✓
  Verifying SHA256 hashes...     ✓

  Environment ready: Node 20.20.2 | 34 packages
```

---

### 🏢 Corporate Friendly — No Admin Required

ven works in restricted enterprise environments where system PATH
modification and admin privileges are unavailable.

```bash
# Portable. No admin. No setup.
.\ven-launcher.exe
```

`ven-launcher` spawns a terminal with the correct environment pre-loaded.
Nothing is written to system config. Nothing requires elevation.

---

### 📊 Full Dependency Visibility

```bash
$ ven graph

  Dependency Graph: my-app
  Runtime: Node 20.20.2

  ├── express@1.3.0
  │   ├── body-parser@1.20.2
  │   │   ├── bytes@3.1.2
  │   │   └── qs@6.11.0
  │   ├── accepts@1.3.8
  │   └── ms@2.1.3
  │
  ├── lodash@4.16.0
  │
  └── axios@1.6.8
      ├── follow-redirects@1.15.4
      └── form-data@4.0.0

  Packages: 12 | Conflicts: 0 | CVEs: 0
```

---

## 🚀 Quick Start

### Install ven

**Windows (PowerShell 5.1+)**
```powershell
# user install (interactive prompt if TTY; defaults to "user" when piped)
irm https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.ps1 | iex

# explicit system install
$env:VEN_INSTALL_MODE='system'; irm https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.ps1 | iex
```

**Windows (offline / corporate — bundled installer)**

Download the release zip, extract, and run `ven-setup.exe`. Two modes are available:

| Mode | Install dir | Admin? |
|------|-------------|--------|
| `--mode user` *(default prompt)* | `%USERPROFILE%\.ven\bin` | No |
| `--mode system`                  | `%ProgramFiles%\ven\bin` | Yes (UAC) |

`ven-setup` copies the binaries, updates the appropriate `PATH` scope (HKCU or HKLM Machine), broadcasts `WM_SETTINGCHANGE`, installs shell hooks, and verifies `ven --version`. Run with `--dry-run` to preview every step. See [docs/cmds/ven-setup.md](docs/cmds/ven-setup.md).

**macOS / Linux**
```bash
# user install
curl -fsSL https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.sh | sh

# system install (requires sudo on Unix; no UAC equivalent)
sudo VEN_INSTALL_MODE=system bash -c "curl -fsSL https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.sh | sh -s -- --mode system"
```

> The full env-var / flag surface and the release asset naming contract live in [docs/install-scripts.md](docs/install-scripts.md). A short `get.ven.sh` host will be wired up once the domain is provisioned; until then the `raw.githubusercontent.com` URLs above are the canonical entry points.

**From source (Rust)**
```bash
git clone https://github.com/bhuwanb23/ven
cd ven
cargo build --release
```

---

### Setup Shell Integration

```bash
ven setup
```

Restart your terminal. That's it.

---

### Initialize a Project

```bash
mkdir my-app && cd my-app
ven init
```

```
? Select runtime:  Node.js
? Version:         20
? Add packages?    Yes
? Packages:        express, lodash

✓ Created ven.toml
✓ Node 20.20.2 activated
```

---

### Add Packages (Safely)

```bash
ven add express
ven add axios
ven check-add lodash     # check before installing
```

---

## 📋 Supported Languages

Every install pulls from the **official source** for the language, verifies a
**SHA256 checksum**, and runs a **post-install binary smoke test** before the
runtime is registered as installed.

| Language   | Install | Package commands keep this in sync | Auto-Switch | Status  |
| ---------- | :-----: | ---------------------------------- | :---------: | :-----: |
| Node.js    | ✅      | `package.json` (npm)               | ✅          | Stable  |
| Bun        | ✅      | `package.json` (bun/npm)           | ✅          | Stable  |
| Python     | ✅      | `requirements.txt` (pip)           | ✅          | Stable  |
| Ruby       | ✅      | `Gemfile` (bundle/gem)             | ✅          | Stable  |
| Java (JDK) | ✅      | `pom.xml` / `build.gradle[.kts]`   | ✅          | Stable  |
| Deno       | ✅      | `deno.json` `imports`              | ✅          | Stable  |
| Go         | ✅      | `go.mod` (`go get`)                | ✅          | Stable  |
| Rust       | ✅      | `Cargo.toml` (`cargo add/remove`)  | ✅          | Stable  |
| PHP        | 🔜      | composer                           | 🔜          | Planned |
| Elixir     | 🔜      | mix                                | 🔜          | Planned |
| .NET       | 🔜      | nuget                              | 🔜          | Planned |

`ven add`, `ven remove`, and `ven upgrade` work uniformly across every
**Stable** language — they invoke the native package manager and keep both the
language-native manifest **and** `ven.toml [packages]` in sync. `ven status
--fix` will auto-install a missing runtime for any of them, not just Node.

---

## 📁 Project Configuration

### `ven.toml`

```toml
[runtime]
node = "20"           # major alias — resolves to 20.20.2
python = "3.11"       # or exact: "3.11.5"

[packages]
express = "4.18.2"
lodash  = "*"
axios   = "^1.6.0"

[env]
NODE_ENV = "development"
PORT     = "3000"

[venv]
auto_path = true      # Prepend ./venv/{Scripts,bin} on activation when present
```

### `ven.lock`

JSON. Format **v2** ships SRI integrity hashes copied straight from the npm
registry's `dist.integrity` (`sha512-...` for current packages). v1 lockfiles
load without `integrity`; `ven sync` prints a hint to regenerate.

```json
{
  "lock_format_version": 2,
  "ecosystem": "npm",
  "runtime_kind": "NpmFamily",
  "runtime_version": "20",
  "roots": ["express"],
  "packages": {
    "express": {
      "version": "4.18.2",
      "integrity": "sha512-5/PsL6iGPdfQ/lKM1UuielYgv3BUoJfz1aUwU9vHZ+J7gyvwdQXFEBIEIaxeGf0GIcreATNyBExtalisDbuMqQ=="
    }
  },
  "edges": [],
  "content_hash": "<sha-256 of canonical payload>"
}
```

`ven sync --check` then audits this lock against `node_modules/` (or
installed pip packages for Python projects), reporting `MISSING`, `STALE`,
`OUT-OF-LOCK`, `MISMATCH`, and informational `ORPHAN` drift. Non-zero exit
on drift makes it CI-safe.

---

## 🖥️ Command Reference

> Pinning a runtime version is done by editing `[runtime]` in `ven.toml` (the
> nearest one wins), then re-entering the directory or running `ven-use`. There
> is no `ven use <lang> <version>` — versions are project-scoped, not global.

### Runtime Management

```bash
ven install <lang> <version>    # ven install node 20.11.0
ven install <lang> <alias>      # latest | lts | stable | bare-major (20)
ven install <lang>              # interactive picker for that lang
ven install                     # full interactive picker
ven list                        # all installed runtimes
ven list <lang> [--verbose|--json]
ven status [--verbose|--json|--fix]
```

### Package Operations

```bash
ven add <package>               # install + manifest sync + ven.toml sync
ven add <package>@<version>     # exact / spec
ven add <package> --dry-run     # preview only
ven add <package> --skip-check  # bypass intelligence (Node/Bun)
ven remove <package>            # safe uninstall
ven remove --cleanup            # find & remove orphans
ven upgrade                     # preview every package
ven upgrade <package>           # preview one
ven upgrade <package> --apply   # actually upgrade
ven upgrade --all --apply       # apply all
```

### Dependency Intelligence (Node.js / Bun)

```bash
ven check-add <package>         # pre-install simulation, no install
ven check-add <package> --json  # machine-readable
ven why <package>               # reverse dependency lookup
ven graph                       # last persisted simulation graph
ven graph --json                # machine-readable
ven resolve                     # find & apply optimal version set
```

### Lockfile & Reproducibility (Node.js / Bun)

```bash
ven lock                        # write ven.lock v2 (SRI integrity + content_hash)
ven sync                        # validate ven.lock + install pins
ven sync --dry-run              # validate, print plan, exit 0
ven sync --check                # CI mode — drift report; exit non-zero on drift
ven sync --check --json         # machine-readable drift report
ven sync --skip-validate        # install without re-checking the lock
```

> For Python projects, `ven sync` runs `pip install -r requirements.txt`
> against the project venv. Other runtimes carry no extra `ven sync` logic
> beyond runtime resolution.

### Shell Integration

```bash
ven setup                       # one-time hook install (PowerShell or bash/zsh rc)
ven shell install               # explicit form of the above
ven shell hook <shell>          # print hook script for: bash | zsh | fish | powershell
ven shell activate <DIR>        # print activation exports for DIR
ven shell deactivate            # print exports that undo the overlay
ven use [DIR]                   # alias for `ven shell activate` (default: .)
ven deactivate                  # alias for `ven shell deactivate`
```

### Standalone Launcher (no admin, no PATH edits)

```bash
ven-launcher                    # spawn shell with cwd's ven.toml pre-loaded
ven-launcher <PATH>             # spawn shell with PATH's ven.toml pre-loaded
ven-launcher --show-env [PATH]  # print resolved env instead of spawning
```

---

## 🗂️ Storage Layout

```
~/.ven/                  ← override with $VEN_STORAGE_PATH
├── node/
│   ├── 20.20.2/        ← node, npm, npx
│   └── 22.11.0/
├── python/
│   ├── 3.11.5/         ← python, pip (Windows embeddable)
│   └── 3.12.0/
├── go/
│   └── 1.21.5/         ← go binary + GOROOT
├── rust/
│   └── 1.75.0/         ← rustc, cargo (isolated CARGO_HOME/RUSTUP_HOME)
├── java/
│   └── 17.0.9/         ← JDK, JAVA_HOME
├── ruby/
│   └── 3.2.2/          ← ruby, gem (isolated GEM_HOME/GEM_PATH)
├── deno/
│   └── 1.40.0/         ← single deno binary
├── bun/
│   └── 1.0.20/         ← single bun binary
├── bin/                ← ven, ven-launcher, ven-setup
└── cache/
    ├── registry.db     ← SQLite: npm registry metadata (24h TTL)
    └── intelligence.db ← SQLite: per-project simulation snapshots
```

---

## 🌍 How Auto-Switching Works

```
You type: cd ~/projects/frontend

Shell hook fires:
  1. Walk up from cwd looking for the nearest ven.toml
  2. Read [runtime]: node = "20"
  3. Resolve "20" against installed versions → 20.20.2
  4. Locate ~/.ven/node/20.20.2/
  5. Prepend to PATH (this terminal only; original PATH cached once)
  6. Export VEN_NODE_VERSION, NODE_PATH, VEN_TOML
  7. Apply every [env] key from ven.toml

You type: node --version
  → v20.20.2

You type: cd ~/projects/backend

Shell hook fires again:
  1. Find new ven.toml: node = "22", python = "3.11"
  2. Recompute overlay (cached by directory + toml-mtime signature)
  3. Re-prepend new bin dirs, swap VEN_*_VERSION markers
  4. Re-apply that project's [env]

You type: node --version
  → v22.11.0
```

**Per-terminal isolation:**
Terminal 1 → Node 20 | Terminal 2 → Node 22 | No conflicts.
Each terminal owns its own process environment block; `ven` never writes to
your shell rc, your registry `Path`, or any other system-level location.

---

## 🏢 Enterprise / Corporate Usage

### Problem

Many corporate machines restrict:
- System PATH modification
- Shell config editing
- Admin privileges

### Solution: ven-launcher

```
┌────────────────────────────────────────────┐
│   Ven Terminal                             │
│   Project: my-app                          │
│   Runtime: Node 20.20.2 | Python 3.11.5   │
├────────────────────────────────────────────┤
│ $ node --version                           │
│ v20.20.2                                   │
│                                            │
│ $ ven add express                          │
│ ✓ express@4.18.2 installed                 │
│                                            │
│ $ _                                        │
└────────────────────────────────────────────┘
```

- No system modifications
- No admin rights
- No shell config changes
- Portable (run from USB drive)
- Just double-click and code

---

## ⚡ Performance

| Operation | Time |
| --------- | ---- |
| Environment switch (cd) | <50ms |
| First compatibility check (network) | ~1-2s |
| Cached compatibility check | <100ms |
| Full dependency graph build | ~200ms |
| Cached graph lookup | <50ms |
| `ven status` | <100ms |

No containers. No VMs. Just PATH manipulation.

---

## 🔐 Security Model

- Runtime downloads verified with **SHA-256 checksums** where the upstream
  publishes them (Node.js does; others fall back to upstream HTTPS).
- **Official sources only** — `nodejs.org`, `python.org`, `go.dev`,
  `static.rust-lang.org`, Adoptium, GitHub releases (Deno / Bun /
  RubyInstaller2 / ruby-builder).
- Each install runs a **post-install binary smoke test** before the runtime
  is registered as available — failures don't pollute `~/.ven/`.
- Lock-file integrity via a **deterministic content hash** over the merged
  resolved graph **plus** per-package SRI hashes (sha512/sha256) copied from
  npm `dist.integrity` in `ven.lock` v2.
- `ven sync --check` performs **drift detection** against `node_modules/`
  (or installed pip packages) — non-zero exit on any mismatch makes it
  CI-safe.
- No telemetry. No phone home. No accounts.

### Built-in security & health (`ven check` / `ven scan`)

| What | Source | Cache TTL | Cross-platform |
|------|--------|-----------|----------------|
| Package CVE scan (8 ecosystems) | [osv.dev](https://osv.dev) `querybatch` | 6 h, stale-on-failure | yes (pure Rust HTTP) |
| Runtime end-of-life alerts | [endoflife.date](https://endoflife.date) | 24 h, stale-on-failure | yes |
| Ghost dependency detection | local source walk (`ignore` crate, gitignore-aware) | n/a | yes (no shell-out) |
| Version-pinned package docs | npm/PyPI/docs.rs/pkg.go.dev/javadoc.io/rubygems/deno | 7 d | yes (`webbrowser` for `--browser`, `termimad` for terminal render) |

```bash
ven check                          # CVE + EOL combined report
ven check --security               # CVE only (npm, PyPI, Go, crates.io, Maven, RubyGems, Deno)
ven check --eol                    # Runtime EOL only
ven scan --ghosts                  # find imports not declared in any manifest
ven scan --ghosts --fix            # add ghosts to ven.toml [packages]
ven docs <pkg>                     # render docs (version-pinned to ven.lock)
ven docs <pkg> --browser           # open canonical URL in default browser
ven docs <pkg> --diff V1 V2        # unified line diff between two versions' READMEs
```

`ven check` exits non-zero on any **HIGH/CRITICAL** CVE or **passed-EOL**
runtime; `ven scan --ghosts` exits non-zero when ghosts are found and
`--fix` was not passed. See [`docs/security-model.md`](docs/security-model.md)
for the full threat model and exit-code contract.

---

## 📖 Documentation

| Topic | Link |
| ----- | ---- |
| **Complete feature reference (all 12 categories)** | [docs/features.md](docs/features.md) |
| Documentation index | [docs/README.md](docs/README.md) |
| Configuration (`ven.toml`) | [docs/ven-toml.md](docs/ven-toml.md) |
| Lockfile (`ven.lock`) | [docs/ven-lock.md](docs/ven-lock.md) |
| Command reference (`ven <cmd>`) | [docs/cmds/INDEX.md](docs/cmds/INDEX.md) |
| Per-language deep dives | [docs/languages.md](docs/languages.md) → [`docs/languages/`](docs/languages/) |
| **Security model** (CVE + EOL + integrity + drift) | [docs/security-model.md](docs/security-model.md) |
| Shell integration | [docs/shell-integration.md](docs/shell-integration.md) |
| Standalone launcher | [docs/ven-launcher.md](docs/ven-launcher.md) |
| Installation (scripts + offline) | [docs/install-scripts.md](docs/install-scripts.md) |
| Contributing | [CONTRIBUTING.md](CONTRIBUTING.md) |

---

## 🤝 Contributing

Contributions are welcome.

```bash
# Clone
git clone https://github.com/bhuwanb23/ven
cd ven

# Build
cargo build

# Test
cargo test

# Run
cargo run -- install node 20
```

### Areas to contribute:
- New language plugins
- Registry integrations (crates.io, PyPI, etc.)
- Shell integrations (Fish, Nushell, etc.)
- Documentation
- Bug reports and fixes

See [CONTRIBUTING.md](CONTRIBUTING.md) for full guide.

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgements

Inspired by the best ideas from:
- **nvm** — per-project Node versions
- **pyenv** — Python version isolation
- **mise** — multi-language runtime management
- **cargo** — dependency graph intelligence
- **volta** — project-local toolchain pinning

Built with ❤️ in **Rust**.

---

<div align="center">

**ven** — *Install once. Switch automatically. Never break.*

[![GitHub Stars](https://img.shields.io/github/stars/bhuwanb23/ven?style=social)](https://github.com/bhuwanb23/ven)

[Documentation](https://bhuwanb23.github.io/ven/docs) • [Releases](https://github.com/bhuwanb23/ven/releases) • [Issues](https://github.com/bhuwanb23/ven/issues) • [Discussions](https://github.com/bhuwanb23/ven/discussions)

</div>
