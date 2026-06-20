import React from "react";
import {
  AbsoluteFill,
  useCurrentFrame,
  interpolate,
  spring,
} from "remotion";
import { Cursor } from "../components/Cursor";
import { ParticleBg } from "../components/ParticleBg";
import { SoundFX } from "../components/SoundFX";

const width = 1920;

const ghostX = 560;
const ghostY = 200;
const ghostW = 800;
const ghostH = 480;

const fileSizes = ["2.4 MB", "1.1 MB", "4.7 MB", "0.8 MB", "3.2 MB"];

const GhostBox: React.FC<{
  label: string;
  x: number;
  y: number;
  revealed: boolean;
  progress: number;
  fileSize: string;
}> = ({ label, x, y, revealed, progress, fileSize }) => {
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
        width: 680,
        padding: "18px 24px",
        borderRadius: 10,
        border: "1px solid rgba(56, 189, 248, 0.15)",
        background: revealed
          ? `rgba(56, 189, 248, ${0.05 * progress})`
          : "rgba(255,255,255,0.02)",
        transform: revealed ? `translateY(${slideY}px)` : "translateY(-8px)",
        opacity: revealed ? progress : 0.3,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
        <span
          style={{
            fontFamily: "monospace",
            fontSize: 24,
            color: revealed
              ? `rgba(56, 189, 248, ${0.5 + 0.5 * progress})`
              : "rgba(255,255,255,0.15)",
            flex: 1,
          }}
        >
          {revealed ? label : "???"}
        </span>
        {revealed && (
          <span
            style={{
              fontFamily: "monospace",
              fontSize: 13,
              color: `rgba(56, 189, 248, ${0.25 * progress})`,
            }}
          >
            {fileSize}
          </span>
        )}
      </div>
    </div>
  );
};

const ghostDirs = [
  { label: "node_modules/.cache/ghost", y: 0 },
  { label: "venv/lib/.venv-track", y: 70 },
  { label: ".npm/_cacache/ghost", y: 140 },
  { label: "target/.rustc_info", y: 210 },
  { label: ".gradle/ghost-jars", y: 280 },
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

  const revealedCount = (() => {
    for (let i = ghostDirs.length - 1; i >= 0; i--) {
      if (frame >= 50 + i * 25) return i + 1;
    }
    return 0;
  })();

  const scanProgress = revealedCount / ghostDirs.length;
  const progressWidth = interpolate(
    Math.min(scanProgress, frame >= 195 ? 1 : scanProgress),
    [0, 1],
    [0, 440],
  );

  return (
    <AbsoluteFill
      style={{
        background:
          "linear-gradient(180deg, #0d0f12 0%, #111318 30%, #131313 100%)",
      }}
    >
      {/* Dot grid background */}
      <svg style={{ position: "absolute", inset: 0, width: "100%", height: "100%", pointerEvents: "none" }}>
        <defs>
          <pattern id="g6" width="40" height="40" patternUnits="userSpaceOnUse">
            <circle cx="20" cy="20" r="0.6" fill="rgba(56, 189, 248, 0.06)" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#g6)" />
      </svg>

      <ParticleBg count={35} color="56, 189, 248" baseOpacity={0.05} />

      {/* Scan line effect */}
      <div
        style={{
          position: "absolute",
          left: 0,
          top: 0,
          width,
          height: 3,
          background: "linear-gradient(90deg, transparent, rgba(56,189,248,0.2), transparent)",
          filter: "blur(2px)",
          transform: `translateY(${interpolate(frame, [0, 240], [0, ghostH + 320])}px)`,
          opacity: interpolate(frame, [0, 20, 220, 240], [0.6, 0.8, 0.8, 0]),
        }}
      />

      {/* Subhead */}
      <div
        style={{
          position: "absolute",
          top: 100,
          left: 0,
          right: 0,
          textAlign: "center",
          opacity: subheadOpacity,
        }}
      >
        <span
          style={{
            fontFamily: "monospace",
            fontSize: 24,
            color: "rgba(255,255,255,0.5)",
            letterSpacing: 3,
          }}
        >
          Scanning for orphaned environments...
        </span>
      </div>

      {/* Scan counter */}
      <div
        style={{
          position: "absolute",
          right: ghostX + 680,
          top: ghostY + 80 + ghostDirs.length * 70 + 10,
          fontFamily: "monospace",
          fontSize: 14,
          color: "rgba(56, 189, 248, 0.3)",
          textAlign: "right",
        }}
      >
        {revealedCount} / {ghostDirs.length} directories
      </div>

      {/* Progress bar */}
      <div
        style={{
          position: "absolute",
          left: ghostX,
          top: ghostY + 80 + ghostDirs.length * 70 + 32,
          width: 680,
          height: 4,
          borderRadius: 2,
          background: "rgba(56, 189, 248, 0.08)",
        }}
      >
        <div
          style={{
            width: progressWidth,
            height: "100%",
            borderRadius: 2,
            background: "linear-gradient(90deg, rgba(56,189,248,0.2), rgba(56,189,248,0.5))",
            transition: "width 0.1s",
          }}
        />
      </div>

      {/* Ghost directory lines */}
      <div
        style={{
          position: "absolute",
          left: ghostX,
          top: ghostY + 80,
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
              y={d.y}
              revealed={revealed}
              progress={progress}
              fileSize={fileSizes[i]}
            />
          );
        })}
      </div>

      {/* Cursor scans through lines */}
      <Cursor
        waypoints={[
          { x: ghostX + ghostW + 40, y: ghostY + 20, frame: 0 },
          { x: ghostX + ghostW - 20, y: ghostY + 75, frame: 35 },
          { x: ghostX + ghostW - 20, y: ghostY + 75, frame: 55 },
          { x: ghostX + ghostW - 20, y: ghostY + 135, frame: 75 },
          { x: ghostX + ghostW - 20, y: ghostY + 135, frame: 95 },
          { x: ghostX + ghostW - 20, y: ghostY + 195, frame: 115 },
          { x: ghostX + ghostW - 20, y: ghostY + 195, frame: 135 },
          { x: ghostX + ghostW - 20, y: ghostY + 255, frame: 155 },
          { x: ghostX + ghostW - 20, y: ghostY + 255, frame: 175 },
          { x: ghostX + ghostW - 20, y: ghostY + 315, frame: 195 },
          { x: width + 40, y: ghostY + 315, frame: 215 },
        ]}
        currentFrame={frame}
        showTrail
      />

      {/* Result: "0 ghost environments found" */}
      {frame >= 195 && (
        <div
          style={{
            position: "absolute",
            bottom: 100,
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
              background: "rgba(74, 222, 128, 0.1)",
              border: "1px solid rgba(74, 222, 128, 0.25)",
              borderRadius: 14,
              padding: "18px 44px",
              transform: `scale(${dotsProgress})`,
            }}
          >
            <span
              style={{
                fontFamily: "monospace",
                fontSize: 28,
                color: "#4ade80",
                fontWeight: "600",
              }}
            >
              0 ghost environments found
            </span>
          </div>
        </div>
      )}

      <SoundFX type="whoosh" startFrame={0} />
      <SoundFX type="scanWobble" startFrame={35} />
      <SoundFX type="typing" startFrame={50} />
      <SoundFX type="typing" startFrame={75} />
      <SoundFX type="typing" startFrame={100} />
      <SoundFX type="typing" startFrame={125} />
      <SoundFX type="typing" startFrame={150} />
      <SoundFX type="success" startFrame={195} />
    </AbsoluteFill>
  );
};
