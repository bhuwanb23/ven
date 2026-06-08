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
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";

export const Beat7CTA: React.FC = () => {
  const frame = useCurrentFrame();

  const logoProgress = spring({
    frame: Math.max(0, frame - 5),
    fps: 30,
    config: { damping: 16, stiffness: 150, mass: 0.4 },
  });

  const logoGlow = interpolate(logoProgress, [0.5, 1], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const installCmdChars = [..."npm install -g ven"];
  const typeStart = 65;
  const typeSpeed = 3;
  const typedCount = Math.min(
    installCmdChars.length,
    Math.max(0, Math.floor((frame - typeStart) / typeSpeed)),
  );
  const isTyped = frame >= typeStart;

  const fadeOut = interpolate(frame, [110, 120], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        background: "#131313",
        opacity: fadeOut,
      }}
    >
      <ParticleBg count={30} />

      <div
        style={{
          position: "absolute",
          top: "38%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 16,
        }}
      >
        <Img
          src={staticFile("Ven_logo.png")}
          style={{
            width: 80 * logoProgress,
            height: 80 * logoProgress,
            opacity: logoProgress,
            filter: `drop-shadow(0 0 ${40 * logoGlow}px rgba(0, 200, 255, ${0.5 * logoGlow}))`,
          }}
        />
      </div>

      {/* Tagline */}
      <div
        style={{
          position: "absolute",
          top: "52%",
          left: 0,
          right: 0,
          textAlign: "center",
        }}
      >
        <KineticText
          text="Install once. Switch automatically. Never break."
          startFrame={30}
          currentFrame={frame}
          fontSize={30}
          color="rgba(255,255,255,0.75)"
          fontWeight="500"
          staggerDelay={5}
          highlightWords={["automatically"]}
          highlightColor="rgba(0, 200, 255, 0.15)"
          style={{ lineHeight: 1.5 }}
        />
      </div>

      {/* Install command with cursor trace */}
      {isTyped && (
        <div
          style={{
            position: "absolute",
            top: "65%",
            left: "50%",
            transform: "translateX(-50%)",
            fontFamily: "monospace",
            fontSize: 20,
            color: "rgba(255,255,255,0.3)",
            background: "rgba(255,255,255,0.03)",
            border: "1px solid rgba(255,255,255,0.06)",
            borderRadius: 10,
            padding: "16px 28px",
            display: "flex",
            alignItems: "center",
            gap: 12,
            opacity: interpolate(frame, [typeStart, typeStart + 10], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            }),
          }}
        >
          <span style={{ color: "rgba(0, 200, 255, 0.5)" }}>$</span>
          <span>
            <span style={{ color: "rgba(255,255,255,0.6)" }}>
              {installCmdChars.slice(0, typedCount).join("")}
            </span>
            {typedCount < installCmdChars.length && (
              <span
                style={{
                  color: "#00c8ff",
                  opacity: Math.floor(frame / 8) % 2 ? 1 : 0,
                }}
              >
                ▌
              </span>
            )}
          </span>
        </div>
      )}

      {/* Badges */}
      {frame >= 85 && (
        <div
          style={{
            position: "absolute",
            bottom: "18%",
            left: "50%",
            transform: "translateX(-50%)",
            display: "flex",
            gap: 24,
            opacity: spring({
              frame: Math.max(0, frame - 85),
              fps: 30,
              config: { damping: 20, stiffness: 150 },
            }),
          }}
        >
          {[
            { label: "GitHub", value: "☆ 4.2k" },
            { label: "npm", value: "↓ 120k/mo" },
            { label: "v2.1.0", value: "Latest" },
          ].map((badge, i) => (
            <div
              key={badge.label}
              style={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: 4,
                padding: "10px 20px",
                borderRadius: 8,
                background: "rgba(255,255,255,0.03)",
                border: "1px solid rgba(255,255,255,0.06)",
                opacity: spring({
                  frame: Math.max(0, frame - 85 - i * 5),
                  fps: 30,
                  config: { damping: 18, stiffness: 120 },
                }),
              }}
            >
              <span
                style={{
                  fontFamily: "Inter, sans-serif",
                  fontSize: 12,
                  color: "rgba(255,255,255,0.3)",
                }}
              >
                {badge.label}
              </span>
              <span
                style={{
                  fontFamily: "Inter, sans-serif",
                  fontSize: 16,
                  fontWeight: "600",
                  color: "rgba(255,255,255,0.7)",
                }}
              >
                {badge.value}
              </span>
            </div>
          ))}
        </div>
      )}

      {/* Cursor traces install command */}
      <Cursor
        waypoints={[
          {
            x: 960 - 200,
            y: 1080 * 0.65 + 10,
            frame: typeStart + installCmdChars.length * typeSpeed + 5,
          },
          {
            x: 960 + 200,
            y: 1080 * 0.65 + 10,
            frame: typeStart + installCmdChars.length * typeSpeed + 20,
          },
        ]}
        currentFrame={frame}
        showTrail
      />
    </AbsoluteFill>
  );
};
