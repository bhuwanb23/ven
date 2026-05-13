import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import clsx from 'clsx'
import Button from '../components/ui/Button.jsx'
import GlassCard from '../components/ui/GlassCard.jsx'
import Icon from '../components/ui/Icon.jsx'
import ScriptedTerminal from '../components/ui/ScriptedTerminal.jsx'
import CodeBlock from '../components/ui/CodeBlock.jsx'
import Terminal from '../components/ui/Terminal.jsx'
import AnimatedDepGraph from '../components/effects/AnimatedDepGraph.jsx'
import Reveal from '../components/effects/Reveal.jsx'
import TiltCard from '../components/effects/TiltCard.jsx'
import useTypewriter from '../hooks/useTypewriter.js'
import useCountUp from '../hooks/useCountUp.js'
import usePrefersReducedMotion from '../hooks/usePrefersReducedMotion.js'
import { LANGUAGES } from '../content/languages.js'
import { COMPARE_HEADERS, COMPARE_ROWS } from '../content/compare.js'
import {
  INSTALL,
  PLATFORM_ORDER,
  GITHUB_URL,
  detectPlatform,
} from '../content/site.js'

// ---- Hero ------------------------------------------------------------------

const HEADLINE = 'One tool. Eight runtimes. Zero conflicts.'

// Mirrors the real CLI surface (`ven init`, `ven add`, `ven status`). Every
// output line is something the actual binary emits in a typical session.
const HERO_SCRIPT = [
  { kind: 'pause', ms: 400 },
  { kind: 'command', text: 'ven init' },
  { kind: 'output', text: '✓ Detected: empty directory', tone: 'muted' },
  { kind: 'output', text: '✓ Selected runtime: node 22.22.2 (LTS)', tone: 'cyan' },
  { kind: 'output', text: '✓ Wrote ven.toml', tone: 'success' },
  { kind: 'pause', ms: 300 },
  { kind: 'command', text: 'ven add express' },
  { kind: 'output', text: '→ Building dependency graph (npm) ...', tone: 'muted' },
  { kind: 'output', text: '  ├── express@4.18.2', tone: 'muted' },
  { kind: 'output', text: '  ├── body-parser@1.20.1', tone: 'muted' },
  { kind: 'output', text: '  └── … 10 transitive deps', tone: 'muted' },
  { kind: 'output', text: '✓ 0 conflicts · 12 packages · 0 CVEs (OSV)', tone: 'success' },
  { kind: 'output', text: '✓ ven.toml updated · package-lock.json written', tone: 'cyan' },
  { kind: 'pause', ms: 400 },
  { kind: 'command', text: 'ven status' },
  { kind: 'output', text: '┌─ Active environment ────────────────────────────', tone: 'cyan' },
  { kind: 'output', text: '│ node       22.22.2     ven.toml',                   tone: 'user' },
  { kind: 'output', text: '│ packages   12          0 missing · 0 drift',        tone: 'user' },
  { kind: 'output', text: '│ security   ok          0 CVE · 0 EOL alert',        tone: 'success' },
  { kind: 'output', text: '└─────────────────────────────────────────────────',  tone: 'cyan' },
  { kind: 'pause', ms: 800 },
]

function HeroInstall() {
  // `detectPlatform` already guards `typeof navigator === 'undefined'`, so it
  // is safe to call inside the lazy state initializer (Vite renders this
  // app client-side only — no SSR window mismatch to worry about).
  const [active, setActive] = useState(() => detectPlatform())

  const tabs = PLATFORM_ORDER.filter((id) => id !== 'source')
  const current = INSTALL[active] ?? INSTALL.windows

  return (
    <div className="w-full max-w-2xl glass-card rounded-xl overflow-hidden z-10 shadow-2xl">
      <div className="flex bg-surface-container-high px-2 border-b border-outline-variant/30">
        {tabs.map((id) => {
          const tab = INSTALL[id]
          const isActive = id === active
          return (
            <button
              key={id}
              type="button"
              onClick={() => setActive(id)}
              className={clsx(
                'px-4 py-2 text-terminal-output font-mono transition-colors',
                isActive
                  ? 'text-primary-fixed-dim border-b-2 border-primary-fixed-dim'
                  : 'text-on-surface-variant hover:text-on-surface border-b-2 border-transparent'
              )}
            >
              {tab.label}
            </button>
          )
        })}
      </div>
      <CodeBlock code={current.cmd} prompt="$" tone="success" language="" className="rounded-none border-0" />
      <div className="px-4 pb-3 -mt-1 font-mono text-[11px] text-on-surface-variant/70">
        {current.note}
      </div>
    </div>
  )
}

