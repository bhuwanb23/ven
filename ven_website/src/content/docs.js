// Curated, precise versions of docs/cmds/* and docs/languages/*.
// Each entry's `sections` array drives DocPage.jsx — schema:
//   { kind: 'p', text }                  paragraph
//   { kind: 'h2', text }                 section heading
//   { kind: 'h3', text }                 subsection heading
//   { kind: 'code', lang, code }         code block
//   { kind: 'ul', items: [str|jsx] }     bullet list
//   { kind: 'table', head: [], rows: [[]] }
//   { kind: 'callout', tone, title?, text }   info / warning callout
//
// Cross-doc links live in `related: [slug]`. TOC is auto-generated from h2s.

import { LANGUAGES } from './languages.js'

// ---- Command docs ---------------------------------------------------------

const CMD_DOCS = {
  init: {
    title: 'ven init',
    category: 'Getting started',
    summary:
      'Interactive project bootstrap. Picks one runtime, optionally adds packages, writes ven.toml, and (optionally) validates the result. Multi-runtime projects are supported via direct ven.toml edits — multi-select wizard ships in the next release.',
    sections: [
      { kind: 'p', text: 'ven init is the project initialization wizard. It writes a ven.toml in the current directory with a runtime, optional packages, and optional [env] variables.' },
      { kind: 'h2', text: 'Scope in v0.1.x' },
      { kind: 'p', text: 'The wizard pins one language at a time. The ven.toml schema and every other ven command (add, status, the activation hook, lock, sync, check) already support multiple runtimes per project — declare them by editing ven.toml directly after ven init. A multi-select wizard (SPACE to toggle several languages, then a version prompt for each) ships in the next release.' },
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven init                  # interactive (single language)\nven init --template       # pick a template\nven init --with-packages  # multi-select popular packages\nven init --validate       # validate after writing\nven init --lang python --ver 3.12   # headless (CI-friendly)' },
      { kind: 'h2', text: 'Templates' },
      {
        kind: 'ul', items: [
          'Express API Server — node + express + cors + dotenv',
          'React + Vite Frontend — node + react + react-dom + vite',
          'Next.js Full-stack — node + next + react + react-dom',
          'Empty Project — pick language + version, no packages',
        ]
      },
      { kind: 'h2', text: 'Generated ven.toml (single runtime)' },
      { kind: 'code', lang: 'toml', code: '[runtime]\nnode = "20"\n\n[packages]\nexpress = "^4.18.2"\ncors    = "^2.8.5"\ndotenv  = "^16.3.1"\n\n[env]\nNODE_ENV = "development"\nPORT     = "3000"' },
      { kind: 'h2', text: 'Multi-runtime projects (hand-edited today)' },
      { kind: 'p', text: 'For projects spanning more than one language — say a Python service with a Node-built frontend, or a Go API with Python data scripts — run ven init for the primary language, then add additional runtimes by hand. Each line is independent; the activation hook will set up PATH for every declared runtime simultaneously.' },
      { kind: 'code', lang: 'toml', code: '[runtime]\npython = "3.12"      # written by ven init\nnode   = "20"        # added by hand\ngo     = "1.22"      # added by hand\n\n[packages]\nflask   = "^3.0.0"   # routes to pip\nexpress = "^4.18.2"  # routes to npm' },
      { kind: 'p', text: 'After editing, run ven install for each new runtime, then cd in or out of the directory to re-trigger the activation hook. ven status will list every active runtime side-by-side.' },
      { kind: 'h2', text: 'Validation' },
      { kind: 'p', text: '--validate runs four checks: ven.toml structure, runtime resolution (will the version install?), declared packages, and optional env vars. Output uses ✓ / ⚠ / ✗ icons.' },
      { kind: 'h2', text: 'Next steps' },
      { kind: 'code', lang: 'bash', code: 'ven install node 20   # install the runtime\nven setup             # one-time shell hook\nven add typescript    # add a package\nven status            # confirm everything\'s healthy' },
    ],
    related: ['install', 'add', 'status'],
  },
  install: {
    title: 'ven install',
    category: 'Getting started',
    summary:
      'Install a language runtime from official sources. SHA256-verified, alias-aware, and isolated under ~/.ven/<lang>/<version>/.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven install <lang> <version>\nven install node 20\nven install python 3.11\nven install rust 1.75' },
      { kind: 'h2', text: 'Version resolution' },
      {
        kind: 'ul', items: [
          '"20"        → resolves to latest patch of 20.x',
          '"latest"    → newest stable release',
          '"lts"       → most recent LTS (Node, Java, Ruby)',
          '"20.20.2"   → exact pin',
        ]
      },
      { kind: 'h2', text: 'What happens' },
      {
        kind: 'ul', items: [
          'Resolves alias → exact version',
          'Downloads from the official source (nodejs.org, python.org, …)',
          'Verifies SHA256 against the upstream checksum file',
          'Extracts into ~/.ven/<lang>/<version>/',
          'Runs a smoke-test (e.g. `node --version`)',
          'Activates in the current shell',
        ]
      },
      { kind: 'callout', tone: 'info', title: 'No admin / sudo', text: 'Everything happens inside ~/.ven/. Nothing is written to system paths.' },
      { kind: 'h2', text: 'Exit codes' },
      {
        kind: 'table', head: ['Code', 'Meaning'], rows: [
          ['0', 'Installed and activated successfully'],
          ['1', 'Download / extract failed'],
          ['2', 'SHA256 mismatch — the artifact was rejected'],
        ]
      },
    ],
    related: ['list', 'init', 'status'],
  },
  status: {
    title: 'ven status',
    category: 'Getting started',
    summary:
      'Report the active environment: runtimes, packages, health, and shell markers. Three verbosity tiers — plain, --verbose, --json.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven status\nven status --verbose\nven status --json' },
      { kind: 'h2', text: 'What it reports' },
      {
        kind: 'ul', items: [
          'Active runtime versions (resolved, not just declared)',
          'Installed vs missing toolchains',
          'Package summary (count, recent additions)',
          'Health overview — short OSV + EOL teaser',
          'VEN_*_VERSION shell markers + activation state',
        ]
      },
      { kind: 'h2', text: '--json output' },
      { kind: 'p', text: 'Stable, documented JSON schema. Designed for CI gates and editor extensions.' },
      { kind: 'code', lang: 'json', code: '{\n  "active": true,\n  "runtimes": [{ "lang": "node", "declared": "20", "resolved": "20.20.2" }],\n  "packages": { "count": 12 },\n  "health": { "cves": 0, "eol_warnings": 0 }\n}' },
    ],
    related: ['init', 'check', 'list'],
  },
  add: {
    title: 'ven add',
    category: 'Commands',
    summary:
      'Install a package — with the dependency graph pre-checked for conflicts and CVEs before the package manager runs.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven add express\nven add express@4.18.2\nven add -D typescript     # dev dependency where the ecosystem supports it' },
      { kind: 'h2', text: 'Pipeline' },
      {
        kind: 'ul', items: [
          'Build the full dependency graph (recursive, includes transitive deps)',
          'Simulate the install — flag conflicts before touching disk',
          'Run a CVE pre-scan via OSV (cached)',
          'Apply via the ecosystem\'s native PM (npm / pip / cargo / …)',
          'Update [packages] in ven.toml',
        ]
      },
      { kind: 'callout', tone: 'success', text: 'You never have to run npm/pip/cargo manually for normal workflows — ven keeps ven.toml as the source of truth.' },
      { kind: 'h2', text: 'Per-ecosystem mapping' },
      {
        kind: 'table', head: ['Runtime', 'Underlying command'], rows: [
          ['node / bun', 'npm install / bun add'],
          ['python', 'pip install (venv-aware)'],
          ['ruby', 'gem install'],
          ['php', 'composer require'],
          ['rust', 'cargo add'],
          ['go / java / deno', 'native — ven coordinates ven.toml + lockfile only'],
        ]
      },
    ],
    related: ['lock', 'check', 'sync'],
  },
  lock: {
    title: 'ven lock',
    category: 'Commands',
    summary:
      'Write ven.lock — a deterministic snapshot of the resolved dependency graph, with SHA256 SRI integrity hashes per package.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven lock           # write ven.lock for current ven.toml\nven lock --check   # exit non-zero if regeneration would change anything' },
      { kind: 'h2', text: 'Lockfile schema (v2)' },
      { kind: 'code', lang: 'toml', code: 'version = 2\ngenerated = "2024-01-15T10:30:00Z"\n\n[runtimes]\nnode = { version = "20.20.2", sha256 = "…" }\n\n[[packages]]\nname      = "express"\nversion   = "4.18.2"\nsource    = "npm"\nintegrity = "sha512-…"   # SRI hash of the tarball\ndependencies = ["body-parser", "cookie-parser"]' },
      { kind: 'h2', text: 'Why SRI hashes?' },
      {
        kind: 'ul', items: [
          'Detect upstream tampering — npm pulls the registry, ven verifies the bytes',
          'CI can use --check to fail builds when a teammate forgets to commit ven.lock',
          'Stable across machines: ven sync rejects any package whose integrity differs',
        ]
      },
      { kind: 'callout', tone: 'info', title: 'v1 → v2 migration', text: 'Running ven lock against a v1 file auto-upgrades it. The CLI prints "lockfile upgraded: v1 → v2" so it shows up in code review.' },
    ],
    related: ['sync', 'add', 'check'],
  },
  sync: {
    title: 'ven sync',
    category: 'Commands',
    summary:
      'Reproduce the locked environment on this machine — or, with --check, fail the build if anything has drifted from ven.lock.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven sync           # bring the local env to match ven.lock\nven sync --check   # report drift; exit 1 if any package is out of sync' },
      { kind: 'h2', text: 'Drift detection' },
      { kind: 'p', text: '--check compares three layers: ven.lock vs ven.toml (declared vs locked), and ven.lock vs the actual installed packages (locked vs reality). It exits non-zero on any mismatch — perfect for CI.' },
      { kind: 'h2', text: 'CI example (GitHub Actions)' },
      { kind: 'code', lang: 'yaml', code: '- run: ven sync --check\n  # Fails the job if ven.lock is stale or someone installed a package without committing.' },
      { kind: 'h2', text: 'Exit codes' },
      {
        kind: 'table', head: ['Code', 'Meaning'], rows: [
          ['0', 'No drift'],
          ['1', 'Drift detected — at least one package mismatched'],
          ['2', 'ven.lock is missing or malformed'],
        ]
      },
    ],
    related: ['lock', 'check', 'add'],
  },
  check: {
    title: 'ven check',
    category: 'Health & security',
    summary:
      'Unified health report — pulls package CVEs from osv.dev and runtime end-of-life status from endoflife.date.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven check                  # CVE + EOL + outdated\nven check --security       # CVE only\nven check --eol            # EOL only\nven check --json           # CI / scripting' },
      { kind: 'h2', text: 'How it works' },
      {
        kind: 'ul', items: [
          'Reads ven.lock when present (otherwise [packages]) for pinned (name, version) pairs',
          'POSTs batches of 1000 to https://api.osv.dev/v1/querybatch',
          'Enriches every vuln via /v1/vulns/<id> for severity + summary + fixed-in version',
          'GETs https://endoflife.date/api/<product>.json per runtime in [runtime]',
          'Caches everything in ~/.ven/intelligence.db with stale-on-failure fallback',
        ]
      },
      { kind: 'h2', text: 'Severity buckets' },
      {
        kind: 'table', head: ['CVSS', 'Bucket'], rows: [
          ['≥ 9.0', 'CRITICAL'],
          ['≥ 7.0', 'HIGH'],
          ['≥ 4.0', 'MODERATE'],
          ['> 0', 'LOW'],
        ]
      },
      { kind: 'h2', text: 'Runtime status labels' },
      {
        kind: 'table', head: ['Label', 'Meaning'], rows: [
          ['[OK]', 'Runtime is up-to-date and supported'],
          ['[OUTDATED]', 'Newer version available'],
          ['[SUPPORT-ENDED]', 'Active support ended, security-only patches'],
          ['[EOL]', 'End-of-life — no further updates'],
        ]
      },
      { kind: 'h2', text: 'Exit codes' },
      {
        kind: 'table', head: ['Code', 'Meaning'], rows: [
          ['0', 'No actionable issues'],
          ['1', 'Any HIGH/CRITICAL CVE, passed-EOL, support-ended, or outdated runtime'],
        ]
      },
      { kind: 'callout', tone: 'info', title: 'Offline-friendly', text: 'OSV cache lives 6 hours, EOL cache 24 hours. On network failure, ven serves the last-known-good entry and prints a "stale" warning.' },
    ],
    related: ['scan', 'lock', 'docs'],
  },
  docs: {
    title: 'ven docs',
    category: 'Documentation',
    summary:
      'Open or render documentation for the exact installed version of a package — in terminal or browser. Also diffs API surfaces between versions.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven docs express                       # terminal-rendered\nven docs express --browser             # open registry page in your browser\nven docs express --diff 4.18.2 5.0.0   # API diff between two versions' },
      { kind: 'h2', text: 'How it works' },
      {
        kind: 'ul', items: [
          'Resolves the version from ven.lock → ven.toml → registry latest',
          'Pulls the README / API metadata from the ecosystem\'s registry (npm, PyPI, crates.io, RubyGems)',
          'Renders Markdown to terminal via termimad — or opens the registry page via the system browser',
          'Caches fetched docs under ~/.ven/cache/docs/<pkg>/<version>/',
        ]
      },
      { kind: 'h2', text: '--diff' },
      { kind: 'p', text: 'Diffs the public API surface between two versions of the same package (function/class signatures). Highlights additions, removals, and signature changes. Useful before any major-version upgrade.' },
    ],
    related: ['check', 'add'],
  },

  // --- Maintenance ----------------------------------------------------------

  update: {
    title: 'ven update',
    category: 'Maintenance',
    summary:
      'Self-update ven (and its sibling ven-launcher) to the latest GitHub release. SHA256-verified, auto-elevates for system installs, in-place — no re-install, no PATH edits.',
    sections: [
      { kind: 'callout', tone: 'info', title: 'Not the same as ven upgrade', text: 'ven upgrade updates project packages (npm / pip / cargo / …). ven update updates the ven binaries themselves.' },
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven update                    # check + apply latest stable\nven update --check            # report only; safe in CI\nven update --version v0.1.6   # roll back to a specific tag\nven update --yes              # skip the confirmation prompt\nven update --force            # reinstall even when already current\nven update --json             # machine-readable for CI gates' },
      { kind: 'h2', text: 'How it works' },
      {
        kind: 'ul', items: [
          'Resolves the install dir from std::env::current_exe().parent()',
          'Fetches the release JSON from api.github.com/repos/bhuwanb23/ven/releases',
          'Compares release.tag_name against CARGO_PKG_VERSION (compile-time)',
          'Downloads the platform-specific combined asset (ven-{os}-{arch}.{zip|tar.gz})',
          'Verifies it against the release\'s SHA256SUMS manifest before extracting',
          'Swaps ven AND ven-launcher in place, atomically per file',
        ]
      },
      { kind: 'h2', text: 'Self-replace strategy' },
      { kind: 'p', text: 'Windows refuses to overwrite a running .exe, so ven renames the current binary to *.exe.old (the classic MoveFileEx trick), then writes the new bytes at the original path. The leftover *.exe.old files are harmless and removable after the next reboot. On Linux and macOS, ven unlink-s the target file (POSIX-safe — the open file descriptor in the running process keeps pointing at the old inode) and writes a fresh file.' },
      { kind: 'h2', text: 'Auto-elevation' },
      { kind: 'p', text: 'If the install dir requires admin (e.g. C:\\Program Files\\ven\\bin or /usr/local/bin), ven re-launches itself elevated through UAC on Windows or sudo on Unix. The elevated child carries an internal --reentry flag so the elevation loop terminates after one hop.' },
      { kind: 'h2', text: 'Exit codes' },
      {
        kind: 'table', head: ['Code', 'Meaning'], rows: [
          ['0', 'No-op (already current) OR update applied successfully'],
          ['1', 'Network failure / SHA256 mismatch / write error'],
          ['2', 'User aborted at the confirmation prompt'],
        ]
      },
      { kind: 'h2', text: 'CI gate example' },
      { kind: 'code', lang: 'yaml', code: '- name: Verify ven is current\n  run: ven update --check --json | jq -e \'.up_to_date == true\'' },
    ],
    related: ['upgrade', 'setup', 'install'],
  },

  doctor: {
    title: 'ven doctor',
    category: 'Maintenance',
    summary:
      'Diagnose ven installation health: multiple copies on disk, PATH shadowing, and whether your build supports ven update.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven doctor\nven doctor --json' },
      { kind: 'h2', text: 'What it checks' },
      {
        kind: 'ul', items: [
          'Known install locations (`~/.ven/bin`, `%ProgramFiles%\\ven\\bin`, `/usr/local/bin`)',
          'Every `ven` binary reported by `where` / `which`',
          'Which copy PATH resolves first',
          'Whether each copy is new enough for `ven update` (v0.1.7+)',
        ]
      },
      { kind: 'h2', text: 'When to run it' },
      {
        kind: 'ul', items: [
          '`ven update` fails with "unrecognized subcommand"',
          'You re-installed but `ven --version` did not change',
          'Both user and system installs exist',
        ]
      },
    ],
    related: ['update', 'setup', 'install'],
  },

  uninstall: {
    title: 'ven uninstall',
    category: 'Maintenance',
    summary:
      'Full-nuke teardown. Removes ven binary, every runtime, cache, state, persisted VEN_HOME, pointer file, PATH entries, and shell rc-file blocks. Idempotent, dry-run-capable.',
    sections: [
      { kind: 'callout', tone: 'warning', title: 'Irreversible', text: 'ven uninstall removes every ven-managed file and setting from your machine. Use --dry-run first to see the plan without touching anything.' },
      { kind: 'h2', text: 'Synopsis' },
      { kind: 'code', lang: 'bash', code: 'ven uninstall                    # interactive: show plan, prompt before nuking\nven uninstall --dry-run          # print the plan; touch nothing\nven uninstall -y                 # skip the confirm prompt (CI / scripts)\nven uninstall --user-only        # skip the system install layer\nven uninstall --system-only      # skip the user install layer\nven uninstall --json -y          # machine-readable result\nven uninstall --json --dry-run   # plan as JSON without executing' },
      { kind: 'h2', text: 'What gets removed' },
      {
        kind: 'ul', items: [
          'User install root (`~/.ven` / `%USERPROFILE%\\.ven`) — binary, every installed runtime, cache, lockfile state',
          'System install (`/usr/local/bin/{ven,ven-launcher,ven-setup}` + `/etc/profile.d/ven.sh` on Unix, `%ProgramFiles%\\ven\\` on Windows)',
          'Relocated storage root if `ven path set` moved it — honored via the same VEN_HOME precedence chain ven uses',
          'Persisted `VEN_HOME` user environment variable written by `ven path set`',
          'Pointer file at `~/.config/ven/config.toml` (or platform equivalent)',
          'Ven-managed blocks from shell rc files: `# >>> ven env >>>`, `# >>> ven-setup PATH >>>`, `# >>> ven shell hook >>>`, plus orphan `.ven/bin` PATH lines',
          'Windows User-scope and Machine-scope PATH entries with WM_SETTINGCHANGE broadcast',
        ]
      },
      { kind: 'h2', text: 'What survives' },
      { kind: 'p', text: 'Per-project files are left alone — `ven.toml`, `ven.lock`, `node_modules/`, `venv/`, language-native lockfiles, editor settings, shell history.' },
      { kind: 'h2', text: 'Flags' },
      {
        kind: 'table', head: ['Flag', 'Effect'], rows: [
          ['(no flag)', 'Print the plan, then prompt "Permanently remove ven and all installed runtimes? [y/N]". Default is No.'],
          ['-y / --yes', 'Skip the confirm prompt. Required for CI / scripted use.'],
          ['--dry-run', 'Build the plan and print it. Do not touch the filesystem or env state. Combine with --json for CI.'],
          ['--user-only', 'Skip the system install layer. No sudo / Admin needed.'],
          ['--system-only', 'Skip the user install layer. For sysadmins. Mutually exclusive with --user-only.'],
          ['--json', 'Emit a structured result. Requires --dry-run (plan) or -y (execute). Pure JSON without one is rejected.'],
        ]
      },
      { kind: 'h2', text: 'Elevation' },
      { kind: 'p', text: 'System install dirs require root / Admin. ven uninstall does NOT auto-elevate (the blast radius is too large). It prints a hint: "sudo ven uninstall" on Unix, "re-run as Administrator" on Windows. Pass --user-only to skip the system layer.' },
      { kind: 'h2', text: 'Windows: the running .exe' },
      { kind: 'p', text: 'Windows refuses to delete the running .exe. ven renames itself to *.exe.old (same trick as ven update), then removes the rest of the install dir. The orphan vanishes on the next reboot.' },
      { kind: 'h2', text: 'Fallback scripts' },
      { kind: 'p', text: 'Standalone teardown scripts are bundled alongside the binary at ~/.ven/bin/ven-uninstall (Unix) and ~\\.ven\\bin\\ven-uninstall.ps1 (Windows). Use these when the ven binary itself is broken. Canonical sources: scripts/uninstall.sh and scripts/uninstall.ps1 in the repo.' },
      { kind: 'h2', text: 'Exit codes' },
      {
        kind: 'table', head: ['Code', 'Meaning'], rows: [
          ['0', 'Uninstall succeeded, or no-op (nothing was installed)'],
          ['1', 'Partial failure (see report), needs elevation, or invalid flag combo'],
        ]
      },
    ],
    related: ['update', 'delete', 'path'],
  },

  // --- Runtime management ---------------------------------------------------

  list: {
    title: 'ven list',
    category: 'Runtimes',
    summary:
      'List installed language runtimes. Pass a language to filter; pass --verbose for disk usage and install dates; pass --json for CI.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven list                # all languages\nven list node           # only Node versions\nven list python --verbose\nven list --json         # for scripting' },
      { kind: 'h2', text: 'What it shows' },
      {
        kind: 'ul', items: [
          'Each language with every installed version under $VEN_HOME/<lang>/<version>/',
          'The "active" marker — the version resolved by the nearest ven.toml',
          '--verbose adds disk size and install date',
          '--json emits a stable object schema (keyed by language when listing all)',
        ]
      },
      { kind: 'callout', tone: 'info', title: 'Where ven stores runtimes', text: 'Default is ~/.ven on Linux/macOS and %USERPROFILE%\\.ven on Windows. Change it with `ven path set <dir>`.' },
    ],
    related: ['install', 'delete', 'path'],
  },

  delete: {
    title: 'ven delete',
    category: 'Runtimes',
    summary:
      'Delete an installed runtime (not a package). Wizard-driven by default; refuses to delete the version currently pinned by ven.toml unless --force.',
    sections: [
      { kind: 'callout', tone: 'warning', title: 'ven delete vs ven remove', text: 'ven delete uninstalls a language runtime. ven remove uninstalls a package from your project. Different surfaces.' },
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven delete                       # wizard: pick language, then version\nven delete python                # pick a Python version to delete\nven delete python 3.12.7         # confirm, then delete\nven delete python 3.12.7 -y      # skip the confirm prompt (CI)\nven delete python 3.12.7 --force # allow deleting the actively-pinned version\nven delete python 3.12.7 -y --json' },
      { kind: 'h2', text: 'Active-version guard' },
      { kind: 'p', text: 'If the chosen version is referenced by the nearest ven.toml, ven refuses and prints the path. Pass --force to override — useful when cleaning up a runtime you have already removed from ven.toml but the activation hook still resolves.' },
      { kind: 'h2', text: 'JSON output' },
      { kind: 'code', lang: 'json', code: '{\n  "language": "python",\n  "version": "3.12.7",\n  "deleted": true,\n  "freed_bytes": 145_802_240\n}' },
    ],
    related: ['list', 'install', 'path'],
  },

  path: {
    title: 'ven path',
    category: 'Runtimes',
    summary:
      'Manage where ven stores its data on disk. Use when ~/.ven (or %USERPROFILE%\\.ven) is on a full drive and you need to relocate every runtime, cache, and lockfile-state file to another drive. v0.1.6+.',
    sections: [
      { kind: 'h2', text: 'Subcommands' },
      { kind: 'code', lang: 'bash', code: 'ven path                       # alias for `ven path show`\nven path show                  # current $VEN_HOME, source, size, free disk\nven path set D:\\ven            # wizard: ask about moving existing data\nven path set D:\\ven --move     # move data, no prompt\nven path set D:\\ven --pointer-only   # leave data, just point future installs at the new dir\nven path set D:\\ven -y --json  # CI: default to move, machine-readable\nven path reset --move          # revert to ~/.ven, move data back' },
      { kind: 'h2', text: 'Resolution precedence' },
      { kind: 'p', text: 'ven walks this chain in order, taking the first hit:' },
      {
        kind: 'ul', items: [
          '$VEN_HOME (env var, wins everything)',
          '$VEN_STORAGE_PATH (env var, legacy fallback)',
          'a portable sibling .ven/ next to the ven binary',
          'the pointer file at ~/.config/ven/config.toml',
          '~/.ven (default)',
        ]
      },
      { kind: 'h2', text: 'What "set" actually does' },
      {
        kind: 'ul', items: [
          'Records the new location in ~/.config/ven/config.toml (ven\'s source of truth)',
          'Persists VEN_HOME in your user environment so npm / pip / new shells inherit it',
          'Optionally moves existing data atomically (with rollback on failure)',
        ]
      },
    ],
    related: ['list', 'delete', 'install'],
  },

  // --- Activation / shell ---------------------------------------------------

  setup: {
    title: 'ven setup',
    category: 'Getting started',
    summary:
      'One-time shell hook installation. Wires bash / zsh / fish / PowerShell to auto-apply ven.toml on cd.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven setup        # interactive: pick shell, confirm rc file' },
      { kind: 'h2', text: 'What it writes' },
      {
        kind: 'ul', items: [
          'A `>>> ven >>>` / `<<< ven <<<` block in your shell rc file',
          'The block calls ven shell hook <shell> and evals its output',
          'Idempotent — re-running ven setup replaces the existing block in place',
        ]
      },
      { kind: 'h2', text: 'After setup' },
      { kind: 'code', lang: 'bash', code: '# Open a new terminal and cd into a project with ven.toml\ncd my-app\nnode --version    # the version pinned by ven.toml — no manual activation' },
      { kind: 'callout', tone: 'info', title: 'Supported shells', text: 'bash, zsh, fish, PowerShell 5.1+ and 7+. Windows cmd.exe is not supported as an activation shell — use PowerShell or ven-launcher.' },
    ],
    related: ['use', 'deactivate', 'install'],
  },

  use: {
    title: 'ven use',
    category: 'Runtimes',
    summary:
      'Print the PATH/env exports for the nearest ven.toml. After `ven setup`, the shell function `ven-use` calls this for you automatically on cd.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven use            # exports for the current dir (evaluate in shell)\nven use .          # explicit\nven use path/to/project\n\n# Manual eval (only needed without ven setup):\n#   bash/zsh:     eval "$(ven use)"\n#   PowerShell:   iex ((ven use) -join "`n")' },
      { kind: 'h2', text: 'Why a separate "use" command?' },
      { kind: 'p', text: 'ven cannot mutate the parent shell\'s environment directly — only the shell itself can do that. So ven prints the exports as shell code, the shell hook installed by ven setup evaluates them, and your PATH ends up with the right runtime versions at the front.' },
    ],
    related: ['setup', 'deactivate', 'status'],
  },

  deactivate: {
    title: 'ven deactivate',
    category: 'Runtimes',
    summary:
      'Print shell code to undo ven\'s PATH overlay in this terminal. After `ven setup`, the alias `ven-deactivate` does this for you.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven deactivate\n\n# Manual eval:\n#   bash/zsh:     eval "$(ven deactivate)"\n#   PowerShell:   iex ((ven deactivate) -join "`n")' },
      { kind: 'h2', text: 'What it does' },
      {
        kind: 'ul', items: [
          'Restores VEN_ORIGINAL_PATH (saved by ven use on first activation in the shell)',
          'Unsets every VEN_*_VERSION marker',
          'Sets VEN_SKIP_PROJECT_VENV=1 to pause Python venv auto-prepend in this shell',
          'Run ven-use again to resume the overlay',
        ]
      },
    ],
    related: ['use', 'setup'],
  },

  // --- Package intelligence -------------------------------------------------

  'check-add': {
    title: 'ven check-add',
    category: 'Insight',
    summary:
      'Pre-flight a package install: compatibility check, conflict detection, CVE scan — without touching disk. Same intelligence ven add runs internally, exposed as a read-only query.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven check-add express\nven check-add react@18\nven check-add lodash@latest --json' },
      { kind: 'h2', text: 'What it reports' },
      {
        kind: 'ul', items: [
          'Resolved version (highest compatible with your runtime)',
          'Engine compatibility (Node engines, Python requires-python, etc.)',
          'Conflict chains against already-installed packages',
          'Open OSV vulnerabilities at that version',
          'Suggested resolution actions when conflicts exist',
        ]
      },
      { kind: 'callout', tone: 'success', text: 'No npm/pip/cargo invocation. Pure simulation against the live registry — safe to run anywhere.' },
    ],
    related: ['add', 'graph', 'check'],
  },

  graph: {
    title: 'ven graph',
    category: 'Insight',
    summary:
      'Render the dependency graph for the current project — either the last cached simulation snapshot, or a live read from node_modules / package manifests.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven graph              # last cached snapshot\nven graph --resolve    # live read (skips the SQLite cache)\nven graph --json       # machine-readable' },
      { kind: 'h2', text: 'Output (tree mode)' },
      { kind: 'code', lang: 'text', code: 'Dependency graph: my-app\nRuntime: node 20.20.2\n├── express@4.18.2\n│   ├── body-parser@1.20.2\n│   └── accepts@1.3.8\n├── lodash@4.17.21\n└── axios@1.6.8\n    └── follow-redirects@1.15.4  ! CVE-2024-28849\n\nConflicts: 0\nWarnings: 1 (CVE)\nOrphans: 0' },
      { kind: 'h2', text: 'Where the data comes from' },
      {
        kind: 'ul', items: [
          'Snapshots are written by ven add and ven check-add into ~/.ven/intelligence.db',
          '--resolve bypasses the cache and re-walks the installed tree from node_modules / installed pip packages / cargo metadata',
          'Conflict + CVE annotations are surfaced inline',
        ]
      },
    ],
    related: ['check-add', 'why', 'resolve'],
  },

  why: {
    title: 'ven why',
    category: 'Insight',
    summary:
      'Reverse dependency lookup — show the chain of packages that pull the named one into your project, and whether it\'s safe to remove.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven why express        # who depends on express?\nven why accepts        # transitive — accepts is pulled by express' },
      { kind: 'h2', text: 'Output' },
      { kind: 'code', lang: 'text', code: 'accepts@1.3.8 is required by:\n  └── express@4.18.2 (declared in ven.toml)\n\nReachable from 1 root. Removing express would also free accepts.' },
      { kind: 'callout', tone: 'info', text: 'ven why complements ven remove --cleanup: why explains the relationship; --cleanup acts on it.' },
    ],
    related: ['remove', 'graph', 'scan'],
  },

  resolve: {
    title: 'ven resolve',
    category: 'Packages',
    summary:
      'Scan the current dependency graph, find every conflict, compute an optimal version set, and apply it in one shot.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven resolve            # scan + propose + ask\nven resolve --yes      # scan + apply without confirming (CI/scripts)\nven resolve -y         # short flag\nven resolve --json     # machine-readable plan' },
      { kind: 'h2', text: 'Output' },
      { kind: 'code', lang: 'text', code: 'Scanning dependency graph...\nFound 2 conflicts:\n  [1] lodash@4.17 <-> express@1.3\n      Fix: lodash -> 4.16  OR  express -> 1.2\n  [2] axios@1.7 <-> Node 20\n      Fix: axios -> 1.6.8\n\nOptimal resolution:\n  lodash:  4.17 -> 4.16\n  axios:   1.7.0 -> 1.6.8\n  express: unchanged\n\nApply? [y/N]:' },
    ],
    related: ['check-add', 'graph', 'upgrade'],
  },

  // --- Package mutations ----------------------------------------------------

  remove: {
    title: 'ven remove',
    category: 'Packages',
    summary:
      'Remove a package with dependency-aware safety checks. Refuses to remove if other packages depend on it (override with --force); --cleanup prunes orphans.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven remove express              # remove with dependency check\nven remove lodash --force       # force-remove (skips the dependents check)\nven remove react react-dom      # multiple at once\nven remove --dry-run            # preview only\nven remove --cleanup            # find + remove every orphan' },
      { kind: 'h2', text: 'Pipeline' },
      {
        kind: 'ul', items: [
          'Build the reverse-dependency graph (who depends on the target?)',
          'If any dependents are found and --force is not set, refuse and list them',
          'Run the ecosystem\'s native uninstall (npm uninstall / pip uninstall / cargo remove / …)',
          'Strip the entry from ven.toml [packages]',
          'Cache invalidate the affected snapshot in ~/.ven/intelligence.db',
        ]
      },
      { kind: 'callout', tone: 'info', title: '--cleanup', text: 'Walks the graph, finds every package not reachable from a ven.toml root entry, and removes all of them in one shot.' },
    ],
    related: ['add', 'why', 'upgrade'],
  },

  upgrade: {
    title: 'ven upgrade',
    category: 'Packages',
    summary:
      'Preview and apply package upgrades — compatibility-checked against your runtime and dependency tree. Distinct from ven update, which updates ven itself.',
    sections: [
      { kind: 'callout', tone: 'warning', title: 'ven upgrade vs ven update', text: 'ven upgrade touches project packages (npm / pip / cargo / …). ven update self-updates the ven binaries. Different commands, different surfaces.' },
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven upgrade express             # preview Express upgrade\nven upgrade express --apply     # apply the upgrade\nven upgrade --all               # preview every package\nven upgrade --all --apply       # upgrade everything in one shot\nven upgrade react --dry-run     # preview without changes\nven upgrade --all --apply --force  # CI mode: no prompts' },
      { kind: 'h2', text: 'Preview output' },
      { kind: 'code', lang: 'text', code: '  express 4.18.2  ->  4.21.2  (latest compatible)\n  Compatibility:  Node 20.11.0 supported\n  Release notes:  npmjs.com/package/express/v/4.21.2\n  Run  ven upgrade express --apply  to upgrade' },
      { kind: 'h2', text: 'Compatibility floor' },
      { kind: 'p', text: 'ven picks the highest version whose declared engine range still satisfies your installed runtime. On Node 18, ven upgrade next will resolve to 13.5.6 instead of 14.x because Next.js 14 requires Node >= 18.17.' },
    ],
    related: ['add', 'remove', 'lock'],
  },

  // --- Health / scanning ----------------------------------------------------

  scan: {
    title: 'ven scan',
    category: 'Insight',
    summary:
      'Walk source files (gitignore-aware) and find packages you import/require but never declared in ven.toml or any native manifest. --fix patches them in.',
    sections: [
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven scan --ghosts          # report only\nven scan --ghosts --fix    # add detected ghosts to ven.toml\nven scan --ghosts --json   # for CI (exits 1 when ghosts found)' },
      { kind: 'h2', text: 'What "ghost" means' },
      { kind: 'p', text: 'A ghost is a package you import in code (e.g. `import requests` in Python, `require("lodash")` in Node) but never list in your manifest. Ghosts work as long as the package happens to be installed transitively — but break the moment a dependency removes them.' },
      { kind: 'h2', text: 'Per-ecosystem inputs' },
      {
        kind: 'table', head: ['Runtime', 'Source patterns scanned', 'Manifests cross-checked'], rows: [
          ['node / bun', '.js .ts .jsx .tsx .mjs .cjs', 'package.json, ven.toml'],
          ['python', '.py', 'requirements.txt, pyproject.toml, ven.toml'],
          ['ruby', '.rb', 'Gemfile, ven.toml'],
          ['rust', '.rs', 'Cargo.toml, ven.toml'],
          ['go', '.go', 'go.mod, ven.toml'],
          ['java', '.java', 'pom.xml, build.gradle, ven.toml'],
          ['deno', '.ts .tsx', 'deno.json, ven.toml'],
        ]
      },
      { kind: 'callout', tone: 'info', title: 'gitignore-aware', text: 'Powered by the `ignore` crate — respects .gitignore, .ignore, and global exclude. Never walks node_modules, target/, .venv/.' },
    ],
    related: ['check', 'add', 'why'],
  },
}

