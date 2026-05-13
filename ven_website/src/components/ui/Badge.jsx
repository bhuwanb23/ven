import clsx from 'clsx'

// Status chip per design.md §Components. Uses 1px coloured border + transparent
// fill, monospace text. Tone picks the colour family.
const TONES = {
  stable:
    'border-secondary-fixed-dim/40 text-secondary-fixed-dim bg-secondary-fixed-dim/5',
  planned: 'border-outline/40 text-outline bg-outline/5',
  major:
    'border-secondary-fixed-dim/50 text-secondary-fixed-dim bg-secondary-fixed-dim/10',
  minor:
    'border-primary-fixed-dim/50 text-primary-fixed-dim bg-primary-fixed-dim/10',
  patch: 'border-outline/50 text-on-surface-variant bg-outline-variant/10',
  security: 'border-error/50 text-error bg-error-container/15',
  cyan: 'border-primary-fixed-dim/40 text-primary-fixed-dim bg-primary-fixed-dim/10',
  red: 'border-error/40 text-error bg-error-container/15',
  neutral:
    'border-outline-variant/40 text-on-surface-variant bg-surface-container-high/40',
}

export default function Badge({ children, tone = 'neutral', className }) {
  return (
    <span
      className={clsx(
        'inline-flex items-center gap-1.5 rounded font-mono text-[10px] uppercase tracking-widest px-2 py-0.5 border',
        TONES[tone],
        className
      )}
    >
      {children}
    </span>
  )
}
