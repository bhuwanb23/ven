import React from "react";
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig, Easing } from "remotion";
import { Terminal } from "../components/Terminal";
import { Logo } from "../components/Logo";
import { Checkmark } from "../components/Checkmark";

const stageTextStyle: React.CSSProperties = {
  fontFamily: "Geist, system-ui, sans-serif",
  textAlign: "center",
};

export const Scene2Solution: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoScale = spring({
    frame: Math.max(0, frame),
    fps,
    config: { damping: 12, stiffness: 100, mass: 0.5 },
  });

  const headlineOpacity = interpolate(frame, [25, 45], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const headlineY = interpolate(frame, [25, 45], [30, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.out(Easing.cubic) });

  const subtitleOpacity = interpolate(frame, [40, 60], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const termOpacity = interpolate(frame, [70, 90], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const termY = interpolate(frame, [70, 90], [40, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.out(Easing.cubic) });

  const line1op = interpolate(frame, [95, 110], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const line2op = interpolate(frame, [125, 140], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const line3op = interpolate(frame, [155, 170], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const line4op = interpolate(frame, [185, 200], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const taglineOpacity = interpolate(frame, [220, 240], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const containerOpacity = interpolate(frame, [0, 270], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `radial-gradient(circle at 50% 30%, rgba(0, 219, 231, 0.08) 0%, transparent 60%)`,
          pointerEvents: "none",
          opacity: containerOpacity,
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
          opacity: containerOpacity,
          gap: 20,
        }}
      >
        {/* Logo */}
        <div style={{ opacity: interpolate(frame, [0, 15], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }), transform: `scale(${logoScale})` }}>
          <Logo frame={frame} size={120} startDelay={0} />
        </div>

        {/* Headline */}
        <div style={{ ...stageTextStyle, opacity: headlineOpacity, transform: `translateY(${headlineY}px)` }}>
          <span style={{ fontSize: 72, fontWeight: 700, color: "#e1fdff", letterSpacing: "-0.03em" }}>
            Meet{" "}
            <span style={{ color: "#00dbe7" }}>ven</span>
          </span>
        </div>

        {/* Subtitle */}
        <div style={{ ...stageTextStyle, opacity: subtitleOpacity, fontSize: 28, color: "#b9cacb", fontWeight: 400, maxWidth: 700 }}>
          The Intelligent Version &amp; Dependency Manager
        </div>

        {/* Terminal - Compatibility Check */}
        <div style={{ opacity: termOpacity, transform: `translateY(${termY}px)`, width: 700, marginTop: 20 }}>
          <Terminal title="~/projects/app — ven" width="100%">
            <div style={{ opacity: line1op }}>
              <span style={{ color: "#849495" }}>$</span>{" "}
              <span style={{ color: "#00e639" }}>ven</span> check-add lodash
            </div>
            <div style={{ opacity: line2op, color: "#b9cacb", marginTop: 8 }}>
              → Building dependency graph...
            </div>
            <div style={{ opacity: line3op, color: "#b9cacb", marginTop: 8 }}>
              → Simulating lodash@4.17 against current stack...
            </div>
            <div style={{ opacity: line4op, color: "#849495", marginTop: 8, display: "flex", alignItems: "center", gap: 8 }}>
              {frame >= 185 && frame < 200 && <span style={{ color: "#00dbe7" }}>⟳</span>}
              {frame >= 200 && <Checkmark frame={frame} startDelay={200} />}
              <span style={{ color: frame >= 200 ? "#00e639" : "#b9cacb" }}>
                {frame >= 200 ? "lodash@4.17 compatible" : "Checking compatibility..."}
              </span>
            </div>
          </Terminal>
        </div>

        {/* Tagline */}
        <div
          style={{
            ...stageTextStyle,
            opacity: taglineOpacity,
            fontSize: 26,
            color: "#00dbe7",
            marginTop: 10,
            maxWidth: 640,
            lineHeight: 1.5,
          }}
        >
          ven is <strong>predictive</strong>. It analyzes your entire dependency graph{" "}
          <em>before</em> touching your environment.
        </div>
      </div>
    </AbsoluteFill>
  );
};
