# ven

Rust CLI for **per-project Node.js runtime + npm deps** driven by **`ven.toml`**. Runtimes live under **`~/.ven`** (or **`%USERPROFILE%\.ven`** on Windows).

## Commands (overview)

| Area | Commands | Notes |
|------|-----------|--------|
| Runtimes | `ven install`, `ven list` | **Node** only today; **`ven install`** uses nodejs.org + SHA256 when available (`src/core/download.rs`). **Python** is not implemented yet (`LanguagePlugin` is Node-only). |
| Shell | `ven setup`, `ven shell hook|activate|install|deactivate`, **`ven use`**, **`ven deactivate`** | Hooks auto-switch on **`cd`** / **`Set-Location`** + prompt (PowerShell/bash/zsh) or **`fish_prompt`**. Outputs of `activate` / `use` / `deactivate` must be evaluated in-shell (hooks define **`ven-use`**). Optional **`VEN_STORAGE_PATH`** overrides `~/.ven`. |
| Packages | `ven add`, `ven remove`, `ven upgrade` | npm + **`ven.toml`** sync (**pip** not wired). |
| Project | `ven init`, `ven status` | Nearest **`ven.toml`** upward (subdir “inherits” ancestor file; **no multi-file merge**). |

### First-time shell setup

1. `cargo build --release` and put **`ven`** on **`PATH`**.
2. Run **`ven setup`** (or **`ven shell install`** on Windows profiles).
3. Open a **new** terminal — or **`ven-use`** / **`eval "$(ven shell activate .)"`** / **`iex …`** to apply manually.

See **`src/shell/mod.rs`** for hook behavior.

## Features (shipping vs roadmap)

**In place:** Multi-version Node under **`~/.ven`**, checksum verification when upstream SHASUMS load, **`ven.toml`** **`[runtime].node`**, **`[packages]`**, **`[env]`**, PATH activation scripts, **`ven add/remove/upgrade`**, **`ven init`** / **`status`**.

**Not in this codebase yet:** Install/manage **Python** runtimes via **`ven install`**; **`pip`** in **`ven add`**. Planned via **`LanguagePlugin`** (`src/plugins/registry.rs`).

## License

MIT
