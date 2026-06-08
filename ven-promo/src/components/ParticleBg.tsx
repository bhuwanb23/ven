import React, { useMemo } from "react";
import { useCurrentFrame } from "remotion";

interface Particle {
  x: number;
  y: number;
  size: number;
  speed: number;
  driftX: number;
  driftY: number;
  opacity: number;
  phase: number;
}

interface ParticleBgProps {
  count?: number;
  color?: string;
  baseOpacity?: number;
  speed?: number;
}

const seededRandom = (seed: number) => {
  const x = Math.sin(seed + 0.1) * 10000;
  return x - Math.floor(x);
};

export const ParticleBg: React.FC<ParticleBgProps> = ({
  count = 30,
  color = "0, 200, 255",
  baseOpacity = 0.15,
  speed = 0.2,
}) => {
  const frame = useCurrentFrame();
  const particles = useMemo<Particle[]>(() => {
    const result: Particle[] = [];
    for (let i = 0; i < count; i++) {
      result.push({
        x: seededRandom(i * 7 + 1) * 1920,
        y: seededRandom(i * 7 + 2) * 1080,
        size: 1 + seededRandom(i * 7 + 3) * 3,
        speed: 0.3 + seededRandom(i * 7 + 4) * 0.7,
        driftX: (seededRandom(i * 7 + 5) - 0.5) * 2,
        driftY: (seededRandom(i * 7 + 6) - 0.5) * 2,
        opacity: 0.3 + seededRandom(i * 7 + 7) * 0.7,
        phase: seededRandom(i * 7 + 8) * Math.PI * 2,
      });
    }
    return result;
  }, [count]);

  return (
    <svg width="1920" height="1080" style={{ position: "absolute", inset: 0 }}>
      <defs>
        <radialGradient id="particleGlow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor={`rgba(${color}, 1)`} stopOpacity="1" />
          <stop offset="100%" stopColor={`rgba(${color}, 0)`} stopOpacity="0" />
        </radialGradient>
      </defs>
      {particles.map((p, i) => {
        const driftOffsetX = Math.sin(frame * 0.005 * p.speed + p.phase) * p.driftX * 30;
        const driftOffsetY = Math.cos(frame * 0.005 * p.speed + p.phase) * p.driftY * 30;
        const breathe = 0.7 + 0.3 * Math.sin(frame * 0.02 * p.speed + p.phase);

        return (
          <circle
            key={i}
            cx={p.x + driftOffsetX}
            cy={p.y + driftOffsetY}
            r={p.size}
            fill={`rgba(${color}, ${baseOpacity * p.opacity * breathe})`}
          />
        );
      })}
    </svg>
  );
};
