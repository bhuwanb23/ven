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
      'Interactive project bootstrap. Picks a runtime, optionally adds packages, writes ven.toml, and (optionally) validates the result.',
    sections: [
      { kind: 'p', text: 'ven init is the project initialization wizard. It writes a ven.toml in the current directory with a runtime, optional packages, and optional [env] variables.' },
      { kind: 'h2', text: 'Usage' },
      { kind: 'code', lang: 'bash', code: 'ven init                  # interactive\nven init --template       # pick a template\nven init --with-packages  # multi-select popular packages\nven init --validate       # validate after writing' },
      { kind: 'h2', text: 'Templates' },
      { kind: 'ul', items: [
        'Express API Server — node + express + cors + dotenv',
        'React + Vite Frontend — node + react + react-dom + vite',
        'Next.js Full-stack — node + next + react + react-dom',
        'Empty Project — pick language + version, no packages',
      ]},
      { kind: 'h2', text: 'Generated ven.toml' },
      { kind: 'code', lang: 'toml', code: '[runtime]\nnode = "20"\n\n[packages]\nexpress = "^4.18.2"\ncors    = "^2.8.5"\ndotenv  = "^16.3.1"\n\n[env]\nNODE_ENV = "development"\nPORT     = "3000"' },
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
      { kind: 'ul', items: [
        '"20"        → resolves to latest patch of 20.x',
        '"latest"    → newest stable release',
        '"lts"       → most recent LTS (Node, Java, Ruby)',
        '"20.20.2"   → exact pin',
      ]},
      { kind: 'h2', text: 'What happens' },
      { kind: 'ul', items: [
        'Resolves alias → exact version',
        'Downloads from the official source (nodejs.org, python.org, …)',
        'Verifies SHA256 against the upstream checksum file',
        'Extracts into ~/.ven/<lang>/<version>/',
        'Runs a smoke-test (e.g. `node --version`)',
        'Activates in the current shell',
      ]},
      { kind: 'callout', tone: 'info', title: 'No admin / sudo', text: 'Everything happens inside ~/.ven/. Nothing is written to system paths.' },
      { kind: 'h2', text: 'Exit codes' },
      { kind: 'table', head: ['Code', 'Meaning'], rows: [
        ['0', 'Installed and activated successfully'],
        ['1', 'Download / extract failed'],
        ['2', 'SHA256 mismatch — the artifact was rejected'],
      ]},
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
      { kind: 'ul', items: [
        'Active runtime versions (resolved, not just declared)',
        'Installed vs missing toolchains',
        'Package summary (count, recent additions)',
        'Health overview — short OSV + EOL teaser',
        'VEN_*_VERSION shell markers + activation state',
      ]},
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
      { kind: 'ul', items: [
        'Build the full dependency graph (recursive, includes transitive deps)',
        'Simulate the install — flag conflicts before touching disk',
        'Run a CVE pre-scan via OSV (cached)',
        'Apply via the ecosystem\'s native PM (npm / pip / cargo / …)',
        'Update [packages] in ven.toml',
      ]},
      { kind: 'callout', tone: 'success', text: 'You never have to run npm/pip/cargo manually for normal workflows — ven keeps ven.toml as the source of truth.' },
      { kind: 'h2', text: 'Per-ecosystem mapping' },
      { kind: 'table', head: ['Runtime', 'Underlying command'], rows: [
        ['node / bun', 'npm install / bun add'],
        ['python', 'pip install (venv-aware)'],
        ['ruby', 'gem install'],
        ['rust', 'cargo add'],
        ['go / java / deno', 'native — ven coordinates ven.toml + lockfile only'],
      ]},
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
      { kind: 'ul', items: [
        'Detect upstream tampering — npm pulls the registry, ven verifies the bytes',
        'CI can use --check to fail builds when a teammate forgets to commit ven.lock',
        'Stable across machines: ven sync rejects any package whose integrity differs',
      ]},
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
      { kind: 'table', head: ['Code', 'Meaning'], rows: [
        ['0', 'No drift'],
        ['1', 'Drift detected — at least one package mismatched'],
        ['2', 'ven.lock is missing or malformed'],
      ]},
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
      { kind: 'code', lang: 'bash', code: 'ven check                  # CVE + EOL\nven check --security       # CVE only\nven check --eol            # EOL only\nven check --json           # CI / scripting' },
      { kind: 'h2', text: 'How it works' },
      { kind: 'ul', items: [
        'Reads ven.lock when present (otherwise [packages]) for pinned (name, version) pairs',
        'POSTs batches of 1000 to https://api.osv.dev/v1/querybatch',
        'Enriches every vuln via /v1/vulns/<id> for severity + summary + fixed-in version',
        'GETs https://endoflife.date/api/<product>.json per runtime in [runtime]',
        'Caches everything in ~/.ven/intelligence.db with stale-on-failure fallback',
      ]},
      { kind: 'h2', text: 'Severity buckets' },
      { kind: 'table', head: ['CVSS', 'Bucket'], rows: [
        ['≥ 9.0', 'CRITICAL'],
        ['≥ 7.0', 'HIGH'],
        ['≥ 4.0', 'MODERATE'],
        ['> 0', 'LOW'],
      ]},
      { kind: 'h2', text: 'Exit codes' },
      { kind: 'table', head: ['Code', 'Meaning'], rows: [
        ['0', 'No actionable issues'],
        ['1', 'Any HIGH/CRITICAL CVE or a passed-EOL runtime'],
      ]},
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
      { kind: 'ul', items: [
        'Resolves the version from ven.lock → ven.toml → registry latest',
        'Pulls the README / API metadata from the ecosystem\'s registry (npm, PyPI, crates.io, RubyGems)',
        'Renders Markdown to terminal via termimad — or opens the registry page via the system browser',
        'Caches fetched docs under ~/.ven/cache/docs/<pkg>/<version>/',
      ]},
      { kind: 'h2', text: '--diff' },
      { kind: 'p', text: 'Diffs the public API surface between two versions of the same package (function/class signatures). Highlights additions, removals, and signature changes. Useful before any major-version upgrade.' },
    ],
    related: ['check', 'add'],
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
      { slug: 'status', label: 'ven status' },
    ],
  },
  {
    title: 'Commands',
    items: [
      { slug: 'add', label: 'ven add' },
      { slug: 'lock', label: 'ven lock' },
      { slug: 'sync', label: 'ven sync' },
    ],
  },
  {
    title: 'Health & docs',
    items: [
      { slug: 'check', label: 'ven check' },
      { slug: 'docs', label: 'ven docs' },
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
