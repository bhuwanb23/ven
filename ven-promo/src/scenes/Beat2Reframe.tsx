import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, spring } from "remotion";
import { KineticText } from "../components/KineticText";

export const Beat2Reframe: React.FC = () => {
  const frame = useCurrentFrame();

  const dotProgress = spring({
    frame,
    fps: 30,
    config: { damping: 14, stiffness: 100, mass: 0.6 },
  });

  const dotPulse = 0.6 + 0.4 * Math.sin(frame * 0.08);

  const dotScale = dotProgress * dotPulse;

  const fadeOut = interpolate(frame, [80, 90], [1, 0], {
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
            gap: 40,
          }}
        >
          <div
            style={{
              width: 8 * dotScale,
              height: 8 * dotScale,
              borderRadius: "50%",
              background: "#00c8ff",
              boxShadow: `0 0 ${40 * dotScale}px rgba(0, 200, 255, ${0.3 * dotScale})`,
              opacity: interpolate(frame, [0, 15], [0, 1], {
                extrapolateLeft: "clamp",
                extrapolateRight: "clamp",
              }),
            }}
          />

          {frame >= 30 && (
            <div style={{ maxWidth: 800, marginTop: 20 }}>
              <KineticText
                text="What if your tool could predict conflicts before they happen?"
                startFrame={30}
                currentFrame={frame}
                fontSize={36}
                color="rgba(255,255,255,0.85)"
                fontWeight="500"
                staggerDelay={4}
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
