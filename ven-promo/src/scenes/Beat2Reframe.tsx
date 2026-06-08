import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, spring } from "remotion";
import { KineticText } from "../components/KineticText";

export const Beat2Reframe: React.FC = () => {
  const frame = useCurrentFrame();

  const dotProgress = spring({
    frame: Math.max(0, frame - 10),
    fps: 30,
    config: { damping: 14, stiffness: 100, mass: 0.6 },
  });

  const dotPulse = 0.5 + 0.5 * Math.sin(frame * 0.06);
  const dotScale = dotProgress * dotPulse;

  const dotOpacity = interpolate(frame, [0, 25], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const fadeOut = interpolate(frame, [165, 180], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        background: "#131313",
        opacity: fadeOut,
      }}
    >
      <div
        style={{
          position: "absolute",
          left: "50%",
          top: "45%",
          transform: "translate(-50%, -50%)",
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            flexDirection: "column",
            gap: 50,
          }}
        >
          <div
            style={{
              width: 10 * dotScale,
              height: 10 * dotScale,
              borderRadius: "50%",
              background: "#00c8ff",
              opacity: dotOpacity,
              boxShadow: `0 0 ${50 * dotScale}px rgba(0, 200, 255, ${0.3 * dotScale})`,
            }}
          />

          {frame >= 60 && (
            <div style={{ maxWidth: 900, marginTop: 10 }}>
              <KineticText
                text="What if your tool could predict conflicts before they happen?"
                startFrame={60}
                currentFrame={frame}
                fontSize={34}
                color="rgba(255,255,255,0.85)"
                fontWeight="500"
                staggerDelay={5}
                highlightWords={["predict", "conflicts"]}
                highlightColor="rgba(0, 200, 255, 0.2)"
                style={{ lineHeight: 1.5, textAlign: "center" }}
              />
            </div>
          )}
        </div>
      </div>
    </AbsoluteFill>
  );
};
