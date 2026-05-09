# `ven check-add`

Non-mutating **dependency intelligence** query: resolves registry metadata, samples engine-compatible versions, runs the same simulation as `ven add`, and prints peer/pin conflicts without installing.

## Usage

```bash
ven check-add <package> [package...]
ven check-add express@4 --json
```

## Options

| Flag | Description |
|------|-------------|
| `--json` | Machine-readable output (check-add struct + simulation summary). |

## See also

- [`ven add`](add.md) — install after confirmation
- [`ven graph`](graph.md) — persisted graph snapshot