function HeroSection() {
  const reduced = usePrefersReducedMotion()
  const { text, done } = useTypewriter(HEADLINE, { speed: 38, startDelay: 250 })

  return (
    <section className="relative min-h-[80vh] flex flex-col items-center justify-center text-center px-margin-mobile hero-gradient overflow-hidden py-24">
      <div
        className="absolute inset-0 z-0 opacity-20 pointer-events-none"
        style={{
          backgroundImage: 'radial-gradient(circle, #00dbe7 1px, transparent 1px)',
          backgroundSize: '40px 40px',
        }}
      />
      <div className="mb-8 z-10">
        <div className="inline-flex items-center justify-center w-24 h-24 rounded-full bg-surface-container-high border border-primary-fixed-dim/30 cyan-glow mb-6">
          <span className="text-primary-fixed-dim text-5xl font-extrabold tracking-tighter">v</span>
        </div>
        <h1 className="font-display-lg text-display-lg text-primary mb-4 max-w-3xl mx-auto min-h-[1.2em]">
          <span aria-label={HEADLINE}>{text || '\u00A0'}</span>
          {!reduced && !done && (
            <span className="inline-block w-[0.5ch] -mb-1 ml-0.5 bg-primary-fixed-dim animate-caret-blink" style={{ height: '0.9em' }} />
          )}
        </h1>
        <p className="font-body-base text-body-base text-on-surface-variant max-w-xl mx-auto">
          ven is a Rust-built version &amp; dependency manager that installs runtimes, checks for CVEs,
          and reproduces environments from a single <code className="text-primary-fixed-dim">ven.toml</code>.
          Works on Windows, macOS, and Linux — no admin required.
        </p>
      </div>

      <div className="flex flex-col md:flex-row gap-4 mb-12 z-10">
        <Button to="/install" size="lg">
          Get Started <Icon name="rocket_launch" />
        </Button>
        <Button href={GITHUB_URL} size="lg" variant="ghost">
          View on GitHub <Icon name="terminal" />
        </Button>
      </div>

      <HeroInstall />
    </section>
  )
}

// ---- Problem ---------------------------------------------------------------

const PROBLEM_LINES = [
  { tone: 'muted', text: '$ npm install -g firebase-tools' },
  { tone: 'muted', text: 'Fetching packages... [34/122]' },
  { tone: 'muted', text: 'Installing dependencies...' },
  { tone: 'error', text: 'npm ERR! code EEXIST' },
  { tone: 'error', text: 'npm ERR! path /usr/local/bin/firebase' },
  { tone: 'error', text: 'npm ERR! File exists: /opt/homebrew/bin/firebase' },
  { tone: 'error', text: 'npm ERR! conflicting versions detected' },
  { tone: 'muted', text: '$ _' },
]

function ProblemSection() {
  return (
    <Reveal as="section" className="py-24 px-margin-desktop max-w-max-width mx-auto grid md:grid-cols-2 gap-16 items-center">
      <div>
        <h2 className="font-display-lg text-display-lg text-primary mb-6">
          The cycle of <span className="text-error">dependency hell</span>
        </h2>
        <p className="font-body-base text-body-base text-on-surface-variant mb-8">
          Single-language managers don't talk to your OS, don't simulate the dependency graph before
          installing, and never check whether the version you're about to pull has an active CVE.
          You only find out after the build breaks.
        </p>
        <ul className="space-y-4">
          {[
            'Conflicting global binary paths',
            'Outdated transitives with known CVEs',
            'Manual environment variable juggling',
            'Lockfiles that drift silently from reality',
          ].map((t) => (
            <li key={t} className="flex items-start gap-3">
              <Icon name="error" className="text-error mt-1" />
              <span className="font-body-base text-body-base">{t}</span>
            </li>
          ))}
        </ul>
      </div>
      <div className="relative">
        <Terminal title="npm — conflicting install" bodyClassName="h-[360px] space-y-1.5">
          {PROBLEM_LINES.map((l, i) => (
            <div
              key={i}
              className={clsx(
                'font-mono text-terminal-output',
                l.tone === 'error' ? 'text-error' : 'text-on-surface-variant',
                l.tone === 'error' && i === 3 && 'red-pulse rounded px-2 py-0.5 inline-block'
              )}
            >
              {l.text}
            </div>
          ))}
        </Terminal>
        <div className="absolute -bottom-6 -right-6 w-32 h-32 bg-error/10 blur-3xl pointer-events-none" />
      </div>
    </Reveal>
  )
}

