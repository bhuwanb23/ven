import { useEffect, useState } from 'react'
import clsx from 'clsx'
import Icon from '../components/ui/Icon.jsx'
import ScriptedTerminal from '../components/ui/ScriptedTerminal.jsx'
import Reveal from '../components/effects/Reveal.jsx'

// Each scenario carries metadata that drives the index card, the breakdown
// panel under the terminal, and the badge tone at the top of the card. The
// `category` field groups scenarios in the index for fast scanning.
const SCENARIOS = [
  {
    id: 'install',
    title: 'Install a runtime',
    blurb: 'Resolve a version alias, verify the SHA-256, and link the binary into PATH.',
    category: 'runtime',
    tone: 'primary',
    icon: 'download',
    teaches: [
      'Aliases like `20` and `lts` expand to a real triplet',
      'Every artifact is SHA-256 verified before extraction',
      'A smoke-test runs before the version is marked installed',
    ],
    script: [
      { kind: 'command', text: 'ven install node 22' },
      { kind: 'output', text: '→ Resolving alias "22"... 22.22.2 (current)', tone: 'muted' },
      { kind: 'output', text: '→ Downloading nodejs-v22.22.2-win-x64.zip   [████████░] 100%', tone: 'cyan' },
      { kind: 'output', text: '→ Verifying SHA-256… 6b1cc4… OK', tone: 'muted' },
      { kind: 'output', text: '→ Extracting to ~/.ven/node/22.22.2/', tone: 'muted' },
      { kind: 'output', text: '→ Smoke-testing node --version → v22.22.2', tone: 'muted' },
      { kind: 'output', text: '✔ Node 22.22.2 ready.', tone: 'success' },
    ],
  },
  {
    id: 'add',
    title: 'Add a dependency',
    blurb: 'Pre-install simulation walks the full dependency graph before touching node_modules.',
    category: 'packages',
    tone: 'primary',
    icon: 'add_circle',
    teaches: [
      'Full transitive graph resolved before any byte hits disk',
      'Peer / range / CVE conflicts surface as soft warnings',
      'ven.toml is updated atomically with the install',
    ],
    script: [
      { kind: 'command', text: 'ven add express' },
      { kind: 'output', text: '→ Building dependency graph (npm)...', tone: 'muted' },
      { kind: 'output', text: '  ├── express@4.18.2', tone: 'muted' },
      { kind: 'output', text: '  ├── body-parser@1.20.1', tone: 'muted' },
      { kind: 'output', text: '  ├── cookie-parser@1.4.6', tone: 'muted' },
      { kind: 'output', text: '  └── … (9 more transitive deps)', tone: 'muted' },
      { kind: 'output', text: '→ Conflict simulation: 0 conflicts', tone: 'success' },
      { kind: 'output', text: '→ CVE scan: 0 advisories (OSV)', tone: 'success' },
      { kind: 'output', text: '→ Applying with npm install express…', tone: 'cyan' },
      { kind: 'output', text: '✔ express@4.18.2 added · ven.toml updated', tone: 'success' },
    ],
  },
  {
    id: 'graph',
    title: 'Visualise the graph',
    blurb: '`ven graph` prints the full transitive tree with peer pins, CVE markers, and orphan callouts.',
    category: 'packages',
    tone: 'primary',
    icon: 'account_tree',
    teaches: [
      'Reads from ven.lock, falls back to ven.toml if not yet locked',
      'CVE-flagged packages are tagged in-line (no second tool needed)',
      'Use `--json` for machine consumption',
    ],
    script: [
      { kind: 'command', text: 'ven graph' },
      { kind: 'output', text: 'Dependency graph: my-app (npm)', tone: 'cyan' },
      { kind: 'output', text: 'Runtime: node 22.22.2', tone: 'muted' },
      { kind: 'output', text: '├── express@4.18.2', tone: 'muted' },
      { kind: 'output', text: '│   ├── body-parser@1.20.1', tone: 'muted' },
      { kind: 'output', text: '│   │   ├── bytes@3.1.2', tone: 'muted' },
      { kind: 'output', text: '│   │   └── qs@6.11.0', tone: 'muted' },
      { kind: 'output', text: '│   └── accepts@1.3.8', tone: 'muted' },
      { kind: 'output', text: '├── lodash@4.17.21', tone: 'muted' },
      { kind: 'output', text: '└── axios@1.6.8', tone: 'muted' },
      { kind: 'output', text: '    └── follow-redirects@1.15.4  ⚠ GHSA-cxjh-pqwp-8mfp', tone: 'warn' },
      { kind: 'output', text: '', tone: 'muted' },
      { kind: 'output', text: 'Conflicts: 0 · CVE warnings: 1 · Orphans: 0', tone: 'cyan' },
    ],
  },
  {
    id: 'why',
    title: 'Trace why a package exists',
    blurb: '`ven why <pkg>` walks back from a transitive dep to the manifest entry that pulled it in.',
    category: 'packages',
    tone: 'primary',
    icon: 'search_insights',
    teaches: [
      'Equivalent to `npm ls`, `cargo tree -i`, `poetry show --tree -r`',
      'Works the same across all 8 ecosystems',
      'Use this before removing anything from ven.toml',
    ],
    script: [
      { kind: 'command', text: 'ven why follow-redirects' },
      { kind: 'output', text: '→ Searching dependency graph for follow-redirects', tone: 'muted' },
      { kind: 'output', text: '', tone: 'muted' },
      { kind: 'output', text: 'follow-redirects@1.15.4', tone: 'cyan' },
      { kind: 'output', text: '└── axios@1.6.8 (declared in ven.toml [packages])', tone: 'muted' },
      { kind: 'output', text: '', tone: 'muted' },
      { kind: 'output', text: '1 path · 1 declared root · 0 cycles', tone: 'success' },
    ],
  },
  {
    id: 'check',
    title: 'Health report',
    blurb: 'Unified CVE + EOL + ghost dependency scan against the current project.',
    category: 'health',
    tone: 'tertiary',
    icon: 'shield_with_heart',
    teaches: [
      'CVE data from osv.dev, EOL data from endoflife.date',
      'Both responses cached in SQLite, served stale on failure',
      'Exit code reflects severity for CI use',
    ],
    script: [
      { kind: 'command', text: 'ven check' },
      { kind: 'output', text: '→ Resolving project at /home/me/api', tone: 'muted' },
      { kind: 'output', text: '→ Querying OSV (12 packages)…', tone: 'muted' },
      { kind: 'output', text: '→ Querying endoflife.date (3 runtimes)…', tone: 'muted' },
      { kind: 'output', text: '', tone: 'muted' },
      { kind: 'output', text: '┌─ CVEs ───────────────────────────────────', tone: 'cyan' },
      { kind: 'output', text: '│ ⚠ axios 0.21.1 → GHSA-wf5p-g6vw-rhxx (HIGH)', tone: 'warn' },
      { kind: 'output', text: '└──────────────────────────────────────────', tone: 'cyan' },
      { kind: 'output', text: '┌─ EOL alerts ────────────────────────────', tone: 'cyan' },
      { kind: 'output', text: '│ ⚠ node 16.x reached EOL on 2023-09-11', tone: 'warn' },
      { kind: 'output', text: '└──────────────────────────────────────────', tone: 'cyan' },
      { kind: 'output', text: '✖ 1 CVE · 1 EOL warning · exit 2', tone: 'error' },
    ],
  },
  {
    id: 'scan-ghosts',
    title: 'Find ghost dependencies',
    blurb: '`ven scan --ghosts` walks the source tree and flags imports that are not declared in any manifest.',
    category: 'health',
    tone: 'tertiary',
    icon: 'travel_explore',
    teaches: [
      'gitignore-aware walker — never scans node_modules / target',
      'Detects ESM, CJS, dynamic imports, scoped packages',
      'Same scanner across Node, Python, Go, Rust, etc.',
    ],
    script: [
      { kind: 'command', text: 'ven scan --ghosts' },
      { kind: 'output', text: '→ Walking source tree (npm project, 42 files)…', tone: 'muted' },
      { kind: 'output', text: '→ Cross-referencing package.json + ven.toml', tone: 'muted' },
      { kind: 'output', text: '', tone: 'muted' },
      { kind: 'output', text: '┌─ Ghost imports ────────────────────────', tone: 'cyan' },
      { kind: 'output', text: '│ src/api/users.js:3   import axios from "axios"', tone: 'warn' },
      { kind: 'output', text: '│ src/utils/log.js:1   require("lodash")', tone: 'warn' },
      { kind: 'output', text: '│ src/index.js:12      await import("@scope/zlib-tools")', tone: 'warn' },
      { kind: 'output', text: '└────────────────────────────────────────', tone: 'cyan' },
      { kind: 'output', text: '', tone: 'muted' },
      { kind: 'output', text: '3 ghost packages found · run `ven add` to declare', tone: 'error' },
    ],
  },
  {
    id: 'lock',
    title: 'Lock & sync',
    blurb: 'Write ven.lock with SHA-256 integrity hashes, then reproduce the env on another machine.',
    category: 'reproducibility',
    tone: 'secondary',
    icon: 'lock',
    teaches: [
      'Canonical-JSON ensures the content hash is stable across machines',
      '`ven sync --check` fails loudly on drift instead of healing silently',
      'Per-package integrity is verified at install time',
    ],
    script: [
      { kind: 'command', text: 'ven lock' },
      { kind: 'output', text: '→ Walking ven.toml (1 runtime, 12 packages)', tone: 'muted' },
      { kind: 'output', text: '→ Resolving via npm registry...', tone: 'muted' },
      { kind: 'output', text: '→ Computing SRI integrity (sha256-…) for 12 tarballs', tone: 'muted' },
      { kind: 'output', text: '✔ Wrote ven.lock (v2) · 12 integrity hashes', tone: 'success' },
      { kind: 'pause', ms: 400 },
      { kind: 'command', text: 'ven sync --check' },
      { kind: 'output', text: '→ Comparing ven.lock ↔ installed packages', tone: 'muted' },
      { kind: 'output', text: '→ Recomputing canonical content_hash…', tone: 'muted' },
      { kind: 'output', text: '✔ No drift. Environment matches lockfile.', tone: 'success' },
    ],
  },
  {
    id: 'switch',
    title: 'Auto-switch on cd',
    blurb: 'cd into a project — ven swaps the runtime + PATH automatically.',
    category: 'runtime',
    tone: 'secondary',
    icon: 'sync_alt',
    teaches: [
      'Shell hook reads ven.toml on every directory change',
      'Per-terminal isolation — Node 22 in one tab, 18 in another',
      'Auto-creates ./venv for Python projects',
    ],
    script: [
      { kind: 'command', text: 'cd ~/projects/data-pipeline' },
      { kind: 'output', text: '→ ven detected ven.toml', tone: 'muted' },
      { kind: 'output', text: '→ Switching: python 3.13.12 · go 1.26.2', tone: 'cyan' },
      { kind: 'output', text: '→ Activated .venv at ./venv/', tone: 'muted' },
      { kind: 'output', text: '✔ Environment ready in 37ms', tone: 'success' },
      { kind: 'pause', ms: 300 },
      { kind: 'command', text: 'python --version' },
      { kind: 'output', text: 'Python 3.13.12', tone: 'user' },
      { kind: 'command', text: 'go version' },
      { kind: 'output', text: 'go version go1.26.2 linux/amd64', tone: 'user' },
    ],
  },
]

