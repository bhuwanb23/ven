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
import { SoundFX } from "../components/SoundFX";

const panelLeft = 180;
const panelTop = 180;
const panelW = 760;
const panelH = 600;
const panelGap = 40;
const panelRight = panelLeft + panelW + panelGap;

const dirs = [
  { name: "frontend/", version: "v20.20.2", y: 80 },
  { name: "backend/", version: "v22.11.0", y: 130 },
  { name: "shared/", version: "v1.5.0", y: 180 },
  { name: "packages/", version: "v3.2.1", y: 230 },
];

const VersionRow: React.FC<{
  dir: typeof dirs[0];
  showVersion: boolean;
  progress: number;
}> = ({ dir, showVersion, progress }) => {
  const yOffset = interpolate(progress, [0, 0.3, 1], [24, -6, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 20,
        padding: "14px 18px",
        borderRadius: 8,
        background:
          showVersion && progress > 0
            ? "rgba(168, 85, 247, 0.08)"
            : "transparent",
        opacity: showVersion
          ? interpolate(progress, [0, 0.5, 1], [0, 0.3, 1])
          : 0.15,
        transform: showVersion ? `translateY(${yOffset}px)` : "translateY(0)",
      }}
    >
      <span
        style={{
          fontFamily: "monospace",
          fontSize: 18,
          color: "rgba(255,255,255,0.5)",
          minWidth: 160,
        }}
      >
        {dir.name}
      </span>
      {showVersion && (
        <span
          style={{
            fontFamily: "monospace",
            fontSize: 30,
            fontWeight: "700",
            color: "#a855f7",
          }}
        >
          {dir.version}
        </span>
      )}
      {!showVersion && (
        <span
          style={{
            fontFamily: "monospace",
            fontSize: 18,
            color: "rgba(255,255,255,0.12)",
          }}
        >
          ---
        </span>
      )}
    </div>
  );
};

export const Beat4Autoswitch: React.FC = () => {
  const frame = useCurrentFrame();

  const panelSlide = spring({
    frame: Math.max(0, frame - 8),
    fps: 30,
    config: { damping: 22, stiffness: 110, mass: 0.6 },
  });
  const panelOpacity = interpolate(panelSlide, [0, 1], [0, 1]);

  const isActive = (i: number) => {
    if (i === 0) return frame >= 60 && frame < 120;
    if (i === 1) return frame >= 120 && frame < 185;
    return false;
  };

  return (
    <AbsoluteFill
      style={{
        background:
          "linear-gradient(180deg, #10081a 0%, #160e24 30%, #131313 100%)",
      }}
    >
      <ParticleBg count={25} color="168, 85, 247" baseOpacity={0.06} />

      {/* Left Panel - Directory */}
      <div
        style={{
          position: "absolute",
          left: panelLeft,
          top: panelTop,
          width: panelW,
          height: panelH,
          borderRadius: 14,
          border: "1px solid rgba(168, 85, 247, 0.1)",
          background: "rgba(168, 85, 247, 0.035)",
          padding: "30px 34px",
          opacity: panelOpacity,
          transform: `translateX(${(1 - panelSlide) * -30}px)`,
        }}
      >
        <div
          style={{
            fontFamily: "Inter, sans-serif",
            fontSize: 16,
            color: "rgba(168, 85, 247, 0.5)",
            marginBottom: 24,
            letterSpacing: 2,
            textTransform: "uppercase",
          }}
        >
          PROJECT
        </div>
        <div style={{ position: "relative" }}>
          {dirs.map((dir, i) => {
            const active = isActive(i);
            return (
              <div
                key={dir.name}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 14,
                  padding: "12px 16px",
                  borderRadius: 8,
                  background: active
                    ? "rgba(168, 85, 247, 0.12)"
                    : "transparent",
                  border: active
                    ? "1px solid rgba(168, 85, 247, 0.3)"
                    : "1px solid transparent",
                  marginBottom: 6,
                }}
              >
                <span
                  style={{
                    fontSize: 20,
                    color: active ? "#a855f7" : "rgba(255,255,255,0.2)",
                  }}
                >
                  📁
                </span>
                <span
                  style={{
                    fontFamily: "monospace",
                    fontSize: 20,
                    color: active
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
          borderRadius: 14,
          border: "1px solid rgba(168, 85, 247, 0.1)",
          background: "rgba(168, 85, 247, 0.035)",
          padding: "30px 34px",
          opacity: panelOpacity,
          transform: `translateX(${(1 - panelSlide) * 30}px)`,
        }}
      >
        <div
          style={{
            fontFamily: "Inter, sans-serif",
            fontSize: 16,
            color: "rgba(168, 85, 247, 0.5)",
            marginBottom: 24,
            letterSpacing: 2,
            textTransform: "uppercase",
          }}
        >
          NODE VERSION
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {dirs.map((dir, i) => {
            const showVersion = i === 0
              ? frame >= 80
              : i === 1
                ? frame >= 140
                : false;
            const progress = i === 0
              ? spring({ frame: Math.max(0, frame - 70), fps: 30, config: { damping: 16, stiffness: 160, mass: 0.4 } })
              : i === 1
                ? spring({ frame: Math.max(0, frame - 130), fps: 30, config: { damping: 16, stiffness: 160, mass: 0.4 } })
                : 1;
            return (
              <VersionRow
                key={dir.name}
                dir={dir}
                showVersion={showVersion}
                progress={progress}
              />
            );
          })}
        </div>
      </div>

      {/* Cursor */}
      <Cursor
        waypoints={[
          { x: panelRight + panelW + 40, y: panelTop + 100, frame: 0 },
          { x: panelLeft + 380, y: panelTop + 112, frame: 50 },
          { x: panelLeft + 380, y: panelTop + 112, frame: 80 },
          { x: panelLeft + 380, y: panelTop + 162, frame: 110 },
          { x: panelLeft + 380, y: panelTop + 162, frame: 140 },
          { x: panelRight + panelW, y: panelTop + 300, frame: 170 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 70, x: panelLeft + 380, y: panelTop + 112 },
          { frame: 130, x: panelLeft + 380, y: panelTop + 162 },
        ]}
        showTrail
      />

      {/* Subtitle */}
      {frame >= 170 && (
        <div
          style={{
            position: "absolute",
            bottom: 120,
            left: 0,
            right: 0,
            textAlign: "center",
          }}
        >
          <KineticText
            text="Auto-switch versions. Seamlessly."
            startFrame={170}
            currentFrame={frame}
            fontSize={28}
            color="rgba(255,255,255,0.5)"
            fontWeight="400"
            staggerDelay={5}
            highlightWords={["Auto-switch"]}
            highlightColor="rgba(168, 85, 247, 0.15)"
          />
        </div>
      )}

      <SoundFX type="whoosh" startFrame={0} />
      <SoundFX type="click" startFrame={70} />
      <SoundFX type="click" startFrame={130} />
    </AbsoluteFill>
  );
};