// ---- Demo ------------------------------------------------------------------

function DemoSection() {
  return (
    <Reveal as="section" className="py-24 bg-surface-container-lowest">
      <div className="max-w-max-width mx-auto px-margin-desktop">
        <div className="text-center mb-16">
          <h2 className="font-headline-md text-headline-md text-primary-fixed-dim uppercase tracking-widest mb-4">
            Live demo
          </h2>
          <p className="font-body-base text-body-base text-on-surface-variant">
            A real <code className="text-primary-fixed-dim">ven init → add → status</code> session, on loop.
          </p>
        </div>
        <div className="max-w-3xl mx-auto">
          <ScriptedTerminal title="ven — interactive demo" script={HERO_SCRIPT} loop autoPlay />
        </div>
      </div>
    </Reveal>
  )
}

// ---- Feature grid ----------------------------------------------------------

const FEATURES = [
  {
    icon: 'sync_alt',
    title: 'Auto-switching shells',
    body: "Drop a ven.toml in your repo. cd into it from PowerShell, Bash, Zsh, or Fish — the runtime, PATH, and language-specific env vars swap automatically. Per-terminal, never global.",
    extra: (
      <div className="bg-black/50 p-3 rounded font-mono text-[12px] border border-outline-variant/20">
        <span className="text-on-surface-variant">→ Switching to: </span>
        <span className="text-secondary-fixed-dim">node 22.22.2</span>
        <span className="text-on-surface-variant"> + </span>
        <span className="text-secondary-fixed-dim">python 3.13.12</span>
      </div>
    ),
  },
  {
    icon: 'hub',
    title: 'Pre-install graph check',
    body: 'Every `ven add` builds the full dependency graph and replays peer constraints, version ranges, and CVE matches before touching node_modules. See the conflict before you create it.',
    extra: (
      <div className="space-y-2">
        <div className="h-1 w-full bg-surface-container-high rounded-full overflow-hidden">
          <div className="h-full bg-primary-fixed-dim" style={{ width: '78%' }} />
        </div>
        <div className="font-mono text-[11px] flex justify-between text-on-surface-variant">
          <span>12 packages walked</span>
          <span className="text-secondary-fixed-dim">0 conflicts</span>
        </div>
      </div>
    ),
  },
  {
    icon: 'security',
    title: 'OSV + EOL, offline-cached',
    body: '`ven check --security` queries osv.dev for advisories; `--eol` queries endoflife.date. Both responses are cached in a local SQLite store and served stale on network failure.',
    extra: (
      <div className="red-pulse bg-error-container/20 p-3 rounded border border-error/30 flex items-center gap-3">
        <Icon name="warning" className="text-error" />
        <span className="font-mono text-[12px] text-error">GHSA-cxjh-pqwp-8mfp blocked</span>
      </div>
    ),
  },
  {
    icon: 'lock',
    title: 'Deterministic ven.lock',
    body: 'One canonical-JSON SHA-256 hash per lockfile. `ven sync --check` will fail loudly if anything has drifted, and every package row carries an integrity hash so you can verify what was installed.',
    extra: (
      <div className="font-mono text-[11px] text-on-surface-variant space-y-1">
        <div>content_hash <span className="text-primary-fixed-dim">7b2f4e91…</span></div>
        <div>integrity   <span className="text-secondary-fixed-dim">sha256-aB3…</span></div>
      </div>
    ),
  },
  {
    icon: 'travel_explore',
    title: 'Ghost-dep scanner',
    body: '`ven scan --ghosts` walks your source tree (gitignore-aware) and flags packages you `import` but never declared. Works across all 8 ecosystems with the same regex-fast scanner.',
    extra: (
      <div className="font-mono text-[11px] text-error/80 leading-relaxed">
        ⚠ src/app.js imports <span className="text-on-surface">axios</span> · not in package.json
      </div>
    ),
  },
  {
    icon: 'workspaces',
    title: 'No-admin launcher',
    body: 'Drop `ven-launcher.exe` on a locked-down corporate box. No installer, no registry writes, no PATH mutation — it spawns a shell with your project\'s ven.toml applied and disappears on exit.',
    extra: (
      <div className="font-mono text-[11px] text-on-surface-variant">
        <span className="text-secondary-fixed-dim">✓</span> Writes nothing to the host machine
      </div>
    ),
  },
]

