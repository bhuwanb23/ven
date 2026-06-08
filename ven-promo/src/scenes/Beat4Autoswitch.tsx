import React from "react";
import {
  AbsoluteFill,
  useCurrentFrame,
  interpolate,
  spring,
} from "remotion";
import { Cursor } from "../components/Cursor";
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";

const panelLeft = 260;
const panelTop = 220;
const panelW = 580;
const panelH = 520;
const panelGap = 80;
const panelRight = panelLeft + panelW + panelGap;

const dirs = [
  { name: "frontend/", version: "v20.20.2", y: 80 },
  { name: "backend/", version: "v22.11.0", y: 130 },
  { name: "shared/", version: "v1.5.0", y: 180 },
  { name: "packages/", version: "v3.2.1", y: 230 },
];

export const Beat4Autoswitch: React.FC = () => {
  const frame = useCurrentFrame();

  const panelSlide = spring({
    frame: Math.max(0, frame - 5),
    fps: 30,
    config: { damping: 20, stiffness: 120, mass: 0.6 },
  });
  const panelOpacity = interpolate(panelSlide, [0, 1], [0, 1]);

  const showVersion = (dirIndex: number): boolean => {
    if (dirIndex === 0) return frame >= 70;
    if (dirIndex === 1) return frame >= 100;
    return frame >= 130;
  };

  const versionSwitchProgress = (dirIndex: number) => {
    if (dirIndex === 0) {
      return spring({
        frame: Math.max(0, frame - 60),
        fps: 30,
        config: { damping: 14, stiffness: 180, mass: 0.5 },
      });
    }
    if (dirIndex === 1) {
      return spring({
        frame: Math.max(0, frame - 90),
        fps: 30,
        config: { damping: 14, stiffness: 180, mass: 0.5 },
      });
    }
    return 1;
  };

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <ParticleBg count={25} />

      {/* Left Panel - Directory */}
      <div
        style={{
          position: "absolute",
          left: panelLeft,
          top: panelTop,
          width: panelW,
          height: panelH,
          borderRadius: 12,
          border: "1px solid rgba(255,255,255,0.08)",
          background: "rgba(255,255,255,0.03)",
          padding: "28px 30px",
          opacity: panelOpacity,
          transform: `translateX(${(1 - panelSlide) * -30}px)`,
        }}
      >
        <div
          style={{
            fontFamily: "Inter, sans-serif",
            fontSize: 14,
            color: "rgba(255,255,255,0.3)",
            marginBottom: 20,
            letterSpacing: 1,
            textTransform: "uppercase",
          }}
        >
          PROJECT
        </div>
        <div style={{ position: "relative" }}>
          {dirs.map((dir, i) => {
            const isActive = i === 0
              ? frame >= 60 && frame < 90
              : i === 1
                ? frame >= 90
                : false;

            return (
              <div
                key={dir.name}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 12,
                  padding: "8px 12px",
                  borderRadius: 6,
                  background: isActive
                    ? "rgba(0, 200, 255, 0.1)"
                    : "transparent",
                  border: isActive
                    ? "1px solid rgba(0, 200, 255, 0.2)"
                    : "1px solid transparent",
                  marginBottom: 4,
                  transition: "none",
                }}
              >
                <span
                  style={{
                    fontSize: 16,
                    color: isActive ? "#00c8ff" : "rgba(255,255,255,0.2)",
                  }}
                >
                  📁
                </span>
                <span
                  style={{
                    fontFamily: "monospace",
                    fontSize: 16,
                    color: isActive
                      ? "#ffffff"
                      : "rgba(255,255,255,0.5)",
                  }}
                >
                  {dir.name}
                </span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Right Panel - Version Display */}
      <div
        style={{
          position: "absolute",
          left: panelRight,
          top: panelTop,
          width: panelW,
          height: panelH,
          borderRadius: 12,
          border: "1px solid rgba(255,255,255,0.08)",
          background: "rgba(255,255,255,0.03)",
          padding: "28px 30px",
          opacity: panelOpacity,
          transform: `translateX(${(1 - panelSlide) * 30}px)`,
        }}
      >
        <div
          style={{
            fontFamily: "Inter, sans-serif",
            fontSize: 14,
            color: "rgba(255,255,255,0.3)",
            marginBottom: 20,
            letterSpacing: 1,
            textTransform: "uppercase",
          }}
        >
          NODE VERSION
        </div>
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 8,
          }}
        >
          {dirs.map((dir, i) => {
            const shown = showVersion(i);
            const progress = versionSwitchProgress(i);

            const yOffset = interpolate(progress, [0, 0.3, 1], [20, -10, 0], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            });

            return (
              <div
                key={dir.name}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 16,
                  padding: "10px 14px",
                  borderRadius: 8,
                  background:
                    shown && progress > 0
                      ? "rgba(0, 200, 255, 0.06)"
                      : "transparent",
                  opacity: shown ? interpolate(progress, [0, 0.5, 1], [0, 0.3, 1]) : 0.15,
                  transform: shown ? `translateY(${yOffset}px)` : "translateY(0)",
                }}
              >
                <span
                  style={{
                    fontFamily: "monospace",
                    fontSize: 14,
                    color: "rgba(255,255,255,0.4)",
                    minWidth: 120,
                  }}
                >
                  {dir.name}
                </span>
                {shown && (
                  <span
                    style={{
                      fontFamily: "monospace",
                      fontSize: 22,
                      fontWeight: "700",
                      color: "#00c8ff",
                    }}
                  >
                    {dir.version}
                  </span>
                )}
                {!shown && (
                  <span
                    style={{
                      fontFamily: "monospace",
                      fontSize: 14,
                      color: "rgba(255,255,255,0.15)",
                    }}
                  >
                    ---
                  </span>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Cursor */}
      <Cursor
        waypoints={[
          { x: panelLeft + panelW + 20, y: panelTop + 30, frame: 0 },
          { x: panelLeft + 300, y: panelTop + 112, frame: 50 },
          { x: panelLeft + 300, y: panelTop + 112, frame: 72 },
          { x: panelLeft + 300, y: panelTop + 162, frame: 82 },
          { x: panelLeft + 300, y: panelTop + 162, frame: 102 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 60, x: panelLeft + 300, y: panelTop + 112 },
          { frame: 90, x: panelLeft + 300, y: panelTop + 162 },
        ]}
        showTrail
      />

      {/* Subtitle */}
      {frame >= 110 && (
        <div
          style={{
            position: "absolute",
            bottom: 160,
            left: 0,
            right: 0,
            textAlign: "center",
          }}
        >
          <KineticText
            text="Auto-switch versions. Seamlessly."
            startFrame={110}
            currentFrame={frame}
            fontSize={26}
            color="rgba(255,255,255,0.5)"
            fontWeight="400"
            staggerDelay={3}
            highlightWords={["Auto-switch"]}
            highlightColor="rgba(0, 200, 255, 0.15)"
          />
        </div>
      )}
    </AbsoluteFill>
  );
};
