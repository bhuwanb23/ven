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

- IDE shortcuts / desktop shortcuts that should always open a shell "inside" a project.
- Users who prefer not to `eval`/`ven-use` in an existing shell.
- Corporate / locked-down environments where modifying the system `PATH` or rc files is not allowed.

For in-shell activation without a new window, use **`ven use`** or hooks after **`ven setup`**.

## Portable mode

`ven-launcher` resolves a single **storage root** (called `VEN_HOME`) on every run, then propagates that value to the spawned shell so every subsequent `ven` call inside it lands in the same place. Resolution is most-specific → least-specific:

| Order | Source                                                       | Notes                                                                 |
|------:|--------------------------------------------------------------|-----------------------------------------------------------------------|
|     1 | `$VEN_HOME` env var (if set & non-empty)                     | Explicit override; trumps everything else.                             |
|     2 | `$VEN_STORAGE_PATH` env var (if set & non-empty)             | Back-compat with the early modules that introduced this convention.    |
|     3 | `<dir-of-launcher-exe>/.ven` if that directory exists        | **Auto-portable.** Drop a `.ven/` next to `ven-launcher`, you're done. |
|     4 | `~/.ven`                                                     | Default for an installed ven.                                          |

### USB-stick / fully-portable workflow

Use the discoverable `ven-launcher-{os}-{arch}.{zip|tar.gz}` release asset (see [install-scripts.md](install-scripts.md#portable-launcher-bundle)) and:

```text
my-bundle/
├── ven                    # or ven.exe on Windows
├── ven-launcher           # or ven-launcher.exe on Windows
├── README.txt
└── .ven/                  # create this folder once; everything ven downloads
                           # (runtimes, lockfile state, doc cache) lives here
```

`./ven-launcher --show-env` will then print `VEN_HOME should be: <bundle>/.ven` and the spawned shell will see the same value via the inherited `VEN_HOME` env var. No system `PATH` change, no rc-file edits, no `~/.ven` writes — the bundle is fully self-contained and movable.

To run the same bundle in shared mode instead (use the host's `~/.ven`), just delete the sibling `.ven/` folder; `ven-launcher` will fall back to tier 4.

### Explicit override

For containers, CI, or one-off testing where you want to point a single launcher invocation at an arbitrary directory:

```sh
VEN_HOME=/srv/ci-cache/ven ./ven-launcher
```

The launcher exports the resolved value to the child shell, so `ven status`, `ven install`, and `ven sync` inside that shell will all use `/srv/ci-cache/ven`.
