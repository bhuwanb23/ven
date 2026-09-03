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
- A runtime you made global with `ven set global` still resolves to an older or different install

## Machine PATH shadows User PATH globals (Windows)

`ven set global` writes only to the **User PATH** so it never needs admin
rights — but on Windows the **Machine PATH is always searched before the
User PATH**. If the same tool (node, python, go, …) is also installed at
machine scope — e.g. a corporate install under `C:\Program Files` — that
copy wins, and the ven-managed global appears to have no effect. This is
standard Windows lookup order, not a ven defect: no User-scope entry can
out-rank a machine-scope one.

To confirm which copy actually runs:

- `where node` (or `where python`, `where go`, …) lists every candidate
  in resolution order — the first hit is what a new shell runs.
- This command reports which copy of **ven itself** PATH resolves first;
  the same ordering applies to the runtime binaries above.

Fixes, in order of preference:

1. Remove or reorder the machine-scope entry (needs admin — your IT
   department for a corporate install).
2. Scope the runtime to the project instead with `ven use` / shell
   hooks: session activation prepends the ven-managed `bin` ahead of
   *everything*, machine scope included.

## See also

- [`ven set global`](set.md) — make an installed runtime globally
  available on the User PATH (no admin)
- [`ven update`](update.md) — self-update (v0.1.7+)
- [Install scripts](../install-scripts.md)
