# ven performance: why commands feel slow, and what we did about it

`ven` is a native Rust binary, so a trivial command like `ven --version` should
return in tens of milliseconds. If it takes **hundreds of milliseconds warm** or
**multiple seconds right after an update**, the bottleneck is almost never ven's
own code — it is the environment the binary runs in. This page documents the
two real causes we found and measured, and the fixes (shipped + manual).

## Measured baseline (Windows, corporate / Defender host)

| Command | Warm | Cold (first run of a fresh binary) |
|---------|------|--------------------------------------|
| `ven --version` | ~220-300 ms | ~4-8 s |
| `ven --help` | ~230 ms | ~4-8 s |
| `git --version` | ~110-140 ms | — |
| `node --version` | ~80-100 ms | — |
| `ven-launcher --version` (1.3 MB) | ~70-80 ms | ~2.7 s |

`ven --version` spends < 20 ms of CPU time. The rest is process startup:
PE image loading plus Windows Defender scanning the unsigned 13 MB executable
on every execution. Signed binaries (`git`, `node`) are trusted and skip the
scan — that is why they return in ~100 ms despite being larger or equally
large.

## Root cause 1: Defender scans the unsigned binary on every run

- `ven.exe` is **unsigned** (no Authenticode signature), so Defender's
  real-time protection inspects the whole image each time it is launched.
- A fresh copy of the same exe in `%TEMP%` (no reputation yet) took **5-8 s
  per run**; the installed copy settled at ~220-300 ms once Defender had
  cached it, but still ~2-4x slower than signed peers.
- After every `ven update`, the new binary is a brand-new file → the cold
  4-8 s scan happens again once.

### Shipped fix: installer adds a Defender exclusion

`ven-setup` now tries to add the ven install root to the Windows Defender
exclusion list at the end of a successful install
(`Add-MpPreference -ExclusionPath <ven-root>`). When the install is elevated
(system install, or a user run from an admin shell) the exclusion lands and
subsequent invocations drop to loader-only cost (~50-80 ms). If the current
process lacks admin rights the step is skipped with a hint — it never fails
the install.

Manual equivalent (run once, elevated PowerShell):

```powershell
Add-MpPreference -ExclusionPath "$env:USERPROFILE\.ven"        # user install
Add-MpPreference -ExclusionPath "$env:ProgramFiles\ven"         # system install
```

You can verify whether an exclusion is active with (elevated):

```powershell
Get-MpPreference | Select-Object -ExpandProperty ExclusionPath
```

### Long-term fix: code-sign the release binaries

The definitive fix is an Authenticode code-signing certificate (OV or EV)
used in the release pipeline. Signed binaries are trusted by SmartScreen /
Application Control and skip Defender's per-execution scan entirely. Until
ven ships signed, the Defender exclusion is the practical mitigation.

## Root cause 2: the shell hook spawned ven on every `cd`

The bash/zsh/fish/PowerShell hooks ran `ven shell activate <dir>` on every
directory change **even when there was no `ven.toml` anywhere up the tree**.
Since activating a project with no manifest is a no-op, that was a pure
~200-300 ms subprocess spawn (plus the Defender scan above) paid on every
`cd` into any ordinary directory.

### Shipped fix: fast path when no `ven.toml` exists

The hooks already compute a TOML "signature" by walking up the tree. When
that signature is empty (no `ven.toml` found), the hook now:

1. restores the baseline `PATH` / clears ven-owned env vars, and
2. **returns without spawning `ven` at all**.

Directories with a `ven.toml` are unaffected — they still call
`ven shell activate` exactly as before. After running `ven setup` /
`ven shell install` once with the updated hooks, `cd` between non-ven
directories is instant.

## Other notes

- **Binary size**: 13 MB is mostly reqwest/rustls, rusqlite (bundled SQLite),
  tokio, and the archive stack. Size is a secondary factor; the scan is what
  dominates, and signing / exclusion neutralizes it regardless of size.
- **`ven-shell-activate` itself** is fast (< 20 ms CPU). The perceived lag was
  process spawn + Defender, not the resolution logic.
- On **Unix** (macOS/Linux) none of this applies — Defender doesn't run there
  and `ven --version` returns in a few milliseconds.