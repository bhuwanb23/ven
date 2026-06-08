import React from "react";
import { interpolate, spring } from "remotion";

interface TerminalLineProps {
  text: string;
  startFrame: number;
  currentFrame: number;
  typingDelay?: number;
  color?: string;
  prompt?: string;
  fontSize?: number;
  x?: number;
  y?: number;
  success?: boolean;
}

export const TerminalLine: React.FC<TerminalLineProps> = ({
  text,
  startFrame,
  currentFrame,
  typingDelay = 2,
  color = "#ffffff",
  prompt = "$",
  fontSize = 22,
  x = 200,
  y = 200,
  success,
}) => {
  const chars = [...text];
  const totalTypeFrames = chars.length * typingDelay;

  const lineProgress = spring({
    frame: Math.max(0, currentFrame - startFrame),
    fps: 30,
    config: { damping: 25, stiffness: 200, mass: 0.3 },
    durationInFrames: totalTypeFrames + 5,
  });

  const visibleChars = Math.floor(
    interpolate(
      Math.max(0, currentFrame - startFrame),
      [0, totalTypeFrames],
      [0, chars.length],
      { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
    ),
  );

  const showCursor =
    currentFrame >= startFrame &&
    visibleChars < chars.length &&
    Math.floor(currentFrame / 10) % 2 === 0;

  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: y,
        fontFamily: "monospace",
        fontSize,
        color: "rgba(255,255,255,0.5)",
        lineHeight: 1.6,
        whiteSpace: "pre",
        textAlign: "left",
        opacity: 0.7 + 0.3 * lineProgress,
      }}
    >
      <span style={{ color: "rgba(0, 200, 255, 0.6)" }}>{prompt}{" "}</span>
      <span style={{ color }}>{text.slice(0, visibleChars)}</span>
      {showCursor && (
        <span style={{ color: "rgba(0, 200, 255, 0.8)" }}>▌</span>
      )}
      {visibleChars >= chars.length && success !== undefined && (
        <span style={{ marginLeft: 12, color: success ? "#4ade80" : "#f87171" }}>
          {success ? "✓" : "✗"}
        </span>
      )}
    </div>
  );
};
