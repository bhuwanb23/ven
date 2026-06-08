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
].map((n, i) => ({
  x: centerX + radius * Math.cos(n.angle),
  y: centerY + radius * Math.sin(n.angle),
  label: n.label,
  color: i === 0 ? "#00c8ff" : "rgba(0, 200, 255, 0.4)",
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
  { from: 1, to: 0 },
  { from: 1, to: 1 },
  { from: 1, to: 2 },
];

export const Beat3Graph: React.FC = () => {
  const frame = useCurrentFrame();

  const logoProgress = spring({
    frame: Math.max(0, frame - 10),
    fps: 30,
    config: { damping: 16, stiffness: 150, mass: 0.4 },
  });

  const logoGlow = interpolate(logoProgress, [0.5, 1], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const subtitleFade = interpolate(frame, [210, 230], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const allGreenStart = 190;
  const allGreenProgress = spring({
    frame: Math.max(0, frame - allGreenStart),
    fps: 30,
    config: { damping: 10, stiffness: 200, mass: 0.5 },
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
          gap: 0,
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

        <div
          style={{
            opacity: subtitleFade,
            marginTop: 20,
          }}
        >
          <KineticText
            text="One tool to rule them all"
            startFrame={30}
            currentFrame={frame}
            fontSize={24}
            color="rgba(255,255,255,0.4)"
            fontWeight="400"
            staggerDelay={3}
            letterSpacing={3}
          />
        </div>
      </div>

      <DependencyGraph
        nodes={graphNodes}
        edges={graphEdges}
        startFrame={60}
        currentFrame={frame}
        nodeStagger={6}
        edgeStagger={3}
        centerIndex={0}
      />

      {frame >= 140 && (
        <DependencyGraph
          nodes={subNodes}
          edges={subEdges}
          startFrame={140}
          currentFrame={frame}
          nodeStagger={3}
          edgeStagger={2}
          centerIndex={0}
        />
      )}

      <Cursor
        waypoints={[
          { x: centerX + 50, y: centerY + 60, frame: 0 },
          { x: centerX + radius + 40, y: centerY - radius - 10, frame: 110 },
          { x: centerX + 50, y: centerY + 60, frame: 135 },
          { x: centerX + 50, y: centerY + 60, frame: 180 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 115, x: langNodes[0].x, y: langNodes[0].y },
        ]}
        showTrail
      />

      {frame >= allGreenStart && (
        <div
          style={{
            position: "absolute",
            bottom: 160,
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
              padding: "12px 28px",
              color: "#4ade80",
              fontSize: 20,
              fontFamily: "Inter, sans-serif",
              fontWeight: "600",
            }}
          >
            <span>All dependencies compatible</span>
            <span style={{ fontSize: 24 }}>✓</span>
          </div>
        </div>
      )}
    </AbsoluteFill>
  );
};
