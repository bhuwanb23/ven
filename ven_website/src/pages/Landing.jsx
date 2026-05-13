import { Link } from 'react-router-dom'
import clsx from 'clsx'
import Button from '../components/ui/Button.jsx'
import GlassCard from '../components/ui/GlassCard.jsx'
import Icon from '../components/ui/Icon.jsx'
import ScriptedTerminal from '../components/ui/ScriptedTerminal.jsx'
import CodeBlock from '../components/ui/CodeBlock.jsx'

const INSTALL_TABS = [
  { id: 'windows', label: 'Windows', cmd: 'irm https://get.ven.sh/install.ps1 | iex' },
  { id: 'macos', label: 'macOS', cmd: 'curl -fsSL https://get.ven.sh/install.sh | sh' },
  { id: 'linux', label: 'Linux', cmd: 'curl -fsSL https://get.ven.sh/install.sh | sh' },
]

// Demo script for the hero terminal — mirrors the "ven install node 20"
// canned session from the original HTML, with cyan/green coloring.
const HERO_SCRIPT = [
  { kind: 'pause', ms: 500 },
  { kind: 'command', text: 'ven install node 20' },
  { kind: 'output', text: '✔ Validating version 20.20.2 (LTS)', tone: 'muted' },
  { kind: 'output', text: '✔ Checking for conflicts with local bin', tone: 'muted' },
  { kind: 'output', text: '✔ Downloading nodejs-v20.20.2-win-x64.zip   100%', tone: 'cyan' },
  { kind: 'output', text: '✔ Node 20.20.2 successfully linked to global path.', tone: 'success' },
  { kind: 'pause', ms: 400 },
  { kind: 'command', text: 'ven add express' },
  { kind: 'output', text: '✔ Resolving dependency graph (npm)', tone: 'muted' },
  { kind: 'output', text: '✔ 0 conflicts · 12 packages · 0 CVEs', tone: 'success' },
  { kind: 'output', text: '✔ Updated ven.toml [packages]', tone: 'cyan' },
  { kind: 'pause', ms: 600 },
]

function HeroSection() {
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
        <h1 className="font-display-lg text-display-lg text-primary mb-4 max-w-3xl mx-auto">
          The Intelligent Version &amp; <br className="hidden md:block" /> Dependency Manager
        </h1>
        <p className="font-body-base text-body-base text-on-surface-variant max-w-xl mx-auto">
          One tool. Every language. Zero conflicts. Engineered for absolute technical mastery and performance.
        </p>
      </div>

      <div className="flex flex-col md:flex-row gap-4 mb-12 z-10">
        <Button to="/install" size="lg">
          Get Started <Icon name="rocket_launch" />
        </Button>
        <Button href="https://github.com/yourorg/ven" size="lg" variant="ghost">
          View on GitHub <Icon name="terminal" />
        </Button>
      </div>

      <div className="w-full max-w-2xl glass-card rounded-xl overflow-hidden z-10 shadow-2xl">
        <div className="flex bg-surface-container-high px-4 border-b border-outline-variant/30">
          {INSTALL_TABS.map((t, i) => (
            <div
              key={t.id}
              className={clsx(
                'px-4 py-2 text-terminal-output font-mono',
                i === 0
                  ? 'text-primary-fixed-dim border-b-2 border-primary-fixed-dim'
                  : 'text-on-surface-variant hover:text-on-surface'
              )}
            >
              {t.label}
            </div>
          ))}
        </div>
        <CodeBlock code={INSTALL_TABS[0].cmd} prompt="$" tone="success" language="" className="rounded-none border-0" />
      </div>
    </section>
  )
}

