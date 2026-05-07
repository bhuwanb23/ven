# Command reference

Unless noted, paths default to the **current working directory**; many commands walk **up** the tree to find `ven.toml`.

## Global

| Command | Purpose |
|---------|---------|
| `ven --help` | Short overview and examples (`after_help` lists common flows). |
| `ven -V` / `ven --version` | Binary version. |

## Project lifecycle

| Command | Purpose |
|---------|---------|
| `ven init` | Create `ven.toml` (interactive with `--template`). |
| `ven status` | Show resolved runtime, packages/env summary; `--verbose`, `--json`, `--fix`. |
| `ven install <runtime> [version]` | Install a language/toolchain version under ven’s store; `-y` / `--dry-run`; `--verbose`; `-q`. Interactive mode lists versions when version omitted (where supported). |
| `ven list [runtime]` | List installed versions (`runtime` optional filter). |
| `ven use [PATH]` | Print shell exports to apply nearest `ven.toml`; **evaluate** output (`eval "$(ven use)"`, PowerShell: parse stderr hint / use hooks). |
| `ven deactivate` | Print exports that undo `ven use` overlay for current shell session. |
| `ven add <packages…>` | Add npm/PyPI/etc. packages per `[packages]` / runtime rules; sync `ven.toml`. |
| `ven remove [packages…]` | Remove packages; `--cleanup` removes orphans. |
| `ven upgrade [packages…]` | Upgrade pins; `--all`, `--apply`, `--dry-run`. |

## Shell integration

| Command | Purpose |
|---------|---------|
| `ven setup` | Install/update shell hooks and optional profiles (bash/zsh/fish/PowerShell). |

Hidden / advanced:

| Command | Purpose |
|---------|---------|
| `ven shell activate` | Same core behavior as `ven use` (machinery for hooks). |
| `ven shell deactivate` | Same as `ven deactivate`. |
| `ven shell hook …` | Internal hook fragments used by `setup`. |

See [shell-integration.md](shell-integration.md).

## Platform spawn

| Binary | Purpose |
|--------|---------|
| `ven-launcher [PROJECT]` | Open a **new** terminal with env for nearest `ven.toml`; `--show-env` prints resolved env instead. See [ven-launcher.md](ven-launcher.md). |
