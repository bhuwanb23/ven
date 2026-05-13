import clsx from 'clsx'

// Thin wrapper around Material Symbols Outlined. Use `fill` to switch to the
// filled variant (CSS data-attribute hook in index.css).
export default function Icon({ name, className, fill = false, weight, style }) {
  const styleVar =
    weight && !fill
      ? { fontVariationSettings: `'FILL' 0, 'wght' ${weight}, 'GRAD' 0, 'opsz' 24`, ...style }
      : weight && fill
        ? { fontVariationSettings: `'FILL' 1, 'wght' ${weight}, 'GRAD' 0, 'opsz' 24`, ...style }
        : style
  return (
    <span
      className={clsx('material-symbols-outlined', className)}
      data-weight={fill ? 'fill' : undefined}
      style={styleVar}
      aria-hidden="true"
    >
      {name}
    </span>
  )
}
