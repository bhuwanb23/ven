# ven

Rust CLI for **per-project language runtimes and dependencies**, driven by **`ven.toml`**. Installed runtimes live under **`~/.ven`** (or **`%USERPROFILE%\.ven`** on Windows).

## Supported runtimes

ven registers multiple **`LanguagePlugin`** implementations: **node**, **python**, **go**, **rust**, **java**, **deno**. Use **`ven install <language> [version]`** and **`ven list [language]`**; omit the version for interactive selection where supported.

Full notes: **[docs/languages.md](docs/languages.md)**.

## Commands (overview)

| Area | Commands | Notes |
|------|-----------|--------|
| Runtimes | `ven install`, `ven list` | Per-language installs (see docs). |
| Shell | `ven setup`, `ven use`, `ven deactivate`, `ven shell …` | Hooks define **`ven-use`** after **`ven setup`**. Evaluate `ven use` output if not using hooks. Optional **`VEN_STORAGE_PATH`** overrides `~/.ven`. |
| Packages | `ven add`, `ven remove`, `ven upgrade` | Ecosystem-specific sync into **`ven.toml`** (npm, pip, etc., per active runtime). |
| Project | `ven init`, `ven status` | Nearest **`ven.toml`** upward from cwd (no multi-file merge). |

**Companion binary:** **`ven-launcher`** opens a new terminal with env for the nearest **`ven.toml`** — see **[docs/ven-launcher.md](docs/ven-launcher.md)**.

### Documentation index

- **[docs/README.md](docs/README.md)** — table of contents  
- **[docs/commands-reference.md](docs/commands-reference.md)** — all commands  
- **[docs/ven-toml.md](docs/ven-toml.md)** — configuration  
- **[docs/shell-integration.md](docs/shell-integration.md)** — hooks and activation  

### First-time shell setup

1. Build or install **`ven`** and put it on **`PATH`** (`cargo build --release` from this repo if developing).
2. Run **`ven setup`** and follow the printed steps for your shell.
3. Open a **new** terminal, or run **`ven-use`** / evaluate **`ven use`** manually.

For CLI details in the terminal: **`ven --help`**, **`ven <command> --help`**.

## License

MIT