const CATEGORY_LABEL = {
  runtime: 'Runtime',
  packages: 'Packages',
  health: 'Health',
  reproducibility: 'Reproducibility',
}

const TONE_CHIP = {
  primary: 'text-primary-fixed-dim border-primary-fixed-dim/30 bg-primary-fixed-dim/5',
  secondary: 'text-secondary-fixed-dim border-secondary-fixed-dim/30 bg-secondary-fixed-dim/5',
  tertiary: 'text-error border-error/30 bg-error/5',
}

const TONE_ICON_BG = {
  primary: 'bg-primary-fixed-dim/10 text-primary-fixed-dim',
  secondary: 'bg-secondary-fixed-dim/10 text-secondary-fixed-dim',
  tertiary: 'bg-error/10 text-error',
}

function ScenarioIndex({ activeId, onPick }) {
  // Group scenarios by category so the list reads like a section index.
  const groups = Object.keys(CATEGORY_LABEL).map((cat) => ({
    category: cat,
    items: SCENARIOS.filter((s) => s.category === cat),
  }))
  return (
    <aside className="glass-card rounded-2xl p-5 lg:sticky lg:top-24">
      <div className="flex items-center justify-between mb-4 px-1">
        <span className="font-mono text-[11px] uppercase tracking-widest text-on-surface-variant">
          Scenarios
        </span>
        <span className="font-mono text-[10px] uppercase tracking-widest text-primary-fixed-dim/80">
          {SCENARIOS.length} total
        </span>
      </div>
      <div className="space-y-5">
        {groups.map((g) => (
          <div key={g.category}>
            <div className="flex items-center gap-2 px-1 mb-2">
              <span className="inline-block w-3 h-px bg-outline-variant/50" />
              <span className="text-[10px] font-bold text-outline uppercase tracking-widest">
                {CATEGORY_LABEL[g.category]}
              </span>
            </div>
            <ul className="space-y-1">
              {g.items.map((s) => {
                const isActive = s.id === activeId
                return (
                  <li key={s.id}>
                    <button
                      type="button"
                      onClick={() => onPick(s.id)}
                      className={clsx(
                        'w-full flex items-center gap-3 px-2 py-2 rounded-lg text-sm transition-all duration-200',
                        isActive
                          ? 'bg-primary-fixed-dim/10 text-primary-fixed-dim font-bold pl-3 border-l-2 border-primary-fixed-dim'
                          : 'text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface hover:translate-x-1'
                      )}
                    >
                      <Icon
                        name={isActive ? 'play_arrow' : s.icon}
                        className={clsx('text-base', !isActive && 'opacity-60')}
                      />
                      <span className="text-left flex-1 truncate">{s.title}</span>
                    </button>
                  </li>
                )
              })}
            </ul>
          </div>
        ))}
      </div>
      <div className="mt-5 pt-4 border-t border-outline-variant/20 font-mono text-[11px] text-on-surface-variant flex items-center justify-between">
        <span>← → to step</span>
        <span className="opacity-60">no install</span>
      </div>
    </aside>
  )
}

