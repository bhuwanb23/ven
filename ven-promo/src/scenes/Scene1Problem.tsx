import React from "react";
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { Terminal } from "../components/Terminal";

const lines = [
  { cmd: "$ npm install express", output: "✗ ERESOLVE peer dependency conflict", error: true },
  { cmd: "$ pip install django", output: "✗ dependency conflict", error: true },
  { cmd: "$ nvm use 18", output: "✗ version 18 not installed", error: true },
  { cmd: "$ npx create-react-app", output: "✗ permission denied", error: true },
];

const lineStyle: React.CSSProperties = { margin: 0, padding: 0 };
const cmdStyle: React.CSSProperties = { color: "#00e639", margin: 0 };
const outputStyle: React.CSSProperties = { color: "#ffb4ab", margin: 0, fontWeight: 600 };
const promptStyle: React.CSSProperties = { color: "#849495", margin: 0 };

const Line: React.FC<{ line: typeof lines[0]; startFrame: number; frame: number }> = ({
  line,
  startFrame,
  frame,
}) => {
  const localFrame = Math.max(0, frame - startFrame);
  const opacity = interpolate(localFrame, [0, 10], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const outputOpacity = interpolate(localFrame, [15, 25], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <div>
      <p style={{ ...lineStyle, opacity }}>
        <span style={promptStyle}>┌─</span>
      </p>
      <p style={{ ...lineStyle, opacity: interpolate(localFrame, [2, 12], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) }}>
        <span style={cmdStyle}>{line.cmd}</span>
      </p>
      <p style={{ ...lineStyle, opacity: outputOpacity, display: "flex", alignItems: "center", gap: 8 }}>
        <span style={outputStyle}>{line.output}</span>
      </p>
    </div>
  );
};

const cursorBlinkStyle: React.CSSProperties = {
  display: "inline-block",
  width: 12,
  height: 28,
  background: "#00dbe7",
  marginLeft: 4,
  verticalAlign: "middle",
};

const Cursor: React.FC<{ frame: number; startFrame: number }> = ({ frame, startFrame }) => {
  const localFrame = Math.max(0, frame - startFrame);
  const visible = Math.floor(localFrame / 8) % 2 === 0;
  return <span style={{ ...cursorBlinkStyle, opacity: visible ? 1 : 0 }} />;
};

export const Scene1Problem: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const termEntry = spring({
    frame,
    fps,
    config: { damping: 15, stiffness: 120, mass: 0.8 },
  });

  const termY = interpolate(termEntry, [0, 1], [-80, 0]);
  const termOpacity = interpolate(frame, [0, 10], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const taglineOpacity = interpolate(frame, [100, 120], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const tagline2Opacity = interpolate(frame, [130, 150], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const fadeOut = interpolate(frame, [160, 180], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <div
        style={{
          position: "absolute",
          inset: 0,
          opacity: 0.025,
          backgroundImage: `url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E")`,
          backgroundRepeat: "repeat",
          backgroundSize: "256px 256px",
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
          opacity: fadeOut,
        }}
      >
        <Terminal
          title="~/projects/api — bash"
          width={760}
          style={{
            transform: `translateY(${termY}px)`,
            opacity: termOpacity,
          }}
        >
          <Line line={lines[0]} startFrame={5} frame={frame} />
          <Line line={lines[1]} startFrame={40} frame={frame} />
          <Line line={lines[2]} startFrame={75} frame={frame} />
          <Line line={lines[3]} startFrame={105} frame={frame} />
          <div style={{ marginTop: 12, display: "flex", alignItems: "center" }}>
            <span style={promptStyle}>$</span>
            <Cursor frame={frame} startFrame={0} />
          </div>
        </Terminal>
      </div>

      <div
        style={{
          position: "absolute",
          bottom: 180,
          left: 0,
          right: 0,
          textAlign: "center",
          opacity: fadeOut,
        }}
      >
        <div
          style={{
            fontFamily: "Geist, system-ui, sans-serif",
            fontSize: 56,
            fontWeight: 700,
            color: "#e1fdff",
            letterSpacing: "-0.02em",
            opacity: taglineOpacity,
            marginBottom: 12,
          }}
        >
          Every tool installs first.
        </div>
        <div
          style={{
            fontFamily: "Geist, system-ui, sans-serif",
            fontSize: 56,
            fontWeight: 700,
            color: "#ff3b30",
            letterSpacing: "-0.02em",
            opacity: tagline2Opacity,
          }}
        >
          Breaks second.
        </div>
      </div>
    </AbsoluteFill>
  );
};
