# ven path

Manage where ven stores its data on disk. Introduced in **v0.1.6**.

Default storage root is `$HOME/.ven` on Linux/macOS and
`%USERPROFILE%\.ven` on Windows. When that drive fills up — a common
problem on company-managed C: drives with a 50 GB quota — `ven path set`
relocates everything (runtimes, cache, lockfile state) to a new location,
atomically, with rollback on failure.

The new location is remembered in two places:

1. A **pointer file** at `~/.config/ven/config.toml` (Linux),
   `~/Library/Application Support/ven/config.toml` (macOS),
   or `%APPDATA%\ven\config.toml` (Windows). This is ven's source of truth.
2. The **`VEN_HOME`** environment variable in your User scope, so any
   external tool (npm, pip, your editor's integrated terminal) sees the
   relocated path automatically in a new shell.

If the env-var write fails (locked-down corporate machine) it's a warning,
not an error — the pointer file is enough for ven itself.

## Resolution precedence

Whenever any `ven` invocation needs to know "where does my data live?",
it walks this list and uses the first hit:

1. `$VEN_HOME` env var (per-process override, highest precedence — CI keeps working)
2. `$VEN_STORAGE_PATH` env var (back-compat with older releases)
3. `<launcher-dir>/.ven/` (portable / USB-stick mode)
4. **Pointer file `[storage].home`** (what `ven path set` writes)
5. `$HOME/.ven` (default)

So `VEN_HOME=/tmp/x ven install python` still wins for a single command,
and dropping a launcher next to a `.ven/` folder still wins for portable
bundles. The pointer is for the steady-state "I moved my data once and
want ven to remember".

## Subcommands

```bash
ven path                            # alias for `ven path show`
ven path show [--json]              # current root, source, size, free space
ven path set <dir> [flags]          # relocate (default: interactive wizard)
ven path reset [flags]              # clear the pointer; revert to $HOME/.ven
```

### `ven path set`

| Flag                | Effect                                                                    |
|---------------------|---------------------------------------------------------------------------|
| _(no flag)_         | Interactive: ask "move data?" / "pointer only?" / "cancel?"               |
| `--move`            | Skip the prompt; move existing data to the new location                   |
| `--no-move`         | Skip the prompt; write the pointer only, leave existing data where it is  |
| `--pointer-only`    | Alias for `--no-move`                                                     |
| `-y` / `--yes`      | Skip the prompt; default to `--move`                                      |
| `--force-unlock`    | Ignore a leftover `.ven-move.lock` (only use if you're certain no other ven is mid-move) |
| `--json`            | Machine-readable. **Requires** `--move`, `--no-move`, or `--pointer-only` (no prompts in JSON mode). |

### `ven path reset`

Drops the pointer file and unsets `VEN_HOME` from your user env.

| Flag             | Effect                                                                |
|------------------|-----------------------------------------------------------------------|
| _(no flag)_      | Interactive: ask "move data back to ~/.ven?"                           |
| `--move`         | Move data back to `$HOME/.ven`                                         |
| `--no-move`      | Just clear the pointer, leave data in place (you can re-set later)    |
| `-y` / `--yes`   | Skip the prompt; default to `--move`                                  |
| `--json`         | Machine-readable                                                      |

## Examples

### Inspect current state

```bash
ven path show
```

```
  Storage root: D:\ven
  Source:       pointer file (C:\Users\you\AppData\Roaming\ven\config.toml)
  Size:         3.4 GB across 4 language(s)
  Pointer:      C:\Users\you\AppData\Roaming\ven\config.toml
```

### Relocate when C: is full

```bash
ven path set D:\ven
```

```
  [ven path] Relocate ven storage root
    From: C:\Users\you\.ven
    To:   D:\ven
    Data: 3.4 GB across 8,217 file(s)

? What should happen to the existing data?
> Move it to the new location (recommended)
  Leave it where it is; just update the pointer
  Cancel

  [progress bar: 3.4 GB / 3.4 GB] [#######>--------]

  [OK] Moved 3.4 GB (8,217 files): C:\Users\you\.ven -> D:\ven
       [i] cross-device copy (target is on a different drive than the source)
  [OK] Pointer: C:\Users\you\AppData\Roaming\ven\config.toml
  [OK] VEN_HOME persisted in your User environment (restart your shell to see it in new sessions)
```

### CI / scripts (no prompts)

```bash
ven path set /mnt/data/ven -y --json
```

```json
{
  "status": "ok",
  "from": "/home/you/.ven",
  "from_source": "default",
  "to": "/mnt/data/ven",
  "pointer": "/home/you/.config/ven/config.toml",
  "moved": true,
  "bytes_moved": 3650534400,
  "files_moved": 8217,
  "used_fast_path": true
}
```

### Pointer-only (don't touch existing data)

Useful when you've manually pre-staged the data at the new location, or
when you want future installs to land on D: but keep the existing C: copy
as a fallback during a migration:

```bash
ven path set D:\ven --pointer-only
```

### Revert to default

```bash
ven path reset --move
```

```
  [OK] ven storage root reverted to default (C:\Users\you\.ven)
  [OK] Moved 3.4 GB from D:\ven
  [OK] VEN_HOME removed from your User environment
```

## Safety semantics

### Cross-drive moves use copy + verify + delete

On Windows, `fs::rename` returns `ERROR_NOT_SAME_DEVICE` when moving
between drives. `ven path set` falls back to a recursive copy with an
`indicatif` progress bar, then **verifies the file count and total byte
count at the target match what was at the source** before it removes the
source. If anything mismatches, the partial target is removed and the
source is left untouched — ven is never left in a half-relocated state.

### A `.ven-move.lock` blocks concurrent moves

Before the copy starts, `ven path set` writes a `.ven-move.lock` file
containing the current PID into the source directory. A second invocation
(or a previous one that crashed) will refuse to start while the lock
exists. To force past a stale lock from a crashed move, pass
`--force-unlock`.

### Active `$VEN_HOME` env var shadows the pointer

If `VEN_HOME` is set in your current shell, it wins over the pointer for
this process (resolver step 1 vs. step 4). `ven path show` calls this out
explicitly with a `[!]` warning, and `ven path set` will warn that the
new pointer won't be observed in your current shell until you `unset
VEN_HOME` (Unix) or close and reopen the terminal (Windows, where the
env update only flows into freshly-spawned shells).

## JSON shapes

### `ven path show --json`

```json
{
  "home": "D:\\ven",
  "source": "pointer",
  "size_bytes": 3650534400,
  "size_human": "3.40 GB",
  "languages_installed": 4,
  "pointer": "D:\\ven",
  "env_VEN_HOME": null,
  "env_VEN_STORAGE_PATH": null
}
```

`source` is one of:
- `env:VEN_HOME` — set explicitly in this process
- `env:VEN_STORAGE_PATH` — back-compat env var
- `portable` — `.ven/` sits next to the launcher
- `pointer` — written by `ven path set`
- `default` — no override

### `ven path set --json`

```json
{
  "status": "ok",                 // "ok" | "cancelled"
  "from": "C:\\Users\\you\\.ven",
  "from_source": "default",
  "to": "D:\\ven",
  "pointer": "C:\\Users\\you\\AppData\\Roaming\\ven\\config.toml",
  "moved": true,
  "bytes_moved": 3650534400,
  "files_moved": 8217,
  "used_fast_path": false,
  "env_warning": null             // present only if persistent env update failed
}
```

### `ven path reset --json`

```json
{
  "status": "reset",              // "reset" | "noop" | "cancelled"
  "from": "D:\\ven",
  "to": "C:\\Users\\you\\.ven",
  "moved": true,
  "bytes_moved": 3650534400,
  "files_moved": 8217
}
```

## What does NOT move

- The `ven` / `ven-launcher` / `ven-setup` binaries themselves stay where
  they were installed by `ven-setup` (e.g. `C:\Users\you\.ven\bin\`). PATH
  is unchanged. Only the **data** moves.
- Portable-mode `.ven/` siblings are untouched — those are per-bundle
  declarations, not per-user preferences. The pointer file is silently
  ignored whenever a portable `.ven/` exists next to the launcher.

## Related

- [`ven list`](list.md) — see what's installed at the current ven home
- [`ven status`](status.md) — project-level configuration
- [`ven-launcher`](../ven-launcher.md) — the portable-mode story

## Implementation

- CLI handler: [`src/cli/path.rs`](../../src/cli/path.rs)
- Storage relocation: [`src/core/storage_move.rs`](../../src/core/storage_move.rs)
- Global config / pointer file: [`src/core/ven_config.rs`](../../src/core/ven_config.rs)
- Resolver: [`src/core/ven_home.rs`](../../src/core/ven_home.rs)
- Persistent user env: [`src/core/user_env.rs`](../../src/core/user_env.rs)
