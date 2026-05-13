import { useMemo, useState } from 'react'
import clsx from 'clsx'
import Icon from '../components/ui/Icon.jsx'
import Badge from '../components/ui/Badge.jsx'
import { RELEASES, TAG_META } from '../content/releases.js'
import { GITHUB_URL, RELEASES_URL } from '../content/site.js'

// Filter chips. We only render the ones that actually match >= 1 release
// (the 'all' chip is always shown). Order is intentional — major / minor
// are common, patch / security are rare, so they sit at the right end.
const FILTERS = [
  { id: 'all', label: 'All' },
  { id: 'major', label: 'Major' },
  { id: 'minor', label: 'Minor' },
  { id: 'patch', label: 'Patch' },
  { id: 'security', label: 'Security' },
]

function ReleaseCard({ release, defaultOpen }) {
  const [open, setOpen] = useState(defaultOpen)
  const meta = TAG_META[release.tag] ?? TAG_META.minor
  const totalChanges =
    release.sections.new.length +
    release.sections.fixed.length +
    release.sections.improved.length

  return (
    <article className="glass-card rounded-xl overflow-hidden">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="w-full flex items-center justify-between gap-4 p-6 text-left hover:bg-surface-container-low/40 transition-colors"
      >
        <div className="flex items-center gap-4 flex-wrap">
          <span className={clsx('w-2.5 h-2.5 rounded-full', meta.dot)} />
          <span className="font-mono text-xl font-bold text-primary">{release.version}</span>
          <span className="font-mono text-xs text-on-surface-variant">·  {release.date}</span>
          <Badge tone={meta.tone}>{meta.label}</Badge>
        </div>
        <div className="flex items-center gap-3 text-on-surface-variant text-sm">
          <span className="hidden md:inline font-mono text-xs">{totalChanges} changes</span>
          <Icon name={open ? 'expand_less' : 'expand_more'} />
        </div>
      </button>
      {open && (
        <div className="border-t border-outline-variant/30 p-6 space-y-6 bg-surface-container-low/30">
          <p className="text-on-surface-variant">{release.summary}</p>

          {release.sections.new.length > 0 && (
            <Group label="New" tone="text-secondary-fixed-dim" items={release.sections.new} />
          )}
          {release.sections.fixed.length > 0 && (
            <Group label="Fixed" tone="text-primary-fixed-dim" items={release.sections.fixed} />
          )}
          {release.sections.improved.length > 0 && (
            <Group label="Improved" tone="text-tertiary-fixed-dim" items={release.sections.improved} />
          )}

          <div className="flex flex-wrap gap-3 pt-2">
            <a
              href={`${GITHUB_URL}/releases/tag/${release.version}`}
              target="_blank"
              rel="noreferrer"
              className="flex items-center gap-2 font-mono text-xs px-3 py-2 border border-primary-fixed-dim/40 text-primary-fixed-dim rounded hover:bg-primary-fixed-dim/10 transition-colors"
            >
              <Icon name="download" className="text-sm" /> Download {release.version}
            </a>
            <a
              href={`${GITHUB_URL}/compare/${release.version}`}
              target="_blank"
              rel="noreferrer"
              className="flex items-center gap-2 font-mono text-xs px-3 py-2 border border-outline-variant/40 text-on-surface-variant rounded hover:bg-surface-container-high transition-colors"
            >
              <Icon name="diff" className="text-sm" /> Full diff
            </a>
          </div>
        </div>
      )}
    </article>
  )
}

function Group({ label, tone, items }) {
  return (
    <section>
      <h3 className={clsx('font-mono text-xs uppercase tracking-widest mb-3', tone)}>
        {label}
      </h3>
      <ul className="space-y-1.5">
        {items.map((it, i) => (
          <li key={i} className="flex items-start gap-3 text-sm text-on-surface-variant">
            <span className="text-outline mt-1.5">•</span>
            <span>{it}</span>
          </li>
        ))}
      </ul>
    </section>
  )
}

export default function Changelog() {
  const [filter, setFilter] = useState('all')

  // Build the per-tag counts once and use them to (a) hide chips that don't
  // match any release, and (b) annotate the visible ones with `· N`. The
  // previous version always rendered Patch + Security even when both were
  // empty, which only ever showed the "No releases match this filter" state.
  const counts = useMemo(() => {
    const c = { all: RELEASES.length, major: 0, minor: 0, patch: 0, security: 0 }
    for (const r of RELEASES) {
      if (c[r.tag] != null) c[r.tag] += 1
    }
    return c
  }, [])

  const visibleFilters = useMemo(
    () => FILTERS.filter((f) => f.id === 'all' || counts[f.id] > 0),
    [counts]
  )

  const filtered = useMemo(
    () => RELEASES.filter((r) => filter === 'all' || r.tag === filter),
    [filter]
  )

  const latest = RELEASES[0]

  return (
    <div className="max-w-3xl mx-auto px-margin-mobile md:px-0 py-16">
      <header className="mb-12">
        <h1 className="font-display-lg text-display-lg text-primary mb-3">Changelog</h1>
        <p className="text-on-surface-variant mb-6">
          Every change to ven, documented. Latest release:{' '}
          <span className="text-primary-fixed-dim font-mono">{latest.version}</span> —{' '}
          {latest.date}
        </p>
        <div className="flex flex-wrap items-center gap-2 mb-6">
          {visibleFilters.map((f) => (
            <button
              key={f.id}
              type="button"
              onClick={() => setFilter(f.id)}
              className={clsx(
                'px-3 py-1 rounded-full font-mono text-xs uppercase tracking-widest border transition-colors',
                filter === f.id
                  ? 'border-primary-fixed-dim text-primary-fixed-dim bg-primary-fixed-dim/10'
                  : 'border-outline-variant/40 text-on-surface-variant hover:text-on-surface'
              )}
            >
              {f.label}
              <span className="opacity-60 ml-1.5">· {counts[f.id]}</span>
            </button>
          ))}
        </div>
        <div className="flex flex-wrap gap-3 text-sm">
          <a
            href={RELEASES_URL}
            target="_blank"
            rel="noreferrer"
            className="text-primary-fixed-dim hover:underline underline-offset-4"
          >
            All releases on GitHub →
          </a>
          <a
            href={`${RELEASES_URL}.atom`}
            target="_blank"
            rel="noreferrer"
            className="text-on-surface-variant hover:underline underline-offset-4"
          >
            RSS feed →
          </a>
        </div>
      </header>

      <div className="relative">
        <div className="absolute left-2.5 top-2 bottom-2 w-px bg-outline-variant/30 hidden md:block" />
        <div className="space-y-6 md:pl-10">
          {filtered.length === 0 ? (
            <p className="text-on-surface-variant text-center py-12">
              No releases match this filter.
            </p>
          ) : (
            filtered.map((r, i) => (
              <ReleaseCard key={r.version} release={r} defaultOpen={i === 0} />
            ))
          )}
        </div>
      </div>

      <footer className="mt-16 pt-8 border-t border-outline-variant/20 text-sm text-on-surface-variant space-y-2">
        <p>
          <span className="text-outline">Versioning:</span> ven follows{' '}
          <a
            href="https://semver.org"
            target="_blank"
            rel="noreferrer"
            className="text-primary-fixed-dim hover:underline"
          >
            semantic versioning
          </a>
          .
        </p>
        <ul className="font-mono text-xs space-y-1">
          <li>MAJOR → breaking changes</li>
          <li>MINOR → new features, backward compatible</li>
          <li>PATCH → bug fixes only</li>
        </ul>
      </footer>
    </div>
  )
}
