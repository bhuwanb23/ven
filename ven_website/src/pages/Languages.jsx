import { Fragment, useEffect, useRef, useState } from 'react'
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

/**
 * Tracks the current Tailwind breakpoint column count for the languages grid
 * (`grid-cols-1 sm:grid-cols-2 lg:grid-cols-4`) so we can position the
 * expand-in-place detail panel right after the *row* that contains the
 * clicked card. The breakpoints match the Tailwind defaults the markup uses.
 */
function useGridColumns() {
  const [cols, setCols] = useState(() => {
    if (typeof window === 'undefined') return 4
    if (window.matchMedia('(min-width: 1024px)').matches) return 4
    if (window.matchMedia('(min-width: 640px)').matches) return 2
    return 1
  })
  useEffect(() => {
    if (typeof window === 'undefined') return undefined
    const sm = window.matchMedia('(min-width: 640px)')
    const lg = window.matchMedia('(min-width: 1024px)')
    const update = () => {
      setCols(lg.matches ? 4 : sm.matches ? 2 : 1)
    }
    sm.addEventListener('change', update)
    lg.addEventListener('change', update)
    return () => {
      sm.removeEventListener('change', update)
      lg.removeEventListener('change', update)
    }
  }, [])
  return cols
}

function HeroBar() {
  // Stats are derived once from the canonical content file so they stay
  // accurate when LANGUAGES grows. Counting unique package managers gives a
  // more interesting figure than just `LANGUAGES.length` repeated twice.
  const totalLanguages = LANGUAGES.length
  const uniqueManagers = new Set(LANGUAGES.flatMap((l) => l.pkgMgr.split(/[\s+/]+/))).size
  const totalVersions = LANGUAGES.reduce((sum, l) => sum + l.versions.length, 0)

  return (
    <section className="relative overflow-hidden border-b border-outline-variant/20">
      {/* Decorative dot grid, ultra-low opacity — gives the hero panel some
          texture without competing with the chip wall on the right. */}
      <div
        className="absolute inset-0 z-0 opacity-[0.04] pointer-events-none"
        style={{
          backgroundImage: 'radial-gradient(circle, #00dbe7 1px, transparent 1px)',
          backgroundSize: '32px 32px',
        }}
      />

      <div className="relative z-10 px-margin-mobile md:px-margin-desktop max-w-max-width mx-auto pt-20 pb-16 grid lg:grid-cols-[1.1fr_1fr] gap-12 lg:gap-16 items-center">
        {/* Left column: tagline + stats + CTAs. */}
        <div>
          <div className="inline-flex items-center gap-2 font-mono text-[11px] uppercase tracking-widest text-primary-fixed-dim/80 mb-6 px-3 py-1 border border-primary-fixed-dim/30 rounded-full bg-primary-fixed-dim/5">
            <span className="inline-block w-1.5 h-1.5 rounded-full bg-primary-fixed-dim animate-pulse" />
            Multi-runtime · Zero conflicts
          </div>

          <h1 className="font-display-lg text-display-lg text-primary mb-5 leading-[1.05]">
            One CLI that speaks{' '}
            <span className="text-primary-fixed-dim">eight</span> languages
          </h1>
          <p className="text-on-surface-variant text-lg max-w-xl mb-8 leading-relaxed">
            ven manages runtimes, packages, and environments for every language below — with the same{' '}
            <code className="text-primary-fixed-dim font-mono">init / add / lock / sync</code> commands and
            one <code className="text-primary-fixed-dim font-mono">ven.toml</code> manifest.
          </p>

          {/* Stats row — three counters, tabular numerals so they snap to a
              uniform width regardless of digit count. */}
          <div className="grid grid-cols-3 gap-4 mb-8 max-w-md">
            <Stat n={totalLanguages} label="Runtimes" tone="primary" />
            <Stat n={totalVersions} label="Pinned versions" tone="secondary" />
            <Stat n={uniqueManagers} label="Package managers" tone="primary" />
          </div>

          <div className="flex flex-wrap gap-3">
            <Button to="/install" size="md">
              Install ven <Icon name="arrow_forward" />
            </Button>
            <Button to="/playground" variant="ghost" size="md">
              Try in playground <Icon name="play_arrow" />
            </Button>
          </div>
        </div>

        {/* Right column: a "language constellation" wall — anchor chips
            scrollspy-style. Visually balances the headline and gives a quick
            jump-to for each language card below. */}
        <div className="relative">
          <div
            className="absolute -inset-8 pointer-events-none blur-2xl"
            style={{
              background:
                'radial-gradient(circle at 50% 50%, rgba(0,219,231,0.12), transparent 70%)',
            }}
          />
          <div className="relative glass-card rounded-2xl p-6 border-primary-fixed-dim/30">
            <div className="flex items-center justify-between mb-5">
              <span className="font-mono text-[11px] uppercase tracking-widest text-on-surface-variant">
                Jump to language
              </span>
              <span className="font-mono text-[10px] uppercase tracking-widest text-secondary-fixed-dim flex items-center gap-1.5">
                <span className="inline-block w-1.5 h-1.5 rounded-full bg-secondary-fixed-dim" />
                all stable
              </span>
            </div>
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
              {LANGUAGES.map((l) => (
                <a
                  key={l.slug}
                  href={`#${l.slug}`}
                  className="group flex items-center gap-2.5 px-3 py-2 rounded-lg bg-surface-container-low border border-outline-variant/30 hover:border-primary-fixed-dim/60 hover:bg-primary-fixed-dim/5 transition-all duration-200"
                >
                  <span className="w-7 h-7 rounded-md bg-surface-container-high text-on-surface-variant group-hover:text-primary-fixed-dim group-hover:bg-primary-fixed-dim/10 flex items-center justify-center font-bold text-[12px] tracking-tighter transition-colors">
                    {l.code}
                  </span>
                  <div className="flex flex-col min-w-0">
                    <span className="text-sm font-bold text-on-surface truncate">{l.name}</span>
                    <span className="font-mono text-[10px] text-outline truncate">
                      {l.versions[l.versions.length - 1]}
                    </span>
                  </div>
                </a>
              ))}
            </div>
            <div className="mt-5 pt-4 border-t border-outline-variant/20 flex items-center justify-between font-mono text-[11px] text-on-surface-variant">
              <span>
                <span className="text-primary-fixed-dim">$</span> ven install &lt;lang&gt; &lt;ver&gt;
              </span>
              <span className="opacity-50">SHA-256 verified</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}

// Small stat sub-component for the hero. Kept local to this file because it's
// only used here and the styling is bespoke to the hero panel.
function Stat({ n, label, tone }) {
  const color = tone === 'primary' ? 'text-primary-fixed-dim' : 'text-secondary-fixed-dim'
  return (
    <div className="border-l-2 border-outline-variant/40 pl-3">
      <div className={clsx('text-2xl font-bold tabular-nums tracking-tighter', color)}>
        {n}
      </div>
      <div className="text-[10px] uppercase tracking-widest text-on-surface-variant mt-0.5">
        {label}
      </div>
    </div>
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

/**
 * Accordion-style detail panel. Rendered inline within the language grid
 * (slotted right after the row containing the active card via `col-span`),
 * so the connection between which card was clicked and where the details
 * appear is always obvious.
 *
 * - Pointer triangle on the top edge visually anchors the panel to its row.
 * - Close button gives an explicit dismissal affordance (the card itself is
 *   also still clickable to toggle).
 * - `key={lang.slug}` on the host fragment in the parent makes this panel
 *   remount on language change, re-firing the reveal entrance animation.
 */
function DetailPanel({ lang, onClose, arrowOffsetPct }) {
  return (
    <div className="relative reveal-init reveal-in">
      {/* Pointer triangle on the top edge, positioned over the active card's
          column. Pure-CSS — uses two stacked borders to draw an outlined
          triangle that matches the panel's border + background. */}
      <div
        className="absolute -top-2 z-10 pointer-events-none"
        style={{ left: `calc(${arrowOffsetPct}% - 8px)` }}
      >
        <div className="w-4 h-4 rotate-45 bg-surface-container-low border-t border-l border-primary-fixed-dim/40" />
      </div>

      <GlassCard tone="neutral" className="relative p-8 border-primary-fixed-dim/40 cyan-glow">
        {/* Top accent strip — fades from cyan into transparent so the eye
            reads the connection between the card row above and the panel. */}
        <div className="absolute inset-x-0 top-0 h-[2px] bg-gradient-to-r from-transparent via-primary-fixed-dim/70 to-transparent rounded-t-2xl" />

        <button
          type="button"
          onClick={onClose}
          aria-label="Close details"
          className="absolute top-4 right-4 w-8 h-8 rounded-full flex items-center justify-center text-on-surface-variant hover:text-primary-fixed-dim hover:bg-surface-container-high transition-colors"
        >
          <Icon name="close" />
        </button>

        <div className="flex items-start justify-between flex-wrap gap-4 mb-6 pr-10">
          <div>
            <div className="flex items-center gap-3 mb-2">
              <span className="font-mono text-[10px] uppercase tracking-widest text-primary-fixed-dim/80 px-2 py-0.5 border border-primary-fixed-dim/30 rounded-full bg-primary-fixed-dim/5">
                {lang.code}
              </span>
              <h2 className="font-display-lg text-3xl text-primary">{lang.name}</h2>
            </div>
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

        <div className="mt-6 pt-6 border-t border-outline-variant/20 flex flex-wrap items-center gap-x-8 gap-y-3 text-sm text-on-surface-variant">
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
    </div>
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
  const cols = useGridColumns()
  const panelRef = useRef(null)

  const openIdx = openSlug ? LANGUAGES.findIndex((l) => l.slug === openSlug) : -1
  const open = openIdx >= 0 ? LANGUAGES[openIdx] : null

  // Drop the panel after the last visible-row tail so it always sits
  // immediately under the row containing the clicked card. Clamped to the
  // last card index so opening the final card still renders correctly.
  const insertAfterIdx =
    openIdx >= 0
      ? Math.min(
          LANGUAGES.length - 1,
          Math.floor(openIdx / cols) * cols + cols - 1
        )
      : -1

  // Horizontal position of the pointer triangle, expressed as a % of the
  // panel's width so it lines up with the column the active card lives in.
  // For `cols = 1` the triangle is centered.
  const columnInRow = openIdx >= 0 ? openIdx % cols : 0
  const arrowOffsetPct =
    cols <= 1 ? 50 : ((columnInRow + 0.5) / cols) * 100

  // Smooth-scroll the freshly-opened panel into view so users on small
  // screens don't miss it when it slots in below.
  useEffect(() => {
    if (openSlug && panelRef.current) {
      panelRef.current.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
    }
  }, [openSlug])

  return (
    <>
      <HeroBar />
      <Reveal as="section" className="px-margin-mobile md:px-margin-desktop max-w-max-width mx-auto pb-12">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {LANGUAGES.map((l, i) => (
            <Fragment key={l.slug}>
              <LanguageCard
                lang={l}
                expanded={openSlug === l.slug}
                onToggle={() =>
                  setOpenSlug((cur) => (cur === l.slug ? null : l.slug))
                }
              />
              {i === insertAfterIdx && open && (
                <div
                  ref={panelRef}
                  className="col-span-1 sm:col-span-2 lg:col-span-4 scroll-mt-24"
                >
                  <DetailPanel
                    // Remount on slug change so the entrance animation
                    // re-fires when the user picks a different language.
                    key={open.slug}
                    lang={open}
                    onClose={() => setOpenSlug(null)}
                    arrowOffsetPct={arrowOffsetPct}
                  />
                </div>
              )}
            </Fragment>
          ))}
        </div>
      </Reveal>
      <ComparisonTable />
      <ComingSoonSection />
      <RequestSection />
    </>
  )
}
