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
const panelTop = 200;
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
    frame: Math.max(0, frame - 10),
    fps: 30,
    config: { damping: 20, stiffness: 100, mass: 0.7 },
  });
  const panelOpacity = interpolate(panelSlide, [0, 1], [0, 1]);

  const versionProgress = (targetFrame: number) =>
    spring({
      frame: Math.max(0, frame - targetFrame),
      fps: 30,
      config: { damping: 14, stiffness: 180, mass: 0.5 },
    });

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
          transform: `translateX(${(1 - panelSlide) * -40}px)`,
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
            const isFrontendActive = i === 0 && frame >= 60 && frame < 90;
            const isBackendActive = i === 1 && frame >= 100;

            return (
              <div
                key={dir.name}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 12,
                  padding: "8px 12px",
                  borderRadius: 6,
                  background: isFrontendActive || isBackendActive
                    ? "rgba(0, 200, 255, 0.1)"
                    : "transparent",
                  border: isFrontendActive || isBackendActive
                    ? "1px solid rgba(0, 200, 255, 0.2)"
                    : "1px solid transparent",
                  marginBottom: 4,
                }}
              >
                <span
                  style={{
                    fontSize: 16,
                    color: isFrontendActive || isBackendActive
                      ? "#00c8ff" : "rgba(255,255,255,0.2)",
                  }}
                >
                  📁
                </span>
                <span
                  style={{
                    fontFamily: "monospace",
                    fontSize: 16,
                    color: isFrontendActive || isBackendActive
                      ? "#ffffff" : "rgba(255,255,255,0.5)",
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
          transform: `translateX(${(1 - panelSlide) * 40}px)`,
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
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {dirs.map((dir, i) => {
            const showVersion = (i === 0 && frame >= 75) || (i === 1 && frame >= 115);
            const progress = i === 0
              ? versionProgress(65)
              : i === 1
                ? versionProgress(105)
                : 0;

            return (
              <div
                key={dir.name}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 16,
                  padding: "10px 14px",
                  borderRadius: 8,
                  background: showVersion && progress > 0
                    ? "rgba(0, 200, 255, 0.06)"
                    : "transparent",
                  opacity: i <= 1
                    ? showVersion
                      ? interpolate(progress, [0, 0.5, 1], [0, 0.4, 1])
                      : 0.15
                    : 0.12,
                  transform: showVersion
                    ? `translateY(${interpolate(progress, [0, 0.3, 1], [20, -10, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })}px)`
                    : "translateY(0)",
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
                {showVersion ? (
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
                ) : (
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
          { x: panelRight + panelW - 60, y: panelTop + 30, frame: 0 },
          { x: panelLeft + 300, y: panelTop + 112, frame: 55 },
          { x: panelLeft + 300, y: panelTop + 112, frame: 78 },
          { x: panelLeft + 300, y: panelTop + 162, frame: 90 },
          { x: panelLeft + 300, y: panelTop + 162, frame: 115 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 70, x: panelLeft + 300, y: panelTop + 112 },
          { frame: 110, x: panelLeft + 300, y: panelTop + 162 },
        ]}
        showTrail
      />

      {/* Subtitle */}
      {frame >= 135 && (
        <div
          style={{
            position: "absolute",
            bottom: 170,
            left: 0,
            right: 0,
            textAlign: "center",
          }}
        >
          <KineticText
            text="Auto-switch versions. Seamlessly."
            startFrame={135}
            currentFrame={frame}
            fontSize={26}
            color="rgba(255,255,255,0.5)"
            fontWeight="400"
            staggerDelay={5}
            highlightWords={["Auto-switch"]}
            highlightColor="rgba(0, 200, 255, 0.15)"
          />
        </div>
      )}
    </AbsoluteFill>
  );
};