function ProblemSection() {
  return (
    <section className="py-24 px-margin-desktop max-w-max-width mx-auto grid md:grid-cols-2 gap-16 items-center">
      <div>
        <h2 className="font-display-lg text-display-lg text-primary mb-6">
          The Cycle of <span className="text-error">Dependency Hell</span>
        </h2>
        <p className="font-body-base text-body-base text-on-surface-variant mb-8">
          Standard package managers operate in silos. They don't talk to your OS, they don't check for
          system-level conflicts, and they certainly don't care about your productivity.
        </p>
        <ul className="space-y-4">
          {[
            'Conflicting global binary paths',
            'Insecure outdated dependencies',
            'Manual environment variable switching',
          ].map((t) => (
            <li key={t} className="flex items-start gap-3">
              <Icon name="error" className="text-error mt-1" />
              <span className="font-body-base text-body-base">{t}</span>
            </li>
          ))}
        </ul>
      </div>
      <div className="relative">
        <div className="glass-card rounded-xl p-1 bg-surface-container-highest/30">
          <div className="bg-black rounded-lg p-6 font-mono text-terminal-output h-[400px] overflow-hidden">
            <div className="text-on-surface-variant mb-2">$ npm install -g firebase-tools</div>
            <div className="text-on-surface-variant mb-2">Fetching packages... [34/122]</div>
            <div className="text-on-surface-variant mb-2">Installing dependencies...</div>
            <div className="text-error font-bold p-2 border border-error/50 bg-error-container/20 rounded mt-4 animate-pulse">
              npm ERR! code EEXIST
              <br />
              npm ERR! path /usr/local/bin/firebase
              <br />
              npm ERR! File exists: /opt/homebrew/bin/firebase
              <br />
              npm ERR! conflicting versions detected
            </div>
            <div className="text-on-surface-variant mt-4">$ _</div>
          </div>
        </div>
        <div className="absolute -bottom-6 -right-6 w-32 h-32 bg-error/10 blur-3xl pointer-events-none" />
      </div>
    </section>
  )
}

function DemoSection() {
  return (
    <section className="py-24 bg-surface-container-lowest">
      <div className="max-w-max-width mx-auto px-margin-desktop">
        <div className="text-center mb-16">
          <h2 className="font-headline-md text-headline-md text-primary-fixed-dim uppercase tracking-widest mb-4">
            Interactive Demo
          </h2>
          <p className="font-body-base text-body-base text-on-surface-variant">
            Experience the lightning speed of ven.
          </p>
        </div>
        <div className="max-w-3xl mx-auto">
          <ScriptedTerminal title="ven — interactive demo" script={HERO_SCRIPT} loop autoPlay />
        </div>
      </div>
    </section>
  )
}

const FEATURES = [
  {
    icon: 'sync_alt',
    title: 'Auto-Switching',
    body: "ven detects your ven.toml on cd and swaps the runtime + PATH instantly — per-terminal, per-project, zero scripts.",
    extra: (
      <div className="bg-black/50 p-3 rounded font-mono text-[12px] border border-outline-variant/20">
        <span className="text-on-surface-variant">Switching to: </span>
        <span className="text-secondary-fixed-dim">Node v20.20.2</span>
      </div>
    ),
  },
  {
    icon: 'hub',
    title: 'Dependency Intelligence',
    body: 'Pre-install graph analysis flags conflicts before they hit your disk. See exactly which package will break which.',
    extra: (
      <div className="flex items-center gap-2">
        <div className="h-1 w-full bg-surface-container-high rounded-full overflow-hidden">
          <div className="h-full bg-primary-fixed-dim w-3/4" />
        </div>
      </div>
    ),
  },
  {
    icon: 'security',
    title: 'Built-In Security',
    body: 'CVE scanning via osv.dev and runtime EOL alerts via endoflife.date — cached locally, served stale on network failure.',
    extra: (
      <div className="bg-error-container/20 p-3 rounded border border-error/30 flex items-center gap-3">
        <Icon name="warning" className="text-error" />
        <span className="font-mono text-[12px] text-error">CVE-2024-1234 Blocked</span>
      </div>
    ),
  },
]

