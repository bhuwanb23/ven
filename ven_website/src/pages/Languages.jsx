import { useState } from 'react'
import { Link } from 'react-router-dom'
import clsx from 'clsx'
import Icon from '../components/ui/Icon.jsx'
import Badge from '../components/ui/Badge.jsx'
import Button from '../components/ui/Button.jsx'
import GlassCard from '../components/ui/GlassCard.jsx'
import Reveal from '../components/effects/Reveal.jsx'
import TiltCard from '../components/effects/TiltCard.jsx'
import { LANGUAGES, COMING_SOON, MOST_REQUESTED } from '../content/languages.js'
import { REQUEST_LANGUAGE_URL, GITHUB_URL } from '../content/site.js'

function HeroBar() {
  return (
    <section className="px-margin-mobile md:px-margin-desktop max-w-max-width mx-auto pt-16 pb-12 text-center">
      <h1 className="font-display-lg text-display-lg text-primary mb-4">
        8 Languages. One Tool. Zero Conflicts.
      </h1>
      <p className="text-on-surface-variant max-w-xl mx-auto mb-8">
        ven manages runtimes, packages, and environments for every language below — with the same commands.
      </p>
      <div className="flex flex-wrap justify-center gap-2">
        {LANGUAGES.map((l) => (
          <a
            key={l.slug}
            href={`#${l.slug}`}
            className="font-mono text-[12px] px-3 py-1 border border-outline-variant/40 rounded text-on-surface-variant hover:text-primary-fixed-dim hover:border-primary-fixed-dim/40 transition-colors"
          >
            {l.name}
          </a>
        ))}
      </div>
    </section>
  )
}

function LanguageCard({ lang, expanded, onToggle }) {
  return (
    <TiltCard
      id={lang.slug}
      max={5}
      className={clsx(
        'glass-card rounded-xl p-6 cursor-pointer transition-colors hover:border-primary-fixed-dim/40',
        expanded && 'border-primary-fixed-dim/60 ring-1 ring-primary-fixed-dim/30'
      )}
      onClick={onToggle}
      onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && onToggle()}
      role="button"
      tabIndex={0}
    >
      <div className="flex items-start justify-between mb-4">
        <div className="text-3xl font-bold text-primary-fixed-dim tracking-tighter">{lang.code}</div>
        <Badge tone="stable">● {lang.status}</Badge>
      </div>
      <h3 className="font-headline-md text-headline-md text-primary mb-1">{lang.name}</h3>
      <p className="font-mono text-[12px] text-on-surface-variant mb-4">
        {lang.versions.join(' · ')}
      </p>
      <dl className="text-sm space-y-1 mb-4">
        <div className="flex justify-between">
          <dt className="text-on-surface-variant">Package mgr</dt>
          <dd className="font-mono text-xs text-on-surface">{lang.pkgMgr}</dd>
        </div>
        <div className="flex justify-between">
          <dt className="text-on-surface-variant">Config</dt>
          <dd className="font-mono text-xs text-on-surface">{lang.config}</dd>
        </div>
      </dl>
      <div className="flex items-center justify-between text-primary-fixed-dim text-sm font-bold">
        <span>{expanded ? 'Hide details' : 'View details'}</span>
        <Icon name={expanded ? 'expand_less' : 'expand_more'} />
      </div>
    </TiltCard>
  )
}

function DetailPanel({ lang }) {
  return (
    <GlassCard tone="neutral" className="p-8 mt-8">
      <div className="flex items-start justify-between flex-wrap gap-4 mb-6">
        <div>
          <h2 className="font-display-lg text-3xl text-primary mb-2">{lang.name}</h2>
          <p className="text-on-surface-variant max-w-2xl">{lang.tagline}</p>
        </div>
        <Button to={`/docs/${lang.slug}`} variant="ghost" size="md">
          Read {lang.name} docs <Icon name="arrow_forward" />
        </Button>
      </div>

      <div className="grid md:grid-cols-2 gap-8">
        <DetailBlock label="Install" lines={lang.install} prompt="$" />
        <DetailBlock
          label="ven.toml"
          lines={lang.venToml.split('\n')}
          prompt=""
          mono
        />
        <DetailBlock
          label="What ven sets"
          lines={lang.env.map(([k, v]) => `${k}  →  ${v}`)}
          prompt=""
          mono
        />
        <DetailBlock
          label="Package operations"
          lines={lang.packageOps.map(([from, to]) => `${from.padEnd(22)} → ${to}`)}
          prompt=""
          mono
        />
      </div>

      <div className="mt-6 flex flex-wrap items-center gap-4 text-sm text-on-surface-variant">
        <div>
          <span className="text-outline mr-2">Includes:</span>
          <span className="font-mono text-on-surface">{lang.includes.join('  ')}</span>
        </div>
        <div>
          <span className="text-outline mr-2">Downloads from:</span>
          <span className="font-mono text-on-surface">{lang.downloads}</span>
        </div>
      </div>
    </GlassCard>
  )
}

