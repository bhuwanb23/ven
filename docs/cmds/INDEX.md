# ven Command Reference

Complete documentation for all `ven` commands and their subcommands.

## 📦 Version Management

| Command | Description | Documentation |
|---------|-------------|---------------|
| `ven install` | Install language versions (native download) | [→ install.md](install.md) |
| `ven list` | List installed versions with metadata | [→ list.md](list.md) |
| `ven status` | Show current project configuration | [→ status.md](status.md) |

## 📋 Package Management

| Command | Description | Documentation |
|---------|-------------|---------------|
| `ven add` | Install packages with compatibility checking | [→ add.md](add.md) |
| `ven check-add` | Query add compatibility without installing | [→ check-add.md](check-add.md) |
| `ven graph` | Inspect dependency graph / last simulation | [→ graph.md](graph.md) |
| `ven remove` | Remove packages with dependency safety | [→ remove.md](remove.md) |
| `ven upgrade` | Preview and apply package upgrades | [→ upgrade.md](upgrade.md) |

## ⚙️ Project Setup

| Command | Description | Documentation |
|---------|-------------|---------------|
| `ven init` | Create ven.toml with templates | [→ init.md](init.md) |
| `ven setup` | Install shell hook for auto-switching | [→ setup.md](setup.md) |

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

- **Native Installation**: Downloads directly from nodejs.org (no fnm/pyenv needed)
- **Plugin Registry**: Extensible architecture for multi-language support
- **Smart Compatibility**: Pre-install checks prevent breaking changes
- **Cross-Platform**: Works on Windows, macOS, and Linux

For detailed architecture documentation, see the main [README](../README.md).
