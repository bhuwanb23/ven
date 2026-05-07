# ven.toml reference

ven discovers **`ven.toml`** by walking up from the working directory (like Git).

Typical sections:

## `[runtime]`

Version pins **per language**. Only set keys for runtimes this project uses; empty keys are ignored.

| Key | Meaning |
|-----|---------|
| **`node`** | Node.js version pin (e.g. `20`, `20.11.1`, `lts`). |
| **`python`** | Python version pin (installed under `~/.ven/python` on supported platforms). |
| **`go`** | Go toolchain version. |
| **`rust`** | Rust toolchain version. |
| **`java`** | JDK version. |
| **`deno`** | Deno version. |
| **`ruby`** | MRI Ruby (`X.Y.Z` under `ven install`). Sets `GEM_HOME` / `GEM_PATH` plus `VEN_RUBY_VERSION` when active. |

Activation builds a **PATH prepend list** (and toolchain vars like `JAVA_HOME`, `GEM_HOME`, `GOROOT`, …) from **every** non-empty `[runtime]` field that resolves to an installed toolchain. Use **`ven status`** and **`ven-launcher --show-env`** to verify what your project applies.

Example:

```toml
[runtime]
node = "20"
python = "3.12"
```

## `[packages]`

Declares dependency pins per ecosystem (npm, pip, etc.). `ven add`, `ven remove`, and `ven upgrade` keep this section consistent when possible.

Exact keys mirror internal serializers — prefer **`ven init`** / **`ven add`** to avoid typos.

## `[env]`

Arbitrary environment variables applied when the shell overlay runs (after PATH/runtime vars).

## `[venv]` (legacy / Python)

Legacy block for Python venv behavior: **`auto_path`** (default `true`). Hooks may prepend `./venv` when it exists.

---

For authoritative output for your ven version, run **`ven init --template`** and inspect the generated `ven.toml`.
