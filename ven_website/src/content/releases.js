// Release timeline driving /changelog. Order: newest first.
// Source: website_design/changelog.md.

export const RELEASES = [
  {
    version: 'v0.1.1',
    date: 'May 15, 2026',
    tag: 'minor',
    summary:
      'Portable launcher bundle + centralized VEN_HOME resolver. Run ven from a USB stick without touching $PATH.',
    sections: {
      new: [
        'Discoverable portable launcher bundle: ven-launcher-{os}-{arch}.{zip|tar.gz} for all 6 platform/arch combos, each with bundled README.txt and per-asset SHA-256 sidecar',
        'Centralized VEN_HOME resolver with 4-tier precedence: $VEN_HOME → $VEN_STORAGE_PATH → <launcher-dir>/.ven → ~/.ven',
        'USB-stick / fully-portable mode: drop a sibling .ven/ folder next to ven-launcher and every runtime, cache entry, and lockfile state lives inside the bundle',
        'ven-launcher --show-env now prints the resolved VEN_HOME so you can confirm portable vs shared mode at a glance',
        '"Which binary should I use?" persona table in the README with a fourth row pointing at the new portable-launcher asset',
      ],
      improved: [
        'Unified all ~17 storage call-sites in src/core/, src/cli/, src/intelligence/, src/bin/setup/ through a single core::ven_home::ven_home() function — kills pre-existing drift between hardcoded ~/.ven/ and VEN_STORAGE_PATH-aware paths',
        'apply_activation_env and apply_launcher_portable_env now export VEN_HOME to every spawned shell, so portable-mode bundles never silently fall back to ~/.ven once you cd into a project',
        'docs/ven-launcher.md rewritten with a Portable mode section, resolver precedence table, and USB-stick layout example',
        'docs/install-scripts.md gained a "Portable launcher bundle" section listing the new asset names',
        'release.yml workflow now emits four assets per matrix entry (combined + launcher + setup + .sha256 sidecars) for all six (os, arch) combos',
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
