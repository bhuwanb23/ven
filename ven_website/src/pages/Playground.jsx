import { useState } from 'react'
import clsx from 'clsx'
import Icon from '../components/ui/Icon.jsx'
import ScriptedTerminal from '../components/ui/ScriptedTerminal.jsx'
import Reveal from '../components/effects/Reveal.jsx'

const SCENARIOS = [
  {
    id: 'install',
    title: 'Install a runtime',
    blurb: 'Resolve a version alias, verify the SHA256, and link the binary into PATH.',
    script: [
      { kind: 'command', text: 'ven install node 20' },
      { kind: 'output', text: '→ Resolving alias "20"... 20.20.2 (LTS)', tone: 'muted' },
      { kind: 'output', text: '→ Downloading nodejs-v20.20.2-win-x64.zip   [████████░] 100%', tone: 'cyan' },
      { kind: 'output', text: '→ Verifying SHA256… 6b1cc4… OK', tone: 'muted' },
      { kind: 'output', text: '→ Extracting to ~/.ven/node/20.20.2/', tone: 'muted' },
      { kind: 'output', text: '→ Smoke-testing node --version → v20.20.2', tone: 'muted' },
      { kind: 'output', text: '✔ Node 20.20.2 ready.', tone: 'success' },
    ],
  },
  {
    id: 'add',
    title: 'Add a dependency (graph-checked)',
    blurb: 'Pre-install simulation walks the full dependency graph before touching node_modules.',
    script: [
      { kind: 'command', text: 'ven add express' },
      { kind: 'output', text: '→ Building dependency graph for npm...', tone: 'muted' },
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
    id: 'check',
    title: 'ven check — health report',
    blurb: 'Unified CVE + EOL + ghost dependency scan against the current project.',
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
    id: 'lock',
    title: 'Lock & sync',
    blurb: 'Write ven.lock with SRI hashes, then reproduce the env on another machine.',
    script: [
      { kind: 'command', text: 'ven lock' },
      { kind: 'output', text: '→ Walking ven.toml (1 runtime, 12 packages)', tone: 'muted' },
      { kind: 'output', text: '→ Resolving via npm registry...', tone: 'muted' },
      { kind: 'output', text: '→ Computing SRI integrity (sha256-…) for 12 tarballs', tone: 'muted' },
      { kind: 'output', text: '✔ Wrote ven.lock (v2) · 12 integrity hashes', tone: 'success' },
      { kind: 'pause', ms: 400 },
      { kind: 'command', text: 'ven sync --check' },
      { kind: 'output', text: '→ Comparing ven.lock ↔ installed packages', tone: 'muted' },
      { kind: 'output', text: '✔ No drift. Environment matches lockfile.', tone: 'success' },
    ],
  },
  {
    id: 'switch',
    title: 'Auto-switch on cd',
    blurb: 'cd into a project — ven swaps the runtime + PATH automatically.',
    script: [
      { kind: 'command', text: 'cd ~/projects/data-pipeline' },
      { kind: 'output', text: '→ ven detected ven.toml', tone: 'muted' },
      { kind: 'output', text: '→ Switching: python 3.12.0 · go 1.21.5', tone: 'cyan' },
      { kind: 'output', text: '→ Activated .venv at .venv/', tone: 'muted' },
      { kind: 'output', text: '✔ Environment ready in 37ms', tone: 'success' },
      { kind: 'pause', ms: 300 },
      { kind: 'command', text: 'python --version' },
      { kind: 'output', text: 'Python 3.12.0', tone: 'user' },
      { kind: 'command', text: 'go version' },
      { kind: 'output', text: 'go version go1.21.5 linux/amd64', tone: 'user' },
    ],
  },
]

export default function Playground() {
  const [activeId, setActiveId] = useState(SCENARIOS[0].id)
  const active = SCENARIOS.find((s) => s.id === activeId) ?? SCENARIOS[0]

  return (
    <div className="max-w-max-width mx-auto px-margin-mobile md:px-margin-desktop py-16">
      <Reveal as="header" className="mb-12 text-center">
        <h1 className="font-display-lg text-display-lg text-primary mb-4">Playground</h1>
        <p className="text-on-surface-variant max-w-2xl mx-auto">
          Watch ven in action. Five canned scenarios — replay them as many times as you like. No install
          required.
        </p>
      </Reveal>

      <div className="flex flex-wrap justify-center gap-2 mb-10">
        {SCENARIOS.map((s) => (
          <button
            key={s.id}
            type="button"
            onClick={() => setActiveId(s.id)}
            className={clsx(
              'px-4 py-2 rounded-lg font-mono text-xs uppercase tracking-widest border transition-colors',
              s.id === activeId
                ? 'border-primary-fixed-dim text-primary-fixed-dim bg-primary-fixed-dim/10'
                : 'border-outline-variant/40 text-on-surface-variant hover:text-on-surface'
            )}
          >
            {s.title}
          </button>
        ))}
      </div>

      <Reveal as="div" className="grid lg:grid-cols-[1fr_2fr] gap-8 items-start">
        <aside className="glass-card rounded-xl p-6">
          <div className="text-[10px] uppercase tracking-widest text-outline mb-2">Scenario</div>
          <h2 className="font-headline-md text-headline-md text-primary mb-3">{active.title}</h2>
          <p className="text-on-surface-variant text-sm leading-relaxed mb-6">{active.blurb}</p>
          <ul className="space-y-2 text-sm">
            {SCENARIOS.map((s) => (
              <li key={s.id}>
                <button
                  type="button"
                  onClick={() => setActiveId(s.id)}
                  className={clsx(
                    'flex items-center gap-2 w-full text-left',
                    s.id === activeId
                      ? 'text-primary-fixed-dim font-bold'
                      : 'text-on-surface-variant hover:text-on-surface'
                  )}
                >
                  <Icon
                    name={s.id === activeId ? 'play_arrow' : 'radio_button_unchecked'}
                    className="text-sm"
                  />
                  {s.title}
                </button>
              </li>
            ))}
          </ul>
        </aside>

        <ScriptedTerminal
          key={active.id}
          title={`ven — ${active.id}`}
          script={active.script}
          autoPlay
          loop={false}
          height={420}
        />
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
