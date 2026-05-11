# Supported languages

`ven` is a multi-language version manager. Each runtime is implemented as a **plugin** behind a single trait (`LanguagePlugin`) — meaning the install, list, activate, and uninstall paths are uniform, but the **download source** and **layout** differ per language.

This page is the entry point. See the per-language deep dives for full detail:

| Language    | Doc                                  | Source                                            | Package tool wired into `ven add` |
|-------------|--------------------------------------|---------------------------------------------------|-----------------------------------|
| Node.js     | [`languages/node.md`](languages/node.md)     | `nodejs.org/dist` (SHA-256 verified)              | npm registry (full intelligence)  |
| Python      | [`languages/python.md`](languages/python.md) | `python.org/ftp` (Windows embeddable zip)         | `pip`                             |
| Go          | [`languages/go.md`](languages/go.md)         | `go.dev/dl`                                       | `go get` / `go mod`               |
| Rust        | [`languages/rust.md`](languages/rust.md)     | `rustup-init` from `static.rust-lang.org`         | `cargo add`                       |
| Java (JDK)  | [`languages/java.md`](languages/java.md)     | Eclipse Adoptium Temurin                          | _(use Maven/Gradle directly)_     |
| Deno        | [`languages/deno.md`](languages/deno.md)     | `github.com/denoland/deno/releases`               | _(use `deno.json` / imports)_     |
| Bun         | [`languages/bun.md`](languages/bun.md)       | `github.com/oven-sh/bun/releases`                 | `bun add` (npm-compatible)        |
| Ruby (MRI)  | [`languages/ruby.md`](languages/ruby.md)     | RubyInstaller2 (Win) / `ruby/ruby-builder` (Unix) | `gem install`                     |

## How a language plugin works

Every plugin implements the same four operations (`src/plugins/mod.rs`):

| Method                         | What it returns                                                  |
|--------------------------------|------------------------------------------------------------------|
| `install_version(version)`     | Downloads + extracts a specific version into `~/.ven/<lang>/<version>/` |
| `list_installed()`             | Reads `~/.ven/<lang>/`, returns sorted versions (newest first)   |
| `bin_path(version)`            | The directory that goes on `PATH` when this version is active    |
| `latest_version()`             | Highest version available from the upstream release index        |

CLI commands (`ven install`, `ven list`, `ven use`, shell hooks) call **only** these four operations — adding a new language is a matter of writing one more plugin + `core/*_install.rs` module.

## Storage layout

After running e.g. `ven install node 20`, `ven install python 3.12.7`, and `ven install ruby 3.4.2`:

```
~/.ven/
├── .cache/                 # downloaded archives (re-used between installs)
├── cache/registry.db       # npm package metadata cache (SQLite)
├── intelligence.db         # snapshots, lock validations, package/dep cache
├── node/
│   └── 20.20.2/            # contents of node-v20.20.2-<os>-<arch>.{zip,tar.gz}
├── python/
│   └── 3.12.7/             # embeddable zip + pip bootstrapped
├── go/
│   └── 1.22.3/
├── rust/
│   └── 1.75.0/             # CARGO_HOME + RUSTUP_HOME live here
├── java/
│   └── 21.0.3+9/           # Adoptium Temurin JDK tree (bin/, lib/, conf/, …)
├── deno/
│   └── 1.40.0/             # single `deno` / `deno.exe` binary
├── bun/
│   └── 1.0.20/             # single `bun` / `bun.exe` binary
└── ruby/
    └── 3.4.2/              # full Ruby tree (bin/, lib/ruby/gems/<abi>/, …)
```

Override the root with **`VEN_STORAGE_PATH`** (every downloader reads it).

## Version specs accepted by `ven install`

All resolvers accept the same shapes, in this order of precedence:

| Input            | Behavior                                                                                        |
|------------------|-------------------------------------------------------------------------------------------------|
| `latest`         | Highest available version from the upstream index                                               |
| `stable` / `lts` | Same as `latest` for languages that don't expose a separate LTS line; Node treats `lts` literally |
| Exact `X.Y.Z`    | Installed verbatim (must exist upstream)                                                        |
| `X.Y`            | Highest patch under that minor                                                                  |
| `X`              | Highest minor+patch under that major                                                            |

If you pass a major-only spec like `ven install node 20`, ven hits the upstream release index, picks the newest matching `20.x.y`, and uses that as the canonical install directory name (`~/.ven/node/20.20.2/`).

The activation path uses the same resolvers (`core/config.rs::resolve_*_version`), but matches against **installed** versions instead of upstream listings.

## What activation actually does

When you `cd` into a directory whose `ven.toml` declares a `[runtime]`, the shell hook calls `ven shell activate <dir>` and evaluates the output. That output (`src/shell/activation.rs`):

1. Walks up to find `ven.toml`.
2. For **every** non-empty `[runtime].<lang>`:
   - Resolves the spec against installed versions.
   - Locates the bin dir via the plugin (`<install_dir>/bin` on Unix, `<install_dir>` for Windows Node/Deno/Bun).
   - Prepends it to `PATH`.
   - Exports a language-specific env var (`VEN_NODE_VERSION`, `JAVA_HOME`, `GOROOT`, `CARGO_HOME`, `GEM_HOME`, …).
3. Applies user `[env]` keys after that.

If the toolchain isn't installed, activation prints a clean **`MissingToolchain`** error telling you which `ven install …` to run.

## When ven manages packages and when it doesn't

`ven add`, `ven remove`, and `ven upgrade` pick a code path based on which `[runtime]` key is set in `ven.toml`:

| Project's primary runtime | `ven add` behavior                                                                 |
|---------------------------|-------------------------------------------------------------------------------------|
| `node`                    | Full dependency intelligence (graph + engine checks) → `npm install <pkg>@<ver>`    |
| `bun`                     | `bun add <pkg>` (same npm registry, simpler resolver)                               |
| `python`                  | `<activated python> -m pip install <spec>`                                          |
| `go`                      | `go mod init` (if missing), then `go get <spec>`                                    |
| `rust`                    | `cargo init` (if missing), then `cargo add <spec>`                                  |
| `ruby`                    | `gem install <name> [-v <version>]`                                                 |
| `java`                    | Notice only — Maven / Gradle owns this                                              |
| `deno`                    | Notice only — `deno.json` / imports own this                                        |

In every case the resulting pin is written back into `[packages]` in `ven.toml` so the project stays reproducible.

## Adding support for a new language

1. Add `src/core/<lang>_install.rs`: implement a `<Lang>Downloader` (`get_install_dir`, `get_bin_path`, `list_installed`, `download`) and `install_<lang>(&dl, version)` / `fetch_<lang>_release_versions()` / `resolve_<lang>_version_spec`.
2. Add `src/plugins/<lang>.rs` exposing a thin `<Lang>Plugin` that implements `LanguagePlugin`.
3. Register it in `src/plugins/registry.rs::PluginRegistry::new()`.
4. Wire the `runtime.<lang>` key into `core/config.rs` (`RuntimeConfig`) and `src/shell/activation.rs`.
5. Extend `src/cli/install/fetch.rs` and `src/cli/install/validate.rs` so version listing and the post-install binary check know about the new key.
6. Optionally implement `cmd_add_<lang>` / `cmd_remove_<lang>` / `cmd_upgrade_<lang>` to manage packages.

The intelligence layer (`src/intelligence/adapters/`) currently has a real adapter only for the **npm family** (Node + Bun); other ecosystems use `GenericStubAdapter` and return a deterministic "stub-compatible" simulation. Pull requests adding real adapters for PyPI, crates.io, etc. are welcome.
