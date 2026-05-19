// Release timeline driving /changelog. Order: newest first.
// Source: website_design/changelog.md.

export const RELEASES = [
  {
    version: 'v0.1.7',
    date: 'May 19, 2026',
    tag: 'minor',
    summary:
      'Two new top-level commands. `ven update` self-updates `ven` + `ven-launcher` to the latest GitHub release with SHA256 verification and in-place binary swap (no re-install, no PATH edits; system installs auto-elevate through UAC / `sudo`). `ven uninstall` replaces the long copy-paste teardown snippet with a single confirmed, dry-run-capable command — removes binary + every runtime + cache + state + persisted `VEN_HOME` + pointer file + PATH entries + shell rc-file blocks, idempotently. Honors a relocated storage root (`ven path set D:\\ven` → uninstall removes both `~/.ven` AND `D:\\ven`). Plus a website docs overhaul: every CLI command now has a dedicated page (was 8 of 22, now 23 of 23) and the sidebar is reorganised into six groups — Getting started / Packages / Insight / Runtimes / Maintenance / Languages.',
    sections: {
      new: [
        '`ven update` self-update command. Detects the install dir from `std::env::current_exe().parent()`, hits the GitHub releases API for `bhuwanb23/ven`, picks the platform-specific combined asset (`ven-{os}-{arch}.{zip|tar.gz}`), verifies it against the release\'s SHA256SUMS manifest, and swaps both binaries atomically. Windows uses the rename-aside trick (`*.exe.old`) since the OS refuses to overwrite a running `.exe`; Unix uses POSIX `unlink + write` (the open file descriptor in the running ven keeps pointing at the dead inode until the process exits). Auto-elevates via UAC on Windows / `sudo` on Unix when the install dir requires admin; the elevated child carries an internal `--reentry` flag so the elevation loop terminates after one hop.',
        '`ven update --check` reports the latest available version without downloading or applying anything — safe in CI. `--json` emits a structured `UpdateReport` (current, target, up_to_date, action, install_dir, install_mode, asset, repo, release_url) for CI gates. `--version v0.1.6` installs a specific tag, letting you roll back to an older release.',
        '`ven uninstall` full-nuke teardown command (replaces the 100-line copy-paste PowerShell/shell snippet). One confirmed, dry-run-capable invocation removes the user install root (`~/.ven`), the system install (`%ProgramFiles%\\ven` / `/usr/local/bin/{ven,ven-launcher,ven-setup}` + `/etc/profile.d/ven.sh`), the persisted `VEN_HOME` user env var, the pointer file (`~/.config/ven/config.toml`), and the ven-managed blocks (`# >>> ven env >>>`, `# >>> ven-setup PATH >>>`, `# >>> ven shell hook >>>`) from every shell rc file. Honors a relocated `$VEN_HOME` so a user who ran `ven path set D:\\ven` gets BOTH dirs cleaned in one shot — fixes the long-standing orphan-data bug in the manual snippet. On Windows the running `.exe` is renamed to `*.exe.old` (same trick `ven update` uses) so the install dir is unblocked even though the OS won\'t delete the running process; the orphan vanishes on reboot.',
        '`ven uninstall` flags: `--dry-run` prints the plan and touches nothing (combine with `--json` to capture the plan in a CI gate); `-y` / `--yes` skips the confirm prompt; `--user-only` skips the system install layer (no sudo / Admin required); `--system-only` skips the user install layer (sysadmin scope); `--json` emits a structured `UninstallPlan` + `UninstallReport` (removed_dirs, removed_files, stripped_path_entries, removed_env_vars, deferred_actions, warnings, errors) for CI gates. JSON without `-y` or `--dry-run` is rejected — the intent is too ambiguous to risk.',
        'Bundled fallback scripts shipped alongside the binary: `~/.ven/bin/ven-uninstall` (POSIX) and `~\\.ven\\bin\\ven-uninstall.ps1` (Windows). Canonical sources at [scripts/uninstall.sh](https://github.com/bhuwanb23/ven/blob/main/scripts/uninstall.sh) and [scripts/uninstall.ps1](https://github.com/bhuwanb23/ven/blob/main/scripts/uninstall.ps1). Same teardown as the native command, idempotent on re-run, with `VEN_UNINSTALL_DRY_RUN=1` / `VEN_UNINSTALL_USER_ONLY=1` / `VEN_UNINSTALL_SYSTEM_ONLY=1` env-var knobs. Use when the `ven` binary itself is broken (the recovery case that motivated splitting the implementation into a shared `core::uninstaller` module that both the CLI and the install-bundled scripts converge on).',
        'New per-command doc pages: [docs/cmds/update.md](https://github.com/bhuwanb23/ven/blob/main/docs/cmds/update.md) and [docs/cmds/uninstall.md](https://github.com/bhuwanb23/ven/blob/main/docs/cmds/uninstall.md) covering the flow end-to-end, the Windows/Unix self-replace semantics, the elevation contract, the exit-code matrix, and CI examples.',
        'Website docs reorganised into six groups (Getting started / Packages / Insight / Runtimes / Maintenance / Languages) and every CLI command got its own page: list, delete, path, use, deactivate, setup, check-add, graph, why, resolve, remove, upgrade, scan, update, uninstall.',
      ],
      improved: [
        '`ven --help` examples now include both `ven update` and `ven uninstall --dry-run` / `ven uninstall` so they\'re discoverable from the top-level surface.',
        'Install page gained an explicit "Upgrade ven" panel between the downloads table and the uninstall section, with a small "v0.1.7+" pill, the standard `ven update` invocation, a CI block (`--yes --json`, `--check --json`, `--version`), and an inline note on the `ven update` vs `ven upgrade` distinction.',
        'Install page Uninstall section restructured: leads with a single `ven uninstall` block (dry-run + execute forms), with the legacy per-OS manual snippets tucked behind a collapsible "Advanced: manual uninstall (no ven binary on PATH)" disclosure for the broken-install recovery case.',
        'The Windows `*.exe.old` self-orphan rename logic that `ven update` introduced was extracted into a shared `core::uninstaller::self_orphan_windows_exe()` helper so both `update` and `uninstall` use the same code path — if you patch one, you patch the other.',
        'The `release.yml` workflow now bundles `ven-uninstall.{ps1,sh}` into the combined and launcher tarballs, and `scripts/install.{ps1,sh}` copy the matching script into the install dir alongside `ven` and `ven-launcher`. Older release archives (< v0.1.7) without the file degrade silently.',
        'docs/features.md gained new sections "Self-update — ven update" and "Uninstall — ven uninstall" plus matching `Maintenance` rows in the quick command index.',
      ],
      fixed: [
        'Long-standing leak: the manual uninstall snippet always hard-coded `%USERPROFILE%\\.ven` / `~/.ven`, so a user who relocated their storage root via `ven path set D:\\ven` ended up with an orphaned `D:\\ven` directory after running the snippet. `ven uninstall` (and the new canonical `scripts/uninstall.{ps1,sh}`) resolve `$VEN_HOME` through the same precedence chain the rest of ven uses, so a relocated root is cleaned automatically.',
        'Long-standing leak: the manual snippet never cleared the global pointer file at `~/.config/ven/config.toml` (or `%APPDATA%\\ven\\config.toml` on Windows), so a subsequent `ven` install would silently inherit the stale pointer and write its data to the dead location. Now removed via `ven_config::clear_storage_home()`.',
        'Long-standing leak: the manual snippet never unset the persisted `VEN_HOME` env var written by `ven path set`, so even after "uninstall" every new shell would still export the old VEN_HOME to npm / pip / etc. Now removed via the same `user_env::unset_user_env` helper `ven path reset` uses.',
        'Shell hook leftovers: the snippet missed the `# >>> ven shell hook >>>` blocks `ven setup` installs into `.bashrc` / `.zshrc` / `config.fish` / `$PROFILE.ps1`. The new scrubber strips all three known marker blocks (`ven env`, `ven-setup PATH`, `ven shell hook`) plus the legacy unmarked `.ven/bin` PATH lines.',
      ],
    },
  },
  {
    version: 'v0.1.6',
    date: 'May 18, 2026',
    tag: 'minor',
    summary:
      'New `ven path` command for relocating your data root to a different drive — atomic, rollback-safe, with persistent `VEN_HOME` so npm / pip / new shells inherit it without you re-exporting anything. Plus a hardening pass on the install pipeline: timeout-aware streaming downloads with retries (fixes "operation timed out" on Ruby / Deno / large tarballs behind corporate proxies), a Rust smoke-test fix (rustup-shim env vars), and an idempotent uninstall path that converges to clean state even after a partial previous run.',
    sections: {
      new: [
        '`ven path show` / `ven path set <dir>` / `ven path reset` — first-class commands for moving `~/.ven` to a different drive (the "my C: drive is full" case). Three calling conventions: interactive wizard (`ven path set D:\\ven`), explicit flags (`--move` / `--no-move` / `--pointer-only`), or fully scripted (`ven path set <dir> --move -y --json`). Moves are atomic: fast-path `rename()` when source + target share a filesystem, fall back to walk-copy-verify-sweep on cross-device. A `.ven-move.lock` blocks concurrent moves; `--force-unlock` breaks a stale lock from a crashed previous run. Any failure rolls back — ven is never left half-relocated.',
        'New `[storage].home` "pointer file" at `~/.config/ven/config.toml` (Linux/macOS) / `%APPDATA%\\ven\\config.toml` (Windows). The `$VEN_HOME` resolver now has five tiers: `$VEN_HOME` → `$VEN_STORAGE_PATH` → `<exe-dir>/.ven` (portable) → **pointer file** → `~/.ven`. Per-process env vars still win so CI scripts keep working; the pointer file is for "I moved my data once and want ven to remember".',
        'Persistent `VEN_HOME` env var management: after `ven path set`, ven writes `VEN_HOME` to your User-scope environment so new shells and child processes (npm, pip, etc.) inherit it without manual re-export. Windows uses `SetEnvironmentVariable` + a `WM_SETTINGCHANGE` broadcast so non-shell apps pick it up too; Unix appends a fenced `# >>> ven env >>>` block to `.bashrc` / `.zshrc` / `.profile` / `config.fish`, idempotent on re-run and cleanly removed on `ven path reset`.',
        'New per-command doc page [docs/cmds/path.md](https://github.com/bhuwanb23/ven/blob/main/docs/cmds/path.md) covering all subcommands, flags, the 5-tier resolver, the lock-file safety semantics, and every `--json` output shape.',
        '`ven path show --json` exposes the resolved storage root, the source that picked it (`env:VEN_HOME` / `env:VEN_STORAGE_PATH` / `portable` / `pointer` / `default`), total size on disk, free space on the volume, and the on-disk pointer path. Designed as the structured surface for "where is my ven data?" tooling and CI checks.',
      ],
      improved: [
        'All seven language installers (`bun`, `deno`, `go`, `java`, `node`, `ruby`, `rust`) now share one timeout-aware streaming downloader (`integrity::download_to_file`): 30 s connect timeout, 45 min total request budget, 60 s per-chunk stall watchdog, and a 3-attempt retry loop with exponential backoff on transient network errors. Same user-agent string everywhere (`ven/0.1.6 (deno-installer)`, etc.) so upstream mirrors can identify ven traffic. Fixes "operation timed out" failures on large tarballs (Ruby ~30 MB, Deno ~32 MB) behind slow corporate proxies / Zscaler.',
        'Every install now streams to a `.partial` file with a progress bar, then atomically renames into place once the checksum passes — a crashed install no longer leaves a half-written archive that the next `ven install` would skip past.',
        '`install.ps1` / `install.sh` uninstall snippet is now fully idempotent. Probes directory existence and PATH presence independently (either alone is enough to enter the cleanup branch), normalizes PATH entries (expand env vars, trim whitespace + trailing slashes, lowercase), and converges to a fully-clean state on every re-run — even if a previous uninstall got interrupted partway through and left orphan PATH entries behind. Unix `sed` loop now skips missing rc files instead of bailing on the first not-found; system-install branch fires when ANY of `/usr/local/bin/{ven,ven-launcher,ven-setup}` or `/etc/profile.d/ven.sh` is present.',
        'README "Runtime Management" block now includes the `ven path` triplet next to `ven install` / `ven list` / `ven delete`, plus a "my C: drive is full" note explaining the relocation flow.',
        'docs/ven-launcher.md gained a "Relocating an installed ven" section clarifying the interaction between portable mode and `ven path set` (portable mode wins; `ven path` is for the system-installed binary).',
      ],
      fixed: [
        '`ven install rust` failing with "cargo --version smoke test failed ... rustup could not choose a version of cargo to run" immediately after a successful install. cargo / rustc inside `$VEN_HOME/rust/<ver>/bin/` are rustup shims that need `CARGO_HOME` / `RUSTUP_HOME` in their environment to resolve the toolchain; ven now passes those explicitly to the post-install smoke test.',
        '`ven install ruby` / `ven install deno` failing with "error decoding response body Caused by: operation timed out" on slow networks. The old downloaders buffered the entire body in memory with no timeout — the new streaming client has explicit connect/read/total deadlines and a per-chunk stall watchdog, so the worst case is a clean retry instead of a 90-second hang followed by a hard failure.',
        '"I moved ven to D: drive and now npm / pip spawned from another tool still write to `C:\\Users\\me\\.ven`": `ven path set` now persists `VEN_HOME` in the User environment, so every child process — not just shells — inherits the new location.',
        'Cross-module test race that surfaced as a macOS-only CI flake (`round_trip_storage_home` panicking with "config should exist after save"). The two test modules that mutate `$HOME` / `$XDG_CONFIG_HOME` / `$APPDATA` now share a single crate-wide env mutex; they can no longer trample each other\'s temp-dir redirections mid-test.',
      ],
    },
  },
  {
    version: 'v0.1.5',
    date: 'May 17, 2026',
    tag: 'minor',
    summary:
      'Installer hardening: detect existing ven installs (user + system) before touching disk, prompt for the right thing (skip / upgrade / shadow-warning), and tidy up the matching uninstall snippets so a "clean slate" actually removes both copies. Closes the failure mode where someone installs system v0.1.1, then user v0.1.4 a month later, and ends up with two `ven` binaries on PATH fighting for precedence.',
    sections: {
      new: [
        '`install.ps1` and `install.sh` now probe both user-scope (`%USERPROFILE%\\.ven\\bin` / `~/.ven/bin`) and system-scope (`%ProgramFiles%\\ven\\bin` / `/usr/local/bin/ven`) install paths before doing anything else, print what they found, and gate the install on three possible outcomes: same mode + same version → exit cleanly with "nothing to do"; same mode + different version → prompt to upgrade; different mode → warn that PATH precedence will shadow one of the two binaries, then ask for explicit confirmation.',
        'Pipe-mode safety: when there is no TTY (CI, `curl | sh`, scripted contexts), the installer aborts cleanly with a hint to set `VEN_FORCE_INSTALL=true` (or pass `-Force` on Windows / `--force` on Unix) to skip the prompt. No more silent double-installs from copy-pasted README snippets running inside an unattended shell.',
        '`-Force` / `--force` flag and `VEN_FORCE_INSTALL` environment variable on both installers, so CI pipelines can opt into "yes, replace whatever is there" without losing the safety prompt for interactive runs.',
        'New FAQ entry on the [Install](https://ven.dev/install) page documenting the re-install detection behaviour end-to-end (what gets probed, what the three prompts mean, how to skip them).',
      ],
      improved: [
        '`ven_website/src/content/site.js` uninstall snippets now clean both user and system installs in one shot. On Unix the snippet auto-uses `sudo` when `/usr/local/bin/ven` exists; on Windows it detects the system install and prints a "re-run this elevated" warning when the current shell is not already running as Administrator. Same UX as the install snippets, mirrored for the teardown direction.',
        'Existing-install probe path list is exhaustive and crash-safe: a missing directory is treated the same as a missing binary, never as an error. So the installer works identically on a brand-new machine and on a machine that has half a previous install left behind from a manual rm.',
      ],
      fixed: [
        'The "system install shadowed by user install (or vice versa)" foot-gun: silently letting both stay on PATH meant `ven --version` could disagree with `which ven` depending on the current shell\'s PATH ordering. v0.1.5 catches this at install time and forces an explicit choice instead of hoping the user notices.',
      ],
    },
  },
  {
    version: 'v0.1.4',
    date: 'May 16, 2026',
    tag: 'minor',
    summary:
      'New `ven delete` command for removing installed runtimes. Refuses to delete the runtime currently pinned in ven.toml unless `--force` is passed, so you can never silently break the next `cd` activation.',
    sections: {
      new: [
        '`ven delete` removes an installed language runtime by deleting its `$VEN_HOME/<lang>/<version>/` directory. Three calling conventions: full wizard (`ven delete`), language-only (`ven delete python` → pick a version), or fully specified (`ven delete python 3.12.7`). Flags: `-y` / `--yes` (skip confirm), `--force` (allow deleting the active runtime), `--json` (machine-readable, requires explicit args + `-y`).',
        'Active-runtime safety guard: refuses to delete the runtime currently resolved by the nearest `ven.toml`. The error message names the exact `ven.toml` path and points at `--force` as the escape hatch. Prevents the silent-shell-breakage class of bugs where users delete a runtime and then `cd` into a project that pinned it.',
        'New per-command doc page [docs/cmds/delete.md](https://github.com/bhuwanb23/ven/blob/main/docs/cmds/delete.md) covering all flags, JSON shapes, the safety guard, and storage layout impact.',
      ],
      improved: [
        'README "Runtime Management" block now includes `ven delete` examples next to `ven install` / `ven list`, plus a short note clarifying the `delete` (runtimes) vs `remove` (packages) split.',
        'docs/cmds/INDEX.md "Version Management" section now lists `ven delete` alongside `install` / `list` / `status`.',
        'docs/cmds/list.md "Remove Deprecated Versions" tip no longer suggests `rm -rf ~/.ven/<lang>/<version>` — it points at `ven delete` instead (manual rm kept as a footnote for pre-v0.1.4 history).',
        'docs/commands-reference.md gained a `ven delete [runtime] [version]` row in the Project lifecycle table.',
        'src/cli/list/helpers.rs helpers (`detect_active_version`, `calculate_dir_size`, `format_bytes`, `get_installation_date`, `get_version_path`) promoted from `pub(super)` to `pub(crate)` so the new delete command can reuse them without duplication.',
      ],
      fixed: [],
    },
  },
  {
    version: 'v0.1.3',
    date: 'May 16, 2026',
    tag: 'patch',
    summary:
      'Trust the OS certificate store. Fixes "error sending request" failures on Zscaler / Netskope / Bluecoat / any SSL-inspecting corporate proxy.',
    sections: {
      new: [],
      improved: [
        'reqwest now loads root CAs from the OS trust store in addition to the bundled Mozilla webpki-roots (rustls-tls-native-roots feature). Browsers worked because they read the Windows / macOS / Linux cert store; ven now does the same.',
        'docs/ven-launcher.md gained a "Corporate proxy / Zscaler" troubleshooting section explaining the failure mode and the fix.',
        'Install page FAQ gained a Zscaler / corporate-proxy entry pointing at v0.1.3 as the minimum version for SSL-inspecting environments.',
      ],
      fixed: [
        '`ven install <lang>` failing with "error sending request for url (https://...)" inside corporate networks where Zscaler / Netskope / Bluecoat MITM HTTPS using a private root CA installed in the OS trust store. ven now picks that root up automatically — no env vars, no flags, no extra config.',
      ],
    },
  },
  {
    version: 'v0.1.2',
    date: 'May 16, 2026',
    tag: 'minor',
    summary:
      'One-click corporate / Zscaler bundle. Download the zip, double-click the bundled terminal shim, get a ven-ready shell — no command-line typing, no admin, no PATH edits.',
    sections: {
      new: [
        'Double-clickable terminal shim shipped inside every ven-launcher-{os}-{arch}.{zip|tar.gz}: Start ven.cmd on Windows, Start ven.command on macOS, start-ven.sh on Linux. Double-click → terminal opens → ven is already activated. Zero command-line knowledge required, designed for non-CLI teammates on locked-down machines.',
        'Bundled README.txt rewritten to lead with the 3-step "Extract → Double-click → ven is ready" flow, plus a "Behind Zscaler / corporate proxy" explainer for users whose firewall blocks irm | iex / curl | sh installers.',
      ],
      improved: [
        'Install page "Corporate & portable" section replaced with a single download button auto-targeted at the visitor\'s OS + arch, a 3-step "Download → Extract → Double-click <shim>" list naming the exact shim filename for the chosen OS, and an "Advanced" disclosure for power users — no more wall of shell commands as the happy path.',
        'Landing page "Built for restricted environments" hero now demonstrates the double-click flow and earns a "Bypasses Zscaler" pill alongside no-sudo / no-UAC / portable.',
        'Install page shares a single /releases-manifest.json fetch between the new Corporate CTA and the Direct downloads table via a useReleasesManifest() hook, so the page renders deterministically without racing the same request twice.',
        'docs/ven-launcher.md gained a "Double-click shim" section with a per-OS table and a "Behind Zscaler / corporate proxy" subsection explaining why zip + double-click passes corporate firewalls; the USB-stick layout example now shows the shim file.',
        'docs/install-scripts.md "Portable launcher bundle" asset table now lists the per-OS shim filename and mode 0755 for the Unix shims.',
        'release.yml launcher-bundle step now stages the per-OS shim alongside ven + ven-launcher and ships a corporate-focused README that opens with the double-click flow.',
      ],
      fixed: [],
    },
  },
  {
    version: 'v0.1.1',
    date: 'May 15, 2026',
    tag: 'minor',
    summary:
      'Portable launcher bundle + centralized VEN_HOME resolver. Double-click a terminal shim and ven is live — no admin, no PATH edits, works behind corporate proxies.',
    sections: {
      new: [
        'One-click corporate / Zscaler download: the portable bundle now ships a double-clickable terminal shim per OS (Start ven.cmd on Windows, Start ven.command on macOS, start-ven.sh on Linux). Double-click → terminal opens → ven is already activated. Zero command-line typing.',
        'Discoverable portable launcher bundle: ven-launcher-{os}-{arch}.{zip|tar.gz} for all 6 platform/arch combos, each with bundled README.txt, terminal shim, and per-asset SHA-256 sidecar',
        'Centralized VEN_HOME resolver with 4-tier precedence: $VEN_HOME → $VEN_STORAGE_PATH → <launcher-dir>/.ven → ~/.ven',
        'USB-stick / fully-portable mode: drop a sibling .ven/ folder next to ven-launcher and every runtime, cache entry, and lockfile state lives inside the bundle',
        'ven-launcher --show-env now prints the resolved VEN_HOME so you can confirm portable vs shared mode at a glance',
        '"Which binary should I use?" persona table in the README with a fourth row pointing at the new portable-launcher asset',
      ],
      improved: [
        'Install page "Corporate & portable" section replaced with a single download button auto-targeted at the visitor\'s OS + arch, a 3-step "Download → Extract → Double-click <shim>" list, and an "Advanced" disclosure for power users — no more wall of shell commands as the happy path',
        'Unified all ~17 storage call-sites in src/core/, src/cli/, src/intelligence/, src/bin/setup/ through a single core::ven_home::ven_home() function — kills pre-existing drift between hardcoded ~/.ven/ and VEN_STORAGE_PATH-aware paths',
        'apply_activation_env and apply_launcher_portable_env now export VEN_HOME to every spawned shell, so portable-mode bundles never silently fall back to ~/.ven once you cd into a project',
        'docs/ven-launcher.md rewritten with a Portable mode section, resolver precedence table, USB-stick layout example, and a "Behind Zscaler" section explaining why the zip + double-click flow passes corporate firewalls',
        'docs/install-scripts.md gained a "Portable launcher bundle" section listing the new asset names',
        'release.yml workflow now emits four assets per matrix entry (combined + launcher + setup + .sha256 sidecars) for all six (os, arch) combos; the launcher bundle includes a per-OS terminal-shim file and a corporate-focused README',
      ],
      fixed: [],
    },
  },
  {
    version: 'v0.1.0',
    date: 'May 14, 2026',
    tag: 'major',
    summary:
      'First public release. Eight runtimes, dependency intelligence, security scanning, version-pinned docs, and a zero-admin launcher.',
    sections: {
      new: [
        'Multi-language runtime management: Node · Python · Go · Rust · Java · Ruby · Deno · Bun (install, alias-resolve, coexist, smoke-test)',
        'ven.toml + automatic shell activation on cd (PowerShell 5.1 + 7, Bash, Zsh, Fish)',
        'Unified package surface — ven add / remove / upgrade — keeps native manifests + ven.toml [packages] in sync across all 8 ecosystems',
        'Dependency Graph Engine: ven graph, ven check-add, ven why, ven resolve — pre-install conflict detection with explanation chains',
        'Security & Health: ven check (CVE via osv.dev + EOL via endoflife.date), ven scan --ghosts (gitignore-aware import audit)',
        'ven.lock v2 with SRI integrity hashes + content_hash, ven sync, ven sync --check (CI-safe drift detection, JSON output)',
        'Version-pinned package docs: ven docs <pkg>, --browser, --diff V1 V2 (terminal-rendered + cached for 7d)',
        'ven-launcher: portable, no-admin terminal spawner — runs from any folder',
        'ven-setup.exe (Windows UAC + HKCU/HKLM) and ven-setup (Unix sudo + /etc/profile.d) self-contained installers',
        'Cross-platform install one-liners: install.ps1 (Windows) and install.sh (Unix) with automatic SHA-256 verification',
        'Automated GitHub Actions release pipeline: 6-cell platform matrix, two-pass cargo build, per-asset .sha256 sidecars, aggregate SHA256SUMS manifest',
        'CI workflow with cargo build + clippy + fmt across windows-latest, ubuntu-latest, macos-latest',
      ],
      improved: [],
      fixed: [],
    },
  },
]

export const TAG_META = {
  major: { label: 'MAJOR', tone: 'major', dot: 'bg-secondary-fixed-dim' },
  minor: { label: 'MINOR', tone: 'minor', dot: 'bg-primary-fixed-dim' },
  patch: { label: 'PATCH', tone: 'patch', dot: 'bg-outline' },
  security: { label: 'SECURITY', tone: 'security', dot: 'bg-error' },
}
