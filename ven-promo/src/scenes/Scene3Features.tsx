import React from "react";
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig, Easing } from "remotion";
import { Terminal } from "../components/Terminal";
import { Checkmark } from "../components/Checkmark";
import { HLine } from "../components/HLine";

const languages = [
  "Node.js", "Python", "Go", "Rust",
  "Java", "Deno", "Bun", "Ruby",
];

const featureLabel: React.CSSProperties = {
  fontFamily: "JetBrains Mono, monospace",
  fontSize: 18,
  color: "#00dbe7",
  letterSpacing: "0.05em",
  textTransform: "uppercase" as const,
  marginBottom: 8,
};

const AutoSwitch: React.FC<{ frame: number; startFrame: number }> = ({ frame, startFrame }) => {
  const localFrame = Math.max(0, frame - startFrame);
  const { fps } = useVideoConfig();

  const opacity = interpolate(localFrame, [0, 10], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const l1op = interpolate(localFrame, [20, 35], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const l1r = interpolate(localFrame, [50, 65], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const l2op = interpolate(localFrame, [70, 85], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const l2r = interpolate(localFrame, [90, 105], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <div style={{ opacity, display: "flex", flexDirection: "column", alignItems: "center", gap: 16, width: "100%" }}>
      <div style={featureLabel}>Auto-Switch</div>
      <Terminal title="~/projects — auto-switch" width={660}>
        <div style={{ opacity: l1op }}>
          <span style={{ color: "#849495" }}>$</span>{" "}
          <span style={{ color: "#e5e2e1" }}>cd frontend</span>
        </div>
        <div style={{ opacity: l1r, marginTop: 4, display: "flex", alignItems: "center", gap: 8 }}>
          <Checkmark frame={localFrame} startDelay={50} />
          <span style={{ color: "#00e639" }}>node -v → v20.20.2</span>
        </div>
        <div style={{ opacity: l2op, marginTop: 12 }}>
          <span style={{ color: "#849495" }}>$</span>{" "}
          <span style={{ color: "#e5e2e1" }}>cd backend</span>
        </div>
        <div style={{ opacity: l2r, marginTop: 4, display: "flex", alignItems: "center", gap: 8 }}>
          <Checkmark frame={localFrame} startDelay={90} />
          <span style={{ color: "#00e639" }}>node -v → v22.11.0</span>
          <span style={{ color: "#b9cacb" }}>| python → 3.11</span>
        </div>
      </Terminal>
      <div style={{ color: "#b9cacb", fontSize: 20, fontFamily: "Geist, system-ui, sans-serif", textAlign: "center" }}>
        Walk in. Version activates. Walk out. It deactivates.
      </div>
    </div>
  );
};

const LangCard: React.FC<{ name: string; index: number; frame: number; startFrame: number }> = ({
  name,
  index,
  frame,
  startFrame,
}) => {
  const localFrame = Math.max(0, frame - startFrame);
  const staggerDelay = index * 5;

  const scale = spring({
    frame: Math.max(0, localFrame - staggerDelay),
    fps: 30,
    config: { damping: 14, stiffness: 140, mass: 0.6 },
  });

  const opacity = interpolate(Math.max(0, localFrame - staggerDelay), [0, 8], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <div
      style={{
        background: "#201f1f",
        border: "2px solid #3a494b",
        borderRadius: 8,
        padding: "16px 20px",
        display: "flex",
        flexDirection: "column",
        gap: 4,
        opacity,
        transform: `scale(${scale})`,
      }}
    >
      <span style={{ fontFamily: "Geist, system-ui, sans-serif", fontSize: 22, fontWeight: 600, color: "#e5e2e1" }}>
        {name}
      </span>
      <span style={{ fontFamily: "JetBrains Mono, monospace", fontSize: 14, color: "#00a3ad" }}>
        ven install {name.toLowerCase().replace(/\.js$/, "")}
      </span>
    </div>
  );
};

const LangGrid: React.FC<{ frame: number; startFrame: number }> = ({ frame, startFrame }) => {
  const localFrame = Math.max(0, frame - startFrame);
  const opacity = interpolate(localFrame, [0, 10], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <div style={{ opacity, display: "flex", flexDirection: "column", alignItems: "center", gap: 20, width: "100%" }}>
      <div style={featureLabel}>8 Languages · One Interface</div>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr 1fr 1fr",
          gap: 12,
          width: 680,
        }}
      >
        {languages.map((lang, i) => (
          <LangCard key={lang} name={lang} index={i} frame={frame} startFrame={startFrame} />
        ))}
      </div>
      <div style={{ color: "#b9cacb", fontSize: 20, fontFamily: "Geist, system-ui, sans-serif", textAlign: "center" }}>
        Same command. Every runtime. Official sources. SHA256 verified.
      </div>
    </div>
  );
};

const typeStyle: React.CSSProperties = {
  margin: 0,
  fontFamily: "JetBrains Mono, monospace",
  fontSize: 20,
};

const Security: React.FC<{ frame: number; startFrame: number }> = ({ frame, startFrame }) => {
  const localFrame = Math.max(0, frame - startFrame);
  const { fps } = useVideoConfig();

  const opacity = interpolate(localFrame, [0, 10], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const l1op = interpolate(localFrame, [15, 30], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const l2op = interpolate(localFrame, [40, 55], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const ghostOp = interpolate(localFrame, [65, 80], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const l3op = interpolate(localFrame, [90, 105], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const shieldOp = interpolate(localFrame, [110, 120], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <div style={{ opacity, display: "flex", flexDirection: "column", alignItems: "center", gap: 16, width: "100%" }}>
      <div style={featureLabel}>Security · Ghost Detection</div>
      <Terminal title="~/projects — ven scan" width={660}>
        <div style={{ opacity: l1op, ...typeStyle }}>
          <span style={{ color: "#849495" }}>$</span>{" "}
          <span style={{ color: "#00e639" }}>ven</span> scan --ghosts
        </div>
        <div style={{ opacity: l2op, ...typeStyle, marginTop: 8, color: "#b9cacb" }}>
          → Walking source tree...
        </div>
        <div
          style={{
            opacity: ghostOp,
            ...typeStyle,
            marginTop: 8,
            color: "#ffb4ab",
            background: "rgba(255, 59, 48, 0.1)",
            border: "1px solid rgba(255, 59, 48, 0.2)",
            borderRadius: 4,
            padding: "6px 12px",
          }}
        >
          ✗ Ghost detected: <strong>lodash</strong> used but not declared
        </div>
        <div style={{ opacity: l3op, ...typeStyle, marginTop: 8, color: "#00e639", display: "flex", alignItems: "center", gap: 8 }}>
          <Checkmark frame={localFrame} startDelay={90} />
          <span>3 ghosts found · 3 fixed</span>
        </div>
      </Terminal>
      <div style={{ opacity: shieldOp, color: "#b9cacb", fontSize: 20, fontFamily: "Geist, system-ui, sans-serif", display: "flex", alignItems: "center", gap: 10 }}>
        <span style={{ color: "#00e639", fontSize: 24 }}>⬡</span> CI-Safe · CVE scan across 8 ecosystems
      </div>
    </div>
  );
};

export const Scene3Features: React.FC = () => {
  const frame = useCurrentFrame();
  const totalDuration = 300; // 450-750 (10s total for scene 3)

  const beatDuration = 100;

  const showAutoSwitch = frame < beatDuration;
  const showLangGrid = frame >= beatDuration && frame < beatDuration * 2;
  const showSecurity = frame >= beatDuration * 2 && frame < beatDuration * 3;
  const fadeToScene4 = interpolate(frame, [beatDuration * 3 - 30, beatDuration * 3], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      {/* Dot grid BG */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          opacity: 0.04,
          backgroundImage: `radial-gradient(circle, rgba(0, 219, 231, 0.6) 1px, transparent 1px)`,
          backgroundSize: "32px 32px",
          pointerEvents: "none",
        }}
      />

      <div
        style={{
          position: "absolute",
          inset: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          padding: "0 80px",
          opacity: fadeToScene4,
        }}
      >
        {showAutoSwitch && <AutoSwitch frame={frame} startFrame={0} />}
        {showLangGrid && <LangGrid frame={frame} startFrame={beatDuration} />}
        {showSecurity && <Security frame={frame} startFrame={beatDuration * 2} />}
      </div>
    </AbsoluteFill>
  );
};
