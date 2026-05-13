import { useState } from 'react'
import clsx from 'clsx'
import useReveal from '../../hooks/useReveal.js'

// Visual mock of a `ven graph` output for the Landing "Total Visibility"
// section. Nodes are positioned in a viewBox so the whole thing scales
// cleanly inside any aspect-square parent. Edges are SVG <path>s with a
// dashed stroke whose offset is animated from `--len` to `0` once the
// graph scrolls into view, giving the lines-drawing-in effect.
//
// Hovering a node highlights both the node and every edge it touches.
// The dedicated `cve` node is rendered with `.red-pulse` so the existing
// keyframe in index.css gets a real use.

const VB = 600 // square viewBox edge

const NODES = [
  { id: 'root',       label: 'my-app',          kind: 'root',    x: VB / 2,        y: VB / 2 },
  { id: 'express',    label: 'express',         kind: 'normal',  x: VB / 2,        y: 70 },
  { id: 'body',       label: 'body-parser',     kind: 'normal',  x: VB - 60,       y: VB / 2 - 80 },
  { id: 'lodash',     label: 'lodash',          kind: 'normal',  x: VB - 80,       y: VB - 100 },
  { id: 'axios',      label: 'axios',           kind: 'normal',  x: 80,            y: VB - 100 },
  { id: 'cve',        label: 'follow-redirects', kind: 'cve',    x: 60,            y: VB / 2 - 80 },
]

const EDGES = [
  ['root', 'express'],
  ['root', 'lodash'],
  ['root', 'axios'],
  ['express', 'body'],
  ['axios', 'cve'],
]

function NodeStyle({ kind, active }) {
  // Drives ring color + label tone for one node. Kept here so the JSX below
  // stays terse and the className strings are easy to scan.
  if (kind === 'root') {
    return {
      ring: 'stroke-primary-fixed-dim',
      fill: 'fill-surface',
      label: 'fill-primary-fixed-dim font-bold',
      r: 52,
    }
  }
  if (kind === 'cve') {
    return {
      ring: 'stroke-error',
      fill: 'fill-surface-container',
      label: 'fill-error',
      r: 40,
    }
  }
  return {
    ring: active ? 'stroke-primary-fixed-dim' : 'stroke-outline-variant',
    fill: 'fill-surface-container',
    label: active ? 'fill-primary-fixed-dim' : 'fill-on-surface-variant',
    r: 40,
  }
}

export default function AnimatedDepGraph({ className }) {
  const [hovered, setHovered] = useState(null)
  const { ref, revealed } = useReveal({ threshold: 0.25 })

  // Edges are "active" when either endpoint is hovered.
  const isEdgeActive = (a, b) => hovered === a || hovered === b

  return (
    <div
      ref={ref}
      className={clsx(
        'glass-card relative aspect-square rounded-2xl overflow-hidden',
        className
      )}
    >
      <svg
        viewBox={`0 0 ${VB} ${VB}`}
        className="absolute inset-0 w-full h-full"
        role="img"
        aria-label="ven dependency graph"
      >
        <defs>
          <radialGradient id="dep-graph-glow" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="rgba(0,219,231,0.18)" />
            <stop offset="100%" stopColor="rgba(0,219,231,0)" />
          </radialGradient>
        </defs>

        {/* Soft cyan halo behind the centre node. */}
        <circle cx={VB / 2} cy={VB / 2} r={VB / 2} fill="url(#dep-graph-glow)" />

        {/* Edges. We draw each as a path so we can dash + animate the offset.
            The CVE edge gets a red stroke so the broken-dep theme reads at a
            glance even before hover. */}
        {EDGES.map(([from, to]) => {
          const a = NODES.find((n) => n.id === from)
          const b = NODES.find((n) => n.id === to)
          if (!a || !b) return null
          const len = Math.hypot(b.x - a.x, b.y - a.y)
          const danger = a.kind === 'cve' || b.kind === 'cve'
          const active = isEdgeActive(from, to)
          return (
            <line
              key={`${from}-${to}`}
              x1={a.x}
              y1={a.y}
              x2={b.x}
              y2={b.y}
              strokeWidth={active ? 2.5 : 1.5}
              strokeLinecap="round"
              className={clsx(
                'transition-[stroke,stroke-width,filter] duration-300',
                danger && !active && 'stroke-error/60',
                danger && active && 'stroke-error',
                !danger && !active && 'stroke-outline-variant/60',
                !danger && active && 'stroke-primary-fixed-dim',
                revealed && 'animate-draw-line'
              )}
              style={{
                strokeDasharray: len,
                strokeDashoffset: revealed ? 0 : len,
                ['--len']: len,
                filter: active ? 'drop-shadow(0 0 6px rgba(0,219,231,0.6))' : undefined,
              }}
            />
          )
        })}

        {/* Nodes — circle + centred text. */}
        {NODES.map((n) => {
          const active = hovered === n.id || hovered == null && n.kind === 'root'
          const s = NodeStyle({ kind: n.kind, active: hovered === n.id })
          return (
            <g
              key={n.id}
              onMouseEnter={() => setHovered(n.id)}
              onMouseLeave={() => setHovered((cur) => (cur === n.id ? null : cur))}
              onFocus={() => setHovered(n.id)}
              onBlur={() => setHovered((cur) => (cur === n.id ? null : cur))}
              tabIndex={0}
              className="cursor-pointer outline-none"
            >
              <circle
                cx={n.x}
                cy={n.y}
                r={s.r}
                strokeWidth={n.kind === 'root' ? 2 : 1.5}
                className={clsx(
                  s.ring,
                  s.fill,
                  'transition-all duration-300',
                  active && 'drop-shadow-[0_0_14px_rgba(0,219,231,0.5)]'
                )}
              />
              <text
                x={n.x}
                y={n.y + 4}
                textAnchor="middle"
                className={clsx('text-[14px] font-mono', s.label)}
              >
                {n.label}
              </text>
            </g>
          )
        })}
      </svg>

      {/* CVE pulse overlay — sits over the cve node and uses the existing
          .red-pulse keyframe. Positioning is in % so it stays glued to the
          node as the SVG scales. */}
      <div
        className="pointer-events-none absolute red-pulse"
        style={{
          left: `${(NODES.find((n) => n.id === 'cve').x / VB) * 100}%`,
          top: `${(NODES.find((n) => n.id === 'cve').y / VB) * 100}%`,
          width: 80,
          height: 80,
          transform: 'translate(-50%, -50%)',
          borderRadius: '50%',
        }}
      />

      {/* Caption strip — pinned to the bottom edge, fades up with the
          reveal so it doesn't compete with the line-draw on mount. */}
      <div
        className={clsx(
          'absolute bottom-3 left-3 right-3 flex items-center justify-between gap-3 text-[11px] font-mono uppercase tracking-widest',
          'transition-opacity duration-700',
          revealed ? 'opacity-100' : 'opacity-0'
        )}
      >
        <span className="text-on-surface-variant/80">5 packages · 1 CVE</span>
        <span className="text-error flex items-center gap-1">
          <span className="inline-block w-1.5 h-1.5 bg-error rounded-full" />
          follow-redirects @ GHSA-cxjh-pqwp-8mfp
        </span>
      </div>
    </div>
  )
}
