import React from "react";
import { interpolate } from "remotion";

interface HLineProps {
  frame: number;
  startFrame?: number;
  duration?: number;
  width?: number;
  color?: string;
}

export const HLine: React.FC<HLineProps> = ({
  frame,
  startFrame = 0,
  duration = 20,
  width = 1,
  color = "#3a494b",
}) => {
  const localFrame = Math.max(0, frame - startFrame);

  const scaleX = interpolate(localFrame, [0, duration], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        height: width,
        background: color,
        width: "100%",
        transform: `scaleX(${scaleX})`,
        transformOrigin: "left center",
      }}
    />
  );
};
