import { Link } from 'react-router-dom'
import {
  GITHUB_URL,
  ISSUES_URL,
  DISCUSSIONS_URL,
  LICENSE_URL,
  CONTACT_EMAIL,
} from '../../content/site.js'

const COLUMNS = [
  {
    title: 'Product',
    links: [
      { to: '/install', label: 'Install' },
      { to: '/languages', label: 'Languages' },
      { to: '/playground', label: 'Playground' },
    ],
  },
  {
    title: 'Docs',
    links: [
      { to: '/docs', label: 'Documentation' },
      { to: '/docs/init', label: 'Quick start' },
      { to: '/docs/check', label: 'Security model' },
      { to: '/changelog', label: 'Changelog' },
    ],
  },
  {
    title: 'Community',
    links: [
      { href: GITHUB_URL, label: 'GitHub' },
      { href: ISSUES_URL, label: 'Issues' },
      { href: DISCUSSIONS_URL, label: 'Discussions' },
    ],
  },
  {
    title: 'Legal',
    // CONTACT_EMAIL is `null`-able — drop the row when not configured so we
    // don't render a broken mailto:.
    links: [
      { href: LICENSE_URL, label: 'MIT License' },
      { to: '/docs/check', label: 'Security' },
      ...(CONTACT_EMAIL
        ? [{ href: `mailto:${CONTACT_EMAIL}`, label: 'Contact' }]
        : []),
    ],
  },
]

export default function Footer() {
  return (
    <footer className="bg-surface-container-lowest w-full border-t border-outline-variant/20 mt-24">
      <div className="max-w-max-width mx-auto px-margin-mobile md:px-margin-desktop py-16 grid grid-cols-2 md:grid-cols-5 gap-8">
        <div className="col-span-2 md:col-span-1">
          <div className="font-headline-md text-headline-md text-on-surface font-bold tracking-tighter mb-4">
            ven
          </div>
          <p className="font-mono text-[12px] text-on-surface-variant leading-relaxed max-w-[14rem]">
            One CLI for eight runtimes. Graph-checked installs. Offline-cached CVE scans.
          </p>
        </div>
        {COLUMNS.map((col) => (
          <div key={col.title}>
            <h4 className="text-[11px] font-bold text-outline uppercase tracking-widest mb-4">
              {col.title}
            </h4>
            <ul className="space-y-2">
              {col.links.map((l) => (
                <li key={l.label}>
                  {l.to ? (
                    <Link
                      to={l.to}
                      className="font-mono text-terminal-output text-on-surface-variant hover:text-primary-fixed-dim transition-colors"
                    >
                      {l.label}
                    </Link>
                  ) : (
                    <a
                      href={l.href}
                      target="_blank"
                      rel="noreferrer"
                      className="font-mono text-terminal-output text-on-surface-variant hover:text-primary-fixed-dim transition-colors"
                    >
                      {l.label}
                    </a>
                  )}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <div className="border-t border-outline-variant/20">
        <div className="max-w-max-width mx-auto px-margin-mobile md:px-margin-desktop py-6 flex flex-col md:flex-row items-center justify-between gap-3 font-mono text-terminal-output text-on-surface-variant">
          <span>© {new Date().getFullYear()} ven core team. Built with Rust. Shipped with care.</span>
          <span className="opacity-60">High-performance dependency management.</span>
        </div>
      </div>
    </footer>
  )
}
