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
import { TerminalLine } from "../components/TerminalLine";
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";
import { SoundFX } from "../components/SoundFX";

export const Beat7CTA: React.FC = () => {
  const frame = useCurrentFrame();

  const logoScale = spring({
    frame: Math.max(0, frame - 15),
    fps: 30,
    config: { damping: 18, stiffness: 150, mass: 0.4 },
  });

  const buttonSpring = spring({
    frame: Math.max(0, frame - 140),
    fps: 30,
    config: { damping: 14, stiffness: 200, mass: 0.35 },
  });

  const buttonGlow = interpolate(buttonSpring, [0.5, 1], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const taglineSpring = spring({
    frame: Math.max(0, frame - 55),
    fps: 30,
    config: { damping: 16, stiffness: 140, mass: 0.5 },
  });

  const decorRing1 = spring({
    frame: Math.max(0, frame - 10),
    fps: 30,
    config: { damping: 30, stiffness: 40, mass: 2 },
  });

  const decorRing2 = spring({
    frame: Math.max(0, frame - 20),
    fps: 30,
    config: { damping: 30, stiffness: 40, mass: 2 },
  });

  return (
    <AbsoluteFill
      style={{
        background:
          "linear-gradient(180deg, #060d1a 0%, #0c1a30 40%, #131313 100%)",
      }}
    >
      {/* Dot grid background */}
      <svg style={{ position: "absolute", inset: 0, width: "100%", height: "100%", pointerEvents: "none" }}>
        <defs>
          <pattern id="g7" width="48" height="48" patternUnits="userSpaceOnUse">
            <circle cx="24" cy="24" r="0.8" fill="rgba(0, 150, 255, 0.08)" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#g7)" />
      </svg>

      <ParticleBg count={40} color="0, 150, 255" baseOpacity={0.06} />

      {/* Decorative rings */}
      <div
        style={{
          position: "absolute",
          left: "50%",
          top: "50%",
          width: 800 * decorRing1,
          height: 800 * decorRing1,
          borderRadius: "50%",
          border: "1px solid rgba(0, 200, 255, 0.04)",
          transform: "translate(-50%, -50%)",
          opacity: 0.6 * (1 - decorRing1),
        }}
      />
      <div
        style={{
          position: "absolute",
          left: "50%",
          top: "50%",
          width: 1100 * decorRing2,
          height: 1100 * decorRing2,
          borderRadius: "50%",
          border: "1px solid rgba(0, 200, 255, 0.025)",
          transform: "translate(-50%, -50%)",
          opacity: 0.4 * (1 - decorRing2),
        }}
      />

      <div
        style={{
          position: "absolute",
          top: "48%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 30,
        }}
      >
        {/* Logo */}
        <Img
          src={staticFile("Ven_logo.png")}
          style={{
            width: 120 * logoScale,
            height: 120 * logoScale,
            opacity: logoScale,
            filter: `drop-shadow(0 0 40px rgba(0, 200, 255, ${0.3 * logoScale}))`,
          }}
        />

        <div
          style={{
            textAlign: "center",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 20,
          }}
        >
          <KineticText
            text="Ready to ship confident?"
            startFrame={50}
            currentFrame={frame}
            fontSize={48}
            color="rgba(255,255,255,0.95)"
            fontWeight="700"
            staggerDelay={5}
            letterSpacing={1}
          />

          {/* Tagline */}
          {frame >= 55 && (
            <div
              style={{
                transform: `translateY(${(1 - taglineSpring) * -12}px)`,
                opacity: taglineSpring,
              }}
            >
              <span
                style={{
                  fontFamily: "Inter, sans-serif",
                  fontSize: 18,
                  color: "rgba(255,255,255,0.35)",
                  letterSpacing: 4,
                  textTransform: "uppercase",
                }}
              >
                ven — environment management, evolved
              </span>
            </div>
          )}

          {/* Terminal line explaining CTA */}
          {frame >= 100 && (
            <div style={{ marginTop: 10 }}>
              <TerminalLine
                text="ven init my-project"
                startFrame={100}
                currentFrame={frame}
                typingDelay={5}
                y={0}
                x={0}
                fontSize={28}
                success
              />
            </div>
          )}

            {/* Action button */}
          {frame >= 130 && (
            <div
              style={{
                marginTop: 10,
                transform: `scale(${buttonSpring * (1 + 0.02 * Math.sin(frame * 0.06))})`,
                opacity: buttonSpring,
              }}
            >
              <div
                style={{
                  background: "linear-gradient(135deg, #00c8ff, #0090ff)",
                  color: "#fff",
                  fontFamily: "Inter, sans-serif",
                  fontSize: 24,
                  fontWeight: "600",
                  padding: "20px 56px",
                  borderRadius: 14,
                  border: "none",
                  cursor: "pointer",
                  boxShadow: `0 0 ${(50 + 10 * Math.sin(frame * 0.06)) * buttonGlow}px rgba(0, 200, 255, ${0.4 * buttonGlow})`,
                  letterSpacing: 2,
                }}
              >
                Get Started with ven
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Stats row */}
      <div
        style={{
          position: "absolute",
          bottom: 40,
          left: 0,
          right: 0,
          display: "flex",
          justifyContent: "center",
          gap: 60,
          opacity: interpolate(frame, [60, 90], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }),
        }}
      >
        {[
          { value: "10k+", label: "stars" },
          { value: "50k+", label: "downloads" },
          { value: "MIT", label: "license" },
        ].map((stat, i) => (
          <div
            key={i}
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 4,
              fontFamily: "Inter, sans-serif",
            }}
          >
            <span
              style={{
                fontSize: 28,
                fontWeight: "700",
                color: "rgba(0, 200, 255, 0.5)",
                letterSpacing: 1,
              }}
            >
              {stat.value}
            </span>
            <span
              style={{
                fontSize: 12,
                color: "rgba(255,255,255,0.2)",
                letterSpacing: 2,
                textTransform: "uppercase",
              }}
            >
              {stat.label}
            </span>
          </div>
        ))}
      </div>

      {/* Feature badges */}
      {frame >= 80 && (
        <div
          style={{
            position: "absolute",
            top: 160,
            left: 0,
            right: 0,
            display: "flex",
            justifyContent: "center",
            gap: 12,
            opacity: interpolate(frame, [80, 110], [0, 0.5], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }),
          }}
        >
          {["Zero Config", "Auto-switch", "Ghost Detection", "Multi-lang"].map((badge, i) => (
            <span
              key={i}
              style={{
                fontFamily: "monospace",
                fontSize: 13,
                color: "rgba(0, 200, 255, 0.3)",
                border: "1px solid rgba(0, 200, 255, 0.08)",
                borderRadius: 20,
                padding: "6px 16px",
                letterSpacing: 0.5,
              }}
            >
              {badge}
            </span>
          ))}
        </div>
      )}

      {/* Cursor: idle entrance then exits */}
      <Cursor
        waypoints={[
          { x: 1920 - 100, y: 500, frame: 0 },
          { x: 700, y: 440, frame: 30 },
          { x: 700, y: 440, frame: 60 },
          { x: 480, y: 440, frame: 70 },
          { x: 480, y: 440, frame: 115 },
          { x: 850, y: 580, frame: 145 },
          { x: 850, y: 580, frame: 175 },
          { x: 1920 + 40, y: 600, frame: 195 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 65, x: 640, y: 440 },
          { frame: 160, x: 850, y: 580 },
          { frame: 170, x: 860, y: 580 },
        ]}
        showTrail
      />

      <SoundFX type="whoosh" startFrame={0} />
      <SoundFX type="whooshShort" startFrame={50} />
      <SoundFX type="typing" startFrame={100} />
      <SoundFX type="success" startFrame={170} />
    </AbsoluteFill>
  );
};
