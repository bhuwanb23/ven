# ven doctor

Diagnose ven installation health: multiple copies on disk, PATH shadowing, and
whether your build supports `ven update`.

## Usage

```bash
ven doctor
ven doctor --json
```

## What it checks

- Known install locations (`~/.ven/bin`, `%ProgramFiles%\ven\bin`, `/usr/local/bin`)
- Every `ven` binary reported by `where` / `which`
- Which copy PATH resolves first
- Whether each copy is new enough for `ven update` (v0.1.7+)

## When to run it

- `ven update` fails with **unrecognized subcommand**
- You re-installed but `ven --version` did not change
- Both user and system installs exist

## See also

- [`ven update`](update.md) — self-update (v0.1.7+)
- [Install scripts](../install-scripts.md)