function FeatureGrid() {
  return (
    <Reveal as="section" className="py-24 overflow-x-hidden">
      <div className="max-w-max-width mx-auto px-margin-desktop mb-12">
        <h2 className="font-display-lg text-display-lg text-primary">Core intelligence</h2>
        <p className="font-body-base text-on-surface-variant mt-3 max-w-2xl">
          Everything below is shipping in the binary today — verified by the 84-case test matrix.
        </p>
      </div>
      <div className="flex gap-gutter px-margin-desktop overflow-x-auto pb-12 snap-x no-scrollbar">
        {FEATURES.map((f) => (
          <TiltCard
            key={f.title}
            className="min-w-[320px] md:min-w-[400px] glass-card p-8 rounded-xl snap-center hover:border-primary-fixed-dim/50 transition-colors group"
          >
            <div className="w-12 h-12 bg-primary-container/20 rounded-lg flex items-center justify-center text-primary-fixed-dim mb-6 group-hover:scale-110 transition-transform">
              <Icon name={f.icon} />
            </div>
            <h3 className="font-headline-md text-headline-md text-primary mb-4">{f.title}</h3>
            <p className="font-body-base text-body-base text-on-surface-variant mb-6">{f.body}</p>
            {f.extra}
          </TiltCard>
        ))}
      </div>
    </Reveal>
  )
}

// ---- Graph -----------------------------------------------------------------

