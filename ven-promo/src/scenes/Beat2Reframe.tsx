import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, spring } from "remotion";
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";
import { SoundFX } from "../components/SoundFX";

export const Beat2Reframe: React.FC = () => {
  const frame = useCurrentFrame();

  const dotSpring = spring({
    frame: Math.max(0, frame),
    fps: 30,
    config: { damping: 14, stiffness: 80, mass: 0.8 },
  });

  const dotGlow = spring({
    frame: Math.max(0, frame - 10),
    fps: 30,
    config: { damping: 20, stiffness: 60, mass: 0.8 },
  });

  const dotPulse = 0.5 + 0.5 * Math.sin(frame * 0.06);

  const dotOpacity = interpolate(frame, [0, 25], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const fadeOut = interpolate(frame, [130, 150], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        background:
          "linear-gradient(180deg, #080e1a 0%, #0c1425 30%, #131313 100%)",
        opacity: fadeOut,
      }}
    >
      <ParticleBg count={30} color="0, 150, 255" baseOpacity={0.06} />

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
              width: 40 * dotSpring * dotPulse,
              height: 40 * dotSpring * dotPulse,
              borderRadius: "50%",
              background: "radial-gradient(circle, #00c8ff 0%, #0066cc 100%)",
              boxShadow: `0 0 ${120 * dotGlow}px rgba(0, 100, 255, ${0.3 * dotGlow})`,
              opacity: dotOpacity,
            }}
          />

          {frame >= 35 && (
            <div style={{ maxWidth: 1400, marginTop: 20 }}>
              <KineticText
                text="What if your tool could predict conflicts before they happen?"
                startFrame={35}
                currentFrame={frame}
                fontSize={42}
                color="rgba(255,255,255,0.85)"
                fontWeight="500"
                staggerDelay={4}
                highlightWords={["predict", "conflicts"]}
                highlightColor="rgba(0, 150, 255, 0.15)"
                style={{ lineHeight: 1.5, textAlign: "center" }}
              />
            </div>
          )}
        </div>
      </div>

      <SoundFX type="whoosh" startFrame={0} />
      <SoundFX type="whoosh" startFrame={50} volume={0.5} />
    </AbsoluteFill>
  );
};
