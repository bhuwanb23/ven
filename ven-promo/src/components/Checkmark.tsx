import React from "react";
import { interpolate, spring, useVideoConfig } from "remotion";

interface CheckmarkProps {
  frame: number;
  startDelay?: number;
}

export const Checkmark: React.FC<CheckmarkProps> = ({
  frame,
  startDelay = 0,
}) => {
  const { fps } = useVideoConfig();
  const localFrame = Math.max(0, frame - startDelay);

  const scale = spring({
    frame: localFrame,
    fps,
    config: { damping: 12, stiffness: 200, mass: 0.3 },
  });

  const opacity = interpolate(localFrame, [0, 10], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <span
      style={{
        color: "#00e639",
        fontFamily: "JetBrains Mono, monospace",
        fontSize: 28,
        fontWeight: 700,
        opacity,
        transform: `scale(${scale})`,
        display: "inline-block",
      }}
    >
      ✓
    </span>
  );
};
