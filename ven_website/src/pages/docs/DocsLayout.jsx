import { useState } from 'react'
import { NavLink, Outlet } from 'react-router-dom'
import clsx from 'clsx'
import Icon from '../../components/ui/Icon.jsx'
import { DOC_GROUPS } from '../../content/docs.js'

function Sidebar({ onNavigate }) {
  return (
    <aside className="lg:w-64 lg:fixed lg:h-[calc(100vh-4rem)] lg:overflow-y-auto border-r border-outline-variant/20 py-8 px-6">
      <div className="mb-6 px-2 flex items-center justify-between">
        <span className="text-xs font-bold uppercase tracking-widest text-outline">
          v1.0.0 (Stable)
        </span>
      </div>
      <NavLink
        to="/docs"
        end
        onClick={onNavigate}
        className={({ isActive }) =>
          clsx(
            'block px-2 py-1.5 rounded font-medium mb-4',
            isActive
              ? 'text-primary-container bg-primary-container/5'
              : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high'
          )
        }
      >
        Introduction
      </NavLink>
      {DOC_GROUPS.map((group) => (
        <div key={group.title} className="space-y-1 mb-4">
          <p className="text-[11px] font-bold text-outline-variant uppercase tracking-widest px-2 mt-6 mb-2">
            {group.title}
          </p>
          {group.items.map((item) => (
            <NavLink
              key={item.slug}
              to={`/docs/${item.slug}`}
              onClick={onNavigate}
              className={({ isActive }) =>
                clsx(
                  'block px-2 py-1.5 rounded text-sm transition-all',
                  isActive
                    ? 'text-primary-container bg-primary-container/5 font-medium'
                    : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high'
                )
              }
            >
              {item.label}
            </NavLink>
          ))}
        </div>
      ))}
    </aside>
  )
}

export default function DocsLayout() {
  const [mobileOpen, setMobileOpen] = useState(false)

  return (
    <div className="max-w-max-width mx-auto">
      <div className="lg:hidden flex items-center justify-between border-b border-outline-variant/20 px-margin-mobile py-3 sticky top-16 bg-surface/95 backdrop-blur-xl z-30">
        <span className="font-mono text-xs uppercase tracking-widest text-on-surface-variant">
          Docs
        </span>
        <button
          type="button"
          onClick={() => setMobileOpen((o) => !o)}
          className="text-primary-fixed-dim flex items-center gap-1 text-sm"
        >
          <Icon name={mobileOpen ? 'close' : 'menu_book'} />
          {mobileOpen ? 'Close' : 'Browse'}
        </button>
      </div>

      <div className={clsx('lg:hidden', mobileOpen ? 'block' : 'hidden')}>
        <Sidebar onNavigate={() => setMobileOpen(false)} />
      </div>

      <div className="hidden lg:block">
        <Sidebar />
      </div>

      <main className="lg:ml-64 px-margin-mobile md:px-margin-desktop py-12">
        <Outlet />
      </main>
    </div>
  )
}
