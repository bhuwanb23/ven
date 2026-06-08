import React from "react";
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig, Easing, Img, staticFile } from "remotion";
import { Terminal } from "../components/Terminal";

const badgeStyle: React.CSSProperties = {
  fontFamily: "JetBrains Mono, monospace",
  fontSize: 18,
  color: "#00dbe7",
  padding: "4px 12px",
  border: "1px solid rgba(0, 219, 231, 0.25)",
  borderRadius: 4,
};

export const Scene4CTA: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoScale = spring({
    frame: Math.max(0, frame),
    fps,
    config: { damping: 12, stiffness: 100, mass: 0.6 },
  });
  const logoOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const headOpacity = interpolate(frame, [20, 40], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const headY = interpolate(frame, [20, 40], [30, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.out(Easing.cubic) });

  const termOpacity = interpolate(frame, [45, 65], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const termY = interpolate(frame, [45, 65], [20, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.out(Easing.cubic) });

  const cmdOpacity = interpolate(frame, [70, 90], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const badge1op = interpolate(frame, [100, 110], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const badge2op = interpolate(frame, [108, 118], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const badge3op = interpolate(frame, [116, 126], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const tagOpacity = interpolate(frame, [120, 140], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const fadeOut = interpolate(frame, [140, 150], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `radial-gradient(circle at 50% 45%, rgba(0, 219, 231, 0.1) 0%, transparent 60%)`,
          pointerEvents: "none",
        }}
      />
      <div
        style={{
          position: "absolute",
          width: 380,
          height: 380,
          borderRadius: "50%",
          border: "2px solid rgba(0, 219, 231, 0.08)",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          pointerEvents: "none",
        }}
      />
      <div
        style={{
          position: "absolute",
          inset: 0,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: 24,
          opacity: fadeOut,
        }}
      >
        {/* Logo */}
        <div
          style={{
            opacity: logoOpacity,
            transform: `scale(${logoScale})`,
          }}
        >
          <Img
            src={staticFile("assets/Ven_logo.png")}
            style={{
              width: 100,
              height: 100,
              borderRadius: "50%",
              boxShadow: "0 0 40px rgba(0, 219, 231, 0.2)",
            }}
          />
        </div>

        {/* Headline */}
        <div
          style={{
            fontFamily: "Geist, system-ui, sans-serif",
            fontSize: 64,
            fontWeight: 700,
            color: "#e1fdff",
            letterSpacing: "-0.03em",
            opacity: headOpacity,
            transform: `translateY(${headY}px)`,
          }}
        >
          Start with <span style={{ color: "#00dbe7" }}>ven</span> today.
        </div>

        {/* Install command */}
        <div style={{ opacity: termOpacity, transform: `translateY(${termY}px)`, width: 620 }}>
          <Terminal title="install — ven" width="100%">
            <div style={{ opacity: cmdOpacity, display: "flex", alignItems: "center", gap: 8 }}>
              <span style={{ color: "#849495" }}>$</span>
              <span style={{ color: "#e5e2e1" }}>
                curl -fsSL{" "}
                <span style={{ color: "#00dbe7" }}>https://get.ven.sh</span> | sh
              </span>
              <span
                style={{
                  display: "inline-block",
                  width: 10,
                  height: 24,
                  background: "#00dbe7",
                  opacity: Math.floor(frame / 6) % 2 === 0 ? 1 : 0,
                }}
              />
            </div>
          </Terminal>
        </div>

        {/* Platform badges */}
        <div style={{ display: "flex", gap: 12, marginTop: 4 }}>
          <span style={{ ...badgeStyle, opacity: badge1op }}>Windows</span>
          <span style={{ ...badgeStyle, opacity: badge2op }}>macOS</span>
          <span style={{ ...badgeStyle, opacity: badge3op }}>Linux</span>
        </div>

        {/* Tagline */}
        <div
          style={{
            fontFamily: "Geist, system-ui, sans-serif",
            fontSize: 22,
            color: "#b9cacb",
            textAlign: "center",
            maxWidth: 500,
            lineHeight: 1.5,
            opacity: tagOpacity,
            marginTop: 8,
          }}
        >
          Install once. Switch automatically. Never break.
        </div>

        {/* Small text */}
        <div
          style={{
            fontFamily: "JetBrains Mono, monospace",
            fontSize: 14,
            color: "#849495",
            opacity: tagOpacity,
            marginTop: 12,
          }}
        >
          MIT License · github.com/bhuwanb23/ven
        </div>
      </div>
    </AbsoluteFill>
  );
};