function BreakdownPanel({ scenario }) {
  return (
    <div className="grid sm:grid-cols-3 gap-4">
      <div className="sm:col-span-2 glass-card rounded-xl p-5">
        <div className="flex items-center gap-2 mb-3">
          <Icon name="school" className="text-primary-fixed-dim text-base" />
          <span className="font-mono text-[11px] uppercase tracking-widest text-on-surface-variant">
            What this teaches
          </span>
        </div>
        <ul className="space-y-2">
          {scenario.teaches.map((t, i) => (
            <li key={i} className="flex items-start gap-2.5 text-sm text-on-surface-variant">
              <span className="text-secondary-fixed-dim mt-0.5">✓</span>
              <span>{t}</span>
            </li>
          ))}
        </ul>
      </div>
      <div className="glass-card rounded-xl p-5 flex flex-col gap-3 bg-surface-container-low">
        <div className="flex items-center gap-2">
          <Icon name="terminal" className="text-primary-fixed-dim text-base" />
          <span className="font-mono text-[11px] uppercase tracking-widest text-on-surface-variant">
            Run it for real
          </span>
        </div>
        <code className="font-mono text-sm text-primary-fixed-dim break-all">
          {firstCommand(scenario)}
        </code>
        <a
          href="/install"
          className="text-xs text-secondary-fixed-dim hover:underline underline-offset-4 mt-auto"
        >
          Install ven →
        </a>
      </div>
    </div>
  )
}

