import React from "react";
import { interpolate } from "remotion";

interface PulseRingProps {
  x: number;
  y: number;
  startFrame: number;
  currentFrame: number;
  duration?: number;
  color?: string;
  maxRadius?: number;
  strokeWidth?: number;
}

export const PulseRing: React.FC<PulseRingProps> = ({
  x,
  y,
  startFrame,
  currentFrame,
  duration = 30,
  color = "rgba(0, 200, 255, 0.4)",
  maxRadius = 80,
  strokeWidth = 2,
}) => {
  const progress = Math.max(
    0,
    Math.min(1, (currentFrame - startFrame) / duration),
  );
  if (progress >= 1) return null;

  const radius = interpolate(progress, [0, 1], [0, maxRadius]);
  const opacity = interpolate(progress, [0, 0.3, 1], [0.6, 0.4, 0]);

  return (
    <circle
      cx={x}
      cy={y}
      r={radius}
      fill="none"
      stroke={color}
      strokeWidth={strokeWidth}
      opacity={opacity}
    />
  );
};
