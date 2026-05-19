# ven uninstall

Full-nuke teardown for the ven install on this machine. Introduced in
**v0.1.7**.

Replaces the long copy-paste PowerShell / shell snippet that used to live on
the install page with a single confirmed, dry-run-capable command. Removes:

- The user install root (`~/.ven` or `%USERPROFILE%\.ven`) — binary, every
  installed runtime, cache, lockfile state.
- The system install if present (`/usr/local/bin/{ven,ven-launcher,ven-setup}`
  + `/etc/profile.d/ven.sh` on Unix, `%ProgramFiles%\ven\` on Windows).
- A relocated storage root if `ven path set` moved it elsewhere — honored
  via the same [`ven_home`](../../src/core/ven_home.rs) precedence the rest
  of ven uses.
- The persisted `VEN_HOME` user environment variable written by `ven path set`.
- The pointer file at `~/.config/ven/config.toml` (or platform equivalent).
- The ven-managed blocks from your shell rc files:
  `# >>> ven env >>>`, `# >>> ven-setup PATH >>>`, `# >>> ven shell hook >>>`,
  plus any orphan unmarked line referencing `.ven/bin` (legacy installs).
- Windows User-scope and Machine-scope PATH entries (with the same
  `WM_SETTINGCHANGE` broadcast as the installer so already-open shells
  pick up the cleaned PATH).

## Survives the teardown

Anything **inside individual projects** is left alone:

- `ven.toml` / `ven.lock` files — those are part of your repo, not ven.
- `node_modules/`, `venv/`, `__pycache__/`, language-native lockfiles.
- Editor settings, shell history, etc.

If you also want to remove the locked language installs that any current
`ven.toml` files would pull back in on the next install, that's a separate
manual step.

## Synopsis

```bash
ven uninstall                    # interactive: show plan, prompt before nuking
ven uninstall --dry-run          # print the plan; touch nothing
ven uninstall -y                 # skip the confirm prompt (CI / scripts)
ven uninstall --user-only        # skip the system install layer
ven uninstall --system-only      # skip the user install layer (rare; for admins)
ven uninstall --json -y          # machine-readable result
ven uninstall --json --dry-run   # plan as JSON without executing
```

## Flags

| Flag | Effect |
|------|--------|
| _(no flag)_ | Print the plan, then prompt `Permanently remove ven and all installed runtimes? [y/N]`. The default answer is **No** so an accidental Enter cancels. |
| `-y` / `--yes` | Skip the confirm prompt. Required for CI / scripted use. |
| `--dry-run` | Build the plan and print it. Don't touch the filesystem or env state. Combine with `--json` to capture the plan in a CI gate. |
| `--user-only` | Skip the system install layer. Useful when you want to drop your personal install without needing sudo / Admin. |
| `--system-only` | Skip the user install layer. For sysadmins cleaning up a shared host without touching the calling user's home dir. Mutually exclusive with `--user-only`. |
| `--json` | Emit a structured result to stdout. **Requires** either `--dry-run` (plan-only) or `-y` (execute). Pure JSON without one of those is rejected because the intent is ambiguous. |

## Elevation

The system install lives in dirs only writable by root / Admin. When `ven
uninstall` detects system artifacts present but the current process lacks
the required privileges, it bails with a clear hint:

- **Unix**: `sudo ven uninstall`
- **Windows**: open a PowerShell as Administrator and re-run.
- Pass `--user-only` to skip the system layer entirely if you just want
  to drop the per-user install for now.

`ven uninstall` does NOT spawn `sudo` / UAC for you the way `ven update`
does. The blast radius is too large to risk a malformed re-launch.

## Windows: the running .exe

On Windows you can't delete an .exe that's currently executing. `ven
uninstall` handles this by renaming the running binary to `*.exe.old`
before walking the install dir — the same `.exe.old` trick `ven update`
uses. The orphan file is freed on the next reboot and the (otherwise
empty) `~/.ven/bin/` folder can be removed by hand at that point.

The deferred action is surfaced in the output:

```
[i] 1 orphan file(s) under C:\Users\you\.ven are still locked by the
    running ven process; they will vanish on reboot, after which
    C:\Users\you\.ven can be removed by hand.
```

This is expected, not an error. Exit code stays 0 unless something else
failed.

## JSON shape

`ven uninstall --json --dry-run`:

```json
{
  "status": "dry-run",
  "scope": "all",
  "needs_elevation": false,
  "plan": {
    "user_install_root": "/home/you/.ven",
    "system_artifacts": [],
    "data_dir": "/home/you/.ven",
    "data_dir_source": "default",
    "data_dir_is_relocated": false,
    "pointer_file": null,
    "user_path_entries": [],
    "system_path_entries": [],
    "user_env_vars": ["VEN_HOME"],
    "rc_files_to_clean": ["/home/you/.bashrc", "/home/you/.profile"],
    "needs_elevation": false,
    "current_exe": "/home/you/.ven/bin/ven"
  }
}
```

`ven uninstall --json -y` (real execution):

```json
{
  "status": "ok",
  "scope": "all",
  "plan": { /* same shape as above */ },
  "report": {
    "removed_dirs": ["/home/you/.ven"],
    "removed_files": ["/home/you/.config/ven/config.toml"],
    "stripped_path_entries": [],
    "removed_env_vars": ["VEN_HOME"],
    "deferred_actions": [],
    "warnings": [],
    "errors": []
  }
}
```

`status` values: `ok` (success), `partial` (completed with errors — see
`report.errors`), `noop` (nothing was installed), `needs_elevation` (system
layer detected without sufficient privileges; exit code 1), `dry-run`.

## Exit codes

| Code | Meaning |
|------|---------|
| `0`  | Uninstall succeeded, or no-op (nothing was installed) |
| `1`  | Partial failure (see report), needs elevation, or invalid flag combo |
| `2`  | (reserved for future "user cancelled at confirm prompt" exit) |

## Fallback scripts

The same teardown is shipped as standalone scripts alongside the binary:

- **Windows**: `~\.ven\bin\ven-uninstall.ps1` (canonical source:
  [`scripts/uninstall.ps1`](../../scripts/uninstall.ps1))
- **Unix**: `~/.ven/bin/ven-uninstall` (canonical source:
  [`scripts/uninstall.sh`](../../scripts/uninstall.sh))

Use these when:

- Your `ven` binary is broken / missing from PATH and can't self-execute.
- You're scripting a CI matrix and don't want to first install ven just
  to uninstall it.
- You prefer reading the shell version before trusting the Rust impl.

Knobs (env vars, since these need to stay sh-pipe-friendly):

```bash
VEN_UNINSTALL_DRY_RUN=1     ven-uninstall      # plan-only run
VEN_UNINSTALL_USER_ONLY=1   ven-uninstall      # skip system layer
VEN_UNINSTALL_SYSTEM_ONLY=1 ven-uninstall      # skip user layer
```

Both scripts are byte-for-byte equivalent in behavior to the native
command. If you find a discrepancy, please open an issue — the two paths
are meant to stay in lock-step.

## Recovery if it fails halfway

`ven uninstall` is fully idempotent. If a single step fails (permission
denied on one rc file, a runtime that's currently in use, etc.) it
records the error in `report.errors` and continues. Re-running converges
to the clean state.

If the native binary refuses to start at all (because of a corrupt
install you're trying to uninstall), fall back to the bundled script:

```bash
# Unix
~/.ven/bin/ven-uninstall

# Windows
& "$env:USERPROFILE\.ven\bin\ven-uninstall.ps1"
```

Or, if even the bundled script is gone, the canonical sources in the
repo can be curled directly:

```bash
# Unix
curl -fsSL https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/uninstall.sh | sh

# Windows
irm https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/uninstall.ps1 | iex
```

## Related

- [`ven path`](path.md) — manage where ven stores its data; uninstall
  honors a relocated storage root automatically.
- [`ven update`](update.md) — self-update the binary in place. Shares the
  Windows `.exe.old` self-orphan helper with uninstall.
- [`ven delete`](delete.md) — delete a single runtime version, NOT the
  whole install.