// ---- Language docs --------------------------------------------------------

function langDoc(slug) {
  const lang = LANGUAGES.find((l) => l.slug === slug)
  if (!lang) return null
  return {
    title: lang.name,
    category: 'Languages',
    summary: lang.tagline,
    sections: [
      { kind: 'h2', text: 'Supported versions' },
      { kind: 'p', text: `ven manages ${lang.versions.join(', ')} (and any other version available from the official source). Use aliases like "latest" or "lts" where supported.` },
      { kind: 'h2', text: 'Install' },
      { kind: 'code', lang: 'bash', code: lang.install.join('\n') },
      { kind: 'h2', text: 'ven.toml' },
      { kind: 'code', lang: 'toml', code: lang.venToml },
      { kind: 'h2', text: 'What ven sets in the environment' },
      { kind: 'table', head: ['Variable', 'Value'], rows: lang.env },
      { kind: 'h2', text: 'Package operations' },
      { kind: 'table', head: ['ven command', 'underlying'], rows: lang.packageOps },
      { kind: 'h2', text: 'Toolchain bundle' },
      { kind: 'ul', items: lang.includes },
      { kind: 'callout', tone: 'info', title: 'Source', text: `Downloads from ${lang.downloads}. Every artifact is SHA256-verified before extraction.` },
    ],
    related: ['init', 'install', 'add'],
  }
}

