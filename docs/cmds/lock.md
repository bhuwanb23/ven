# `ven lock`

Generate **`ven.lock`** for npm-based projects (Node or Bun in `ven.toml`).

## What it does

1. For each package in `[packages]`, runs the same **simulate-add** graph the CLI uses for `ven add`.
2. **Merges** all graphs into one pinned view (errors if the same package resolves to two different versions).
3. Writes JSON including **roots**, **packages** (exact versions), **edges** (with constraints and kinds), and a **`content_hash`** for integrity.

## Usage

```bash
ven lock
```

Requires `ven.toml` with `[runtime] node` or `bun` and at least one `[packages]` entry.

## See also

- [`ven sync`](sync.md) — validate and install from the lockfile
