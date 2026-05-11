# Rust in ven

Rust is installed via the **official `rustup-init`** bootstrapper, but with `CARGO_HOME` and `RUSTUP_HOME` both redirected to **`~/.ven/rust/<version>/`**. That way every ven-managed toolchain is fully self-contained — no global `~/.cargo` or `~/.rustup` mutation.

| Aspect                | Detail |
|-----------------------|--------|
| `ven.toml` key        | `runtime.rust` |
| Install dir           | `~/.ven/rust/<version>/` |
| Bootstrapper          | `rustup-init.exe` / `rustup-init` cached in `~/.ven/rust/<version>/.cache/` |
| Release index         | `https://api.github.com/repos/rust-lang/rust/releases?per_page=100` |
| `rustup-init` source  | `https://win.rustup.rs/x86_64` · `https://static.rust-lang.org/rustup/dist/<triple>/rustup-init` |
| Profile               | `minimal` (only `rustc`, `cargo`, `rust-std`; no `clippy`/`rustfmt` by default) |
| Architectures         | Windows-x86_64 · Linux-x86_64/aarch64 · macOS-x86_64/aarch64 |
| Package manager       | `cargo add` / `cargo remove` |
| Plugin                | `src/plugins/rust.rs` |
| Downloader            | `src/core/rust_install.rs` |

## Install

```bash
ven install rust 1.75.0        # exact
ven install rust 1.75          # latest 1.75.x from GitHub releases
ven install rust stable        # newest release
ven install rust latest        # same as `stable`
ven install rust               # interactive picker
```

### What the installer actually does

1. Wipes `~/.ven/rust/<version>/` if it exists (clean slate).
2. Downloads the platform-appropriate `rustup-init` into `~/.ven/rust/<version>/.cache/` (skipped if already cached).
3. Makes it executable on Unix (`chmod 755`).
4. Runs:

   ```bash
   CARGO_HOME=~/.ven/rust/<v>  RUSTUP_HOME=~/.ven/rust/<v>  \
   rustup-init -y --no-modify-path --profile minimal --default-toolchain <v>
   ```

5. Verifies `bin/cargo[.exe]` exists, otherwise the install fails.

Because `CARGO_HOME` and `RUSTUP_HOME` are the same directory, you'll see both the rustup layout (`toolchains/`, `update-hashes/`, `settings.toml`) **and** the cargo layout (`bin/`, `registry/`, `git/`, `config.toml`) merged in `~/.ven/rust/<version>/`.

## Activation

```toml
[runtime]
rust = "1.75"
```

When active:

| Variable             | Value                                          |
|----------------------|------------------------------------------------|
| `PATH` (prepended)   | `~/.ven/rust/<v>/bin`                          |
| `VEN_RUST_VERSION`   | Resolved version (e.g. `1.75.0`)               |
| `CARGO_HOME`         | `~/.ven/rust/<v>`                              |
| `RUSTUP_HOME`        | `~/.ven/rust/<v>`                              |

This means **the registry cache, the `~/.cargo/config.toml` equivalent, the toolchain installs, and the credentials file are all per-ven-version**. Switching between projects with different Rust pins gives each one a clean, isolated cargo environment.

## Packages — `cargo add`

When Rust is the primary runtime:

```bash
ven add serde                      # cargo add serde
ven add serde@1                    # cargo add serde@1
ven add serde --features derive    # any extra flags pass through to cargo
```

Before the first add, ven runs `cargo init --name <folder>` if `Cargo.toml` is missing.

`ven upgrade <crate>` runs `cargo update -p <crate>`. `ven remove` runs `cargo remove`.

Like Go, there is **no real dependency-intelligence adapter** for Rust — `ven add` defers to cargo's own resolver and writes the pin into `ven.toml`'s `[packages]`.

### Configuration example

```toml
[runtime]
rust = "1.75"

[packages]
serde = "@1"
tokio = "@1.32"
anyhow = "@latest"
```

## Common errors

| Symptom                                                                | Cause / fix                                                                          |
|------------------------------------------------------------------------|--------------------------------------------------------------------------------------|
| `rustup-init failed to install Rust <v>`                               | Often a network / proxy issue. Check `https://static.rust-lang.org/` is reachable.   |
| `Unsupported platform for rustup-init download`                        | ven currently only lists `rustup-init` URLs for x86_64/aarch64 macOS/Linux/Windows. |
| `Rust <v> is not installed. Run: ven install rust <v>`                 | The pin in `ven.toml` doesn't match `~/.ven/rust/<v>/bin/cargo`.                      |
| Want `clippy` / `rustfmt`                                              | The default profile is `minimal`. After activation, run `rustup component add clippy rustfmt` — it lands inside the project's `RUSTUP_HOME`. |
| Mixing system `~/.cargo` with ven's                                    | Don't. ven's activation explicitly overrides `CARGO_HOME` so they can't collide.     |
