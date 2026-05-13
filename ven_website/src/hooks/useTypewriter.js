import { useEffect, useState } from 'react'
import usePrefersReducedMotion from './usePrefersReducedMotion.js'

/**
 * One-shot typewriter for headline text.
 *
 *   const { text, done } = useTypewriter('The Intelligent...', { speed: 45 })
 *
 * Distinct from `ScriptedTerminal`, which is a full multi-step terminal
 * session player. This hook just types a single string from empty -> full
 * once. Reduced-motion users see the full string immediately and `done = true`
 * on first render — no half-typed text on a screen reader.
 */
export default function useTypewriter(value, { speed = 45, startDelay = 200 } = {}) {
  const reduced = usePrefersReducedMotion()
  // Track only the animated state; when reduced-motion is on we return the
  // full string + done=true from the hook directly, so the effect can be a
  // clean no-op (no synchronous setState inside the body, which the React 19
  // hooks plugin disallows).
  const [text, setText] = useState('')
  const [done, setDone] = useState(false)

  useEffect(() => {
    if (reduced) return undefined
    let i = 0
    let raf = 0
    let interval = 0
    const start = setTimeout(() => {
      interval = setInterval(() => {
        i += 1
        raf = requestAnimationFrame(() => setText(value.slice(0, i)))
        if (i >= value.length) {
          clearInterval(interval)
          setDone(true)
        }
      }, speed)
    }, startDelay)
    return () => {
      clearTimeout(start)
      clearInterval(interval)
      cancelAnimationFrame(raf)
    }
  }, [value, speed, startDelay, reduced])

  if (reduced) return { text: value, done: true }
  return { text, done }
}
