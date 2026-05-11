# Deno in ven

Deno installs are a **single statically-linked binary** pulled from GitHub Releases. Deno owns its own dependency model (URL imports, `deno.json`, `deno.lock`), so `ven` doesn't manage packages for Deno projects.

| Aspect                | Detail |
|-----------------------|--------|
| `ven.toml` key        | `runtime.deno` |
| Install dir           | `~/.ven/deno/<version>/` |
| Source                | `https://github.com/denoland/deno/releases/download/v<X.Y.Z>/deno-<target>.zip` |
| Release index         | `https://api.github.com/repos/denoland/deno/tags?per_page=100` |
| Architectures         | Windows-x86_64 · Linux-x86_64/aarch64 · macOS-x86_64/aarch64 |
| Package manager       | None — Deno uses URL imports / `deno.json` |
| Plugin                | `src/plugins/deno.rs` |
| Downloader            | `src/core/deno_install.rs` |

## Install

```bash
ven install deno 1.40.0        # exact
ven install deno 1.40          # latest 1.40.x
ven install deno latest        # newest tag
ven install deno               # interactive picker
```

### Target asset names

ven picks the right asset based on host OS + arch:

| Target                | Asset                                  |
|-----------------------|----------------------------------------|
| Windows x86_64        | `deno-x86_64-pc-windows-msvc.zip`      |
| Linux x86_64          | `deno-x86_64-unknown-linux-gnu.zip`    |
| Linux aarch64         | `deno-aarch64-unknown-linux-gnu.zip`   |
| macOS x86_64          | `deno-x86_64-apple-darwin.zip`         |
| macOS aarch64         | `deno-aarch64-apple-darwin.zip`        |

The zip contains exactly one file — the `deno` (or `deno.exe`) binary. ven extracts it directly into `~/.ven/deno/<version>/` and `chmod 755`s it on Unix.

## Activation

```toml
[runtime]
deno = "1.40"
```

When active:

| Variable           | Value                            |
|--------------------|----------------------------------|
| `PATH` (prepended) | `~/.ven/deno/<v>/` (the binary lives directly here, not under `bin/`) |
| `VEN_DENO_VERSION` | Resolved version                  |

No `DENO_DIR` is exported by ven; Deno will use its default (`~/.cache/deno` on Unix, `%LOCALAPPDATA%\deno` on Windows). Set `[env].DENO_DIR = "..."` in `ven.toml` if you want per-project caching.

## Packages

Deno doesn't have a package manager in the npm sense. Dependencies are:

- **URL imports** in your source (`import oak from "https://deno.land/x/oak@v12.6.1/mod.ts"`).
- **Import maps** in `deno.json` (recommended for refactor-friendliness).
- **`deno.lock`** for pinning.

`ven add` for a Deno project prints a notice pointing you at `deno.json` / `imports` and does nothing else. Same for `ven remove` and `ven upgrade`. The intelligence layer uses the stub adapter — Deno's web-friendly module graph isn't simulated.

### Configuration example

```toml
[runtime]
deno = "1.40"

[env]
DENO_DIR = ".deno-cache"
DENO_NO_PROMPT = "1"
```

You'll still want a `deno.json` next to `ven.toml`:

```json
{
  "imports": {
    "oak/": "https://deno.land/x/oak@v12.6.1/"
  },
  "tasks": {
    "dev": "deno run --watch --allow-net main.ts"
  }
}
```

## Common errors

| Symptom                                                                | Cause / fix                                                                |
|------------------------------------------------------------------------|----------------------------------------------------------------------------|
| `Deno <v> is not installed. Run: ven install deno <v>`                 | Pin doesn't match `~/.ven/deno/`. Install it.                              |
| `Unsupported platform for Deno download`                               | Host isn't in the supported asset matrix above.                            |
| `404` from GitHub                                                      | Often a malformed version (Deno tags are `v1.40.0`, no `v1.40`). ven strips the `v` automatically — use bare semver like `1.40.0`. |
| `ven add` says "Deno dependencies are managed by imports/deno.json"    | Expected — that's the design. Edit `deno.json` directly.                   |
