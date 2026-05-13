import clsx from 'clsx'
import useReveal from '../../hooks/useReveal.js'

/**
 * Drop-in scroll-reveal wrapper.
 *
 *   <Reveal as="section" className="py-24">
 *     ...
 *   </Reveal>
 *
 * Renders the requested tag with `reveal-init` baseline classes that the CSS
 * in `index.css` consumes. When the element scrolls into view (or the user
 * has reduced-motion enabled) we add `reveal-in` to flip opacity / transform.
 */
export default function Reveal({
  as: Tag = 'div',
  children,
  className,
  delay = 0,
  ...rest
}) {
  const { ref, revealed } = useReveal()
  return (
    <Tag
      ref={ref}
      className={clsx('reveal-init', revealed && 'reveal-in', className)}
      style={delay ? { transitionDelay: `${delay}ms` } : undefined}
      {...rest}
    >
      {children}
    </Tag>
  )
}
