import { useEffect, useState } from 'react'
import { NavLink, Outlet, useLocation } from 'react-router-dom'
import clsx from 'clsx'
import Icon from '../../components/ui/Icon.jsx'
import { DOC_GROUPS } from '../../content/docs.js'

/**
 * Track how far down the page the user has scrolled, as a 0–1 fraction.
 * Used to draw the progress bar at the top of the sidebar.
 */
function useScrollProgress() {
  const [progress, setProgress] = useState(0)
  useEffect(() => {
    const onScroll = () => {
      const doc = document.documentElement
      const max = (doc.scrollHeight - doc.clientHeight) || 1
      const p = Math.min(1, Math.max(0, doc.scrollTop / max))
      setProgress(p)
    }
    onScroll()
    window.addEventListener('scroll', onScroll, { passive: true })
    return () => window.removeEventListener('scroll', onScroll)
  }, [])
  return progress
}

function Sidebar({ onNavigate, progress }) {
  return (
    <aside
      className={clsx(
        // `sticky` (not `fixed`) is the key fix — the sidebar stays pinned
        // while the docs main column is in view, then scrolls away naturally
        // when the footer comes up, so it can never overlap the footer.
        'lg:sticky lg:top-20 lg:self-start',
        'lg:max-h-[calc(100vh-6rem)] lg:overflow-y-auto',
        'py-6 px-4 border-r border-outline-variant/20'
      )}
    >
      {/* Scroll-progress strip — thin, tone-coloured, sits on the inner edge
          of the sidebar so it doesn't compete with the page content. */}
      <div className="relative h-1 mb-6 bg-surface-container-high rounded-full overflow-hidden">
        <div
          className="absolute inset-y-0 left-0 bg-gradient-to-r from-primary-fixed-dim to-secondary-fixed-dim transition-[width] duration-150"
          style={{ width: `${Math.round(progress * 100)}%` }}
        />
      </div>

      <div className="mb-6 px-2 flex items-center justify-between">
        <span className="text-xs font-bold uppercase tracking-widest text-outline">
          v0.1.1
        </span>
        <span className="text-[10px] font-mono uppercase tracking-widest text-secondary-fixed-dim flex items-center gap-1.5">
          <span className="inline-block w-1.5 h-1.5 rounded-full bg-secondary-fixed-dim animate-pulse" />
          stable
        </span>
      </div>

      <NavLink
        to="/docs"
        end
        onClick={onNavigate}
        className={({ isActive }) =>
          clsx(
            'block px-3 py-2 rounded-lg font-medium mb-4 transition-all duration-200 relative overflow-hidden',
            isActive
              ? 'text-primary-fixed-dim bg-primary-fixed-dim/10 pl-4 before:absolute before:left-0 before:top-1/2 before:-translate-y-1/2 before:h-4 before:w-0.5 before:bg-primary-fixed-dim before:rounded-r'
              : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high hover:translate-x-1'
          )
        }
      >
        Introduction
      </NavLink>

      {DOC_GROUPS.map((group) => (
        <div key={group.title} className="space-y-1 mb-5">
          <p className="text-[10px] font-bold text-outline uppercase tracking-widest px-3 mt-5 mb-2 flex items-center gap-2">
            <span className="inline-block w-3 h-px bg-outline-variant/50" />
            {group.title}
          </p>
          {group.items.map((item) => (
            <NavLink
              key={item.slug}
              to={`/docs/${item.slug}`}
              onClick={onNavigate}
              className={({ isActive }) =>
                clsx(
                  'block px-3 py-1.5 rounded-lg text-sm transition-all duration-200 relative overflow-hidden',
                  isActive
                    ? 'text-primary-fixed-dim bg-primary-fixed-dim/10 font-medium pl-4 before:absolute before:left-0 before:top-1/2 before:-translate-y-1/2 before:h-4 before:w-0.5 before:bg-primary-fixed-dim before:rounded-r'
                    : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high hover:translate-x-1'
                )
              }
            >
              {item.label}
            </NavLink>
          ))}
        </div>
      ))}

      {/* Bottom edge fade — soft visual stop so a long sidebar list doesn't
          look like it was abruptly cropped. */}
      <div className="sticky bottom-0 h-8 -mx-4 mt-4 pointer-events-none bg-gradient-to-t from-surface to-transparent" />
    </aside>
  )
}

export default function DocsLayout() {
  const [mobileOpen, setMobileOpen] = useState(false)
  const progress = useScrollProgress()
  const location = useLocation()

  // Scroll back to the top whenever the route changes so the user always
  // lands at the doc heading. The mobile drawer already closes itself via
  // the `onNavigate` callback wired into every NavLink below.
  useEffect(() => {
    if (typeof window !== 'undefined') {
      window.scrollTo({ top: 0, behavior: 'instant' in window ? 'instant' : 'auto' })
    }
  }, [location.pathname])

  return (
    <div className="max-w-max-width mx-auto">
      {/* Mobile drawer header — sticks below the nav. */}
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

      {mobileOpen && (
        <div className="lg:hidden border-b border-outline-variant/20 bg-surface-container-low">
          <Sidebar onNavigate={() => setMobileOpen(false)} progress={progress} />
        </div>
      )}

      {/* Two-column grid layout — sidebar gets its own column, so its `sticky`
          positioning works correctly and it never sits on top of the footer. */}
      <div className="lg:grid lg:grid-cols-[16rem_1fr] lg:gap-8">
        <div className="hidden lg:block">
          <Sidebar progress={progress} />
        </div>
        <main className="px-margin-mobile md:px-margin-desktop py-12 min-w-0">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
