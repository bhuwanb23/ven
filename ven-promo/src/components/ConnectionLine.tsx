import React from "react";
import { interpolate, spring } from "remotion";
import { evolvePath } from "@remotion/paths";

interface ConnectionLineProps {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  startFrame: number;
  currentFrame: number;
  duration?: number;
  color?: string;
  strokeWidth?: number;
  opacity?: number;
  glowColor?: string;
}

const buildCubicBezier = (
  x1: number,
  y1: number,
  x2: number,
  y2: number,
) => {
  const midX = (x1 + x2) / 2;
  const midY = (y1 + y2) / 2;
  const dx = x2 - x1;
  const dy = y2 - y1;
  const dist = Math.sqrt(dx * dx + dy * dy) || 1;
  const arc = Math.min(dist * 0.25, 80);
  const px = -dy / dist;
  const py = dx / dist;
  const cx1 = { x: midX + px * arc - dx * 0.1, y: midY + py * arc - dy * 0.1 };
  const cx2 = { x: midX + px * arc + dx * 0.1, y: midY + py * arc + dy * 0.1 };
  return `M ${x1} ${y1} C ${cx1.x} ${cx1.y}, ${cx2.x} ${cx2.y}, ${x2} ${y2}`;
};

export const ConnectionLine: React.FC<ConnectionLineProps> = ({
  startX,
  startY,
  endX,
  endY,
  startFrame,
  currentFrame,
  duration = 20,
  color = "rgba(0, 200, 255, 0.5)",
  strokeWidth = 2,
  opacity = 1,
  glowColor = "rgba(0, 200, 255, 0.15)",
}) => {
  const d = buildCubicBezier(startX, startY, endX, endY);
  const progress = spring({
    frame: Math.max(0, currentFrame - startFrame),
    fps: 30,
    config: { damping: 18, stiffness: 120, mass: 0.6 },
    durationInFrames: duration,
  });
  const { strokeDasharray, strokeDashoffset } = evolvePath(progress, d);
  const lineOpacity = interpolate(progress, [0, 0.1, 1], [0, 0.5, opacity]);

  return (
    <g style={{ opacity: lineOpacity }}>
      <path
        d={d}
        fill="none"
        stroke={glowColor}
        strokeWidth={strokeWidth * 4}
        strokeLinecap="round"
        style={{
          strokeDasharray,
          strokeDashoffset,
        }}
      />
      <path
        d={d}
        fill="none"
        stroke={color}
        strokeWidth={strokeWidth}
        strokeLinecap="round"
        style={{
          strokeDasharray,
          strokeDashoffset,
        }}
      />
    </g>
  );
};
