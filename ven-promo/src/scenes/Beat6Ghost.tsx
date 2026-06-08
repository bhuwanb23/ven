import React from "react";
import {
  AbsoluteFill,
  useCurrentFrame,
  interpolate,
  spring,
} from "remotion";
import { Cursor } from "../components/Cursor";
import { KineticText } from "../components/KineticText";
import { ConnectionLine } from "../components/ConnectionLine";
import { ParticleBg } from "../components/ParticleBg";

const ghostX = 720;
const ghostY = 380;
const centerX = 960;
const centerY = 540;

const normalNodes = [
  { x: 800, y: 540, label: "Node.js" },
  { x: 960, y: 400, label: "Python" },
  { x: 1100, y: 480, label: "Go" },
  { x: 1040, y: 620, label: "Rust" },
];

const edges = normalNodes.map((n, i) => ({
  startX: centerX,
  startY: centerY,
  endX: n.x,
  endY: n.y,
  startFrame: 10 + i * 5,
}));

const ghostEdges = [
  {
    startX: centerX,
    startY: centerY,
    endX: ghostX,
    endY: ghostY,
    startFrame: 30,
  },
];

export const Beat6Ghost: React.FC = () => {
  const frame = useCurrentFrame();

  const ghostPulse = 0.4 + 0.6 * Math.sin(frame * 0.12);

  const scanStart = 65;
  const scanProgress = spring({
    frame: Math.max(0, frame - scanStart),
    fps: 30,
    config: { damping: 8, stiffness: 60, mass: 0.8 },
    durationInFrames: 50,
  });

  const ghostToGreen = spring({
    frame: Math.max(0, frame - 100),
    fps: 30,
    config: { damping: 12, stiffness: 150, mass: 0.5 },
  });

  const scanX = interpolate(scanProgress, [0, 1], [ghostX - 200, ghostX + 200], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const scanOpacity = interpolate(scanProgress, [0, 0.3, 0.7, 1], [0, 0.8, 0.8, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const ghostColor = ghostToGreen > 0.5
    ? `rgba(74, 222, 128, ${0.3 + 0.7 * ghostToGreen})`
    : `rgba(255, 80, 80, ${ghostPulse})`;

  const ghostFill = ghostToGreen > 0.5
    ? `rgba(74, 222, 128, ${0.15 * ghostToGreen})`
    : `rgba(255, 80, 80, ${0.1 * ghostPulse})`;

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <ParticleBg count={25} />

      <svg
        width="1920"
        height="1080"
        style={{ position: "absolute", inset: 0 }}
      >
        {edges.map((e, i) => (
          <ConnectionLine
            key={i}
            startX={e.startX}
            startY={e.startY}
            endX={e.endX}
            endY={e.endY}
            startFrame={e.startFrame}
            currentFrame={frame}
            color="rgba(0, 200, 255, 0.2)"
            strokeWidth={1.5}
          />
        ))}
        {ghostEdges.map((e, i) => (
          <ConnectionLine
            key={`ghost-${i}`}
            startX={e.startX}
            startY={e.startY}
            endX={e.endX}
            endY={e.endY}
            startFrame={e.startFrame}
            currentFrame={frame}
            color={
              ghostToGreen > 0.5
                ? "rgba(74, 222, 128, 0.3)"
                : `rgba(255, 80, 80, ${0.2 * ghostPulse})`
            }
            strokeWidth={1.5}
          />
        ))}
      </svg>

      {/* Center node */}
      <div
        style={{
          position: "absolute",
          left: centerX - 30,
          top: centerY - 30,
          width: 60,
          height: 60,
          borderRadius: "50%",
          background: "rgba(0, 200, 255, 0.12)",
          border: "2px solid rgba(0, 200, 255, 0.4)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontFamily: "Inter, sans-serif",
          fontSize: 13,
          fontWeight: "600",
          color: "#00c8ff",
        }}
      >
        ven
      </div>

      {/* Ghost node */}
      <div
        style={{
          position: "absolute",
          left: ghostX - 35,
          top: ghostY - 35,
          width: 70,
          height: 70,
          borderRadius: "50%",
          background: ghostFill,
          border: `2px solid ${ghostColor}`,
          boxShadow: ghostToGreen > 0.5
            ? `0 0 30px rgba(74, 222, 128, ${0.2 * ghostToGreen})`
            : `0 0 ${15 + 20 * ghostPulse}px rgba(255, 80, 80, ${0.2 * ghostPulse})`,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexDirection: "column",
          opacity: ghostToGreen > 0.9 ? 1 : 1,
          transition: "none",
        }}
      >
        {ghostToGreen > 0.5 ? (
          <span style={{ color: "#4ade80", fontSize: 18 }}>✓</span>
        ) : (
          <>
            <span
              style={{
                color: `rgba(255, 80, 80, ${ghostPulse})`,
                fontSize: 16,
                fontWeight: "700",
              }}
            >
              ??
            </span>
            <span
              style={{
                color: `rgba(255, 80, 80, ${0.4 * ghostPulse})`,
                fontSize: 9,
                marginTop: 2,
              }}
            >
              GHOST
            </span>
          </>
        )}
      </div>

      {/* Scan wave */}
      {frame >= scanStart && frame < scanStart + 50 && (
        <div
          style={{
            position: "absolute",
            left: scanX - 3,
            top: ghostY - 200,
            width: 6,
            height: 400,
            background: `linear-gradient(180deg, transparent 0%, rgba(0, 200, 255, ${scanOpacity}) 50%, transparent 100%)`,
            opacity: scanOpacity,
            filter: "blur(4px)",
          }}
        />
      )}

      {/* Normal node labels */}
      {normalNodes.map((node, i) => {
        const nodeFrame = 10 + i * 5;
        const nodeOpacity = spring({
          frame: Math.max(0, frame - nodeFrame),
          fps: 30,
          config: { damping: 20, stiffness: 100 },
        });

        return (
          <div
            key={node.label}
            style={{
              position: "absolute",
              left: node.x - 15,
              top: node.y + 20,
              textAlign: "center",
              fontFamily: "Inter, sans-serif",
              fontSize: 11,
              color: "rgba(255,255,255,0.35)",
              opacity: 0.8 * nodeOpacity,
            }}
          >
            {node.label}
          </div>
        );
      })}

      {/* Ghost label */}
      <div
        style={{
          position: "absolute",
          left: ghostX - 30,
          top: ghostY + 45,
          textAlign: "center",
          fontFamily: "Inter, sans-serif",
          fontSize: 11,
          color: ghostToGreen > 0.5 ? "rgba(74, 222, 128, 0.5)" : "rgba(255, 80, 80, 0.4)",
          opacity: 0.8,
        }}
      >
        ghost-dep
      </div>

      {/* Cursor */}
      <Cursor
        waypoints={[
          { x: 1400, y: 800, frame: 0 },
          { x: ghostX + 50, y: ghostY - 20, frame: 50 },
          { x: ghostX + 200, y: ghostY - 200, frame: 75 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 62, x: ghostX, y: ghostY },
        ]}
        showTrail
      />

      {/* Closing text */}
      {frame >= 115 && (
        <div
          style={{
            position: "absolute",
            bottom: 140,
            left: 0,
            right: 0,
            textAlign: "center",
          }}
        >
          <KineticText
            text="Ghost detection. Zero surprises."
            startFrame={115}
            currentFrame={frame}
            fontSize={28}
            color="rgba(255,255,255,0.6)"
            fontWeight="500"
            staggerDelay={4}
            highlightWords={["Ghost", "Zero"]}
            highlightColor="rgba(0, 200, 255, 0.15)"
          />
        </div>
      )}
    </AbsoluteFill>
  );
};
