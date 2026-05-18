import { useEffect } from 'react'

/**
 * Continuously publish the current scroll position as CSS custom properties
 * on `document.documentElement`. Lets purely-presentational components (the
 * animated background, decorative parallax layers, etc.) respond to scroll
 * without each one wiring up its own listener or causing a React re-render
 * on every scroll tick.
 *
 * Published vars:
 *
 *   --scroll-y         pixel offset, like `420px`
 *   --scroll-progress  0-1 (clamped) over the document's scrollable range
 *
 * Notes:
 * - The listener is `passive: true` so it never blocks the scroll thread.
 * - All writes are coalesced through a single `requestAnimationFrame`, so
 *   even a hyperactive trackpad caps us at one DOM write per frame.
 * - Honors `prefers-reduced-motion`: the listener still runs (consumers may
 *   still want the value for non-animated reasons), but downstream CSS
 *   should already be gating its own animations on the media query.
 * - SSR safe: bails out cleanly when `window` is undefined.
 *
 * Mount this exactly once near the root of the React tree. Multiple mounts
 * are harmless but redundant — every consumer reads the same root-level
 * vars.
 */
export default function useScrollY() {
  useEffect(() => {
    if (typeof window === 'undefined') return undefined

    const root = document.documentElement
    let frame = 0

    const write = () => {
      frame = 0
      const y = window.scrollY || window.pageYOffset || 0
      const max = Math.max(
        1,
        document.documentElement.scrollHeight - window.innerHeight
      )
      const progress = Math.min(1, Math.max(0, y / max))
      root.style.setProperty('--scroll-y', `${y}px`)
      root.style.setProperty('--scroll-progress', progress.toFixed(4))
    }

    const onScroll = () => {
      if (frame) return
      frame = window.requestAnimationFrame(write)
    }

    // Seed the vars once before any scroll event so the first paint is
    // already in sync (otherwise the background pops once on first scroll).
    write()
    window.addEventListener('scroll', onScroll, { passive: true })
    window.addEventListener('resize', onScroll, { passive: true })
    return () => {
      window.removeEventListener('scroll', onScroll)
      window.removeEventListener('resize', onScroll)
      if (frame) window.cancelAnimationFrame(frame)
    }
  }, [])
}
