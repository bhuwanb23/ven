import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate } from "remotion";
import { Cursor } from "../components/Cursor";
import { TerminalLine } from "../components/TerminalLine";
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";
import { PulseRing } from "../components/PulseRing";
import { SoundFX } from "../components/SoundFX";

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

  const shakeIntensity = interpolate(frame, [165, 172, 185, 200], [0, 3, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const shakeX = shakeIntensity * Math.sin(frame * 1.7) * 2;
  const shakeY = shakeIntensity * Math.cos(frame * 1.3) * 1.5;

  return (
    <AbsoluteFill
      style={{
        background:
          frame < 165
            ? "#131313"
            : `linear-gradient(180deg, #1a0808 0%, #0d0505 50%, #131313 100%)`,
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: 0,
          transform: `translate(${shakeX}px, ${shakeY}px)`,
        }}
      >
      <ParticleBg count={25} color="255, 50, 50" baseOpacity={0.08} />

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
        x={420}
        fontSize={36}
        success
      />
      <TerminalLine
        text="pip install"
        startFrame={80}
        currentFrame={frame}
        typingDelay={3}
        y={390}
        x={420}
        fontSize={36}
        success
      />
      <TerminalLine
        text="npm install react"
        startFrame={155}
        currentFrame={frame}
        typingDelay={1}
        y={480}
        x={420}
        fontSize={36}
        success={false}
      />

      <Cursor
        waypoints={[
          { x: 350, y: 270, frame: 0 },
          { x: 750, y: 316, frame: 38 },
          { x: 350, y: 270, frame: 48 },
          { x: 350, y: 270, frame: 68 },
          { x: 750, y: 406, frame: 113 },
          { x: 350, y: 270, frame: 125 },
          { x: 350, y: 270, frame: 140 },
          { x: 820, y: 496, frame: 158 },
          { x: 350, y: 270, frame: 178 },
          { x: 350, y: 400, frame: 195 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 45, x: 780, y: 316 },
          { frame: 120, x: 780, y: 406 },
          { frame: 165, x: 860, y: 496 },
        ]}
        showTrail
      />

      {frame >= 165 && frame < 195 && (
        <PulseRing
          x={860}
          y={496}
          startFrame={165}
          currentFrame={frame}
          color="rgba(255, 50, 50, 0.5)"
          maxRadius={200}
        />
      )}

      {frame >= 178 && (
        <div style={{ position: "absolute", bottom: 160, left: 0, right: 0 }}>
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

      <SoundFX type="typing" startFrame={0} />
      <SoundFX type="whooshShort" startFrame={38} />
      <SoundFX type="click" startFrame={45} />
      <SoundFX type="typing" startFrame={80} />
      <SoundFX type="whooshShort" startFrame={113} />
      <SoundFX type="click" startFrame={120} />
      <SoundFX type="typing" startFrame={155} />
      <SoundFX type="error" startFrame={165} />
      <SoundFX type="click" startFrame={165} />
      </div>
    </AbsoluteFill>
  );
};
