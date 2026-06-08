import React from "react";
import {
  AbsoluteFill,
  useCurrentFrame,
  interpolate,
  spring,
} from "remotion";
import { Cursor } from "../components/Cursor";
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";

const orbRadius = 220;
const cx = 960;
const cy = 460;

interface Orb {
  label: string;
  angle: number;
}

const orbs: Orb[] = [
  { label: "js", angle: 0 },
  { label: "py", angle: Math.PI / 4 },
  { label: "java", angle: Math.PI / 2 },
  { label: "go", angle: (3 * Math.PI) / 4 },
  { label: "rs", angle: Math.PI },
  { label: "php", angle: (5 * Math.PI) / 4 },
  { label: "rb", angle: (3 * Math.PI) / 2 },
  { label: "ts", angle: (7 * Math.PI) / 4 },
];

const OrbNode: React.FC<{
  orb: Orb;
  glowProgress: number;
}> = ({ orb, glowProgress }) => {
  const x = cx + orbRadius * Math.cos(orb.angle);
  const y = cy + orbRadius * Math.sin(orb.angle);

  const scale = interpolate(glowProgress, [0, 0.4, 1], [0, 1.15, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const glow = interpolate(glowProgress, [0, 0.6, 1], [0, 0.5, 0.2], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        position: "absolute",
        left: x - 28,
        top: y - 28,
        width: 56,
        height: 56,
        borderRadius: "50%",
        border: `2px solid rgba(0, 200, 255, ${0.3 + 0.7 * glowProgress})`,
        background: `rgba(0, 200, 255, ${0.05 * glowProgress})`,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        transform: `scale(${scale})`,
        boxShadow:
          glowProgress > 0
            ? `0 0 ${40 * glow}px rgba(0, 200, 255, ${0.3 * glow})`
            : "none",
      }}
    >
      <span
        style={{
          fontFamily: "monospace",
          fontSize: 13,
          color: `rgba(255,255,255,${0.4 + 0.6 * glowProgress})`,
          fontWeight: "600",
        }}
      >
        {orb.label}
      </span>
    </div>
  );
};

export const Beat5Languages: React.FC = () => {
  const frame = useCurrentFrame();

  const activeOrbCount = (() => {
    if (frame < 45) return 0;
    if (frame < 75) return 1;
    if (frame < 95) return 2;
    if (frame < 110) return 4;
    if (frame < 125) return 6;
    return 8;
  })();

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <ParticleBg count={30} />

      {/* Title */}
      <div
        style={{
          position: "absolute",
          top: 100,
          left: 0,
          right: 0,
          textAlign: "center",
        }}
      >
        <KineticText
          text="ven speaks your language"
          startFrame={0}
          currentFrame={frame}
          fontSize={38}
          color="rgba(255,255,255,0.9)"
          fontWeight="600"
          staggerDelay={5}
          letterSpacing={1}
        />
      </div>

      {/* Orbs */}
      {orbs.map((orb, i) => (
        <OrbNode
          key={orb.label}
          orb={orb}
          glowProgress={
            i < activeOrbCount
              ? spring({
                  frame: Math.max(0, frame - (45 + i * 10)),
                  fps: 30,
                  config: { damping: 14, stiffness: 140, mass: 0.5 },
                })
              : 0
          }
        />
      ))}

      {/* Center glow */}
      {activeOrbCount > 0 && (
        <div
          style={{
            position: "absolute",
            left: cx - 30,
            top: cy - 30,
            width: 60,
            height: 60,
            borderRadius: "50%",
            background: "rgba(0, 200, 255, 0.15)",
            boxShadow: `0 0 80px rgba(0, 200, 255, ${0.1 + 0.08 * Math.sin(frame * 0.04)})`,
            opacity: interpolate(
              activeOrbCount,
              [0, 1, 8],
              [0, 0.6, 1]
            ),
          }}
        />
      )}

      {/* Cursor clicks orb 0 and orb 2 */}
      <Cursor
        waypoints={[
          { x: cx, y: cy - orbRadius - 60, frame: 0 },
          { x: cx + orbRadius + 10, y: cy - 10, frame: 40 },
          { x: cx + orbRadius + 10, y: cy - 10, frame: 75 },
          { x: cx - 10, y: cy - orbRadius - 10, frame: 95 },
          { x: 1920 + 40, y: 400, frame: 130 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 55, x: cx + orbRadius, y: cy },
          { frame: 85, x: cx, y: cy - orbRadius },
        ]}
        showTrail
      />

      {/* Closing text */}
      {frame >= 145 && (
        <div
          style={{
            position: "absolute",
            bottom: 130,
            left: 0,
            right: 0,
            textAlign: "center",
          }}
        >
          <KineticText
            text="Switch between any runtime in seconds."
            startFrame={145}
            currentFrame={frame}
            fontSize={24}
            color="rgba(255,255,255,0.5)"
            fontWeight="400"
            staggerDelay={5}
          />
        </div>
      )}
    </AbsoluteFill>
  );
};
