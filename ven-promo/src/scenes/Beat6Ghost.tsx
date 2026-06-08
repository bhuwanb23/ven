import React from "react";
import {
  AbsoluteFill,
  useCurrentFrame,
  interpolate,
  spring,
} from "remotion";
import { Cursor } from "../components/Cursor";
import { ParticleBg } from "../components/ParticleBg";

const width = 1920;

const ghostX = 700;
const ghostY = 240;
const ghostW = 520;
const ghostH = 380;

const GhostBox: React.FC<{
  label: string;
  x: number;
  y: number;
  revealed: boolean;
  progress: number;
}> = ({ label, x, y, revealed, progress }) => {
  const slideY = interpolate(progress, [0, 0.4, 1], [-16, 4, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: y,
        width: 480,
        padding: "16px 20px",
        borderRadius: 10,
        border: "1px solid rgba(0, 200, 255, 0.15)",
        background: revealed
          ? `rgba(0, 200, 255, ${0.04 * progress})`
          : "rgba(255,255,255,0.02)",
        transform: revealed ? `translateY(${slideY}px)` : "translateY(-8px)",
        opacity: revealed ? progress : 0.3,
      }}
    >
      <span
        style={{
          fontFamily: "monospace",
          fontSize: 15,
          color: revealed
            ? `rgba(0, 200, 255, ${0.5 + 0.5 * progress})`
            : "rgba(255,255,255,0.15)",
        }}
      >
        {revealed ? label : "???"}
      </span>
    </div>
  );
};

const ghostDirs = [
  { label: "node_modules/.cache/ghost", y: ghostY + 0 },
  { label: "venv/lib/.venv-track", y: ghostY + 60 },
  { label: ".npm/_cacache/ghost", y: ghostY + 120 },
  { label: "target/.rustc_info", y: ghostY + 180 },
  { label: ".gradle/ghost-jars", y: ghostY + 240 },
];

export const Beat6Ghost: React.FC = () => {
  const frame = useCurrentFrame();

  const subheadOpacity = interpolate(frame, [0, 30, 205, 240], [0, 0.6, 0.6, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const dotsProgress = spring({
    frame: Math.max(0, frame - 190),
    fps: 30,
    config: { damping: 10, stiffness: 60, mass: 0.8 },
  });

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <ParticleBg count={25} />

      {/* Scan line effect */}
      <div
        style={{
          position: "absolute",
          left: 0,
          top: 0,
          width,
          height: 2,
          background: "linear-gradient(90deg, transparent, rgba(0,200,255,0.15), transparent)",
          filter: "blur(1px)",
          transform: `translateY(${interpolate(frame, [0, 240], [0, ghostH + 320])}px)`,
          opacity: interpolate(frame, [0, 20, 220, 240], [0.6, 0.8, 0.8, 0]),
        }}
      />

      {/* Subhead */}
      <div
        style={{
          position: "absolute",
          top: 120,
          left: 0,
          right: 0,
          textAlign: "center",
          opacity: subheadOpacity,
        }}
      >
        <span
          style={{
            fontFamily: "monospace",
            fontSize: 20,
            color: "rgba(255,255,255,0.5)",
            letterSpacing: 2,
          }}
        >
          Scanning for orphaned environments...
        </span>
      </div>

      {/* Ghost directory lines */}
      <div
        style={{
          position: "absolute",
          left: ghostX,
          top: ghostY + 60,
        }}
      >
        {ghostDirs.map((d, i) => {
          const revealFrame = 50 + i * 25;
          const revealed = frame >= revealFrame;

          let progress = 0;
          if (revealed) {
            progress = spring({
              frame: Math.max(0, frame - revealFrame),
              fps: 30,
              config: { damping: 18, stiffness: 160, mass: 0.4 },
            });
          }

          return (
            <GhostBox
              key={d.label}
              label={d.label}
              x={0}
              y={d.y - ghostY - 60}
              revealed={revealed}
              progress={progress}
            />
          );
        })}
      </div>

      {/* Cursor scans through lines */}
      <Cursor
        waypoints={[
          { x: ghostX + ghostW + 40, y: ghostY + 40, frame: 0 },
          { x: ghostX + ghostW - 20, y: ghostY + 60, frame: 35 },
          { x: ghostX + ghostW - 20, y: ghostY + 60, frame: 55 },
          { x: ghostX + ghostW - 20, y: ghostY + 120, frame: 75 },
          { x: ghostX + ghostW - 20, y: ghostY + 120, frame: 95 },
          { x: ghostX + ghostW - 20, y: ghostY + 180, frame: 115 },
          { x: ghostX + ghostW - 20, y: ghostY + 180, frame: 135 },
          { x: ghostX + ghostW - 20, y: ghostY + 240, frame: 155 },
          { x: ghostX + ghostW - 20, y: ghostY + 240, frame: 175 },
          { x: ghostX + ghostW - 20, y: ghostY + 300, frame: 195 },
          { x: width + 40, y: ghostY + 300, frame: 215 },
        ]}
        currentFrame={frame}
        showTrail
      />

      {/* Result: "0 ghost environments found" */}
      {frame >= 195 && (
        <div
          style={{
            position: "absolute",
            bottom: 160,
            left: 0,
            right: 0,
            textAlign: "center",
          }}
        >
          <div
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 14,
              background: "rgba(74, 222, 128, 0.08)",
              border: "1px solid rgba(74, 222, 128, 0.2)",
              borderRadius: 12,
              padding: "14px 36px",
              transform: `scale(${dotsProgress})`,
            }}
          >
            <span
              style={{
                fontFamily: "monospace",
                fontSize: 24,
                color: "#4ade80",
                fontWeight: "600",
              }}
            >
              0 ghost environments found
            </span>
          </div>
        </div>
      )}
    </AbsoluteFill>
  );
};
