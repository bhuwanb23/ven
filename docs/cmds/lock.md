# `ven lock`

Generate **`ven.lock`** for npm-based projects (Node or Bun in `ven.toml`).

## What it does

1. For each package in `[packages]`, runs the same **simulate-add** graph the CLI uses for `ven add`.
2. **Merges** all graphs into one pinned view (errors if the same package resolves to two different versions).
3. Copies each package's **`dist.integrity`** (npm SRI) — sha512/sha256 — into the lock entry, so `ven sync --check` and downstream tools can verify on-disk tarballs.
4. Writes JSON including **roots**, **packages** (exact versions + integrity), **edges** (with constraints and kinds), and a top-level **`content_hash`** computed over the canonical payload.

## Lockfile format

`ven.lock` is JSON. Current writer is **format v2**; v1 lockfiles are read-compatible (their packages just have no `integrity` field).

```json
{
  "lock_format_version": 2,
  "ecosystem": "npm",
  "runtime_kind": "NpmFamily",
  "runtime_version": "20",
  "roots": ["express"],
  "packages": {
    "express": {
      "version": "4.18.2",
      "integrity": "sha512-5/PsL6iGPdfQ/lKM1UuielYgv3BUoJfz1aUwU9vHZ+J7gyvwdQXFEBIEIaxeGf0GIcreATNyBExtalisDbuMqQ=="
    }
  },
  "edges": [],
  "content_hash": "<sha-256 of canonical payload>"
}
```

The `integrity` field is the same SRI string npm publishes (`sha512-...` for current registry releases). When upstream omits it (very old packages, mirrors), the field is left out.

## Usage

```bash
ven lock
```

Requires `ven.toml` with `[runtime] node` or `bun` and at least one `[packages]` entry. Output prints a one-line **integrity coverage** summary (`X/Y packages have SRI hashes`).

## See also

- [`ven sync`](sync.md) — validate, drift-audit, and install from the lockfile
