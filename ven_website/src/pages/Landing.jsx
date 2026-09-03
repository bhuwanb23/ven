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
  AFTER_INSTALL_COMMANDS,
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
          <p className="text-center text-xs text-on-surface-variant mt-6 font-mono">
            After installing:{' '}
            <span className="text-primary-fixed-dim">
              {AFTER_INSTALL_COMMANDS.join(' → ')}
            </span>
          </p>
        </div>
      </div>
    </Reveal>
  )
}

// ---- Feature grid ----------------------------------------------------------

// Each feature carries a `tone` so the card paints itself with a coordinated
// icon ring, accent bar, and command-pill colour. Three tones — primary (cyan),
// secondary (green), tertiary (red) — keep the grid scannable in one glance
// while still letting CVE/EOL items read as warnings.
const TONE = {
  primary: {
    accent: 'from-primary-fixed-dim/80 via-primary-fixed-dim/40 to-transparent',
    ring: 'border-primary-fixed-dim/40 group-hover:border-primary-fixed-dim',
    iconBg: 'bg-primary-fixed-dim/10',
    iconText: 'text-primary-fixed-dim',
    pill: 'text-primary-fixed-dim border-primary-fixed-dim/30 bg-primary-fixed-dim/5',
    glow: 'group-hover:shadow-cyan-glow',
  },
  secondary: {
    accent: 'from-secondary-fixed-dim/80 via-secondary-fixed-dim/40 to-transparent',
    ring: 'border-secondary-fixed-dim/40 group-hover:border-secondary-fixed-dim',
    iconBg: 'bg-secondary-fixed-dim/10',
    iconText: 'text-secondary-fixed-dim',
    pill: 'text-secondary-fixed-dim border-secondary-fixed-dim/30 bg-secondary-fixed-dim/5',
    glow: 'group-hover:shadow-[0_0_20px_rgba(0,230,57,0.18)]',
  },
  tertiary: {
    accent: 'from-error/80 via-error/40 to-transparent',
    ring: 'border-error/30 group-hover:border-error/60',
    iconBg: 'bg-error/10',
    iconText: 'text-error',
    pill: 'text-error border-error/30 bg-error/5',
    glow: 'group-hover:shadow-red-glow',
  },
}

const FEATURES = [
  {
    icon: 'sync_alt',
    tone: 'primary',
    title: 'Auto-switching shells',
    cmd: 'cd ./my-project',
    body: 'cd into any directory with a ven.toml — PowerShell, Bash, Zsh, and Fish all swap the runtime, PATH, and language env vars automatically. Per-terminal isolation, never global.',
    extra: (
      <div className="bg-[#050505] p-3 rounded font-mono text-[12px] border border-outline-variant/30">
        <span className="text-on-surface-variant">→ Switching to </span>
        <span className="text-secondary-fixed-dim">node 22.22.2</span>
        <span className="text-on-surface-variant"> + </span>
        <span className="text-secondary-fixed-dim">python 3.13.12</span>
      </div>
    ),
  },
  {
    icon: 'hub',
    tone: 'primary',
    title: 'Pre-install graph check',
    cmd: 'ven add express',
    body: 'Every `ven add` walks the full dependency graph and replays peer constraints, version ranges, and OSV CVE matches before any byte hits node_modules.',
    extra: (
      <div className="space-y-2">
        <div className="h-1.5 w-full bg-surface-container-high rounded-full overflow-hidden">
          <div className="h-full bg-gradient-to-r from-primary-fixed-dim to-secondary-fixed-dim" style={{ width: '78%' }} />
        </div>
        <div className="font-mono text-[11px] flex justify-between text-on-surface-variant">
          <span>12 packages walked</span>
          <span className="text-secondary-fixed-dim font-bold">0 conflicts</span>
        </div>
      </div>
    ),
  },
  {
    icon: 'security',
    tone: 'tertiary',
    title: 'OSV + EOL, offline-cached',
    cmd: 'ven check --security',
    body: '`ven check --security` queries osv.dev; `--eol` hits endoflife.date. Both responses are cached in a local SQLite store and served stale on network failure.',
    extra: (
      <div className="red-pulse bg-error/10 p-3 rounded border border-error/30 flex items-center gap-3">
        <Icon name="warning" className="text-error" />
        <span className="font-mono text-[12px] text-error">GHSA-cxjh-pqwp-8mfp blocked</span>
      </div>
    ),
  },
  {
    icon: 'lock',
    tone: 'secondary',
    title: 'Deterministic ven.lock',
    cmd: 'ven sync --check',
    body: 'One canonical-JSON SHA-256 content hash per lockfile. `ven sync --check` fails loudly on drift, and every package row carries its own SRI integrity string.',
    extra: (
      <div className="font-mono text-[11px] text-on-surface-variant space-y-1 bg-[#050505] p-3 rounded border border-outline-variant/30">
        <div>content_hash <span className="text-primary-fixed-dim">7b2f4e91…</span></div>
        <div>integrity   <span className="text-secondary-fixed-dim">sha256-aB3…</span></div>
      </div>
    ),
  },
  {
    icon: 'travel_explore',
    tone: 'tertiary',
    title: 'Ghost-dependency scanner',
    cmd: 'ven scan --ghosts',
    body: 'Walks your source tree (gitignore-aware) and flags packages you `import` but never declared. Same regex-fast scanner across all 8 ecosystems.',
    extra: (
      <div className="font-mono text-[11px] text-error/80 leading-relaxed bg-error/5 p-3 rounded border border-error/20">
        ⚠ src/app.js imports <span className="text-on-surface">axios</span> · not in package.json
      </div>
    ),
  },
  {
    icon: 'workspaces',
    tone: 'secondary',
    title: 'One-click corporate bundle',
    cmd: 'Start ven.cmd',
    body: 'Download the portable zip, extract anywhere, double-click `Start ven.cmd` (or `Start ven.command` on macOS). A terminal opens with ven activated — no installer, no registry writes, no PATH mutation, passes Zscaler.',
    extra: (
      <div className="font-mono text-[11px] text-on-surface-variant flex items-center gap-2">
        <span className="text-secondary-fixed-dim font-bold">✓</span>
        <span>Writes nothing to the host machine</span>
      </div>
    ),
  },
]

