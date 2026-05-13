import { useEffect, useState } from 'react'

/**
 * Reactive `prefers-reduced-motion: reduce` matcher.
 *
 * Returns `true` when the user has asked the OS to minimise non-essential
 * motion (Windows: "Show animations in Windows", macOS: "Reduce motion",
 * iOS / Android: Accessibility → Reduce motion). The handful of animation
 * hooks below all bail out early when this returns `true`, which is the
 * agreed master kill-switch for the whole site.
 */
export default function usePrefersReducedMotion() {
  const [reduced, setReduced] = useState(() => {
    if (typeof window === 'undefined' || !window.matchMedia) return false
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches
  })

  useEffect(() => {
    if (typeof window === 'undefined' || !window.matchMedia) return undefined
    const mq = window.matchMedia('(prefers-reduced-motion: reduce)')
    const handler = (e) => setReduced(e.matches)
    // `addEventListener` is the modern API; Safari < 14 needs `addListener`.
    if (mq.addEventListener) {
      mq.addEventListener('change', handler)
      return () => mq.removeEventListener('change', handler)
    }
    mq.addListener(handler)
    return () => mq.removeListener(handler)
  }, [])

  return reduced
}
