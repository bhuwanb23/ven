# ven Command Reference

Complete documentation for all `ven` commands and their subcommands.

> Looking for a top-down feature view instead of per-command pages? See
> [**docs/features.md**](../features.md) — every capability mapped to the
> exact command syntax that implements it.

## 📦 Version Management

| Command | Description | Documentation |
|---------|-------------|---------------|
| `ven install` | Install language versions (native download) | [→ install.md](install.md) |
| `ven list` | List installed versions with metadata | [→ list.md](list.md) |
| `ven delete` | Delete an installed runtime (refuses active version without `--force`) | [→ delete.md](delete.md) |
| `ven set global` | Make an installed runtime globally available on the User PATH (no admin) | [→ set.md](set.md) |
| `ven status` | Show current project configuration | [→ status.md](status.md) |

## 📋 Package Management

`ven add` / `ven remove` / `ven upgrade` are **unified** across every supported
language: each call invokes the native package manager **and** keeps both the
language-native manifest and `ven.toml [packages]` in sync.

| Language    | Native manifest kept in sync             | Tool used                                    |
|-------------|------------------------------------------|----------------------------------------------|
| Node.js     | `package.json` (+ `package-lock.json`)   | `npm`                                        |
| Bun         | `package.json` (+ `bun.lockb`)           | `bun`                                        |
| Python      | `requirements.txt`                       | `pip` (venv-aware)                           |
| Ruby        | `Gemfile`                                | `bundle add/remove/update` when Gemfile present, else `gem install/uninstall` + direct Gemfile edit |
| Java        | `pom.xml` / `build.gradle[.kts]`         | direct manifest edit; coords `group:artifact[@version]` |
| Deno        | `deno.json` `imports`                    | `deno add`/`deno remove` (≥ 1.42), else direct JSON edit |
| Go          | `go.mod`                                 | `go get` / `go get pkg@none` / `go get -u` + `go mod tidy` |
| Rust        | `Cargo.toml`                             | `cargo add` / `cargo remove` / `cargo update -p` |

| Command | Description | Documentation |
|---------|-------------|---------------|
| `ven add` | Install packages (compatibility check + native install + manifest + `ven.toml` sync) | [→ add.md](add.md) |
| `ven check-add` | Query add compatibility without installing | [→ check-add.md](check-add.md) |
| `ven graph` | Inspect dependency graph / last simulation | [→ graph.md](graph.md) |
| `ven remove` | Remove packages (native uninstall + manifest + `ven.toml` cleanup) | [→ remove.md](remove.md) |
| `ven upgrade` | Preview and apply package upgrades (native upgrade + manifest + `ven.toml` sync) | [→ upgrade.md](upgrade.md) |
| `ven lock` | Write `ven.lock` from resolved graphs | [→ lock.md](lock.md) |
| `ven sync` | Validate `ven.lock` and install pins (Node/Bun); `pip install -r requirements.txt` for Python projects | [→ sync.md](sync.md) |
| `ven resolve` | Auto-resolve conflicts and apply fixes | [→ resolve.md](resolve.md) |

## 🛡️ Health & Security

| Command | Description | Documentation |
|---------|-------------|---------------|
| `ven check` | Combined CVE (OSV) + runtime EOL (endoflife.date) report | [→ check.md](check.md) |
| `ven scan --ghosts` | Find packages imported in source but not declared in any manifest | [→ scan.md](scan.md) |

For the underlying threat model, caching strategy, and exit-code semantics, see [→ security-model.md](../security-model.md).

## 📖 Documentation

| Command | Description | Documentation |
|---------|-------------|---------------|
| `ven docs <pkg>` | Render version-pinned docs in the terminal, or open in browser; supports `--diff v1 v2` | [→ docs.md](docs.md) |

## ⚙️ Project Setup

| Command | Description | Documentation |
|---------|-------------|---------------|
| `ven init` | Create ven.toml with templates | [→ init.md](init.md) |
| `ven setup` | Install shell hook for auto-switching | [→ setup.md](setup.md) |
| `ven path` | Move ven's storage root to another drive (v0.1.6+) | [→ path.md](path.md) |

## 🔄 Maintenance

| Command | Description | Documentation |
|---------|-------------|---------------|
| `ven doctor` | Diagnose multiple installs, PATH shadowing, and upgrade paths | [→ doctor.md](doctor.md) |
| `ven update` | Self-update `ven` + `ven-launcher` to the latest release (auto-elevates for system installs; v0.1.7+) | [→ update.md](update.md) |
| `ven uninstall` | Full-nuke teardown: removes ven binary, every runtime, cache, state, persisted env, and PATH entries (v0.1.7+) | [→ uninstall.md](uninstall.md) |

## 💿 Installer / Spawner

| Binary / Script | Description | Documentation |
|-----------------|-------------|---------------|
| `ven-setup` | Cross-platform installer (GUI wizard v0.2+; CLI via `--cli`) | [→ ven-setup.md](ven-setup.md) · [GUI screens](ven-setup-gui.md) |
| `install.ps1` / `install.sh` | One-liner install scripts that wrap `ven-setup` from a GitHub release | [→ ../install-scripts.md](../install-scripts.md) |

## 🔧 Internal Commands

| Command | Description | Documentation |
|---------|-------------|---------------|
| `ven shell` | Shell integration (hidden) | [→ shell.md](shell.md) |

---

## Quick Examples

```bash
# Install Node.js
ven install node 20              # Latest 20.x.x
ven install node lts             # Latest LTS

# Create project
ven init --template              # Interactive template selection
ven add express                  # Install with compatibility check

# Auto-switching
cd myproject/                    # Automatically activates correct Node version
node --version                   # Shows version from ven.toml
```

## Architecture

- **Native Installation**: Downloads directly from each language's official source (nodejs.org, python.org, go.dev, static.rust-lang.org, Adoptium, deno.com, oven-sh, RubyInstaller2 / ruby-builder). SHA256 verified + binary smoke-tested before being marked installed.
- **Plugin Registry**: Extensible architecture for multi-language support
- **Smart Compatibility**: Pre-install checks prevent breaking changes
- **Cross-Platform**: Works on Windows (PowerShell 5.1+, PowerShell 7+), macOS (bash, zsh), and Linux (bash, zsh, fish). Windows `cmd.exe` is **not** a supported activation shell — use PowerShell, or use `ven-launcher` for a no-install portable shell.

For detailed architecture documentation, see the main [README](../README.md).
