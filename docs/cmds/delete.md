# ven delete

Delete an installed language runtime by removing its directory under
`$VEN_HOME/<language>/<version>/`.

Distinct from [`ven remove`](remove.md), which uninstalls **packages**
(npm / pip / cargo / gem / ...). `delete` removes a **runtime** — the
files Node, Python, Go, etc. themselves live in.

> Added in **v0.1.4**. Before that, the equivalent was a manual
> `rm -rf ~/.ven/<lang>/<version>` (Unix) or `Remove-Item -Recurse`
> (Windows).

## Overview

- Three calling conventions: full wizard, language-only, or fully specified.
- Refuses to delete the runtime currently resolved by the nearest
  `ven.toml` (so the next `cd` activation in that project doesn't silently
  fail). Pass `--force` to override.
- Shows the runtime's disk size before asking to confirm so you know what
  you're freeing.
- Auto-confirms in non-interactive shells (CI, piped stdin) for scripted use.

## Usage

```bash
ven delete [OPTIONS] [LANGUAGE] [VERSION]
```

## Calling conventions

| Invocation                            | Behaviour                                                         |
|---------------------------------------|-------------------------------------------------------------------|
| `ven delete`                          | Wizard: pick language (only ones with installs) → pick version → confirm → delete |
| `ven delete python`                   | Skip language picker; pick a Python version → confirm → delete    |
| `ven delete python 3.12.7`            | Skip both pickers; show confirm prompt only                       |
| `ven delete python 3.12.7 -y`         | Skip the confirm prompt too (CI / scripts)                        |
| `ven delete python 3.12.7 --force`    | Allow deleting the version that is currently active in `ven.toml` |
| `ven delete python 3.12.7 -y --json`  | Machine-readable result (requires `-y` + explicit args)           |

## Flags

| Flag        | Short | Description                                                       |
|-------------|-------|-------------------------------------------------------------------|
| `--yes`     | `-y`  | Skip the confirmation prompt.                                     |
| `--force`   | -     | Bypass the active-runtime safety check (otherwise deletion is refused when the chosen version is pinned in the nearest `ven.toml`). |
| `--json`    | -     | Machine-readable output. Requires explicit `<language> <version>` + `-y` (no interactive prompts in JSON mode). |

## Examples

### Wizard

```text
$ ven delete

[WIZARD] Delete a language runtime
? Select language ›
  python  (2 versions installed)
  node    (1 version installed)
  go      (1 version installed)

? Select python version to delete ›
  3.12.7  (231.4 MB - installed 2026-04-12)
  3.11.9  (218.7 MB - installed 2026-03-02)

  [DELETE] About to permanently delete python 3.11.9
    Path: C:\Users\you\.ven\python\3.11.9
    Size: 218.7 MB (installed 2026-03-02)

? Permanently delete this runtime? (y/N) › y

  [OK] Deleted python 3.11.9 (218.7 MB freed)
  [PATH] C:\Users\you\.ven\python\3.11.9
```

### Direct delete

```bash
ven delete python 3.11.9 -y          # CI / scripts: no prompts
ven delete node 18.20.2              # interactive confirm only
```

### JSON for automation

```bash
ven delete python 3.11.9 -y --json
```

```json
{
  "status": "deleted",
  "language": "python",
  "version": "3.11.9",
  "path": "/home/you/.ven/python/3.11.9",
  "freed_bytes": 229332910,
  "freed_human": "218.7 MB",
  "force": false
}
```

## Safety: the active-runtime guard

By default, if the resolved `(language, version)` matches the version that
the nearest `ven.toml` would activate, `ven delete` refuses and exits with
a clear error:

```text
[ERROR] Cannot delete python 3.12.7: it is the active runtime in C:\proj\ven.toml.

  Deleting it would break the next `cd` activation in that project.
  Pin a different version in ven.toml first, or pass --force to override.
```

This prevents the surprise where you delete the runtime, then re-enter your
project and the shell hook errors out because the version it was supposed to
activate is gone.

Two ways past the guard:

1. **Recommended:** edit `ven.toml` to pin a different version, then run
   `ven delete <lang> <old-version>`. Now no project resolves to the
   deleted version.
2. **Quick:** pass `--force`. The deletion still happens, and the next
   `cd <pinned-project>` will surface a "runtime not installed" error that
   you can fix with `ven install`.

### JSON refusal

In `--json` mode the refusal is non-zero-exit + structured:

```bash
ven delete python 3.12.7 -y --json
```

```json
{
  "status": "refused",
  "reason": "active_runtime",
  "language": "python",
  "version": "3.12.7",
  "ven_toml": "C:\\proj\\ven.toml",
  "hint": "pass --force to override"
}
```

## Other JSON shapes

| `status`     | When                                                                        |
|--------------|-----------------------------------------------------------------------------|
| `deleted`    | Runtime directory removed successfully.                                     |
| `cancelled`  | Interactive confirm prompt declined (no-op).                                |
| `noop`       | No versions of the chosen language are installed — nothing to delete.       |
| `refused`    | Active-runtime guard blocked the deletion (exits non-zero).                 |

## Storage layout

`ven delete <lang> <version>` is equivalent to a recursive remove of:

```text
$VEN_HOME/<lang>/<version>/
```

where `$VEN_HOME` follows the resolver order documented in
[`ven-launcher.md`](../ven-launcher.md#portable-mode):
`$VEN_HOME` → `$VEN_STORAGE_PATH` → `<launcher-dir>/.ven` → `~/.ven`.

The deletion does **not** touch:

- `$VEN_HOME/bin/` (the `ven`, `ven-launcher`, `ven-setup` binaries stay)
- `$VEN_HOME/cache/` (OSV / EOL / docs lookups remain warm)
- `$VEN_HOME/storage/` (lockfile state and drift snapshots)
- Other versions of the same language

## Related commands

- [`ven list`](list.md) — see what's installed before deciding what to delete
- [`ven install`](install.md) — install a runtime version (reverse operation)
- [`ven remove`](remove.md) — uninstall **packages** from a project (not a runtime)
- [`ven status`](status.md) — check which runtime is active in the current project

## Implementation

- CLI handler: [`src/cli/delete.rs`](../../src/cli/delete.rs)
- Shared helpers (also used by `ven list`): [`src/cli/list/helpers.rs`](../../src/cli/list/helpers.rs)
  — `detect_active_version`, `calculate_dir_size`, `format_bytes`,
  `get_installation_date`, `get_version_path`
- Storage root resolver: [`src/core/ven_home.rs`](../../src/core/ven_home.rs)
