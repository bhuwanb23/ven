import { useMemo } from 'react'
import { Link, useParams } from 'react-router-dom'
import clsx from 'clsx'
import Icon from '../../components/ui/Icon.jsx'
import CodeBlock from '../../components/ui/CodeBlock.jsx'
import { getDoc } from '../../content/docs.js'
import NotFound from '../NotFound.jsx'

const CALLOUT_TONE = {
  info: {
    border: 'border-primary-fixed-dim/40',
    bg: 'bg-primary-fixed-dim/5',
    icon: 'info',
    iconColor: 'text-primary-fixed-dim',
  },
  success: {
    border: 'border-secondary-fixed-dim/40',
    bg: 'bg-secondary-fixed-dim/5',
    icon: 'check_circle',
    iconColor: 'text-secondary-fixed-dim',
  },
  warning: {
    border: 'border-error/40',
    bg: 'bg-error-container/10',
    icon: 'warning',
    iconColor: 'text-error',
  },
}

function slugify(text) {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, '')
    .trim()
    .replace(/\s+/g, '-')
}

function Section({ block, slug }) {
  switch (block.kind) {
    case 'p':
      return (
        <p className="text-on-surface-variant text-base leading-relaxed mb-4">
          {block.text}
        </p>
      )
    case 'h2':
      return (
        <h2
          id={slug}
          className="font-headline-md text-headline-md text-on-surface mt-12 mb-4 scroll-mt-24"
        >
          {block.text}
        </h2>
      )
    case 'h3':
      return (
        <h3 className="font-bold text-primary mt-8 mb-3 text-lg">
          {block.text}
        </h3>
      )
    case 'code':
      return (
        <CodeBlock
          code={block.code}
          language={block.lang ?? 'shell'}
          showPrompt={false}
          tone={block.lang === 'bash' || block.lang === 'shell' ? 'success' : 'cyan'}
          className="mb-6"
        />
      )
    case 'ul':
      return (
        <ul className="space-y-2 mb-6">
          {block.items.map((it, i) => (
            <li key={i} className="flex items-start gap-3 text-on-surface-variant">
              <Icon name="check" className="text-primary-fixed-dim mt-1 text-sm" />
              <span>{it}</span>
            </li>
          ))}
        </ul>
      )
    case 'table':
      return (
        <div className="overflow-x-auto mb-6">
          <table className="w-full border-collapse text-sm">
            <thead>
              <tr className="border-b border-outline-variant text-on-surface-variant uppercase text-[10px] tracking-widest">
                {block.head.map((h) => (
                  <th key={h} className="text-left py-2 px-3">
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-outline-variant/20">
              {block.rows.map((row, i) => (
                <tr key={i} className="hover:bg-surface-container-low/50">
                  {row.map((cell, j) => (
                    <td
                      key={j}
                      className={clsx(
                        'py-2 px-3',
                        j === 0 ? 'font-mono text-on-surface' : 'text-on-surface-variant'
                      )}
                    >
                      {cell}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )
    case 'callout': {
      const tone = CALLOUT_TONE[block.tone] ?? CALLOUT_TONE.info
      return (
        <div className={clsx('rounded-xl border p-4 mb-6 flex gap-3', tone.border, tone.bg)}>
          <Icon name={tone.icon} className={clsx('mt-0.5', tone.iconColor)} />
          <div>
            {block.title && (
              <div className="font-bold text-on-surface mb-1">{block.title}</div>
            )}
            <div className="text-sm text-on-surface-variant">{block.text}</div>
          </div>
        </div>
      )
    }
    default:
      return null
  }
}

function TOC({ headings }) {
  if (headings.length === 0) return null
  return (
    <aside className="hidden xl:block w-56 fixed right-[max(2rem,calc((100vw-1280px)/2))] top-32">
      <h4 className="text-[11px] font-bold text-outline uppercase tracking-widest mb-4">
        On this page
      </h4>
      <nav className="space-y-2 text-sm">
        {headings.map((h) => (
          <a
            key={h.slug}
            href={`#${h.slug}`}
            className="block text-on-surface-variant hover:text-primary-fixed-dim transition-colors"
          >
            {h.text}
          </a>
        ))}
      </nav>
    </aside>
  )
}

export default function DocPage() {
  const { slug } = useParams()
  const doc = getDoc(slug)

  const headings = useMemo(() => {
    if (!doc) return []
    return doc.sections
      .filter((s) => s.kind === 'h2')
      .map((s) => ({ text: s.text, slug: slugify(s.text) }))
  }, [doc])

  if (!doc) {
    return <NotFound title="Doc not found" sub={`No entry for "${slug}".`} />
  }

  return (
    <div className="grid xl:grid-cols-[1fr_14rem] gap-8 max-w-4xl">
      <article>
        <nav className="flex items-center gap-2 text-sm text-outline mb-4">
          <Link to="/docs" className="hover:text-primary-fixed-dim">Docs</Link>
          <Icon name="chevron_right" className="text-xs" />
          <span className="text-on-surface-variant">{doc.category}</span>
          <Icon name="chevron_right" className="text-xs" />
          <span className="text-on-surface">{doc.title}</span>
        </nav>

        <header className="mb-8">
          <h1 className="font-display-lg text-display-lg text-primary-container mb-4">
            {doc.title}
          </h1>
          <p className="font-body-base text-lg text-on-surface-variant leading-relaxed">
            {doc.summary}
          </p>
        </header>

        {doc.sections.map((block, i) => {
          const sectionSlug = block.kind === 'h2' ? slugify(block.text) : undefined
          return <Section key={i} block={block} slug={sectionSlug} />
        })}

        {doc.related && doc.related.length > 0 && (
          <section className="mt-16 pt-8 border-t border-outline-variant/20">
            <h3 className="text-[11px] uppercase tracking-widest text-outline mb-4">
              Related
            </h3>
            <div className="flex flex-wrap gap-3">
              {doc.related
                .map((s) => ({ slug: s, doc: getDoc(s) }))
                .filter((r) => r.doc)
                .map((r) => (
                  <Link
                    key={r.slug}
                    to={`/docs/${r.slug}`}
                    className="group flex items-center gap-2 px-3 py-2 rounded-lg border border-outline-variant/30 hover:border-primary-fixed-dim/50 transition-colors"
                  >
                    <span className="font-mono text-sm text-on-surface group-hover:text-primary-fixed-dim">
                      {r.doc.title}
                    </span>
                    <Icon name="arrow_forward" className="text-xs text-on-surface-variant" />
                  </Link>
                ))}
            </div>
          </section>
        )}
      </article>

      <TOC headings={headings} />
    </div>
  )
}
