# `ven-setup` (cross-platform installer)

`ven-setup` is a **single self-contained installer**. `ven` and `ven-launcher` are **embedded as bytes** inside `ven-setup` at build time; the installer extracts them, updates `PATH`, optionally configures `$VEN_HOME`, installs shell hooks, optionally pre-installs runtimes, and verifies `ven --version`.

> **v0.2+**: Running `ven-setup` with no flags opens a **native GUI wizard** (Welcome → Done). See [`ven-setup-gui.md`](ven-setup-gui.md). Use `--cli` for the legacy terminal flow (SSH, CI, headless).

The installer also falls back to **sibling files on disk** when an embedded payload is empty, so development builds and `cargo run --bin ven-setup` still work.

> **Supported shells**: bash, zsh, fish, PowerShell (5.1+ and 7+). Windows `cmd.exe` is **not** a supported activation shell — install via PowerShell, or use [`ven-launcher`](../ven-launcher.md) when no install at all is allowed.

## Install modes

### Windows

| Mode      | Install dir                  | PATH scope                                                   | Admin? |
|-----------|------------------------------|--------------------------------------------------------------|--------|
| `user`    | `%USERPROFILE%\.ven\bin`     | `HKCU\Environment\Path`                                      | No     |
| `system`  | `%ProgramFiles%\ven\bin`     | `HKLM Machine Path`                                          | Yes (UAC) |

System mode saves wizard choices to `%TEMP%\ven-setup-resume.toml` and relaunches with `Start-Process -Verb RunAs`.

### Unix (Linux / macOS)

| Mode      | Install dir         | PATH wiring                                                | Root? |
|-----------|---------------------|------------------------------------------------------------|-------|
| `user`    | `~/.ven/bin`        | Block in `~/.bashrc` / `~/.zshrc` / `~/.profile`           | No  |
| `system`  | `/usr/local/bin`    | `/etc/profile.d/ven.sh`                                    | Yes (`sudo`) |

## Pipeline (shared by GUI and CLI)

```text
1. Extract ven + ven-launcher
2. Configure storage path ($VEN_HOME pointer + user env, if customized)
3. Update PATH (optional)
4. Install shell hook via `ven setup` (optional)
5. Pre-install selected runtimes (optional)
6. Verify `ven --version`
```

## Usage

```bash
# GUI wizard (default on desktop)
ven-setup

# Legacy CLI
ven-setup --cli
ven-setup --mode user --no-input

# Automation
ven-setup --mode user --no-input --with-runtimes node,python
ven-setup --mode user --dry-run
ven-setup --storage-path D:\ven-data --no-hook

# Headless Linux (auto-falls back to CLI when no DISPLAY)
DISPLAY= ven-setup
```

## Options

| Flag | Description |
|------|-------------|
| `--mode <user\|system>` | Install scope |
| `--cli` | Force terminal flow (skip GUI) |
| `--dry-run` | Print plan only |
| `--no-input` | Non-interactive; requires `--mode` (implies CLI) |
| `--storage-path <PATH>` | Set `$VEN_HOME` / pointer to this directory |
| `--with-runtimes <langs>` | Comma-separated slugs to pre-install (`node,python`, …) |
| `--no-hook` | Skip `ven setup` shell hook |
| `--no-path` | Skip PATH update |
| `--elevated-child` | Internal: elevated child after UAC |
| `--resume <PATH>` | Internal: TOML resume file from parent |

## Binary embedding

Two-pass release build:

```bash
cargo build --release --bin ven --bin ven-launcher
cargo build --release --bin ven-setup
```

## See also

- [`ven-setup-gui.md`](ven-setup-gui.md) — wizard screens
- [`ven setup`](setup.md) — shell hook
- [`ven-launcher`](../ven-launcher.md) — portable mode without install