function firstCommand(scenario) {
  const cmd = scenario.script.find((s) => s.kind === 'command')
  return cmd ? cmd.text : 'ven'
}

export default function Playground() {
  const [activeId, setActiveId] = useState(SCENARIOS[0].id)
  const active = SCENARIOS.find((s) => s.id === activeId) ?? SCENARIOS[0]

  // Keyboard nav — left / right cycles through scenarios. Ignored when the
  // user is typing in any input/textarea (none on this page right now, but
  // future-proofing costs nothing).
  useEffect(() => {
    const onKey = (e) => {
      if (e.target instanceof HTMLElement) {
        const tag = e.target.tagName
        if (tag === 'INPUT' || tag === 'TEXTAREA') return
      }
      if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return
      const idx = SCENARIOS.findIndex((s) => s.id === activeId)
      if (idx < 0) return
      const next =
        e.key === 'ArrowRight'
          ? (idx + 1) % SCENARIOS.length
          : (idx - 1 + SCENARIOS.length) % SCENARIOS.length
      setActiveId(SCENARIOS[next].id)
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [activeId])

  return (
    <div className="max-w-max-width mx-auto px-margin-mobile md:px-margin-desktop py-16">
      <Reveal as="header" className="mb-12 text-center">
        <div className="inline-flex items-center gap-2 font-mono text-[11px] uppercase tracking-widest text-primary-fixed-dim/80 mb-5 px-3 py-1 border border-primary-fixed-dim/30 rounded-full bg-primary-fixed-dim/5">
          <span className="inline-block w-1.5 h-1.5 rounded-full bg-primary-fixed-dim animate-pulse" />
          Live · {SCENARIOS.length} canned demos
        </div>
        <h1 className="font-display-lg text-display-lg text-primary mb-4">Playground</h1>
        <p className="text-on-surface-variant max-w-2xl mx-auto">
          Watch ven in action. Eight scripted scenarios across runtime, packages, health, and
          reproducibility — replay them as many times as you like.{' '}
          <span className="text-primary-fixed-dim font-mono text-sm">No install required.</span>
        </p>
      </Reveal>

      <Reveal as="div" className="grid lg:grid-cols-[18rem_1fr] gap-6 items-start">
        <ScenarioIndex activeId={activeId} onPick={setActiveId} />

        <div className="space-y-6 min-w-0">
          {/* Scenario header card — title, category chip, tone-coloured icon
              ring. Mirrors the scenario card surface visually so the terminal
              underneath reads as the "live region" of this header. */}
          <div className="glass-card rounded-2xl p-6">
            <div className="flex items-start gap-4">
              <div
                className={clsx(
                  'w-12 h-12 rounded-xl flex items-center justify-center text-2xl shrink-0',
                  TONE_ICON_BG[active.tone] ?? TONE_ICON_BG.primary
                )}
              >
                <Icon name={active.icon} />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex flex-wrap items-center gap-2 mb-2">
                  <h2 className="font-headline-md text-headline-md text-primary truncate">
                    {active.title}
                  </h2>
                  <span
                    className={clsx(
                      'font-mono text-[10px] uppercase tracking-widest px-2 py-0.5 border rounded',
                      TONE_CHIP[active.tone] ?? TONE_CHIP.primary
                    )}
                  >
                    {CATEGORY_LABEL[active.category]}
                  </span>
                </div>
                <p className="text-on-surface-variant text-sm leading-relaxed">{active.blurb}</p>
              </div>
            </div>
          </div>

          <ScriptedTerminal
            key={active.id}
            title={`ven — ${active.id}`}
            script={active.script}
            autoPlay
            loop={false}
            height={460}
          />

          <BreakdownPanel scenario={active} />
        </div>
      </Reveal>

      <div className="mt-16 text-center text-sm text-on-surface-variant">
        Ready for the real thing?{' '}
        <a className="text-primary-fixed-dim hover:underline underline-offset-4" href="/install">
          Install ven →
        </a>
      </div>
    </div>
  )
}
