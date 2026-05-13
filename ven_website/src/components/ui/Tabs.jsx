import { useState } from 'react'
import clsx from 'clsx'

// Headless tab strip used by Install + Landing hero install bar.
// Items: [{ id, label, content }].
export default function Tabs({ items, initial, className, tabsClassName, contentClassName }) {
  const [active, setActive] = useState(initial ?? items[0]?.id)
  const current = items.find((i) => i.id === active) ?? items[0]
  return (
    <div className={className}>
      <div
        className={clsx(
          'flex flex-wrap gap-1 bg-surface-container-high border-b border-outline-variant/30 rounded-t-xl overflow-hidden',
          tabsClassName
        )}
      >
        {items.map((item) => {
          const on = item.id === current.id
          return (
            <button
              key={item.id}
              type="button"
              onClick={() => setActive(item.id)}
              className={clsx(
                'px-4 py-2 font-mono text-terminal-output transition-colors',
                on
                  ? 'text-primary-fixed-dim border-b-2 border-primary-fixed-dim'
                  : 'text-on-surface-variant hover:text-on-surface border-b-2 border-transparent'
              )}
            >
              {item.label}
            </button>
          )
        })}
      </div>
      <div className={contentClassName}>{current?.content}</div>
    </div>
  )
}