function FeatureGrid() {
  return (
    <section className="py-24 overflow-x-hidden">
      <div className="max-w-max-width mx-auto px-margin-desktop mb-12">
        <h2 className="font-display-lg text-display-lg text-primary">Core Intelligence</h2>
      </div>
      <div className="flex gap-gutter px-margin-desktop overflow-x-auto pb-12 snap-x no-scrollbar">
        {FEATURES.map((f) => (
          <div
            key={f.title}
            className="min-w-[320px] md:min-w-[400px] glass-card p-8 rounded-xl snap-center hover:border-primary-fixed-dim/50 transition-colors group"
          >
            <div className="w-12 h-12 bg-primary-container/20 rounded-lg flex items-center justify-center text-primary-fixed-dim mb-6 group-hover:scale-110 transition-transform">
              <Icon name={f.icon} />
            </div>
            <h3 className="font-headline-md text-headline-md text-primary mb-4">{f.title}</h3>
            <p className="font-body-base text-body-base text-on-surface-variant mb-6">{f.body}</p>
            {f.extra}
          </div>
        ))}
      </div>
    </section>
  )
}

function GraphSection() {
  return (
    <section className="py-24 bg-surface-container">
      <div className="max-w-max-width mx-auto px-margin-desktop grid md:grid-cols-2 gap-16 items-center">
        <div className="order-2 md:order-1">
          <div className="relative glass-card aspect-square rounded-2xl flex items-center justify-center overflow-hidden">
            <div className="absolute w-24 h-24 border border-primary-fixed-dim/30 rounded-full flex items-center justify-center z-10 bg-surface text-primary-fixed-dim font-bold">
              Node.js
            </div>
            <div className="absolute top-10 left-10 w-20 h-20 border border-outline-variant/30 rounded-full flex items-center justify-center bg-surface-container text-on-surface-variant text-sm">
              Express
            </div>
            <div className="absolute bottom-10 right-10 w-20 h-20 border border-outline-variant/30 rounded-full flex items-center justify-center bg-surface-container text-on-surface-variant text-sm">
              Lodash
            </div>
            <div className="absolute top-10 right-10 w-20 h-20 border border-error/50 rounded-full flex items-center justify-center bg-surface-container text-error text-sm animate-pulse">
              Legacy
            </div>
            <svg className="absolute inset-0 w-full h-full opacity-20" xmlns="http://www.w3.org/2000/svg">
              <line stroke="white" strokeWidth="1" x1="50%" x2="20%" y1="50%" y2="20%" />
              <line stroke="white" strokeWidth="1" x1="50%" x2="80%" y1="50%" y2="80%" />
              <line stroke="#ffb4ab" strokeWidth="2" x1="50%" x2="80%" y1="50%" y2="20%" />
            </svg>
          </div>
        </div>
        <div className="order-1 md:order-2">
          <h2 className="font-display-lg text-display-lg text-primary mb-6">Total Visibility</h2>
          <p className="font-body-base text-body-base text-on-surface-variant mb-8">
            Stop guessing why a specific version is breaking your build. ven maps the internal structure of
            your environments so you can pinpoint conflicts in seconds.
          </p>
          <div className="space-y-4">
            {[
              { icon: 'search_insights', text: 'Trace binary origins back to project manifests' },
              { icon: 'troubleshoot', text: 'Identify shadow dependencies taking up disk space' },
              { icon: 'shield_with_heart', text: 'Block CVE-tagged versions before they reach your disk' },
            ].map((row) => (
              <div key={row.text} className="flex items-center gap-4 p-4 rounded-lg bg-surface-container-high">
                <Icon name={row.icon} className="text-primary-fixed-dim" />
                <span className="font-body-base text-body-base">{row.text}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  )
}

const LANGS = [
  { code: 'JS', name: 'Node.js', stable: true },
  { code: 'PY', name: 'Python', stable: true },
  { code: 'GO', name: 'Go', stable: true },
  { code: 'RS', name: 'Rust', stable: true },
  { code: 'JV', name: 'Java', stable: true },
  { code: 'RB', name: 'Ruby', stable: true },
  { code: 'DN', name: 'Deno', stable: true },
  { code: 'BN', name: 'Bun', stable: true },
]

function LanguagesStrip() {
  return (
    <section className="py-24 px-margin-desktop max-w-max-width mx-auto text-center">
      <h2 className="font-headline-md text-headline-md text-primary mb-4">Universal Support</h2>
      <p className="text-on-surface-variant mb-12 max-w-xl mx-auto">
        Eight runtimes today. Same commands. Same lockfile. Same guarantees.
      </p>
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-8 gap-6">
        {LANGS.map((l) => (
          <Link
            key={l.name}
            to="/languages"
            className="glass-card p-6 rounded-xl flex flex-col items-center gap-3 hover:border-primary-fixed-dim transition-all group"
          >
            <div className="text-3xl text-on-surface-variant group-hover:text-primary-fixed-dim transition-colors">
              {l.code}
            </div>
            <div className="font-mono text-[10px] px-2 py-1 border border-secondary-fixed-dim/40 text-secondary-fixed-dim rounded uppercase">
              Stable
            </div>
            <div className="font-body-base text-sm font-bold">{l.name}</div>
          </Link>
        ))}
      </div>
    </section>
  )
}

const COMPARE_ROWS = [
  ['Pre-install conflict check', true, false, true],
  ['Cross-language unification', true, false, true],
  ['Real-time CVE scanning (OSV)', true, false, false],
  ['Visual dependency graph', true, false, false],
  ['Runs without admin / sudo', true, false, false],
]

function CompareSection() {
  return (
    <section className="py-24 bg-surface-container-lowest">
      <div className="max-w-max-width mx-auto px-margin-desktop">
        <h2 className="font-display-lg text-display-lg text-primary text-center mb-16">
          Beyond basic package managers
        </h2>
        <div className="overflow-x-auto">
          <table className="w-full border-collapse">
            <thead>
              <tr className="border-b border-outline-variant">
                <th className="text-left p-6 font-headline-md text-on-surface-variant">Capability</th>
                <th className="p-6 font-headline-md text-primary-fixed-dim bg-primary-fixed-dim/5">ven</th>
                <th className="p-6 font-headline-md text-on-surface-variant">npm / nvm</th>
                <th className="p-6 font-headline-md text-on-surface-variant">mise / asdf</th>
              </tr>
            </thead>
            <tbody className="font-body-base">
              {COMPARE_ROWS.map(([label, ven, npm, mise]) => (
                <tr key={label} className="border-b border-outline-variant/30">
                  <td className="p-6">{label}</td>
                  <td className="p-6 text-center text-secondary-fixed-dim">
                    {ven ? <Icon name="check_circle" fill /> : <Icon name="cancel" />}
                  </td>
                  <td className="p-6 text-center text-on-surface-variant">
                    {npm ? <Icon name="check_circle" fill /> : <Icon name="cancel" />}
                  </td>
                  <td className="p-6 text-center text-on-surface-variant">
                    {mise ? <Icon name="check_circle" fill /> : <Icon name="cancel" />}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  )
}

function NumbersSection() {
  return (
    <section className="py-24 border-y border-outline-variant/20">
      <div className="max-w-max-width mx-auto px-margin-desktop grid grid-cols-1 md:grid-cols-3 gap-12 text-center">
        <div>
          <div className="text-6xl font-bold text-primary-fixed-dim mb-4 tracking-tighter">&lt; 50ms</div>
          <div className="font-headline-md text-on-surface-variant uppercase text-sm tracking-widest">
            Switching Time
          </div>
        </div>
        <div>
          <div className="text-6xl font-bold text-secondary-fixed-dim mb-4 tracking-tighter">0</div>
          <div className="font-headline-md text-on-surface-variant uppercase text-sm tracking-widest">
            Unresolved Conflicts
          </div>
        </div>
        <div>
          <div className="text-6xl font-bold text-primary-fixed-dim mb-4 tracking-tighter">8</div>
          <div className="font-headline-md text-on-surface-variant uppercase text-sm tracking-widest">
            Core Languages
          </div>
        </div>
      </div>
    </section>
  )
}

function EnterpriseSection() {
  return (
    <section className="py-24 px-margin-desktop max-w-max-width mx-auto">
      <GlassCard className="p-12 flex flex-col md:flex-row gap-16 items-center">
        <div className="md:w-1/2">
          <h2 className="font-display-lg text-display-lg text-primary mb-6">
            Built for Restricted Environments
          </h2>
          <p className="font-body-base text-body-base text-on-surface-variant mb-8">
            Operating in a locked-down enterprise machine? ven doesn't require admin rights. It lives
            entirely in your user space — shimmed and isolated — so you stay compliant and unblocked.
          </p>
          <div className="flex flex-wrap gap-3">
            <div className="flex items-center gap-2 text-primary-fixed-dim bg-primary-fixed-dim/10 px-4 py-2 rounded-lg border border-primary-fixed-dim/20">
              <Icon name="verified_user" />
              <span className="text-sm font-bold">No Sudo Required</span>
            </div>
            <div className="flex items-center gap-2 text-primary-fixed-dim bg-primary-fixed-dim/10 px-4 py-2 rounded-lg border border-primary-fixed-dim/20">
              <Icon name="shield_with_heart" />
              <span className="text-sm font-bold">Portable Launcher</span>
            </div>
          </div>
        </div>
        <div className="md:w-1/2">
          <div className="rounded-xl border border-outline-variant/30 bg-surface-container-low p-8">
            <div className="font-mono text-sm space-y-2">
              <div className="text-on-surface-variant"># Drop ven-launcher.exe on a locked-down machine</div>
              <div>
                <span className="text-secondary-fixed-dim">$</span>{' '}
                <span className="text-on-surface">./ven-launcher.exe</span>
              </div>
              <div className="text-on-surface-variant"># Spawns a shell with Node 20 + Python 3.11 + deps</div>
              <div className="text-secondary-fixed-dim">✔ Environment ready: 34 packages</div>
            </div>
          </div>
        </div>
      </GlassCard>
    </section>
  )
}

function QuickStartSection() {
  return (
    <section className="py-24 bg-surface-container-high/20">
      <div className="max-w-max-width mx-auto px-margin-desktop">
        <h2 className="font-display-lg text-display-lg text-primary text-center mb-16">
          Three steps to mastery
        </h2>
        <div className="grid md:grid-cols-3 gap-12">
          {[
            {
              n: '01',
              title: 'Install',
              body: 'Run the one-liner. SHA256-verified, no admin required.',
              cmd: 'curl -fsSL https://get.ven.sh/install.sh | sh',
            },
            {
              n: '02',
              title: 'Init',
              body: 'Create a ven.toml in your project — runtimes, packages, env vars in one place.',
              cmd: 'ven init',
            },
            {
              n: '03',
              title: 'Code',
              body: 'cd into your project. ven applies the environment automatically. No activation scripts.',
              cmd: 'ven status',
            },
          ].map((s) => (
            <div key={s.n} className="flex flex-col">
              <div className="text-4xl font-bold text-primary-fixed-dim/30 mb-6">{s.n}</div>
              <h3 className="font-headline-md text-headline-md text-primary mb-4">{s.title}</h3>
              <p className="font-body-base text-on-surface-variant mb-6">{s.body}</p>
              <div className="bg-black p-4 rounded border border-outline-variant/30 font-mono text-xs">
                <span className="text-secondary-fixed-dim">$</span>{' '}
                <span className="text-on-surface">{s.cmd}</span>
              </div>
            </div>
          ))}
        </div>
        <div className="text-center mt-16">
          <Button to="/install" size="lg">
            Install ven now <Icon name="arrow_forward" />
          </Button>
        </div>
      </div>
    </section>
  )
}

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
