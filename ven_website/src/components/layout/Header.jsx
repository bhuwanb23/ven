import { useState } from 'react'
import { Link, NavLink } from 'react-router-dom'
import clsx from 'clsx'
import Brand from '../ui/Brand.jsx'
import Icon from '../ui/Icon.jsx'
import { GITHUB_URL } from '../../content/site.js'

const NAV = [
  { to: '/docs', label: 'Docs' },
  { to: '/languages', label: 'Languages' },
  { to: '/changelog', label: 'Changelog' },
  { to: '/playground', label: 'Playground' },
]

const linkClass = ({ isActive }) =>
  clsx(
    'font-body-base text-body-base transition-colors duration-200',
    isActive
      ? 'text-primary-fixed-dim'
      : 'text-on-surface-variant hover:text-primary-fixed-dim'
  )

export default function Header() {
  const [open, setOpen] = useState(false)
  return (
    <nav className="bg-surface/70 backdrop-blur-xl border-b border-outline-variant/30 shadow-nav-glow fixed top-0 w-full z-50">
      <div className="flex justify-between items-center max-w-max-width mx-auto px-margin-mobile md:px-margin-desktop h-16">
        <Link
          to="/"
          aria-label="ven — home"
          className="flex items-center font-headline-md text-headline-md font-bold text-primary-fixed-dim tracking-tighter"
        >
          <Brand size="md" />
        </Link>

        <div className="hidden md:flex gap-8 items-center">
          {NAV.map((n) => (
            <NavLink key={n.to} to={n.to} className={linkClass}>
              {n.label}
            </NavLink>
          ))}
        </div>

        <div className="hidden md:flex gap-3 items-center">
          <a
            href={GITHUB_URL}
            target="_blank"
            rel="noreferrer"
            className="font-body-base text-body-base text-on-surface-variant hover:text-primary-fixed-dim transition-colors duration-200"
          >
            GitHub
          </a>
          <Link
            to="/install"
            className="bg-primary-fixed-dim text-on-primary-fixed font-bold px-4 py-2 rounded-lg text-sm shadow-[0_0_10px_rgba(0,219,231,0.4)] active:scale-95 transition-all"
          >
            Get Started
          </Link>
        </div>

        <button
          type="button"
          className="md:hidden text-primary-fixed-dim"
          onClick={() => setOpen((o) => !o)}
          aria-label="Toggle navigation"
        >
          <Icon name={open ? 'close' : 'menu'} />
        </button>
      </div>

      {open && (
        <div className="md:hidden border-t border-outline-variant/30 bg-surface/95 backdrop-blur-xl px-margin-mobile py-4 flex flex-col gap-3">
          {NAV.map((n) => (
            <NavLink
              key={n.to}
              to={n.to}
              className={linkClass}
              onClick={() => setOpen(false)}
            >
              {n.label}
            </NavLink>
          ))}
          <Link
            to="/install"
            onClick={() => setOpen(false)}
            className="bg-primary-fixed-dim text-on-primary-fixed font-bold px-4 py-2 rounded-lg text-sm text-center"
          >
            Get Started
          </Link>
        </div>
      )}
    </nav>
  )
}