function FeatureCard({ feature, index }) {
  const t = TONE[feature.tone]
  return (
    <TiltCard
      max={4}
      className={clsx(
        'group relative flex flex-col h-full p-7 rounded-2xl bg-surface-container-low border transition-all duration-300',
        'hover:-translate-y-1',
        t.ring,
        t.glow
      )}
    >
      {/* Top accent strip — fades from tone-coloured to transparent so the
          eye is drawn to the icon corner. */}
      <div
        className={clsx(
          'absolute top-0 left-0 right-0 h-[3px] rounded-t-2xl bg-gradient-to-r',
          t.accent
        )}
      />

      <div className="flex items-start justify-between mb-5">
        <div
          className={clsx(
            'w-12 h-12 rounded-xl flex items-center justify-center text-2xl transition-transform duration-300 group-hover:scale-110',
            t.iconBg,
            t.iconText
          )}
        >
          <Icon name={feature.icon} />
        </div>
        <span className="font-mono text-[10px] text-outline tabular-nums tracking-widest">
          {String(index + 1).padStart(2, '0')} / 06
        </span>
      </div>

      <h3 className="font-headline-md text-xl font-bold text-on-surface mb-3">
        {feature.title}
      </h3>

      <p className="font-body-base text-sm text-on-surface-variant leading-relaxed mb-5 flex-grow">
        {feature.body}
      </p>

      <div className="space-y-3">
        <div
          className={clsx(
            'inline-flex items-center gap-2 font-mono text-[11px] px-2.5 py-1 rounded border tracking-wide',
            t.pill
          )}
        >
          <span className="opacity-60">$</span>
          {feature.cmd}
        </div>
        {feature.extra}
      </div>
    </TiltCard>
  )
}

