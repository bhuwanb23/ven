import { Link } from 'react-router-dom'
import Icon from '../../components/ui/Icon.jsx'
import Terminal from '../../components/ui/Terminal.jsx'
import GlassCard from '../../components/ui/GlassCard.jsx'
import Reveal from '../../components/effects/Reveal.jsx'
import { DOC_GROUPS, getDoc } from '../../content/docs.js'
import {
  INSTALL,
  GITHUB_URL,
  ISSUES_URL,
  DISCUSSIONS_URL,
} from '../../content/site.js'

// Map a sidebar group title to the icon we want next to its hub card. Falls
// back to a generic glyph if `docs.js` ever introduces a new group we haven't
// listed here yet.
const GROUP_ICON = {
  'Getting started': 'play_arrow',
  Commands: 'terminal',
  'Health & docs': 'shield_with_heart',
  Languages: 'language',
}

const PRINCIPLES = [
  {
    title: 'Predict, don\'t react',
    body: 'Every `ven add` builds the full dependency graph and replays peer + version + CVE constraints before touching the disk. The build never breaks because of something ven could have caught.',
  },
  {
    title: 'Per-terminal isolation',
    body: 'No global activation. Each shell session resolves its own environment, so Node 22 in one tab and Node 18 in another never collide. The shell hook is the only thing that knows about ven.',
  },
  {
    title: 'Single source of truth',
    body: '`ven.toml` declares the runtime, the packages, and the env in one file. `ven.lock` makes it reproducible with a canonical SHA-256 content hash + per-package integrity strings.',
  },
  {
    title: 'Offline-friendly',
    body: 'CVE (osv.dev) and EOL (endoflife.date) responses are SQLite-cached with stale-on-failure. You can develop on a plane and still get a meaningful health report.',
  },
]

export default function DocsHub() {
  return (
    <div className="max-w-4xl">
      <Reveal as="header" className="mb-12">
        <nav className="flex items-center gap-2 text-sm text-outline mb-4">
          <span>Docs</span>
        </nav>
        <h1 className="font-display-lg text-display-lg text-primary-container mb-6">
          Introduction
        </h1>
        <p className="font-body-base text-xl text-on-surface-variant leading-relaxed">
          ven is a multi-language version and dependency manager written in Rust. It manages eight runtimes
          with one CLI, simulates the dependency graph before installing, locks every byte with a canonical
          SHA-256 content hash, and runs without admin rights.
        </p>
      </Reveal>

      <Reveal as="section" className="mb-16">
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
      </Reveal>

      <Reveal as="section" className="mb-16">
        <h2 className="font-headline-md text-headline-md text-on-surface mb-6">
          Get started in seconds
        </h2>
        <Terminal title="bash" bodyClassName="space-y-2">
          <div className="flex gap-3">
            <span className="text-primary-container">$</span>
            <span className="text-on-surface">{INSTALL.linux.cmd}</span>
          </div>
          <div className="flex gap-3 text-outline">
            <span>#</span>
            <span>Detecting architecture and shell…</span>
          </div>
          <div className="flex gap-3 text-secondary-fixed-dim">
            <Icon name="check_circle" fill className="text-sm" />
            <span>ven installed in ~/.ven/bin</span>
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
      </Reveal>

      <Reveal as="section" className="mb-16">
        <h2 className="font-headline-md text-headline-md text-on-surface mb-6">Browse the docs</h2>
        <div className="grid md:grid-cols-3 gap-6">
          {DOC_GROUPS.map((group) => (
            <GlassCard key={group.title} tone="neutral" className="p-6">
              <div className="flex items-center gap-3 mb-4">
                <Icon
                  name={GROUP_ICON[group.title] ?? 'menu_book'}
                  className="text-primary-fixed-dim"
                />
                <h3 className="font-bold text-primary">{group.title}</h3>
              </div>
              <ul className="space-y-2 text-sm">
                {group.items.map((item) => {
                  // Pull the canonical summary from the doc itself when the
                  // sidebar label is just the command/language name.
                  const doc = getDoc(item.slug)
                  const sub =
                    group.title === 'Languages'
                      ? null
                      : doc?.summary?.split('.').shift()
                  return (
                    <li key={item.slug}>
                      <Link
                        to={`/docs/${item.slug}`}
                        className="group flex items-start gap-2 text-on-surface-variant hover:text-primary-fixed-dim transition-colors"
                      >
                        <Icon
                          name="arrow_forward"
                          className="text-xs opacity-50 group-hover:opacity-100 mt-1"
                        />
                        <span>
                          <span className="block">{item.label}</span>
                          {sub && (
                            <span className="block text-[11px] opacity-60 leading-snug">
                              {sub}.
                            </span>
                          )}
                        </span>
                      </Link>
                    </li>
                  )
                })}
              </ul>
            </GlassCard>
          ))}
        </div>
      </Reveal>

      <Reveal as="section" className="grid md:grid-cols-2 gap-8 mb-16">
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
                combines OSV + EOL + SHA-256 integrity hashes.
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
            Run into an issue? Open a GitHub issue or join the discussion — we triage every report.
          </p>
          <div className="flex flex-wrap gap-3">
            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noreferrer"
              className="bg-surface-container-highest px-3 py-1.5 rounded text-xs font-bold hover:bg-outline-variant transition-colors"
            >
              GitHub
            </a>
            <a
              href={ISSUES_URL}
              target="_blank"
              rel="noreferrer"
              className="bg-surface-container-highest px-3 py-1.5 rounded text-xs font-bold hover:bg-outline-variant transition-colors"
            >
              Issues
            </a>
            <a
              href={DISCUSSIONS_URL}
              target="_blank"
              rel="noreferrer"
              className="bg-surface-container-highest px-3 py-1.5 rounded text-xs font-bold hover:bg-outline-variant transition-colors"
            >
              Discussions
            </a>
          </div>
        </GlassCard>
      </Reveal>
    </div>
  )
}
