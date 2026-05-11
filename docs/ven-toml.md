# `ven.toml` reference

`ven.toml` is the per-project manifest. ven discovers it by **walking up** from the working directory (like Git's `.git`) and parses it with `serde` / `toml` into `VenConfig` (`src/core/config.rs`).

Every section is optional. An empty file is valid but useless — ven won't apply anything without at least one `[runtime]` key.

## Full schema

```toml
[runtime]
# At least one of these should be non-empty. Multiple are allowed —
# every set field contributes to PATH and toolchain env vars at activation.
node   = "20"            # Node.js          → ~/.ven/node/<resolved>/
python = "3.12"          # Python           → ~/.ven/python/<resolved>/   (Windows install only; venv on Unix)
go     = "1.22"          # Go               → ~/.ven/go/<resolved>/
rust   = "1.75"          # Rust toolchain   → ~/.ven/rust/<resolved>/     (CARGO_HOME + RUSTUP_HOME)
java   = "21"            # Adoptium Temurin → ~/.ven/java/<resolved>/     (JAVA_HOME)
deno   = "1.40"          # Deno             → ~/.ven/deno/<resolved>/
bun    = "1.0"            # Bun              → ~/.ven/bun/<resolved>/
ruby   = "3.4"           # MRI Ruby         → ~/.ven/ruby/<resolved>/     (GEM_HOME + GEM_PATH)

[packages]
# Free-form pins per ecosystem. `ven add` / `ven remove` / `ven upgrade`
# keep this section consistent with the package manager native to the
# project's primary runtime (see below).
express = "^4.18.2"
lodash  = "*"

[env]
# Arbitrary key/value pairs applied after PATH + toolchain vars.
# Useful for app config that should follow the project around.
NODE_ENV = "development"
PORT     = "3000"
DATABASE_URL = "postgres://..."

[venv]
# Optional. Hooks prepend `./venv` (or legacy `./.venv`) before the
# ven-managed Python when present. Set `auto_path = false` to disable.
auto_path = true
```

## `[runtime]` keys in detail

Every key accepts one of:

| Spec       | Behavior                                                       |
|------------|----------------------------------------------------------------|
| `latest`   | Highest installed version                                       |
| `lts`      | Same as `latest` for non-Node runtimes; Node treats it specially (even majors only) |
| `X`        | Highest installed `X.*.*`                                       |
| `X.Y`      | Highest installed `X.Y.*`                                       |
| `X.Y.Z`    | Used verbatim — must exist under `~/.ven/<lang>/<X.Y.Z>/`       |
| `stable`   | Rust only — alias for `latest`                                 |

The resolvers live in `src/core/config.rs` (`resolve_<lang>_version`). Empty / missing keys are skipped. A project can pin many languages at once:

```toml
[runtime]
node   = "20"
python = "3.12"
go     = "1.22"
```

`ven status` and `ven-launcher --show-env` print exactly what activation would apply for the current directory.

For per-language download sources, install layout, and activation env vars, see the [language deep dives](languages.md).

## `[packages]` — primary-runtime semantics

The `[packages]` table is **typed by the primary runtime** that ven detects from `[runtime]`. There's an ordering inside `src/intelligence/adapters/mod.rs::adapter_from_ven_config`, but the practical rule is:

| Primary runtime | What `ven add <name>[@<ver>]` does                                      |
|-----------------|--------------------------------------------------------------------------|
| Node / Bun      | Dependency-intelligence graph simulation, then `npm install` / `bun add` |
| Python          | `pip install` (uses venv when present, ven-managed Python otherwise)     |
| Go              | `go mod init` (first time), then `go get`                                |
| Rust            | `cargo init` (first time), then `cargo add`                              |
| Ruby            | `gem install` (honors activation's `GEM_HOME`)                           |
| Java            | Prints a notice — use Maven/Gradle                                       |
| Deno            | Prints a notice — edit `deno.json` / imports                             |

The pin format follows each ecosystem's conventions:

- **npm / Bun:** `^X.Y.Z`, `~X.Y`, `*`, `latest`, exact `X.Y.Z`
- **pip:** `==X.Y.Z`, `>=X.Y`, `~=X.Y`, `<X`, `*`
- **Go modules:** `@vX.Y.Z`, `@latest`
- **Cargo:** `@X`, `@X.Y.Z`, `@latest`
- **Rubygems:** `>=X.Y`, exact `X.Y.Z`, `*`

Prefer **`ven init`** / **`ven add`** over hand-editing — they pick the right pin syntax for you.

## `[env]`

A free-form map of strings → strings. ven applies it **after** PATH and toolchain vars, so user keys can reference things ven already set (e.g. `RAILS_ENV`, `DATABASE_URL`, `NODE_ENV`). Avoid setting `PATH` here — the activation layer skips that key on purpose so it can't clobber the runtime overlay.

## `[venv]` (Python only)

Currently exposes one field:

| Field        | Default | Meaning                                                                  |
|--------------|---------|--------------------------------------------------------------------------|
| `auto_path`  | `true`  | If `true`, hooks prepend `./venv` (or legacy `./.venv`) to PATH when a `pyvenv.cfg` is present. Set `false` to keep the ven-managed Python on top instead. |

The `VEN_SKIP_PROJECT_VENV=1` env var (set by `ven deactivate`) takes precedence and disables the prepend until you run `ven-use` again.

## End-to-end example

```toml
# Full-stack project: a Node API server and a Python data pipeline,
# both pinned to specific runtimes with secure-by-default env vars.

[runtime]
node   = "20"
python = "3.12"

[packages]
# Node side
express = "^4.18.2"
zod = "^3.22.0"

# Python side
fastapi = ">=0.110"
sqlalchemy = "==2.0.30"

[env]
NODE_ENV = "production"
PYTHONDONTWRITEBYTECODE = "1"
DATABASE_URL = "postgres://localhost:5432/app"

[venv]
auto_path = true
```

Running `ven status` here will report **both** Node and Python, the highest matching installed versions, and any persisted dependency-intelligence snapshot from the last `ven add`.