function FeatureGrid() {
  return (
    <Reveal as="section" className="py-24">
      <div className="max-w-max-width mx-auto px-margin-desktop">
        <div className="mb-12 flex flex-col md:flex-row md:items-end md:justify-between gap-4">
          <div>
            <div className="font-mono text-xs uppercase tracking-widest text-primary-fixed-dim/80 mb-3">
              · Core intelligence
            </div>
            <h2 className="font-display-lg text-display-lg text-primary">
              Six engines, one binary
            </h2>
            <p className="font-body-base text-on-surface-variant mt-4 max-w-2xl">
              Every capability below is shipping in the binary today — verified by the 84-case test matrix
              that runs on every commit.
            </p>
          </div>
          <div className="hidden md:flex items-center gap-2 text-xs font-mono text-on-surface-variant">
            <span className="inline-block w-1.5 h-1.5 rounded-full bg-secondary-fixed-dim animate-pulse" />
            v0.2.6 · all green
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {FEATURES.map((f, i) => (
            <FeatureCard key={f.title} feature={f} index={i} />
          ))}
        </div>
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

// One-line demos cycled through the standalone "live demo" strip below the
// language grid (the grid itself stays perfectly uniform).
const LANG_DEMOS = {
  node:   { cmd: 'ven add express',   out: '✓ 12 packages · 0 CVE',     mgr: 'npm' },
  python: { cmd: 'ven add flask',     out: '✓ installed in ./venv',     mgr: 'pip' },
  go:     { cmd: 'ven add gin',       out: '✓ go.mod updated',          mgr: 'go mod' },
  rust:   { cmd: 'ven add serde',     out: '✓ Cargo.toml updated',      mgr: 'cargo' },
  java:   { cmd: 'ven add guava',     out: '✓ pom.xml updated',         mgr: 'maven' },
  ruby:   { cmd: 'ven add rails',     out: '✓ Gemfile updated',         mgr: 'gem' },
  deno:   { cmd: 'ven add npm:chalk', out: '✓ deno.json updated',       mgr: 'deno' },
  bun:    { cmd: 'ven add chalk',     out: '✓ package.json updated',    mgr: 'bun' },
}

function LangTile({ lang, active, onActivate }) {
  const pinned = lang.versions[lang.versions.length - 1]
  return (
    <TiltCard
      max={4}
      as={Link}
      to={`/docs/${lang.slug}`}
      onMouseEnter={onActivate}
      onFocus={onActivate}
      className={clsx(
        'group relative flex flex-col items-start gap-3 p-5 rounded-2xl bg-surface-container-low border transition-all duration-300 hover:-translate-y-1',
        active
          ? 'border-primary-fixed-dim cyan-glow'
          : 'border-outline-variant/30 hover:border-primary-fixed-dim/50'
      )}
    >
      <div className="flex items-start justify-between w-full">
        <div
          className={clsx(
            'w-10 h-10 rounded-lg flex items-center justify-center font-bold text-base tracking-tighter transition-colors',
            active
              ? 'bg-primary-fixed-dim/15 text-primary-fixed-dim'
              : 'bg-surface-container-high text-on-surface-variant group-hover:text-primary-fixed-dim'
          )}
        >
          {lang.code}
        </div>
        <span className="font-mono text-[10px] tracking-widest uppercase text-secondary-fixed-dim/80 px-2 py-0.5 border border-secondary-fixed-dim/30 rounded bg-secondary-fixed-dim/5">
          stable
        </span>
      </div>
      <div className="text-left w-full">
        <div className="font-bold text-on-surface text-base mb-0.5">{lang.name}</div>
        <div className="font-mono text-[11px] text-on-surface-variant tabular-nums">v{pinned}</div>
      </div>
      <div className="font-mono text-[10px] text-outline uppercase tracking-widest">
        {lang.pkgMgr}
      </div>
      {active && (
        <div className="absolute inset-x-0 -bottom-[1px] h-[2px] bg-gradient-to-r from-transparent via-primary-fixed-dim to-transparent" />
      )}
    </TiltCard>
  )
}

function LiveDemoStrip({ lang }) {
  const demo = LANG_DEMOS[lang.slug] ?? LANG_DEMOS.node
  return (
    <div className="glass-card rounded-2xl border border-primary-fixed-dim/30 overflow-hidden">
      <div className="flex items-center justify-between px-5 py-3 border-b border-outline-variant/20 bg-surface-container-low">
        <div className="flex items-center gap-3">
          <span className="inline-flex w-2 h-2 rounded-full bg-secondary-fixed-dim animate-pulse" />
          <span className="font-mono text-[11px] uppercase tracking-widest text-on-surface-variant">
            Live · {lang.name}
          </span>
        </div>
        <span className="font-mono text-[10px] tracking-widest uppercase text-outline">
          ven → {demo.mgr}
        </span>
      </div>
      <div
        // Remount via key on every language change so the reveal transition
        // re-runs and the cmd/output appear to type in from blank.
        key={lang.slug}
        className="p-5 font-mono text-sm bg-[#050505] reveal-init reveal-in flex flex-col gap-1.5"
      >
        <div>
          <span className="text-secondary-fixed-dim mr-2">$</span>
          <span className="text-on-surface">{demo.cmd}</span>
          <span className="inline-block w-1.5 h-3.5 align-middle bg-primary-fixed-dim/70 animate-caret-blink ml-0.5" />
        </div>
        <div className="text-secondary-fixed-dim">{demo.out}</div>
      </div>
    </div>
  )
}

function LanguagesStrip() {
  const reduced = usePrefersReducedMotion()
  const [activeIdx, setActiveIdx] = useState(0)
  const activeLang = LANGUAGES[activeIdx] ?? LANGUAGES[0]

  useEffect(() => {
    if (reduced) return undefined
    const t = setInterval(() => {
      setActiveIdx((cur) => (cur + 1) % LANGUAGES.length)
    }, 4500)
    return () => clearInterval(t)
  }, [reduced])

  return (
    <Reveal as="section" className="py-24 px-margin-desktop max-w-max-width mx-auto">
      <div className="mb-12 flex flex-col md:flex-row md:items-end md:justify-between gap-4">
        <div>
          <div className="font-mono text-xs uppercase tracking-widest text-primary-fixed-dim/80 mb-3">
            · Universal support
          </div>
          <h2 className="font-display-lg text-display-lg text-primary">
            Eight runtimes, one CLI surface
          </h2>
          <p className="font-body-base text-on-surface-variant mt-4 max-w-2xl">
            Same `ven init / add / status / lock` commands across every language. Same `ven.toml`
            manifest. Same SHA-256 verified install pipeline. Verified by the 84-case test matrix.
          </p>
        </div>
        <Button to="/languages" variant="ghost">
          See all languages <Icon name="arrow_forward" />
        </Button>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 mb-8">
        {LANGUAGES.map((l, i) => (
          <LangTile
            key={l.slug}
            lang={l}
            active={i === activeIdx}
            onActivate={() => setActiveIdx(i)}
          />
        ))}
      </div>

      <LiveDemoStrip lang={activeLang} />
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
          <p className="font-body-base text-body-base text-on-surface-variant mb-6">
            Corporate laptop where <code className="text-on-surface">irm | iex</code> and{' '}
            <code className="text-on-surface">curl | sh</code> are blocked by Zscaler? Download the
            portable zip from the browser, extract anywhere, and{' '}
            <strong className="text-on-surface">double-click the bundled terminal shim</strong> —{' '}
            <code className="text-on-surface">Start ven.cmd</code> on Windows,{' '}
            <code className="text-on-surface">Start ven.command</code> on macOS,{' '}
            <code className="text-on-surface">start-ven.sh</code> on Linux. A shell opens with your
            project's <code className="text-on-surface">ven.toml</code> already applied.
          </p>
          <p className="font-body-base text-body-base text-on-surface-variant mb-8">
            Nothing is written to <code className="text-on-surface">Program Files</code>, nothing is
            added to PATH, and nothing remains on exit.
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
            <div className="flex items-center gap-2 text-secondary-fixed-dim bg-secondary-fixed-dim/10 px-4 py-2 rounded-lg border border-secondary-fixed-dim/20">
              <Icon name="block" />
              <span className="text-sm font-bold">Bypasses Zscaler</span>
            </div>
          </div>
        </div>
        <div className="md:w-1/2 w-full">
          <Terminal title="locked-down corporate box" bodyClassName="space-y-1.5">
            <div className="text-on-surface-variant"># 1. Download zip from the browser (HTTPS, passes proxy)</div>
            <div className="text-on-surface-variant"># 2. Extract anywhere (Desktop, USB, network share)</div>
            <div className="text-on-surface-variant"># 3. Double-click Start ven.cmd</div>
            <div>
              <span className="text-secondary-fixed-dim">PS&gt;</span>{' '}
              <span className="text-on-surface">ven --version</span>
            </div>
            <div className="text-on-surface">ven 0.1.6 (x86_64-pc-windows-msvc)</div>
            <div className="text-secondary-fixed-dim">✓ Environment ready: node 22 · python 3.13 · 34 packages</div>
            <div className="text-on-surface-variant"># Close the window — the host is untouched.</div>
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
