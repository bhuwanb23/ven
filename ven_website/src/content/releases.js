// Release timeline driving /changelog. Order: newest first.
// Source: website_design/changelog.md.

export const RELEASES = [
  {
    version: 'v0.1.6',
    date: 'May 17, 2026',
    tag: 'minor',
    summary:
      'New `ven path` command for relocating ven\'s storage root (default `~/.ven`) to a different drive — atomic move with rollback, persistent pointer file, and a User-scope `VEN_HOME` so npm / pip / new shells inherit the new location automatically. Built for the "my C: drive is full" case on company-managed Windows machines.',
    sections: {
      new: [
        '`ven path show` prints the resolved `$VEN_HOME`, the resolver source (env / portable / pointer / default), size on disk, and the active pointer. `--json` for scripting.',
        '`ven path set <dir>` relocates the storage root. Interactive prompt by default ("move existing data? / pointer only? / cancel"); explicit flags `--move`, `--no-move`, `--pointer-only`, `-y / --yes`, `--json` for non-interactive use. Cross-drive moves fall back from `fs::rename` to recursive copy + verify + delete with an `indicatif` progress bar; the source is never touched until the target is fully populated and the file count + byte count match, so a failed move never leaves ven half-relocated.',
        '`ven path reset` clears the pointer (with or without moving data back) and reverts to `~/.ven`. Same flag surface as `set`.',
        'Persistent pointer file at `~/.config/ven/config.toml` (Linux), `~/Library/Application Support/ven/config.toml` (macOS), or `%APPDATA%\\ven\\config.toml` (Windows). Survives a user wiping `~/.ven`. Atomic write (`.tmp` + rename) so a crash mid-write can\'t corrupt it.',
        'Persistent `VEN_HOME` user-env mutation: Windows uses `[Environment]::SetEnvironmentVariable(..., \'User\')` + `WM_SETTINGCHANGE` broadcast (same pattern as `ven-setup` PATH); Unix adds/replaces a `# >>> ven env >>>` block in `~/.bashrc`, `~/.zshrc`, `~/.profile`, and `~/.config/fish/config.fish` (whichever exist). Failures are warnings, not errors — the pointer file is ven\'s source of truth.',
        'A `.ven-move.lock` file containing the PID protects against concurrent moves. `--force-unlock` clears a stale lock from a crashed previous attempt.',
        'New per-command doc page [docs/cmds/path.md](https://github.com/bhuwanb23/ven/blob/main/docs/cmds/path.md) covering all flags, JSON shapes, the safety semantics (cross-drive copy/verify, lock file, env-shadow warning), and what does NOT move (binaries, portable sibling `.ven/`s).',
      ],
      improved: [
        '`src/core/ven_home.rs` resolver now has 5 steps (was 4): the pointer file slots in between portable mode and the `~/.ven` default. `$VEN_HOME` and `$VEN_STORAGE_PATH` env vars still take precedence over the pointer so per-process overrides keep working unchanged.',
        'New `ven_home_source()` helper returns a `HomeSource` enum (`EnvVenHome` / `EnvVenStoragePath` / `PortableSibling` / `Pointer` / `Default`) so `ven path show` can tell the user exactly which knob is currently in effect — and warn when an active env var would shadow a freshly-written pointer.',
        'README "Runtime Management" block now includes `ven path show` / `ven path set <dir>` / `ven path reset` examples alongside `ven install` / `ven list` / `ven delete`, with a short note about when to use it.',
        'docs/cmds/INDEX.md "Project Setup" section now lists `ven path` next to `ven init` and `ven setup`.',
        'docs/ven-launcher.md got a new "Relocating an installed ven" section that disambiguates the pointer-file flow from portable mode (portable sibling `.ven/` still wins; pointer is the per-user "I moved my data" preference).',
      ],
      fixed: [],
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
