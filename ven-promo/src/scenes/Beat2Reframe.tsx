import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, spring } from "remotion";
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";
import { SoundFX } from "../components/SoundFX";

const cx = 960;
const cy = 486;

const conflictPaths = [
  { d: "M 0,0 C 60,-80 120,-40 180,-120", x: cx - 240, y: cy + 20, label: "" },
  { d: "M 0,0 C 40,-60 100,-20 140,-100", x: cx - 200, y: cy + 30, label: "v2.1.1" },
  { d: "M 0,0 C -40,-60 -100,-20 -140,-100", x: cx + 200, y: cy - 50, label: "v1.8.3" },
  { d: "M 0,0 C 50,60 120,40 180,100", x: cx - 200, y: cy + 100, label: "" },
  { d: "M 0,0 C -60,80 -120,40 -180,120", x: cx + 200, y: cy + 80, label: "v3.0.0-beta" },
];

const orbitingParticles = Array.from({ length: 16 }, (_, i) => {
  const angle = (i / 16) * Math.PI * 2;
  return { angle, speed: 0.5 + Math.sin(i * 1.3) * 0.3, radius: 120 + i * 18 };
});

const CLAMP = { extrapolateLeft: "clamp" as const, extrapolateRight: "clamp" as const };

export const Beat2Reframe: React.FC = () => {
  const frame = useCurrentFrame();

  const dotSpring = spring({
    frame: Math.max(0, frame),
    fps: 30,
    config: { damping: 14, stiffness: 80, mass: 0.8 },
  });

  const dotGlow = spring({
    frame: Math.max(0, frame - 10),
    fps: 30,
    config: { damping: 20, stiffness: 60, mass: 0.8 },
  });

  const dotPulse = 0.5 + 0.5 * Math.sin(frame * 0.06);
  const ringPulse = 1 + 0.03 * Math.sin(frame * 0.03);

  const dotOpacity = interpolate(frame, [0, 25], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const fadeOut = interpolate(frame, [130, 150], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const branchOpacity = interpolate(frame, [15, 40], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        background:
          "linear-gradient(180deg, #080e1a 0%, #0c1425 30%, #131313 100%)",
        opacity: fadeOut,
      }}
    >
      <ParticleBg count={30} color="0, 150, 255" baseOpacity={0.06} />

      {/* Orbital ring system */}
      <svg
        style={{ position: "absolute", inset: 0, pointerEvents: "none", opacity: dotOpacity * 0.6 }}
        width="1920" height="1080"
      >
        <defs>
          <radialGradient id="ringGrad" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="rgba(0, 150, 255, 0)" />
            <stop offset="70%" stopColor="rgba(0, 150, 255, 0.04)" />
            <stop offset="100%" stopColor="rgba(0, 150, 255, 0)" />
          </radialGradient>
        </defs>
        {[180, 260, 360, 480].map((r, i) => (
          <circle
            key={i}
            cx={cx}
            cy={cy}
            r={r * ringPulse}
            fill="none"
            stroke={`rgba(0, 150, 255, ${0.04 + 0.02 * Math.sin(frame * 0.02 + i * 1.2)})`}
            strokeWidth={1}
            strokeDasharray={i % 2 === 0 ? "4 8" : "2 12"}
          />
        ))}
        {/* Orbiting dots */}
        {orbitingParticles.map((p, i) => {
          const a = p.angle + frame * 0.008 * p.speed;
          const r = p.radius * ringPulse;
          return (
            <circle
              key={i}
              cx={cx + r * Math.cos(a)}
              cy={cy + r * Math.sin(a)}
              r={2 + Math.sin(frame * 0.05 + i) * 0.8}
              fill={`rgba(0, 200, 255, ${0.15 + 0.1 * Math.sin(frame * 0.04 + i)})`}
            />
          );
        })}
      </svg>

      {/* Conflict branch paths */}
      <svg
        style={{ position: "absolute", inset: 0, pointerEvents: "none", opacity: branchOpacity * dotOpacity }}
        width="1920" height="1080"
      >
        {conflictPaths.map((path, i) => {
          const visible = interpolate(frame, [25 + i * 5, 35 + i * 5], [0, 1], CLAMP);
          return (
            <g key={i} opacity={visible}>
              <path
                d={path.d}
                fill="none"
                stroke={`rgba(255, ${100 + i * 30}, 0, ${0.15 + 0.05 * Math.sin(frame * 0.04 + i)})`}
                strokeWidth={1.5}
                strokeDasharray="3 4"
                transform={`translate(${cx}, ${cy})`}
              />
              <circle
                cx={cx}
                cy={cy}
                r={3}
                fill={`rgba(255, ${100 + i * 30}, 0, 0.3)`}
                opacity={visible}
              />
              {path.label && (
                <text
                  x={path.x}
                  y={path.y}
                  fill="rgba(255,255,255,0.12)"
                  fontSize={11}
                  fontFamily="monospace"
                  opacity={visible}
                >
                  {path.label}
                </text>
              )}
            </g>
          );
        })}
      </svg>

      <div
        style={{
          position: "absolute",
          left: "50%",
          top: "45%",
          transform: "translate(-50%, -50%)",
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            flexDirection: "column",
            gap: 50,
          }}
        >
          <div
            style={{
              width: 40 * dotSpring * dotPulse,
              height: 40 * dotSpring * dotPulse,
              borderRadius: "50%",
              background: "radial-gradient(circle, #00c8ff 0%, #0066cc 100%)",
              boxShadow: `0 0 ${120 * dotGlow}px rgba(0, 100, 255, ${0.3 * dotGlow})`,
              opacity: dotOpacity,
            }}
          />

          {frame >= 35 && (
            <div style={{ maxWidth: 1400, marginTop: 20 }}>
              <KineticText
                text="What if your tool could predict conflicts before they happen?"
                startFrame={35}
                currentFrame={frame}
                fontSize={42}
                color="rgba(255,255,255,0.85)"
                fontWeight="500"
                staggerDelay={4}
                highlightWords={["predict", "conflicts"]}
                highlightColor="rgba(0, 150, 255, 0.15)"
                style={{ lineHeight: 1.5, textAlign: "center" }}
              />
            </div>
          )}
        </div>
      </div>

      <SoundFX type="whoosh" startFrame={0} />
      <SoundFX type="whoosh" startFrame={50} volume={0.5} />
    </AbsoluteFill>
  );
};
