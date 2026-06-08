# ven update

Self-update ven to the latest published release — no need to re-run the installer or touch PATH.

## Overview

`ven update` is **self-update for the ven binaries themselves**.

It is the canonical upgrade path once ven is installed. The command:

- detects where the currently-running `ven` lives (user install / system install / portable)
- fetches the latest release metadata from `https://api.github.com/repos/bhuwanb23/ven/releases/latest`
- downloads the platform-specific *combined* asset (`ven-{os}-{arch}.zip` on Windows, `.tar.gz` everywhere else)
- verifies it against the `SHA256SUMS` manifest published with the same release
- swaps `ven` **and** `ven-launcher` in place, atomically per file
- auto-elevates (UAC on Windows, `sudo` on Unix) if the install directory needs admin

> **Not the same as `ven upgrade`** — `ven upgrade` updates *project packages*; `ven update` updates *ven itself*.

## Upgrading from ven &lt; 0.1.7

`ven update` was added in **v0.1.7**. Older binaries do not have this subcommand
(`error: unrecognized subcommand 'update'`).

**Bootstrap once** with the install one-liner or `ven-setup`, then use
`ven update` on the new binary:

```powershell
# Windows (elevated PowerShell for system mode)
$env:VEN_FORCE_INSTALL = 'true'
irm https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.ps1 | iex
```

```bash
# Linux / macOS
VEN_FORCE_INSTALL=true curl -fsSL https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.sh | sh
```

Or download `ven-setup-{os}-{arch}` from the [releases](https://github.com/bhuwanb23/ven/releases) page.

### Multiple installs / PATH still shows an old version

If you installed a newer ven but `ven --version` is unchanged, another copy
is winning on PATH (common on Windows: `%ProgramFiles%\ven\bin` before
`%USERPROFILE%\.ven\bin`). Run:

```bash
ven doctor
```

`ven doctor` lists every install, which one PATH uses, and what to do next.

## Usage

```bash
ven update                    # check for + apply the latest stable
ven update --check            # only report what's available; no download
ven update --version v0.1.6   # install a specific tag (rollback)
ven update --yes              # skip the "Apply?" prompt (CI / scripts)
ven update --force            # reinstall even when already on the target
ven update --json             # machine-readable report for CI gates
```

## Command Reference

### Syntax

```bash
ven update [OPTIONS]
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--check` | Report the available version without downloading anything. | `false` |
| `--version <tag>` | Install a specific release tag (`v0.1.6` or `0.1.6`). | latest stable |
| `-y`, `--yes` | Skip the confirmation prompt. Default in non-TTY shells. | `false` |
| `--force` | Reinstall even if the running version already matches the target. | `false` |
| `--json` | Emit a machine-readable `UpdateReport` and continue. | `false` |

`--reentry` is an internal flag set automatically after an elevation re-launch. Do not pass it by hand.

### Exit codes

| Code | Meaning |
|------|---------|
| `0`  | No-op (already current) **or** update applied successfully |
| `1`  | Network failure / SHA256 mismatch / write error |
| `2`  | User aborted at the confirmation prompt |

## How it works

```
1. resolve current install dir from `std::env::current_exe().parent()`
2. classify mode by path prefix: ~/.ven/bin -> user, /usr/local/bin or %ProgramFiles%\ven -> system, else portable
3. GET https://api.github.com/repos/bhuwanb23/ven/releases/{latest | tags/<tag>}
4. compare release.tag_name (stripped of leading v) against CARGO_PKG_VERSION
5. if --check OR up-to-date (and not --force): exit
6. write-probe install dir — if EACCES/ERROR_ACCESS_DENIED, re-launch self via UAC/sudo with --reentry
7. download the combined asset to a TempDir
8. fetch SHA256SUMS from the release, look up the line for our asset, verify(file, expected)
9. extract:
     Windows -> zip::ZipArchive
     Unix    -> flate2::GzDecoder + tar::Archive
10. find `ven` (or ven.exe) and `ven-launcher` inside the extracted tree (depth-limited walk)
11. self-replace each binary:
     Windows -> rename target to *.exe.old, write new bytes at the original path
     Unix    -> unlink target (POSIX-safe while running), write new bytes, chmod 0755
12. print summary; tell user to open a fresh terminal and run `ven --version`
```

### Why the Windows `.exe.old` files?

Windows refuses to overwrite an executable that is currently running — even by the running process itself. The official workaround (used by Chrome, VS Code, rustup, …) is `MoveFileExW(target, target + ".old", MOVEFILE_REPLACE_EXISTING)` which renames the directory entry *without* touching the in-memory image, then writes a brand-new file at the original path. The leftover `*.exe.old` files in `%USERPROFILE%\.ven\bin` (or `%ProgramFiles%\ven\bin`) are harmless and can be deleted at any time after the next reboot.

### Why no `.old` file on Linux/macOS?

POSIX `unlink()` removes the directory entry but keeps the underlying inode alive as long as any process has the file open. The running ven keeps a valid file descriptor to its own binary; creating a new file at the same path is harmless.

## Auto-elevation

When the install dir is not writable by the current user, ven re-launches itself elevated.

- **Windows**: spawns `powershell.exe -NoProfile -NonInteractive -Command "Start-Process -FilePath '<ven.exe>' -Verb RunAs -ArgumentList @('update','--reentry',…) -Wait"`. The user sees a UAC consent prompt.
- **Unix**: re-execs through `sudo -- /path/to/ven update --reentry …`. If `sudo` is not installed, the command fails with a clear error.

The elevated child carries `--reentry` and `--yes`. If even the elevated child can't write to the install dir, it aborts instead of looping.

## Examples

### Already up to date

```bash
$ ven update
ven update
  current : 0.1.7
  target  : 0.1.7 (bhuwanb23/ven)
  dir     : C:\Users\you\.ven\bin
  mode    : user
  release : https://github.com/bhuwanb23/ven/releases/tag/v0.1.7

[ok] ven 0.1.7 is already the latest release. Nothing to do.
```

### Apply an update

```bash
$ ven update
ven update
  current : 0.1.6
  target  : 0.1.7 (bhuwanb23/ven)
  dir     : C:\Users\you\.ven\bin
  mode    : user

Apply ven 0.1.6 -> 0.1.7? [Y/n]: y
  [DL] Downloading ven-windows-x64.zip ...
  [###########################################] 100% (5.6 MB)
  [ok] SHA256 verified (da8cb4f69a9c...)
  [ok] ven 0.1.6 -> 0.1.7
  [ok] ven-launcher updated

Updated.
Open a new terminal and run `ven --version` to confirm.
```

### CI gate — fail the build if ven is stale

```bash
ven update --check --json | jq -e '.up_to_date == true'
```

### Roll back to an older release

```bash
ven update --version v0.1.5 --yes
```

## Related commands

- [`ven doctor`](doctor.md) — diagnose PATH shadowing and multiple installs
- [`ven upgrade`](upgrade.md) — upgrade *project packages*, not ven itself
- [`ven setup`](setup.md) — first-time shell hook setup (run once after first install)
- [`ven --version`](../commands-reference.md) — print the currently-running version
- [`ven-setup`](ven-setup.md) — first-time installer; only needed once

## See also

- [Distribution & installation](../features.md#16-distribution--installation)
- [Install scripts](../install-scripts.md) — `install.ps1` / `install.sh` (first-time install)
