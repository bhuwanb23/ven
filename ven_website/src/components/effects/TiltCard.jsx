import { useRef } from 'react'
import clsx from 'clsx'
import usePrefersReducedMotion from '../../hooks/usePrefersReducedMotion.js'

/**
 * Cursor-tracking 3D tilt wrapper. Stays flat on touch and reduced-motion.
 *
 * Implementation note: we deliberately do NOT trigger a React re-render on
 * every mousemove — we mutate `style.setProperty('--rx' / '--ry', ...)` on
 * the DOM node directly. The `.tilt` class in `index.css` reads those vars.
 * One paint per frame, no layout work, no React reconciliation.
 */
export default function TiltCard({
  as: Tag = 'div',
  max = 6,
  children,
  className,
  ...rest
}) {
  const ref = useRef(null)
  const reduced = usePrefersReducedMotion()

  const onMove = (e) => {
    if (reduced) return
    const el = ref.current
    if (!el) return
    const rect = el.getBoundingClientRect()
    const px = (e.clientX - rect.left) / rect.width - 0.5
    const py = (e.clientY - rect.top) / rect.height - 0.5
    el.style.setProperty('--ry', `${px * max}deg`)
    el.style.setProperty('--rx', `${-py * max}deg`)
  }

  const onLeave = () => {
    const el = ref.current
    if (!el) return
    el.style.setProperty('--ry', '0deg')
    el.style.setProperty('--rx', '0deg')
  }

  return (
    <Tag
      ref={ref}
      onMouseMove={onMove}
      onMouseLeave={onLeave}
      className={clsx('tilt', className)}
      {...rest}
    >
      {children}
    </Tag>
  )
}