function GraphSection() {
  return (
    <Reveal as="section" className="py-24 bg-surface-container">
      <div className="max-w-max-width mx-auto px-margin-desktop grid md:grid-cols-2 gap-16 items-center">
        <div className="order-2 md:order-1">
          <AnimatedDepGraph />
        </div>
        <div className="order-1 md:order-2">
          <h2 className="font-display-lg text-display-lg text-primary mb-6">Total visibility</h2>
          <p className="font-body-base text-body-base text-on-surface-variant mb-8">
            Stop guessing why a build broke. <code className="text-primary-fixed-dim">ven graph</code> renders
            the full transitive tree, <code className="text-primary-fixed-dim">ven why &lt;pkg&gt;</code> traces
            any package back to the root manifest that pulled it in, and CVE-tagged nodes glow red before they
            reach your disk.
          </p>
          <div className="space-y-4">
            {[
              { icon: 'account_tree', text: <><code className="text-primary-fixed-dim">ven graph</code> — full dependency tree, peer-pin annotated</> },
              { icon: 'search_insights', text: <><code className="text-primary-fixed-dim">ven why express</code> — root cause, in one line</> },
              { icon: 'troubleshoot', text: <><code className="text-primary-fixed-dim">ven scan --ghosts</code> — undeclared imports across 8 ecosystems</> },
              { icon: 'shield_with_heart', text: <><code className="text-primary-fixed-dim">ven check --security</code> — OSV-backed CVE scan, cached offline</> },
            ].map((row, i) => (
              <div key={i} className="flex items-center gap-4 p-4 rounded-lg bg-surface-container-high">
                <Icon name={row.icon} className="text-primary-fixed-dim" />
                <span className="font-body-base text-body-base">{row.text}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </Reveal>
  )
}

// ---- Languages strip -------------------------------------------------------

// 2-line mini-demos cycled through one rotating tile in LanguagesStrip.
const LANG_DEMOS = {
  node:   [{ kind: 'command', text: 'ven add express' }, { kind: 'output', text: '✓ 12 packages · 0 CVE', tone: 'success' }],
  python: [{ kind: 'command', text: 'ven add flask' },   { kind: 'output', text: '✓ installed in ./venv', tone: 'success' }],
  go:     [{ kind: 'command', text: 'ven add gin' },     { kind: 'output', text: '✓ go.mod updated',     tone: 'success' }],
  rust:   [{ kind: 'command', text: 'ven add serde' },   { kind: 'output', text: '✓ Cargo.toml updated', tone: 'success' }],
  java:   [{ kind: 'command', text: 'ven add guava' },   { kind: 'output', text: '✓ pom.xml updated',    tone: 'success' }],
  ruby:   [{ kind: 'command', text: 'ven add rails' },   { kind: 'output', text: '✓ Gemfile updated',    tone: 'success' }],
  deno:   [{ kind: 'command', text: 'ven add npm:chalk' }, { kind: 'output', text: '✓ deno.json updated', tone: 'success' }],
  bun:    [{ kind: 'command', text: 'ven add chalk' },   { kind: 'output', text: '✓ package.json updated', tone: 'success' }],
}

function LanguagesStrip() {
  const reduced = usePrefersReducedMotion()
  const [rotIdx, setRotIdx] = useState(0)

  useEffect(() => {
    if (reduced) return undefined
    const t = setInterval(() => {
      setRotIdx((cur) => (cur + 1) % LANGUAGES.length)
    }, 5000)
    return () => clearInterval(t)
  }, [reduced])

  return (
    <Reveal as="section" className="py-24 px-margin-desktop max-w-max-width mx-auto text-center">
      <h2 className="font-headline-md text-headline-md text-primary mb-4">Universal support</h2>
      <p className="text-on-surface-variant mb-12 max-w-xl mx-auto">
        Eight runtimes. Same commands. Same lockfile. Same guarantees. Verified by the 84-case test matrix
        on every release.
      </p>
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-8 gap-6">
        {LANGUAGES.map((l, i) => {
          const isRot = !reduced && i === rotIdx
          if (isRot) {
            const demo = LANG_DEMOS[l.slug] ?? LANG_DEMOS.node
            const cmd = demo[0].text
            const out = demo[1].text
            return (
              <Link
                key={l.slug}
                to="/languages"
                // Use rotIdx as key so the whole tile remounts on each rotation,
                // re-firing the reveal transition for the typing-feel effect.
                className="glass-card rounded-xl overflow-hidden flex flex-col gap-2 p-5 text-left border-primary-fixed-dim/60 cyan-glow transition-colors hover:border-primary-fixed-dim"
              >
                <div key={`rot-${rotIdx}`} className="flex flex-col gap-2 reveal-init reveal-in">
                  <div className="flex items-center justify-between">
                    <span className="text-primary-fixed-dim text-[11px] uppercase font-mono tracking-widest">
                      {l.name}
                    </span>
                    <span className="text-secondary-fixed-dim text-[10px] uppercase font-mono tracking-widest flex items-center gap-1">
                      <span className="inline-block w-1.5 h-1.5 bg-secondary-fixed-dim rounded-full animate-pulse" />
                      live
                    </span>
                  </div>
                  <div className="font-mono text-[12px] text-on-surface">
                    <span className="text-secondary-fixed-dim mr-1">$</span>
                    {cmd}
                  </div>
                  <div className="font-mono text-[11px] text-secondary-fixed-dim">{out}</div>
                </div>
              </Link>
            )
          }
          return (
            <TiltCard
              key={l.slug}
              max={4}
              as={Link}
              to="/languages"
              className="glass-card p-6 rounded-xl flex flex-col items-center gap-3 hover:border-primary-fixed-dim transition-all group"
            >
              <div className="text-3xl text-on-surface-variant group-hover:text-primary-fixed-dim transition-colors font-bold tracking-tighter">
                {l.code}
              </div>
              <div className="font-mono text-[10px] px-2 py-1 border border-secondary-fixed-dim/40 text-secondary-fixed-dim rounded uppercase">
                Stable
              </div>
              <div className="font-body-base text-sm font-bold">{l.name}</div>
            </TiltCard>
          )
        })}
      </div>
    </Reveal>
  )
}

// ---- Compare ---------------------------------------------------------------

function CompareCell({ value }) {
  if (value === true) {
    return <Icon name="check_circle" fill className="text-secondary-fixed-dim" />
  }
  if (value === false) {
    return <Icon name="cancel" className="text-on-surface-variant/40" />
  }
  return (
    <span className="font-mono text-[11px] uppercase tracking-widest text-tertiary-fixed-dim">
      {value}
    </span>
  )
}

function CompareSection() {
  return (
    <Reveal as="section" className="py-24 bg-surface-container-lowest">
      <div className="max-w-max-width mx-auto px-margin-desktop">
        <h2 className="font-display-lg text-display-lg text-primary text-center mb-4">
          Beyond basic package managers
        </h2>
        <p className="text-on-surface-variant text-center mb-16 max-w-2xl mx-auto">
          What ven does that nvm, mise, and Docker can't — without the daemon, the layers, or the admin rights.
        </p>
        <div className="overflow-x-auto">
          <table className="w-full border-collapse">
            <thead>
              <tr className="border-b border-outline-variant">
                {COMPARE_HEADERS.map((h, i) => (
                  <th
                    key={h}
                    className={clsx(
                      'p-6 font-headline-md',
                      i === 0 && 'text-left text-on-surface-variant',
                      i === 1 && 'text-center text-primary-fixed-dim bg-primary-fixed-dim/5',
                      i > 1 && 'text-center text-on-surface-variant'
                    )}
                  >
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="font-body-base">
              {COMPARE_ROWS.map((row) => (
                <tr key={row[0]} className="border-b border-outline-variant/30 shimmer-row">
                  <td className="p-6">{row[0]}</td>
                  <td className="p-6 text-center">
                    <CompareCell value={row[1]} />
                  </td>
                  <td className="p-6 text-center">
                    <CompareCell value={row[2]} />
                  </td>
                  <td className="p-6 text-center">
                    <CompareCell value={row[3]} />
                  </td>
                  <td className="p-6 text-center">
                    <CompareCell value={row[4]} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </Reveal>
  )
}

// ---- Numbers ---------------------------------------------------------------

function Stat({ value, label, tone = 'primary', suffix = '' }) {
  const { ref, value: counted } = useCountUp(value, { duration: 1400 })
  const color = tone === 'primary' ? 'text-primary-fixed-dim' : 'text-secondary-fixed-dim'
  return (
    <div ref={ref}>
      <div className={clsx('text-6xl font-bold mb-4 tracking-tighter tabular-nums', color)}>
        {counted}
        {suffix}
      </div>
      <div className="font-headline-md text-on-surface-variant uppercase text-sm tracking-widest">
        {label}
      </div>
    </div>
  )
}

function NumbersSection() {
  return (
    <Reveal as="section" className="py-24 border-y border-outline-variant/20">
      <div className="max-w-max-width mx-auto px-margin-desktop grid grid-cols-1 md:grid-cols-4 gap-12 text-center">
        <Stat value={8}   label="Supported runtimes" />
        <Stat value={84}  label="Verified test cases" tone="secondary" />
        <Stat value={50}  label="Switching time" suffix="ms" tone="primary" />
        <Stat value={0}   label="Admin rights required" tone="secondary" />
      </div>
    </Reveal>
  )
}

// ---- Enterprise ------------------------------------------------------------

function EnterpriseSection() {
  return (
    <Reveal as="section" className="py-24 px-margin-desktop max-w-max-width mx-auto">
      <GlassCard className="p-12 flex flex-col md:flex-row gap-16 items-center">
        <div className="md:w-1/2">
          <h2 className="font-display-lg text-display-lg text-primary mb-6">
            Built for restricted environments
          </h2>
          <p className="font-body-base text-body-base text-on-surface-variant mb-8">
            ven-launcher is a single portable executable. Drop it on a USB stick, a network share, or your
            Downloads folder, double-click it, and a shell opens with your project's ven.toml already
            applied. Nothing is written to <code className="text-on-surface">Program Files</code>, nothing
            is added to PATH, and nothing remains on exit.
          </p>
          <div className="flex flex-wrap gap-3">
            <div className="flex items-center gap-2 text-primary-fixed-dim bg-primary-fixed-dim/10 px-4 py-2 rounded-lg border border-primary-fixed-dim/20">
              <Icon name="verified_user" />
              <span className="text-sm font-bold">No sudo / no UAC</span>
            </div>
            <div className="flex items-center gap-2 text-primary-fixed-dim bg-primary-fixed-dim/10 px-4 py-2 rounded-lg border border-primary-fixed-dim/20">
              <Icon name="usb" />
              <span className="text-sm font-bold">Portable</span>
            </div>
            <div className="flex items-center gap-2 text-primary-fixed-dim bg-primary-fixed-dim/10 px-4 py-2 rounded-lg border border-primary-fixed-dim/20">
              <Icon name="shield_with_heart" />
              <span className="text-sm font-bold">Read-only host</span>
            </div>
          </div>
        </div>
        <div className="md:w-1/2 w-full">
          <Terminal title="locked-down corporate box" bodyClassName="space-y-1.5">
            <div className="text-on-surface-variant"># Run from anywhere, no install:</div>
            <div>
              <span className="text-secondary-fixed-dim">$</span>{' '}
              <span className="text-on-surface">./ven-launcher.exe</span>
            </div>
            <div className="text-on-surface-variant"># Spawns shell with this project's ven.toml applied</div>
            <div className="text-secondary-fixed-dim">✓ Environment ready: node 22 · python 3.13 · 34 packages</div>
            <div className="text-on-surface-variant"># Exit and the host is untouched.</div>
          </Terminal>
        </div>
      </GlassCard>
    </Reveal>
  )
}

// ---- Quick start -----------------------------------------------------------

function QuickStartSection() {
  return (
    <Reveal as="section" className="py-24 bg-surface-container-high/20">
      <div className="max-w-max-width mx-auto px-margin-desktop">
        <h2 className="font-display-lg text-display-lg text-primary text-center mb-16">
          Three commands to a reproducible env
        </h2>
        <div className="grid md:grid-cols-3 gap-12">
          {[
            {
              n: '01',
              title: 'Install',
              body: 'One-liner installer. SHA-256-verified, no admin required. Drops the binary into ~/.ven/bin.',
              cmd: INSTALL.linux.cmd,
            },
            {
              n: '02',
              title: 'Init',
              body: 'Pick a runtime and (optionally) some packages. Writes a ven.toml the rest of your team can use.',
              cmd: 'ven init',
            },
            {
              n: '03',
              title: 'Sync',
              body: 'On a fresh checkout, `ven sync` reproduces the env byte-for-byte from ven.lock. Drift, if any, is reported.',
              cmd: 'ven sync',
            },
          ].map((s) => (
            <div key={s.n} className="flex flex-col">
              <div className="text-4xl font-bold text-primary-fixed-dim/30 mb-6">{s.n}</div>
              <h3 className="font-headline-md text-headline-md text-primary mb-4">{s.title}</h3>
              <p className="font-body-base text-on-surface-variant mb-6">{s.body}</p>
              <CodeBlock code={s.cmd} prompt="$" tone="success" language="" />
            </div>
          ))}
        </div>
        <div className="text-center mt-16">
          <Button to="/install" size="lg">
            Install ven now <Icon name="arrow_forward" />
          </Button>
        </div>
      </div>
    </Reveal>
  )
}

// ---- Top-level -------------------------------------------------------------

export default function Landing() {
  return (
    <>
      <HeroSection />
      <ProblemSection />
      <DemoSection />
      <FeatureGrid />
      <GraphSection />
      <LanguagesStrip />
      <CompareSection />
      <NumbersSection />
      <EnterpriseSection />
      <QuickStartSection />
    </>
  )
}
