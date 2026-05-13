import clsx from 'clsx'

// Level-1 glass surface from design.md. Defaults to soft cyan border; pass
// `tone="neutral"` for the outline-variant border used by docs/install.
export default function GlassCard({ children, className, tone = 'cyan', as: Tag = 'div', ...rest }) {
  return (
    <Tag
      className={clsx(
        tone === 'cyan' ? 'glass-card' : 'glass-surface',
        'rounded-xl',
        className
      )}
      {...rest}
    >
      {children}
    </Tag>
  )
}
