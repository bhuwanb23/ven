import { useCallback, useState } from 'react'
import clsx from 'clsx'
import Icon from './Icon.jsx'

// Single-line or multi-line code block with a copy button. Visual match for the
// `bg-surface-container-lowest` snippets used throughout the existing HTML.
export default function CodeBlock({
  code,
  language = 'shell',
  className,
  prompt = '$',
  showPrompt = true,
  tone = 'cyan',
  copyable = true,
}) {
  const [copied, setCopied] = useState(false)

  const onCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code)
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch {
      // Clipboard API can be unavailable in restricted contexts; silently no-op.
    }
  }, [code])

  const codeColor =
    tone === 'cyan' ? 'text-primary-fixed-dim' : 'text-secondary-fixed-dim'

  return (
    <div
      className={clsx(
        'glass-surface bg-surface-container-lowest rounded-lg overflow-hidden border border-outline-variant/30',
        className
      )}
    >
      <div className="flex items-center justify-between gap-4 p-4 group">
        <code
          className={clsx(
            'font-mono text-sm md:text-base block whitespace-pre-wrap break-all',
            codeColor
          )}
        >
          {showPrompt && (
            <span className="text-on-surface-variant mr-2 select-none">
              {prompt}
            </span>
          )}
          {code}
        </code>
        {copyable && (
          <button
            type="button"
            onClick={onCopy}
            className="shrink-0 text-on-surface-variant hover:text-primary-fixed-dim transition-colors"
            aria-label="Copy command"
          >
            <Icon name={copied ? 'check' : 'content_copy'} className="text-base" />
          </button>
        )}
      </div>
      {language && (
        <div className="text-[10px] uppercase tracking-widest text-on-surface-variant/40 font-mono px-4 pb-2">
          {language}
        </div>
      )}
    </div>
  )
}
