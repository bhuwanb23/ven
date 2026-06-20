import React from "react";
import {
  AbsoluteFill,
  useCurrentFrame,
  interpolate,
  spring,
  staticFile,
  Img,
} from "remotion";
import { Cursor } from "../components/Cursor";
import { DependencyGraph } from "../components/DependencyGraph";
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";
import { SoundFX } from "../components/SoundFX";

const centerX = 960;
const centerY = 500;
const radius = 420;

const langNodes = [
  { label: "Node.js", angle: -Math.PI / 2 },
  { label: "Python", angle: -Math.PI / 6 },
  { label: "Java", angle: Math.PI / 6 },
  { label: "Go", angle: Math.PI / 2 },
  { label: "Rust", angle: 5 * Math.PI / 6 },
  { label: "PHP", angle: -5 * Math.PI / 6 },
].map((n) => ({
  x: centerX + radius * Math.cos(n.angle),
  y: centerY + radius * Math.sin(n.angle),
  label: n.label,
  color: "rgba(0, 200, 255, 0.4)",
  size: 60,
}));

const graphNodes = [
  { x: centerX, y: centerY, label: "ven", color: "#00c8ff", size: 120 },
  ...langNodes,
];

const graphEdges = langNodes.map((_, i) => ({
  from: 0,
  to: i + 1,
}));

const subNodes = [
  { x: 500, y: 700, label: "Express", color: "rgba(0, 200, 255, 0.5)", size: 40 },
  { x: 380, y: 620, label: "Next.js", color: "rgba(0, 200, 255, 0.5)", size: 40 },
  { x: 620, y: 620, label: "Prisma", color: "rgba(0, 200, 255, 0.5)", size: 40 },
];

const subEdges = [
  { from: 0, to: 1 },
  { from: 0, to: 2 },
];

export const Beat3Graph: React.FC = () => {
  const frame = useCurrentFrame();

  const logoProgress = spring({
    frame: Math.max(0, frame - 10),
    fps: 30,
    config: { damping: 18, stiffness: 130, mass: 0.5 },
  });

  const logoGlow = interpolate(logoProgress, [0.5, 1], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const nodePulse = 1 + 0.04 * Math.sin(frame * 0.04);

  const allGreenProgress = spring({
    frame: Math.max(0, frame - 300),
    fps: 30,
    config: { damping: 12, stiffness: 180, mass: 0.4 },
  });

  const dataPulse = (frame * 0.02) % 1;

  return (
    <AbsoluteFill
      style={{
        background:
          "linear-gradient(180deg, #080f0f 0%, #0a1414 30%, #131313 100%)",
      }}
    >
      {/* Dot grid background */}
      <svg style={{ position: "absolute", inset: 0, width: "100%", height: "100%", pointerEvents: "none" }}>
        <defs>
          <pattern id="g3" width="48" height="48" patternUnits="userSpaceOnUse">
            <circle cx="24" cy="24" r="0.8" fill="rgba(0, 200, 255, 0.12)" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#g3)" />
      </svg>

      {/* Data-pulse dots traveling along graph edges */}
      <svg style={{ position: "absolute", inset: 0, width: "100%", height: "100%", pointerEvents: "none", zIndex: 5 }}>
        {graphEdges.map((_, i) => {
          const t = (dataPulse + i * 0.22) % 1;
          const target = langNodes[i];
          const px = centerX + t * (target.x - centerX);
          const py = centerY + t * (target.y - centerY);
          return (
            <circle
              key={i}
              cx={px}
              cy={py}
              r={3 + Math.sin(frame * 0.1 + i * 2) * 1}
              fill="#4ade80"
              opacity={Math.sin(t * Math.PI) * 0.5}
            />
          );
        })}
        {/* Sub-graph data pulse */}
        {frame >= 270 && subEdges.map((edge, i) => {
          const t = ((frame - 270) * 0.025 + i * 0.35) % 1;
          const from = subNodes[edge.from];
          const to = subNodes[edge.to];
          const px = from.x + t * (to.x - from.x);
          const py = from.y + t * (to.y - from.y);
          return (
            <circle
              key={`sub-${i}`}
              cx={px}
              cy={py}
              r={2.5}
              fill="#4ade80"
              opacity={Math.sin(t * Math.PI) * 0.4}
            />
          );
        })}
      </svg>

      <ParticleBg count={50} color="0, 180, 120" baseOpacity={0.05} />

      <div
        style={{
          position: "absolute",
          top: 100,
          left: "50%",
          transform: "translateX(-50%)",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          zIndex: 10,
        }}
      >
        <Img
          src={staticFile("Ven_logo.png")}
          style={{
            width: 60 * logoProgress,
            height: 60 * logoProgress,
            opacity: logoProgress,
            filter: `drop-shadow(0 0 ${30 * logoGlow}px rgba(0, 200, 255, ${0.4 * logoGlow}))`,
          }}
        />

        <div style={{ marginTop: 20 }}>
          <KineticText
            text="One tool to rule them all"
            startFrame={45}
            currentFrame={frame}
            fontSize={22}
            color="rgba(255,255,255,0.4)"
            fontWeight="400"
            staggerDelay={5}
            letterSpacing={3}
          />
        </div>
      </div>

      <div
        style={{
          position: "absolute",
          inset: 0,
          transform: `scale(${nodePulse})`,
        }}
      >
        <DependencyGraph
          nodes={graphNodes}
          edges={graphEdges}
          startFrame={90}
          currentFrame={frame}
          nodeStagger={8}
          edgeStagger={4}
          centerIndex={0}
        />
      </div>

      {frame >= 270 && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            transform: `scale(${1 + 0.03 * Math.sin(frame * 0.05 + 1)})`,
          }}
        >
          <DependencyGraph
            nodes={subNodes}
            edges={subEdges}
            startFrame={270}
            currentFrame={frame}
            nodeStagger={5}
            edgeStagger={3}
            centerIndex={0}
          />
        </div>
      )}

      <Cursor
        waypoints={[
          { x: centerX + 100, y: centerY + 100, frame: 0 },
          { x: langNodes[1].x + 20, y: langNodes[1].y + 10, frame: 220 },
          { x: langNodes[1].x + 20, y: langNodes[1].y + 10, frame: 285 },
          { x: centerX + 100, y: centerY + 100, frame: 300 },
          { x: centerX + 100, y: centerY + 100, frame: 340 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 275, x: langNodes[1].x, y: langNodes[1].y },
        ]}
        showTrail
      />

      {frame >= 300 && (
        <div
          style={{
            position: "absolute",
            bottom: 120,
            left: 0,
            right: 0,
            textAlign: "center",
            opacity: allGreenProgress,
          }}
        >
          <div
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 14,
              background: "rgba(74, 222, 128, 0.12)",
              border: "1px solid rgba(74, 222, 128, 0.3)",
              borderRadius: 14,
              padding: "18px 40px",
              color: "#4ade80",
              fontSize: 26,
              fontFamily: "Inter, sans-serif",
              fontWeight: "600",
            }}
          >
            <span>All dependencies compatible</span>
            <span style={{ fontSize: 30 }}>✓</span>
          </div>
        </div>
      )}

      <SoundFX type="whoosh" startFrame={0} />
      <SoundFX type="whoosh" startFrame={90} volume={0.5} />
      <SoundFX type="click" startFrame={275} />
      <SoundFX type="success" startFrame={300} />
    </AbsoluteFill>
  );
};
