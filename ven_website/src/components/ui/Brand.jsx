import clsx from 'clsx'

/**
 * Site-wide brand mark: the ven logo paired (by default) with the
 * lowercase "ven" wordmark. Use everywhere a "ven" label currently
 * appears so swapping the logotype across the site is a one-file change.
 *
 *   <Brand />                        // header / hero default
 *   <Brand size="sm" />              // footer / sidebars
 *   <Brand size="lg" wordmark={false} />  // logo-only hero variant
 *   <Brand href={null} />            // disable the implicit <Link to="/">
 *
 * The logo path goes through Vite's `BASE_URL` so it resolves to
 * `/ven-logo.png` in dev and `/ven/ven-logo.png` on the GitHub Pages
 * project site without each caller having to think about it.
 */
const SIZES = {
  sm: { img: 'h-6 w-6', text: 'text-base' },
  md: { img: 'h-8 w-8', text: 'text-headline-md' },
  lg: { img: 'h-12 w-12', text: 'text-4xl' },
}

export default function Brand({
  size = 'md',
  wordmark = true,
  className,
  imgClassName,
  textClassName,
  alt = 'ven logo',
}) {
  const s = SIZES[size] ?? SIZES.md
  const src = `${import.meta.env.BASE_URL}ven-logo.png`
  return (
    <span
      className={clsx('inline-flex items-center gap-2 select-none', className)}
    >
      <img
        src={src}
        alt={alt}
        width={48}
        height={48}
        decoding="async"
        loading="eager"
        className={clsx(
          s.img,
          // Soft cyan halo on hover only — keeps the chrome calm at rest
          // but rewards the cursor for landing on the brand mark, mirroring
          // the `cyan-glow` treatment used by the Get Started button.
          'rounded-md transition-shadow duration-300',
          'hover:drop-shadow-[0_0_10px_rgba(0,219,231,0.4)]',
          imgClassName
        )}
      />
      {wordmark && (
        <span
          className={clsx(
            'font-bold tracking-tighter text-primary-fixed-dim',
            s.text,
            textClassName
          )}
        >
          ven
        </span>
      )}
    </span>
  )
}
