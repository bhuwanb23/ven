# `ven-setup` GUI wizard (v0.2+)

Since **v0.2.0**, double-clicking `ven-setup` (or running it with no flags on a machine with a display) opens a native **eight-screen wizard** built with [eframe](https://github.com/emilk/egui) / egui. The CLI flow remains available via `--cli`.

**v0.2.1** is a full visual redesign: website-aligned cyan-on-dark theme, bundled Inter / JetBrains Mono fonts, a left step rail, clickable option cards (replacing stock radios/checkboxes), hero Welcome/Done screens, a real progress spinner, and a 920×640 viewport with the Ven logo as the window icon. On Windows, release builds link the *windows* subsystem so double-clicking does not flash a cmd window; `--cli` re-attaches to the parent console so scripted installs still print normally.

## Screens

| # | Screen | What you choose |
|---|--------|-----------------|
| 1 | **Welcome** | Logo, version, link to documentation |
| 2 | **Install mode** | User (recommended) vs System (UAC / sudo) |
| 3 | **Storage location** | `$VEN_HOME` path + Browse folder picker |
| 4 | **Shell integration** | Add to PATH; install shell hook |
| 5 | **Runtimes** | Optional pre-install: Node, Python, Go, Rust, Java, Deno, Bun, Ruby (`latest` each) |
| 6 | **Review** | Read-only summary → **Install** |
| 7 | **Progress** | Live step list (6 pipeline steps) + log output |
| 8 | **Done** | Open documentation / open terminal / Finish |

Navigation: **Back** / **Next** on every screen except Progress (no back during install) and Done. **Cancel** asks for confirmation.

## Elevation

- **Windows system install**: when you click Install, the wizard saves your choices to `%TEMP%\ven-setup-resume.toml` and launches an elevated child via UAC (`--mode system --elevated-child --resume <path>`). Complete the UAC prompt in the elevated window.
- **Unix system install**: the Progress screen shows the exact `sudo ven-setup --mode system --elevated-child --resume "…"` command to run in a terminal.

## Headless / CI

The GUI is skipped when:

- `--cli` or `--no-input` is passed
- `--elevated-child` is set (resume install)
- On Linux, neither `$DISPLAY` nor `$WAYLAND_DISPLAY` is set

Use the [CLI installer](ven-setup.md) flags in those cases.

## Build without GUI

```bash
cargo build --release --no-default-features --bin ven-setup
```

Produces a smaller CLI-only installer (~3 MB less) without eframe/wgpu.

## See also

- [`ven-setup`](ven-setup.md) — full installer reference (embedding, PATH, flags)
- [`ven setup`](setup.md) — shell hook installed during step 4 of the pipeline
