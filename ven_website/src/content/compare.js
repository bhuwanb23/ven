// Fact-checked comparison rows for the Landing CompareSection.
//
// Each row is `[capability, ven, npm/nvm, mise/asdf, docker?]` — boolean for a
// pure yes/no, or a short string ("partial", "manual", "container") for a
// qualified answer. The Landing table renders booleans as check/cancel icons
// and strings verbatim, so keep them short.
//
// Sourcing notes (so future you can defend each row):
//   - nvm:   Node only; no cross-language; no graph; PATH-managed.
//             https://github.com/nvm-sh/nvm
//   - mise:  multi-runtime version manager; no package management surface;
//             no CVE / EOL data.
//             https://mise.jdx.dev
//   - docker: provides full isolation but requires the daemon + admin and
//             rebuilds for every dependency change.

export const COMPARE_HEADERS = ['Capability', 'ven', 'npm + nvm', 'mise / asdf', 'Docker']

export const COMPARE_ROWS = [
  ['Manages 9 runtimes with one CLI',           true,        false,       true,        'container'],
  ['Auto-switching on `cd`',                    true,        false,       true,        false],
  ['Unified package surface (`ven add`)',       true,        'Node only', false,       false],
  ['Pre-install dependency-graph simulation',   true,        false,       false,       false],
  ['OSV CVE scanning, offline-cached',          true,        false,       false,       false],
  ['Endoflife.date runtime EOL alerts',         true,        false,       false,       false],
  ['Reproducible lock with SHA-256 hashes',     true,        'package-lock', false,    'image digest'],
  ['Drift detection vs lockfile',               true,        'partial',   false,       false],
  ['Ghost-dependency scanner',                  true,        false,       false,       false],
  ['Works without admin / sudo',                true,        true,        true,        false],
  ['Portable .exe for locked-down boxes',       true,        false,       false,       false],
  ['Per-terminal isolation (no global state)',  true,        false,       'partial',   true],
]
