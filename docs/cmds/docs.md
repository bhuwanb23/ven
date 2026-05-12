# `ven docs`

Show **version-pinned documentation** for an installed package — render in
the terminal, open in your browser, or diff two versions.

## What it does

1. Loads `ven.toml` and identifies the **primary runtime** (Node/Bun, Python,
   Go, Rust, Java, Ruby, Deno).
2. **Resolves the version pin** in this order:
   - `ven.lock` packages (when present)
   - `ven.toml [packages]` (strips `^` / `~` / `=` prefixes; skips
     `latest` / `*`)
   - Installed manifest (`node_modules/<pkg>/package.json` for npm)
3. Fetches the docs body from the ecosystem's canonical source and renders
   it for the terminal via [`termimad`](https://crates.io/crates/termimad).
4. Caches the body in `~/.ven/intelligence.db` (`doc_cache`) for **7 days**.

## Per-ecosystem source

| Ecosystem | Body source | Browser URL |
|-----------|-------------|--------------|
| Node / Bun | `registry.npmjs.org/<pkg>/<v>` `.readme` | `npmjs.com/package/<pkg>/v/<v>` |
| Python | `pypi.org/pypi/<pkg>/<v>/json` `info.description` | `pypi.org/project/<pkg>/<v>/` |
| Rust | docs.rs HTML (URL only — terminal render points at the URL) | `docs.rs/<pkg>/<v>/<pkg>/` |
| Go | pkg.go.dev HTML (URL only) | `pkg.go.dev/<module>@<v>` |
| Java | javadoc.io (URL only); `<pkg>` may be `groupId:artifactId` | `javadoc.io/doc/<group>/<artifact>/<v>` |
| Ruby | `rubygems.org/api/v1/gems/<pkg>.json` `info` | gem version page |
| Deno | URL passthrough | `npm:` → npmjs, `jsr:` → jsr.io, otherwise `deno.land/x/<pkg>@<v>` |

## Usage

```bash
ven docs <pkg>                       # render in terminal (markdown via termimad)
ven docs <pkg> --browser             # open canonical URL in default browser
ven docs <pkg> --diff V1 V2          # unified line diff between two versions' READMEs
ven docs <pkg> --json                # machine-readable
```

## Renderer behavior

| Context | Output |
|---------|--------|
| **TTY** (interactive shell) | `termimad` markdown render (auto-width) |
| **Non-TTY** (pipe, CI) | raw markdown text passes through unchanged |

## `--browser`

Opens the canonical URL using the [`webbrowser`](https://crates.io/crates/webbrowser)
crate, which dispatches to `cmd /c start` on Windows, `open` on macOS, and
`xdg-open` on Linux. Set `VEN_BROWSER_DRY_RUN=1` to print the URL instead
of spawning (used by tests to avoid opening real browsers in CI).

## `--diff V1 V2`

Fetches the README at both versions (cached identically), then prints a
unified line diff via [`similar::TextDiff`](https://docs.rs/similar). Per-API
surface diff (function/class signatures) is a future stretch goal — needs
per-ecosystem AST parsers.

## Cross-platform

Pure Rust deps: `reqwest`, `termimad`, `webbrowser`, `similar`. No native
bindings; identical on Windows / macOS / Linux.

## See also

- [`ven add`](add.md), [`ven lock`](lock.md), [`ven sync`](sync.md) — the version pin chain
- [security-model.md](../security-model.md) — caching strategy and TTLs
