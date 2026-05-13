import { Link } from 'react-router-dom'
import Icon from '../../components/ui/Icon.jsx'
import Terminal from '../../components/ui/Terminal.jsx'
import GlassCard from '../../components/ui/GlassCard.jsx'

const CATEGORIES = [
  {
    title: 'Getting started',
    icon: 'play_arrow',
    links: [
      { slug: 'init', label: 'ven init — interactive project setup' },
      { slug: 'install', label: 'ven install — runtime installation' },
      { slug: 'status', label: 'ven status — observability' },
    ],
  },
  {
    title: 'Commands',
    icon: 'terminal',
    links: [
      { slug: 'add', label: 'ven add — install packages (graph-checked)' },
      { slug: 'lock', label: 'ven lock — write ven.lock' },
      { slug: 'sync', label: 'ven sync — reproduce + drift check' },
      { slug: 'check', label: 'ven check — CVE + EOL health report' },
      { slug: 'docs', label: 'ven docs — version-pinned package docs' },
    ],
  },
  {
    title: 'Languages',
    icon: 'language',
    links: [
      { slug: 'node', label: 'Node.js' },
      { slug: 'python', label: 'Python' },
      { slug: 'go', label: 'Go' },
      { slug: 'rust', label: 'Rust' },
      { slug: 'java', label: 'Java' },
      { slug: 'ruby', label: 'Ruby' },
      { slug: 'deno', label: 'Deno' },
      { slug: 'bun', label: 'Bun' },
    ],
  },
]

const PRINCIPLES = [
  {
    title: 'Predict, don\'t react',
    body: "Every install begins with a dependency-graph simulation. ven shows you what will break before it touches the disk.",
  },
  {
    title: 'Per-terminal isolation',
    body: 'No global activation. Each shell session gets its own resolved environment, so Node 20 in one tab and Node 22 in another never collide.',
  },
  {
    title: 'Single source of truth',
    body: 'ven.toml declares the runtime, packages, and env in one file. ven.lock makes it reproducible with SRI integrity hashes.',
  },
  {
    title: 'Offline-friendly',
    body: 'CVE and EOL lookups are SQLite-cached with stale-on-failure. You can develop on a plane and still get a meaningful health report.',
  },
]

