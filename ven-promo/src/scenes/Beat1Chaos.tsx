import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, spring } from "remotion";
import { Cursor } from "../components/Cursor";
import { TerminalLine } from "../components/TerminalLine";
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";
import { PulseRing } from "../components/PulseRing";

export const Beat1Chaos: React.FC = () => {
  const frame = useCurrentFrame();

  const redFlash = interpolate(
    frame,
    [70, 72, 90],
    [0, 0.6, 0],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  const recoilProgress = spring({
    frame: Math.max(0, frame - 76),
    fps: 30,
    config: { damping: 12, stiffness: 300, mass: 0.8 },
  });

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <ParticleBg count={20} />

      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `rgba(255, 50, 50, ${redFlash})`,
          zIndex: 998,
        }}
      />

      <TerminalLine
        text="npm install"
        startFrame={0}
        currentFrame={frame}
        typingDelay={3}
        y={300}
        x={250}
        success
      />
      <TerminalLine
        text="pip install"
        startFrame={25}
        currentFrame={frame}
        typingDelay={3}
        y={360}
        x={250}
        success
      />
      <TerminalLine
        text="npm install react"
        startFrame={50}
        currentFrame={frame}
        typingDelay={3}
        y={420}
        x={250}
        success={false}
      />

      <Cursor
        waypoints={[
          { x: 200, y: 280, frame: 0 },
          { x: 400, y: 315, frame: 18 },
          { x: 200, y: 280, frame: 22 },
          { x: 400, y: 375, frame: 43 },
          { x: 200, y: 280, frame: 47 },
          { x: 450, y: 435, frame: 68 },
          { x: 200, y: 280, frame: 72 + recoilProgress * 30 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 20, x: 440, y: 315 },
          { frame: 45, x: 440, y: 375 },
          { frame: 70, x: 490, y: 435 },
        ]}
        showTrail
      />

      {frame >= 70 && frame < 100 && (
        <PulseRing
          x={490}
          y={435}
          startFrame={70}
          currentFrame={frame}
          color="rgba(255, 50, 50, 0.6)"
          maxRadius={120}
        />
      )}

      {frame >= 100 && (
        <div style={{ position: "absolute", bottom: 200, left: 0, right: 0 }}>
          <KineticText
            text="You install first. And pray it works."
            startFrame={100}
            currentFrame={frame}
            fontSize={28}
            color="rgba(255,255,255,0.5)"
            fontWeight="400"
            staggerDelay={4}
          />
        </div>
      )}
    </AbsoluteFill>
  );
};
