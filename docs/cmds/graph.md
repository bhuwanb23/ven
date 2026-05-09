# `ven graph`

Inspect dependency intelligence output for the current project (nearest `ven.toml`).

## Behavior

1. By default, loads the **last SQLite snapshot** for this project (`~/.ven/intelligence.db`) from a prior `ven add` / simulation.
2. If no snapshot exists, prints a **manifest snapshot**: declared `[packages]` with versions from `node_modules` when available.

## Usage

```bash
ven graph
ven graph --json
ven graph --resolve   # skip snapshot; show live manifest / install probe
```

## Options

| Flag | Description |
|------|-------------|
| `--json` | Full `SimulationResult` when a snapshot exists; otherwise `IntelGraph` JSON. |
| `--resolve` | Do not read the snapshot; use the current environment graph only. |
