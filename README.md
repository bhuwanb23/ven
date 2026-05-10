<div align="center">

<img src="assets/ven-logo.png" alt="Ven Logo" width="120" />

# ven

### The Intelligent Version & Dependency Manager

*One tool. Every language. Zero configuration overhead.*

[![Build Status](https://img.shields.io/github/actions/workflow/status/yourorg/ven/ci.yml?style=flat-square&logo=github)](https://github.com/yourorg/ven/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/github/v/release/yourorg/ven?style=flat-square)](https://github.com/yourorg/ven/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)]()
[![Stars](https://img.shields.io/github/stars/yourorg/ven?style=flat-square)](https://github.com/yourorg/ven/stargazers)

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
| CVE security scanning            |     ❌     |      ❌      |      ❌      |    ✅    |
| EOL alerts                       |     ❌     |      ❌      |      ❌      |    ✅    |
| Ghost dependency detection       |     ❌     |      ❌      |      ❌      |    ✅    |
| Team environment sync            |     ✅     |      ❌      |      ❌      |    ✅    |
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

### 🛡️ Built-In Security

CVE scanning, EOL alerts, and ghost dependency detection — built in, not
bolted on.

```bash
$ ven check

  Security     1 CRITICAL, 2 HIGH
  EOL          Node 20.20.2 — OK (2 years remaining)
  Ghosts       2 undeclared imports found

  ✗ lodash@4.17.19  CVE-2021-23337  CVSS 9.1
    Fix: ven upgrade lodash

  ✗ axios@1.6.0  CVE-2024-28849  CVSS 7.5
    Fix: ven upgrade axios

  Ghost deps in source (not in config):
  • dotenv  (found in src/config.js)
  • chalk   (found in src/cli.js)
    Fix: ven scan --fix
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

**Windows (PowerShell)**
```powershell
irm https://get.ven.sh/install.ps1 | iex
```

**macOS / Linux**
```bash
curl -fsSL https://get.ven.sh/install.sh | sh
```

**From source (Rust)**
```bash
git clone https://github.com/yourorg/ven
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

| Language   | Install | Packages     | Auto-Switch | Status |
| ---------- | :-----: | :----------: | :---------: | :----: |
| Node.js    | ✅      | npm          | ✅          | Stable |
| Python     | ✅      | pip          | ✅          | Stable |
| Go         | ✅      | go mod       | ✅          | Stable |
| Rust       | ✅      | cargo        | ✅          | Stable |
| Java (JDK) | ✅      | Maven/Gradle | ✅          | Stable |
| Ruby       | ✅      | gem/bundler  | ✅          | Stable |
| Deno       | ✅      | native/npm   | ✅          | Stable |
| Bun        | ✅      | bun/npm      | ✅          | Stable |
| PHP        | 🔜      | composer     | 🔜          | Planned|
| Elixir     | 🔜      | mix          | 🔜          | Planned|
| .NET       | 🔜      | nuget        | 🔜          | Planned|

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
path = ".venv"        # Python venv location
```

### `ven.lock`

```toml
version    = "1"
created_at = "2024-01-15T14:23:11Z"

[runtime]
node = "20.20.2"

[[packages]]
name    = "express"
version = "4.18.2"
sha256  = "a3f8b2c1d4e5f6..."

[[packages]]
name    = "lodash"
version = "4.16.0"
sha256  = "b4c5d6e7f8a9b0..."
```

---

## 🖥️ Command Reference

### Runtime Management

```bash
ven install <lang> <version>    # Install runtime
ven list                        # List installed versions
ven use <lang> <version>        # Set global default
ven status                      # Show active environment
ven status --verbose            # Detailed view
ven status --json               # Machine-readable output
```

### Package Operations

```bash
ven add <package>               # Install package
ven add <package>@<version>     # Install specific version
ven remove <package>            # Uninstall package
ven upgrade                     # Check all upgrades
ven upgrade <package> --apply   # Apply upgrade
```

### Dependency Intelligence

```bash
ven check-add <package>         # Pre-install compatibility check
ven check-add <package> --explain  # Show full conflict chain
ven why <package>               # Reverse dependency lookup
ven graph                       # Full dependency tree
ven resolve                     # Auto-fix all conflicts
```

### Security & Health

```bash
ven check                       # Full health report
ven check --security            # CVE scan only
ven check --eol                 # EOL check only
ven scan --ghosts               # Find undeclared dependencies
ven scan --fix                  # Auto-add ghost dependencies
```

### Team Sync

```bash
ven lock                        # Generate ven.lock
ven sync                        # Install from ven.lock
ven sync --check                # Drift check only (CI-safe)
```

### Export

```bash
ven export dockerfile           # Generate Dockerfile
ven export github-actions       # Generate CI workflow
ven export gitlab-ci            # Generate GitLab CI config
```

### Shell Integration

```bash
ven setup                       # Install shell hooks
ven shell activate              # Manual activation
ven deactivate                  # Clear ven environment
```

### Launcher (No Admin)

```bash
ven-launcher                    # Open ven terminal in current dir
ven-launcher <path>             # Open ven terminal in project
```

---

## 🗂️ Storage Layout

```
~/.ven/
├── node/
│   ├── 20.20.2/        ← node, npm, npx
│   └── 22.11.0/
├── python/
│   ├── 3.11.5/         ← python, pip
│   └── 3.12.0/
├── go/
│   └── 1.21.5/         ← go binary + GOROOT
├── rust/
│   └── 1.75.0/         ← rustc, cargo
├── java/
│   └── 17.0.9/         ← JDK, JAVA_HOME
├── ruby/
│   └── 3.2.2/          ← ruby, gem, GEM_HOME
├── deno/
│   └── 1.40.0/         ← single deno binary
├── bun/
│   └── 1.0.20/         ← single bun binary
├── versions/
│   ├── node            ← global default: "20.20.2"
│   └── python          ← global default: "3.11.5"
└── cache/
    └── ven.db          ← SQLite: package metadata cache
```

---

## 🌍 How Auto-Switching Works

```
You type: cd ~/projects/frontend

Shell hook fires:
  1. Search for ven.toml (current → parent directories)
  2. Read: node = "20"
  3. Resolve: 20 → 20.20.2
  4. Find: ~/.ven/node/20.20.2/
  5. Prepend to PATH (this terminal only)
  6. Export: VEN_NODE_VERSION=20.20.2

You type: node --version
  → v20.20.2

You type: cd ~/projects/backend

Shell hook fires again:
  1. Find new ven.toml: node = "22", python = "3.11"
  2. Remove old PATH entries
  3. Add new PATH entries
  4. Export new VEN_* markers

You type: node --version
  → v22.11.0
```

**Per-terminal isolation:**
Terminal 1 → Node 20 | Terminal 2 → Node 22 | No conflicts

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

- All runtime downloads verified with **SHA256 checksums**
- Official sources only (nodejs.org, python.org, golang.org, etc.)
- **CVE scanning** via OSV database (api.osv.dev)
- **EOL monitoring** via endoflife.date API
- Lock file integrity via **SHA256 package hashes**
- No telemetry. No phone home. No accounts.

---

## 📖 Documentation

| Topic | Link |
| ----- | ---- |
| Getting Started | [docs/getting-started.md](docs/getting-started.md) |
| Configuration Reference | [docs/configuration.md](docs/configuration.md) |
| Command Reference | [docs/commands.md](docs/commands.md) |
| Language Guides | [docs/languages/](docs/languages/) |
| Dependency Graph | [docs/dependency-graph.md](docs/dependency-graph.md) |
| Team Workflows | [docs/team-workflows.md](docs/team-workflows.md) |
| Enterprise Usage | [docs/enterprise.md](docs/enterprise.md) |
| Contributing | [CONTRIBUTING.md](CONTRIBUTING.md) |

---

## 🤝 Contributing

Contributions are welcome.

```bash
# Clone
git clone https://github.com/yourorg/ven
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

[![GitHub Stars](https://img.shields.io/github/stars/yourorg/ven?style=social)](https://github.com/yourorg/ven)
[![Follow](https://img.shields.io/twitter/follow/ven_sh?style=social)](https://twitter.com/ven_sh)

[Website](https://ven.sh) • [Documentation](https://docs.ven.sh) • [Discord](https://discord.gg/ven) • [Twitter](https://twitter.com/ven_sh)

</div>