// ---- Assemble final index -------------------------------------------------

export const DOC_INDEX = {
  ...CMD_DOCS,
  ...Object.fromEntries(
    LANGUAGES.map((l) => [l.slug, langDoc(l.slug)]).filter(([, v]) => v)
  ),
}

// Sidebar grouping for DocsLayout.
export const DOC_GROUPS = [
  {
    title: 'Getting started',
    items: [
      { slug: 'init', label: 'ven init' },
      { slug: 'install', label: 'ven install' },
      { slug: 'setup', label: 'ven setup' },
      { slug: 'status', label: 'ven status' },
    ],
  },
  {
    title: 'Packages',
    items: [
      { slug: 'add', label: 'ven add' },
      { slug: 'remove', label: 'ven remove' },
      { slug: 'upgrade', label: 'ven upgrade' },
      { slug: 'lock', label: 'ven lock' },
      { slug: 'sync', label: 'ven sync' },
      { slug: 'check-add', label: 'ven check-add' },
      { slug: 'resolve', label: 'ven resolve' },
    ],
  },
  {
    title: 'Insight',
    items: [
      { slug: 'graph', label: 'ven graph' },
      { slug: 'why', label: 'ven why' },
      { slug: 'scan', label: 'ven scan' },
      { slug: 'check', label: 'ven check' },
      { slug: 'docs', label: 'ven docs' },
    ],
  },
  {
    title: 'Runtimes',
    items: [
      { slug: 'list', label: 'ven list' },
      { slug: 'delete', label: 'ven delete' },
      { slug: 'path', label: 'ven path' },
      { slug: 'use', label: 'ven use' },
      { slug: 'deactivate', label: 'ven deactivate' },
    ],
  },
  {
    title: 'Maintenance',
    items: [
      { slug: 'update', label: 'ven update' },
      { slug: 'doctor', label: 'ven doctor' },
      { slug: 'uninstall', label: 'ven uninstall' },
    ],
  },
  {
    title: 'Languages',
    items: LANGUAGES.map((l) => ({ slug: l.slug, label: l.name })),
  },
]

export function getDoc(slug) {
  return DOC_INDEX[slug] ?? null
}
