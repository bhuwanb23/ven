import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate } from "remotion";
import { Cursor } from "../components/Cursor";
import { TerminalLine } from "../components/TerminalLine";
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";
import { PulseRing } from "../components/PulseRing";

export const Beat1Chaos: React.FC = () => {
  const frame = useCurrentFrame();

  const redPeak = interpolate(frame, [165, 172, 185], [0, 0.55, 0.55], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const redFade = interpolate(frame, [185, 200], [0.55, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const redFlash = frame < 185 ? redPeak : redFade;

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
        y={280}
        x={250}
        success
      />
      <TerminalLine
        text="pip install"
        startFrame={80}
        currentFrame={frame}
        typingDelay={3}
        y={350}
        x={250}
        success
      />
      <TerminalLine
        text="npm install react"
        startFrame={155}
        currentFrame={frame}
        typingDelay={1}
        y={420}
        x={250}
        success={false}
      />

      <Cursor
        waypoints={[
          { x: 200, y: 260, frame: 0 },
          { x: 450, y: 296, frame: 38 },
          { x: 200, y: 260, frame: 48 },
          { x: 200, y: 260, frame: 68 },
          { x: 450, y: 366, frame: 113 },
          { x: 200, y: 260, frame: 125 },
          { x: 200, y: 260, frame: 140 },
          { x: 480, y: 436, frame: 158 },
          { x: 200, y: 260, frame: 178 },
          { x: 200, y: 360, frame: 195 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 45, x: 480, y: 296 },
          { frame: 120, x: 480, y: 366 },
          { frame: 165, x: 520, y: 436 },
        ]}
        showTrail
      />

      {frame >= 165 && frame < 195 && (
        <PulseRing
          x={520}
          y={436}
          startFrame={165}
          currentFrame={frame}
          color="rgba(255, 50, 50, 0.5)"
          maxRadius={140}
        />
      )}

      {frame >= 178 && (
        <div style={{ position: "absolute", bottom: 180, left: 0, right: 0 }}>
          <KineticText
            text="You install first. And pray it works."
            startFrame={178}
            currentFrame={frame}
            fontSize={28}
            color="rgba(255,255,255,0.5)"
            fontWeight="400"
            staggerDelay={5}
          />
        </div>
      )}
    </AbsoluteFill>
  );
};
