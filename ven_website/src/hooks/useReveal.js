import { useEffect, useRef, useState } from 'react'
import usePrefersReducedMotion from './usePrefersReducedMotion.js'

/**
 * One-shot scroll-reveal observer.
 *
 *   const { ref, revealed } = useReveal()
 *   <section ref={ref} className={revealed ? 'reveal-in' : 'reveal-init'}>
 *
 * Uses `IntersectionObserver` once: as soon as 15% of the target enters the
 * viewport we flip `revealed` to `true` and unobserve, so the animation is
 * triggered exactly once per page load. Browsers without `IntersectionObserver`
 * (very old; effectively none in the React 19 baseline) and users with
 * `prefers-reduced-motion: reduce` get `revealed = true` from the start so
 * content is never trapped invisible.
 */
export default function useReveal({ threshold = 0.15, rootMargin = '0px 0px -10% 0px' } = {}) {
  const ref = useRef(null)
  const reduced = usePrefersReducedMotion()
  // Internal state only — we OR it with `reduced` / IO-unavailable below so
  // we never need to call setState synchronously inside the effect body.
  const [revealedFromIO, setRevealedFromIO] = useState(false)
  const noIO = typeof window !== 'undefined' && typeof IntersectionObserver === 'undefined'

  useEffect(() => {
    if (reduced || noIO) return undefined
    const el = ref.current
    if (!el) return undefined
    const io = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setRevealedFromIO(true)
            io.unobserve(entry.target)
          }
        }
      },
      { threshold, rootMargin }
    )
    io.observe(el)
    return () => io.disconnect()
  }, [reduced, noIO, threshold, rootMargin])

  return { ref, revealed: revealedFromIO || reduced || noIO }
}
