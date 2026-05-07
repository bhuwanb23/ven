# Supported languages / runtimes

ven treats each runtime as a **language plugin**: discovery, install paths, version normalization, and `ven.toml` keys differ slightly.

General workflow:

1. **Install a runtime**: `ven install <language> [version]` (omit version for interactive pick where implemented).
2. **Pin in project**: `ven init` / edit `[runtime]` in `ven.toml`.
3. **Activate in shell**: hooks (`ven setup`) call **`ven-use`** on directory change, or run `ven use` manually and evaluate output.

## Node (`node`)

- **Install**: `ven install node 20` or `ven install node lts` (aliases depend on plugin).
- **Binary**: `node`, `npm`, `npx` on PATH when active.
- **Packages**: `[packages]` npm entries; `ven add` / `ven remove` / `ven upgrade`.

## Python (`python`)

- **Install**: `ven install python 3.12` (exact availability depends on upstream listings).
- **Venv**: `[venv]` legacy section or runtime-driven venv paths; Windows vs Unix paths handled in resolver.
- **Packages**: pip/PyPI via `[packages]` where configured.

## Go (`go`)

- **Install**: `ven install go 1.22.x` etc.
- **Env**: `GOROOT`, `GOPATH` / module cache as required by the plugin when active.

## Rust (`rust`)

- **Install**: `ven install rust stable` or explicit versions where listed.
- **Notes**: Typically uses rustup-style installs managed through ven’s store layout.

## Java (`java`)

- **Install**: `ven install java 21` (Temurin/OpenJDK style builds depending on source).
- **`JAVA_HOME`**: Set when runtime is active.

## Deno (`deno`)

- **Install**: `ven install deno 2.x` etc.
- **Packages**: JSR/npm compatibility layers follow Deno conventions in `[packages]` mapping.

---

If `ven install <lang>` fails, check `ven install <lang> --dry-run -v` and ensure your OS/architecture is supported by that plugin’s fetch logic.

See also [ven-toml.md](ven-toml.md) for **`[runtime]`** keys (`node`, `python`, `go`, …) — you can pin more than one language per project.
