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

const centerX = 960;
const centerY = 500;
const radius = 280;

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
  size: 30,
}));

const graphNodes = [
  { x: centerX, y: centerY, label: "ven", color: "#00c8ff", size: 55 },
  ...langNodes,
];

const graphEdges = langNodes.map((_, i) => ({
  from: 0,
  to: i + 1,
}));

const subNodes = [
  { x: centerX - 180, y: centerY - 200, label: "Express", color: "rgba(0, 200, 255, 0.5)", size: 22 },
  { x: centerX + 180, y: centerY - 200, label: "Next.js", color: "rgba(0, 200, 255, 0.5)", size: 22 },
  { x: centerX, y: centerY - 320, label: "Prisma", color: "rgba(0, 200, 255, 0.5)", size: 22 },
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

  const allGreenProgress = spring({
    frame: Math.max(0, frame - 300),
    fps: 30,
    config: { damping: 12, stiffness: 180, mass: 0.4 },
  });

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <ParticleBg count={40} />

      <div
        style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
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

        <div style={{ marginTop: 24 }}>
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

      <DependencyGraph
        nodes={graphNodes}
        edges={graphEdges}
        startFrame={90}
        currentFrame={frame}
        nodeStagger={8}
        edgeStagger={4}
        centerIndex={0}
      />

      {frame >= 270 && (
        <DependencyGraph
          nodes={subNodes}
          edges={subEdges}
          startFrame={270}
          currentFrame={frame}
          nodeStagger={5}
          edgeStagger={3}
          centerIndex={0}
        />
      )}

      <Cursor
        waypoints={[
          { x: centerX + 80, y: centerY + 80, frame: 0 },
          { x: centerX + radius + 20, y: centerY - radius + 10, frame: 220 },
          { x: centerX + radius + 20, y: centerY - radius + 10, frame: 285 },
          { x: centerX + 80, y: centerY + 80, frame: 300 },
          { x: centerX + 80, y: centerY + 80, frame: 340 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 275, x: langNodes[0].x, y: langNodes[0].y },
        ]}
        showTrail
      />

      {frame >= 300 && (
        <div
          style={{
            position: "absolute",
            bottom: 140,
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
              gap: 12,
              background: "rgba(74, 222, 128, 0.1)",
              border: "1px solid rgba(74, 222, 128, 0.3)",
              borderRadius: 12,
              padding: "14px 32px",
              color: "#4ade80",
              fontSize: 22,
              fontFamily: "Inter, sans-serif",
              fontWeight: "600",
            }}
          >
            <span>All dependencies compatible</span>
            <span style={{ fontSize: 26 }}>✓</span>
          </div>
        </div>
      )}
    </AbsoluteFill>
  );
};
