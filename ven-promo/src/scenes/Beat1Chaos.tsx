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
    [195, 198, 210],
    [0, 0.5, 0],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  const recoilX = spring({
    frame: Math.max(0, frame - 198),
    fps: 30,
    config: { damping: 10, stiffness: 250, mass: 0.9 },
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
        typingDelay={5}
        y={280}
        x={250}
        success
      />
      <TerminalLine
        text="pip install"
        startFrame={80}
        currentFrame={frame}
        typingDelay={5}
        y={360}
        x={250}
        success
      />
      <TerminalLine
        text="npm install react"
        startFrame={160}
        currentFrame={frame}
        typingDelay={5}
        y={440}
        x={250}
        success={false}
      />

      <Cursor
        waypoints={[
          { x: 200, y: 260, frame: 0 },
          { x: 450, y: 295, frame: 58 },
          { x: 200, y: 260, frame: 78 },
          { x: 450, y: 375, frame: 148 },
          { x: 200, y: 260, frame: 168 },
          { x: 500, y: 455, frame: 193 },
          { x: 150, y: 240, frame: 198 + recoilX * 40 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 68, x: 490, y: 295 },
          { frame: 158, x: 490, y: 375 },
          { frame: 195, x: 540, y: 455 },
        ]}
        showTrail
      />

      {frame >= 195 && frame < 195 + 30 && (
        <PulseRing
          x={540}
          y={455}
          startFrame={195}
          currentFrame={frame}
          color="rgba(255, 50, 50, 0.5)"
          maxRadius={100}
        />
      )}

      {frame >= 80 && frame < 80 + 25 && (
        <PulseRing
          x={490}
          y={295}
          startFrame={80}
          currentFrame={frame}
          color="rgba(74, 222, 128, 0.4)"
          maxRadius={60}
        />
      )}

      {frame >= 170 && frame < 170 + 25 && (
        <PulseRing
          x={490}
          y={375}
          startFrame={170}
          currentFrame={frame}
          color="rgba(74, 222, 128, 0.4)"
          maxRadius={60}
        />
      )}

      {frame >= 200 && (
        <div style={{ position: "absolute", bottom: 180, left: 0, right: 0 }}>
          <KineticText
            text="You install first. And pray it works."
            startFrame={200}
            currentFrame={frame}
            fontSize={28}
            color="rgba(255,255,255,0.45)"
            fontWeight="400"
            staggerDelay={5}
          />
        </div>
      )}
    </AbsoluteFill>
  );
};
