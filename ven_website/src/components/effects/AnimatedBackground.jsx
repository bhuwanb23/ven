import { useEffect, useRef } from 'react'
import usePrefersReducedMotion from '../../hooks/usePrefersReducedMotion.js'

/**
 * Site-wide ambient background. Mounts once near the root of <AppLayout />,
 * sits behind everything else (`-z-10`, `pointer-events-none`), and:
 *
 *   1) Drifts two large radial-gradient blobs — cyan + coral — using a
 *      pure-CSS keyframe so the work stays on the compositor thread.
 *   2) Reacts to scroll by reading `--scroll-y` and `--scroll-progress`
 *      from <html> (published by `useScrollY`, mounted once in AppLayout).
 *      The blobs translate slightly slower than the page (parallax) and
 *      shift hue dominance from cyan -> coral as the user scrolls, so
 *      long pages feel like a journey rather than a flat surface.
 *   3) Overlays a faint dot grid for terminal-style texture.
 *
 * `prefers-reduced-motion` is honored: the animation + parallax are
 * dropped, leaving a static gradient + grid (still on-brand, no chrome
 * jitter for vestibular-sensitive users).
 *
 * Performance budget: two `radial-gradient`s on one fixed div + a CSS
 * mask on another. No JS-driven RAF loop here — `useScrollY` (mounted
 * once globally) does the rAF coalescing and writes the vars; the CSS
 * rules in `index.css` (`.bg-aurora` family) consume them.
 */
export default function AnimatedBackground() {
  const reduced = usePrefersReducedMotion()
  const ref = useRef(null)

  // Pointer-driven micro-parallax: when the cursor sits in the viewport,
  // tilt the blobs ~12px toward the cursor so the layer feels alive even
  // before the user scrolls. Coalesced through rAF so it stays free; the
  // CSS read of `--mouse-x/--mouse-y` ignores them when reduced motion is
  // on (the keyframes are paused and the parallax delta is gated to 0).
  useEffect(() => {
    if (reduced) return undefined
    if (typeof window === 'undefined') return undefined

    const root = document.documentElement
    let frame = 0
    let nextX = 0
    let nextY = 0

    const write = () => {
      frame = 0
      root.style.setProperty('--mouse-x', nextX.toFixed(4))
      root.style.setProperty('--mouse-y', nextY.toFixed(4))
    }
    const onMove = (e) => {
      // Normalize to roughly -0.5..0.5 around the viewport center so the
      // CSS can multiply by a small px scalar (e.g. `calc(var(--mouse-x) * 24px)`)
      // without worrying about screen size.
      nextX = e.clientX / window.innerWidth - 0.5
      nextY = e.clientY / window.innerHeight - 0.5
      if (frame) return
      frame = window.requestAnimationFrame(write)
    }
    window.addEventListener('pointermove', onMove, { passive: true })
    return () => {
      window.removeEventListener('pointermove', onMove)
      if (frame) window.cancelAnimationFrame(frame)
      root.style.removeProperty('--mouse-x')
      root.style.removeProperty('--mouse-y')
    }
  }, [reduced])

  return (
    <div
      ref={ref}
      aria-hidden="true"
      data-reduced-motion={reduced ? 'true' : 'false'}
      className="bg-aurora fixed inset-0 -z-10 pointer-events-none overflow-hidden"
    >
      {/* Cyan blob — primary brand colour. Drifts slowly + reacts to scroll
          progress; fades out as the page is scrolled deeper. */}
      <div className="bg-aurora__cyan absolute" />
      {/* Coral blob — secondary brand colour from the logo's right half.
          Mirrors the cyan blob with opposite drift + opposite scroll-tied
          intensity so the colour balance shifts as the user reads down. */}
      <div className="bg-aurora__coral absolute" />
      {/* Dot grid — terminal-style texture. Pure CSS, no animation, masked
          with a vignette so the edges fade away. */}
      <div className="bg-aurora__grid absolute inset-0" />
      {/* Vignette — kills the brightness near content-heavy strips so text
          on top of the background never loses contrast. */}
      <div className="bg-aurora__vignette absolute inset-0" />
    </div>
  )
}
