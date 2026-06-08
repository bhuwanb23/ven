import React from "react";
import { Img, interpolate, spring, useVideoConfig, staticFile } from "remotion";

interface LogoProps {
  frame: number;
  size?: number;
  startDelay?: number;
  animate?: boolean;
}

export const Logo: React.FC<LogoProps> = ({
  frame,
  size = 140,
  startDelay = 0,
  animate = true,
}) => {
  const { fps } = useVideoConfig();
  const localFrame = Math.max(0, frame - startDelay);

  const scale = animate
    ? spring({
        frame: localFrame,
        fps,
        config: { damping: 12, stiffness: 100, mass: 0.5 },
      })
    : 1;

  const opacity = animate
    ? interpolate(localFrame, [0, 15], [0, 1], {
        extrapolateLeft: "clamp",
        extrapolateRight: "clamp",
      })
    : 1;

  const glowOpacity = animate
    ? interpolate(localFrame, [0, 20], [0, 0.3], {
        extrapolateLeft: "clamp",
        extrapolateRight: "clamp",
      })
    : 0.3;

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        position: "relative",
      }}
    >
      <div
        style={{
          position: "absolute",
          width: size * 2.5,
          height: size * 2.5,
          borderRadius: "50%",
          border: "3px solid rgba(0, 219, 231, 0.1)",
          opacity: glowOpacity,
          pointerEvents: "none",
        }}
      />
      <div
        style={{
          position: "absolute",
          width: size * 3.5,
          height: size * 3.5,
          borderRadius: "50%",
          background: `radial-gradient(circle, rgba(0, 219, 231, ${glowOpacity * 0.5}) 0%, transparent 70%)`,
          pointerEvents: "none",
        }}
      />
      <Img
        src={staticFile("assets/Ven_logo.png")}
        style={{
          width: size,
          height: size,
          borderRadius: "50%",
          opacity,
          transform: `scale(${scale})`,
        }}
      />
    </div>
  );
};