export default function DocsHub() {
  return (
    <div className="max-w-4xl">
      <header className="mb-12">
        <nav className="flex items-center gap-2 text-sm text-outline mb-4">
          <span>Docs</span>
        </nav>
        <h1 className="font-display-lg text-display-lg text-primary-container mb-6">
          Introduction
        </h1>
        <p className="font-body-base text-xl text-on-surface-variant leading-relaxed">
          ven is a high-velocity, multi-language version and dependency manager. It manages eight runtimes
          with one CLI, predicts conflicts before installing, locks every byte with SRI hashes, and runs
          without admin rights.
        </p>
      </header>

      <section className="mb-16">
        <h2 className="font-headline-md text-headline-md text-on-surface mb-6 flex items-center gap-3">
          <Icon name="bolt" className="text-primary-fixed-dim" />
          Design principles
        </h2>
        <div className="grid md:grid-cols-2 gap-6">
          {PRINCIPLES.map((p) => (
            <GlassCard key={p.title} tone="neutral" className="p-6 terminal-glow">
              <h3 className="font-bold text-primary mb-2">{p.title}</h3>
              <p className="text-sm text-on-surface-variant">{p.body}</p>
            </GlassCard>
          ))}
        </div>
      </section>

      <section className="mb-16">
        <h2 className="font-headline-md text-headline-md text-on-surface mb-6">
          Get started in seconds
        </h2>
        <Terminal title="bash" bodyClassName="space-y-2">
          <div className="flex gap-3">
            <span className="text-primary-container">$</span>
            <span className="text-on-surface">curl -fsSL https://get.ven.sh/install.sh | sh</span>
          </div>
          <div className="flex gap-3 text-outline">
            <span>#</span>
            <span>Detecting architecture and shell…</span>
          </div>
          <div className="flex gap-3 text-secondary-fixed-dim">
            <Icon name="check_circle" fill className="text-sm" />
            <span>ven 1.0.0 installed in ~/.ven/bin</span>
          </div>
          <div className="flex gap-3 mt-4">
            <span className="text-primary-container">$</span>
            <span className="text-on-surface">ven init</span>
          </div>
          <div className="flex gap-3 text-outline">
            <span>#</span>
            <span>Interactive runtime + package selection</span>
          </div>
        </Terminal>
      </section>

      <section className="mb-16">
        <h2 className="font-headline-md text-headline-md text-on-surface mb-6">Browse the docs</h2>
        <div className="grid md:grid-cols-3 gap-6">
          {CATEGORIES.map((c) => (
            <GlassCard key={c.title} tone="neutral" className="p-6">
              <div className="flex items-center gap-3 mb-4">
                <Icon name={c.icon} className="text-primary-fixed-dim" />
                <h3 className="font-bold text-primary">{c.title}</h3>
              </div>
              <ul className="space-y-2 text-sm">
                {c.links.map((l) => (
                  <li key={l.slug}>
                    <Link
                      to={`/docs/${l.slug}`}
                      className="text-on-surface-variant hover:text-primary-fixed-dim transition-colors flex items-center gap-2"
                    >
                      <Icon name="arrow_forward" className="text-xs opacity-50" />
                      {l.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </GlassCard>
          ))}
        </div>
      </section>

      <section className="grid md:grid-cols-2 gap-8 mb-16">
        <div>
          <h3 className="font-headline-md text-lg text-primary mb-4">What's next?</h3>
          <ul className="space-y-4">
            <li className="flex items-start gap-3">
              <Icon name="arrow_forward" className="text-primary-fixed-dim mt-1" />
              <span className="text-on-surface-variant">
                <Link to="/docs/init" className="text-primary underline-offset-4 hover:underline">
                  Read the quick-start
                </Link>{' '}
                to see how a new project gets bootstrapped end-to-end.
              </span>
            </li>
            <li className="flex items-start gap-3">
              <Icon name="arrow_forward" className="text-primary-fixed-dim mt-1" />
              <span className="text-on-surface-variant">
                Learn how{' '}
                <Link to="/docs/check" className="text-primary underline-offset-4 hover:underline">
                  the security model
                </Link>{' '}
                combines OSV + EOL + SRI hashes.
              </span>
            </li>
            <li className="flex items-start gap-3">
              <Icon name="arrow_forward" className="text-primary-fixed-dim mt-1" />
              <span className="text-on-surface-variant">
                Pick your language in the{' '}
                <Link to="/languages" className="text-primary underline-offset-4 hover:underline">
                  languages directory
                </Link>{' '}
                for runtime-specific details.
              </span>
            </li>
          </ul>
        </div>
        <GlassCard tone="neutral" className="p-6">
          <h3 className="font-bold text-on-surface mb-2">Community support</h3>
          <p className="text-sm text-on-surface-variant mb-4">
            Run into an issue? Open a GitHub discussion or drop into Discord — we're a small team and we
            triage every issue.
          </p>
          <div className="flex gap-3">
            <a
              href="https://github.com/yourorg/ven/issues"
              target="_blank"
              rel="noreferrer"
              className="bg-surface-container-highest px-3 py-1.5 rounded text-xs font-bold hover:bg-outline-variant transition-colors"
            >
              GitHub
            </a>
            <a
              href="https://github.com/yourorg/ven/discussions"
              target="_blank"
              rel="noreferrer"
              className="bg-surface-container-highest px-3 py-1.5 rounded text-xs font-bold hover:bg-outline-variant transition-colors"
            >
              Discussions
            </a>
          </div>
        </GlassCard>
      </section>
    </div>
  )
}