function DetailBlock({ label, lines, prompt, mono }) {
  return (
    <div>
      <h4 className="text-[11px] uppercase tracking-widest text-outline mb-3">{label}</h4>
      <div className="bg-surface-container-lowest border border-outline-variant/30 rounded-lg p-4 font-mono text-sm space-y-1">
        {lines.map((l, i) => (
          <div key={i} className="whitespace-pre">
            {prompt && (
              <span className="text-secondary-fixed-dim mr-2 select-none">{prompt}</span>
            )}
            <span className={clsx(mono ? 'text-on-surface-variant' : 'text-on-surface')}>{l}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

function ComingSoonSection() {
  return (
    <Reveal as="section" className="px-margin-mobile md:px-margin-desktop max-w-max-width mx-auto py-16">
      <h2 className="font-headline-md text-headline-md text-primary mb-8">Coming soon</h2>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {COMING_SOON.map((c) => (
          <div
            key={c.name}
            className="glass-surface rounded-xl p-6 text-center border border-outline-variant/30 hover:border-primary-fixed-dim/40 transition-colors"
          >
            <div className="font-bold text-primary mb-2">{c.name}</div>
            <div className="font-mono text-[11px] text-on-surface-variant mb-3">{c.pkgMgr}</div>
            <Badge tone="planned">Planned</Badge>
          </div>
        ))}
      </div>
    </Reveal>
  )
}

function RequestSection() {
  return (
    <Reveal as="section" className="px-margin-mobile md:px-margin-desktop max-w-max-width mx-auto py-16">
      <GlassCard tone="neutral" className="p-8">
        <h2 className="font-headline-md text-headline-md text-primary mb-2">
          Don't see your language?
        </h2>
        <p className="text-on-surface-variant mb-6 max-w-2xl">
          We add languages based on community demand. Vote on existing requests or open a new issue.
        </p>
        <div className="flex flex-wrap gap-3 mb-8">
          <Button href={REQUEST_LANGUAGE_URL}>
            Request a language <Icon name="open_in_new" />
          </Button>
          <Button variant="ghost" href={`${GITHUB_URL}#contributing`}>
            Plugin system guide <Icon name="arrow_forward" />
          </Button>
        </div>

        <h3 className="text-[11px] uppercase tracking-widest text-outline mb-4">
          Most-requested
        </h3>
        <div className="space-y-3 font-mono text-sm">
          {MOST_REQUESTED.map((r) => (
            <div key={r.name} className="flex items-center gap-4 group">
              <span className="w-24 text-on-surface">{r.name}</span>
              <div className="flex-1 h-2 bg-surface-container-high rounded-full overflow-hidden">
                <div
                  className="h-full bg-primary-fixed-dim transition-all duration-700 group-hover:bg-secondary-fixed-dim"
                  style={{ width: `${(r.votes / r.max) * 100}%` }}
                />
              </div>
              <span className="w-20 text-right text-on-surface-variant">{r.votes} votes</span>
            </div>
          ))}
        </div>
      </GlassCard>
    </Reveal>
  )
}

function ComparisonTable() {
  return (
    <Reveal as="section" className="px-margin-mobile md:px-margin-desktop max-w-max-width mx-auto py-16">
      <h2 className="font-headline-md text-headline-md text-primary mb-8">Quick reference</h2>
      <div className="overflow-x-auto">
        <table className="w-full border-collapse font-mono text-sm">
          <thead>
            <tr className="border-b border-outline-variant text-on-surface-variant uppercase text-[10px] tracking-widest">
              <th className="text-left py-3 px-2">Language</th>
              <th className="text-left py-3 px-2">Package manager</th>
              <th className="text-left py-3 px-2">Isolation</th>
              <th className="text-left py-3 px-2">Config</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-outline-variant/20">
            {LANGUAGES.map((l) => (
              <tr
                key={l.slug}
                className="hover:bg-surface-container-low/50 transition-colors shimmer-row"
              >
                <td className="py-3 px-2 font-bold text-primary-fixed-dim">
                  <Link to={`/docs/${l.slug}`}>{l.name}</Link>
                </td>
                <td className="py-3 px-2 text-on-surface-variant">{l.pkgMgr}</td>
                <td className="py-3 px-2 text-on-surface-variant">
                  {isolationFor(l.slug)}
                </td>
                <td className="py-3 px-2 text-on-surface-variant">{l.config}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Reveal>
  )
}

function isolationFor(slug) {
  switch (slug) {
    case 'node':
    case 'bun':
      return 'node_modules/'
    case 'python':
      return '.venv/lib/'
    case 'go':
      return 'GOPATH cache'
    case 'rust':
      return 'target/'
    case 'java':
      return '~/.m2/'
    case 'ruby':
      return 'GEM_HOME'
    case 'deno':
      return '~/.cache/deno/'
    default:
      return '—'
  }
}

export default function Languages() {
  const [openSlug, setOpenSlug] = useState(null)
  const open = LANGUAGES.find((l) => l.slug === openSlug)
  return (
    <>
      <HeroBar />
      <Reveal as="section" className="px-margin-mobile md:px-margin-desktop max-w-max-width mx-auto pb-12">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {LANGUAGES.map((l) => (
            <LanguageCard
              key={l.slug}
              lang={l}
              expanded={openSlug === l.slug}
              onToggle={() =>
                setOpenSlug((cur) => (cur === l.slug ? null : l.slug))
              }
            />
          ))}
        </div>
        {open && <DetailPanel lang={open} />}
      </Reveal>
      <ComparisonTable />
      <ComingSoonSection />
      <RequestSection />
    </>
  )
}
