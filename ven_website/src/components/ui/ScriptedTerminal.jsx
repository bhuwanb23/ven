import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import clsx from 'clsx'
import Icon from './Icon.jsx'

// Renders a scripted terminal session. Each script entry is one of:
//   { kind: 'command', text, prompt? }   -> typewriter-typed user input
//   { kind: 'output',  text, tone? }     -> instant-rendered output line(s)
//   { kind: 'pause',   ms }              -> sleep before next step
// tones map to terminal colours used throughout the site.
const TONE_CLASS = {
  default: 'text-on-surface-variant',
  user: 'text-on-surface',
  prompt: 'text-secondary-fixed-dim',
  success: 'text-secondary-fixed-dim',
  cyan: 'text-primary-fixed-dim',
  error: 'text-error',
  warn: 'text-tertiary-fixed-dim',
  muted: 'text-outline',
}

// Per-character typing speed in ms (kept low to feel snappy).
const TYPE_MS = 28

export default function ScriptedTerminal({
  script,
  title = 'ven',
  autoPlay = true,
  loop = false,
  controls = true,
  height = 'auto',
  className,
}) {
  const [step, setStep] = useState(0)
  const [typedCount, setTypedCount] = useState(0)
  const [playing, setPlaying] = useState(autoPlay)
  const [renderedLines, setRenderedLines] = useState([])
  const timeoutRef = useRef(null)
  const bodyRef = useRef(null)

  const flatScript = useMemo(() => script ?? [], [script])

  const reset = useCallback(() => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current)
    setStep(0)
    setTypedCount(0)
    setRenderedLines([])
    setPlaying(true)
  }, [])

  // Drive the script forward.
  useEffect(() => {
    if (!playing) return
    if (step >= flatScript.length) {
      if (loop) {
        timeoutRef.current = setTimeout(() => reset(), 2000)
      }
      return
    }
    const entry = flatScript[step]

    if (entry.kind === 'pause') {
      timeoutRef.current = setTimeout(() => {
        setStep((s) => s + 1)
      }, entry.ms ?? 400)
      return
    }

    if (entry.kind === 'output') {
      setRenderedLines((prev) => [
        ...prev,
        { kind: 'output', text: entry.text, tone: entry.tone ?? 'default' },
      ])
      timeoutRef.current = setTimeout(() => {
        setStep((s) => s + 1)
      }, entry.ms ?? 220)
      return
    }

    // command — typewriter effect.
    if (entry.kind === 'command') {
      if (typedCount < entry.text.length) {
        timeoutRef.current = setTimeout(() => {
          setTypedCount((c) => c + 1)
        }, TYPE_MS)
      } else {
        // commit typed line, advance.
        setRenderedLines((prev) => [
          ...prev,
          {
            kind: 'command',
            text: entry.text,
            prompt: entry.prompt ?? '$',
          },
        ])
        timeoutRef.current = setTimeout(() => {
          setTypedCount(0)
          setStep((s) => s + 1)
        }, 250)
      }
      return
    }
  }, [playing, step, typedCount, flatScript, loop, reset])

  // Auto-scroll the body as new lines appear.
  useEffect(() => {
    if (bodyRef.current) {
      bodyRef.current.scrollTop = bodyRef.current.scrollHeight
    }
  }, [renderedLines, typedCount])

  const current = flatScript[step]
  const typingPartial =
    current && current.kind === 'command'
      ? current.text.slice(0, typedCount)
      : null

  return (
    <div
      className={clsx(
        'glass-surface terminal-glow rounded-xl overflow-hidden shadow-2xl border border-outline-variant/40',
        className
      )}
    >
      <div className="terminal-header h-10 flex items-center justify-between px-4">
        <div className="flex gap-1.5">
          <div className="terminal-header-dot bg-[#FF5F56]" />
          <div className="terminal-header-dot bg-[#FFBD2E]" />
          <div className="terminal-header-dot bg-[#27C93F]" />
        </div>
        <span className="font-mono text-[11px] text-on-surface-variant uppercase tracking-widest">
          {title}
        </span>
        {controls ? (
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setPlaying((p) => !p)}
              className="text-on-surface-variant hover:text-primary-fixed-dim text-sm"
              aria-label={playing ? 'Pause' : 'Play'}
            >
              <Icon name={playing ? 'pause' : 'play_arrow'} />
            </button>
            <button
              type="button"
              onClick={reset}
              className="text-on-surface-variant hover:text-primary-fixed-dim text-sm"
              aria-label="Restart"
            >
              <Icon name="restart_alt" />
            </button>
          </div>
        ) : (
          <span className="w-12" />
        )}
      </div>
      <div
        ref={bodyRef}
        className="p-6 font-mono text-terminal-output bg-[#050505] overflow-y-auto"
        style={{ height, minHeight: '260px', maxHeight: height === 'auto' ? '480px' : height }}
      >
        {renderedLines.map((line, i) =>
          line.kind === 'command' ? (
            <div key={i} className="mb-2">
              <span className="text-secondary-fixed-dim mr-2">{line.prompt}</span>
              <span className="text-on-surface">{line.text}</span>
            </div>
          ) : (
            <div
              key={i}
              className={clsx('mb-1 whitespace-pre-wrap', TONE_CLASS[line.tone] ?? TONE_CLASS.default)}
            >
              {line.text}
            </div>
          )
        )}
        {typingPartial !== null && (
          <div className="mb-2">
            <span className="text-secondary-fixed-dim mr-2">
              {current.prompt ?? '$'}
            </span>
            <span className="text-on-surface">{typingPartial}</span>
            <span className="inline-block w-2 h-4 align-middle bg-primary-fixed-dim/80 animate-caret-blink ml-0.5" />
          </div>
        )}
      </div>
    </div>
  )
}
