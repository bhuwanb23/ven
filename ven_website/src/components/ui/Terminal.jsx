import clsx from 'clsx'

// Terminal window chrome: traffic-light dots + optional title bar + black body.
// Children fill the body. Used by Landing, Install, Playground, and docs pages.
export default function Terminal({
  title,
  children,
  className,
  bodyClassName,
  glow = true,
}) {
  return (
    <div
      className={clsx(
        'glass-surface overflow-hidden rounded-xl shadow-2xl border border-outline-variant/40',
        glow && 'terminal-glow',
        className
      )}
    >
      <div className="terminal-header h-10 flex items-center justify-between px-4">
        <div className="flex gap-1.5">
          <div className="terminal-header-dot bg-[#FF5F56]" />
          <div className="terminal-header-dot bg-[#FFBD2E]" />
          <div className="terminal-header-dot bg-[#27C93F]" />
        </div>
        {title && (
          <span className="font-mono text-[11px] text-on-surface-variant opacity-70 uppercase tracking-widest">
            {title}
          </span>
        )}
        <span className="w-12" />
      </div>
      <div
        className={clsx(
          'p-6 font-mono text-terminal-output bg-[#050505]',
          bodyClassName
        )}
      >
        {children}
      </div>
    </div>
  )
}
