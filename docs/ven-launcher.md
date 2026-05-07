# ven-launcher

`ven-launcher` is a companion binary shipped with this repo. It:

1. Takes an optional **project path** (directory or path to `ven.toml`).
2. Walks up from that root to find **`ven.toml`** (same rules as `ven use`).
3. Either **opens a new terminal** with the resolved runtime PATH/env, or prints env.

## Usage

```text
ven-launcher [PROJECT]
```

- **`PROJECT`** omitted: use current working directory as search root.
- **`--show-env`**: print resolved environment / PATH information instead of spawning a terminal.

## When to use it

- IDE shortcuts / desktop shortcuts that should always open a shell “inside” a project.
- Users who prefer not to `eval`/`ven-use` in an existing shell.

For in-shell activation without a new window, use **`ven use`** or hooks after **`ven setup`**.
