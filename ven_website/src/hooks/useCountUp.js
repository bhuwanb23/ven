import { useEffect, useRef, useState } from 'react'
import useReveal from './useReveal.js'
import usePrefersReducedMotion from './usePrefersReducedMotion.js'

/**
 * Animates a number from `0` to `target` over `duration` ms with a soft
 * ease-out curve. Fires only when the host element scrolls into view and runs
 * exactly once per page load.
 *
 *   const { ref, value } = useCountUp(84, { duration: 1500 })
 *   <div ref={ref}>{value}</div>
 *
 * Reduced-motion users see the final value on first paint (no animation, no
 * intermediate ticks) so screen readers announce the real number.
 */
export default function useCountUp(target, { duration = 1200 } = {}) {
  const { ref, revealed } = useReveal({ threshold: 0.4 })
  const reduced = usePrefersReducedMotion()
  const [value, setValue] = useState(0)
  const started = useRef(false)

  useEffect(() => {
    if (reduced) return undefined
    if (!revealed || started.current) return undefined
    started.current = true
    const startTs = performance.now()
    let raf = 0
    const tick = (now) => {
      const t = Math.min(1, (now - startTs) / duration)
      // ease-out cubic — fast then settle.
      const eased = 1 - Math.pow(1 - t, 3)
      setValue(Math.round(target * eased))
      if (t < 1) {
        raf = requestAnimationFrame(tick)
      }
    }
    raf = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(raf)
  }, [revealed, reduced, target, duration])

  return { ref, value: reduced ? target : value }
}
