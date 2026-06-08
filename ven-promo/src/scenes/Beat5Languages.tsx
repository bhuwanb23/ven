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

const languages = [
  "Node.js", "Python", "Java", "Go",
  "Rust", "PHP", "Ruby", "C#",
];

const orbCount = languages.length;
const cx = 960;
const cy = 480;
const ringR = 260;

interface OrbPosition {
  x: number;
  y: number;
}

const orbPositions: OrbPosition[] = languages.map((_, i) => ({
  x: cx + ringR * Math.cos((2 * Math.PI * i) / orbCount - Math.PI / 2),
  y: cy + ringR * Math.sin((2 * Math.PI * i) / orbCount - Math.PI / 2),
}));

export const Beat5Languages: React.FC = () => {
  const frame = useCurrentFrame();

  const ringProgress = spring({
    frame: Math.max(0, frame - 10),
    fps: 30,
    config: { damping: 18, stiffness: 100, mass: 0.6 },
  });

  const ringOpacity = interpolate(ringProgress, [0, 0.3, 1], [0, 0.5, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const cursorWaypoints = orbPositions.map((pos, i) => ({
    x: pos.x,
    y: pos.y,
    frame: 30 + i * 10,
  }));

  const activeOrbIndex = (() => {
    for (let i = cursorWaypoints.length - 1; i >= 0; i--) {
      if (frame >= cursorWaypoints[i].frame) return i;
    }
    return -1;
  })();

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <ParticleBg count={35} />

      {/* Connection ring border */}
      <svg
        width="1920"
        height="1080"
        style={{ position: "absolute", inset: 0 }}
      >
        <circle
          cx={cx}
          cy={cy}
          r={ringR + 40}
          fill="none"
          stroke="rgba(0, 200, 255, 0.05)"
          strokeWidth={1}
          strokeDasharray="4 8"
          opacity={ringOpacity}
        />
        <circle
          cx={cx}
          cy={cy}
          r={ringR}
          fill="none"
          stroke="rgba(0, 200, 255, 0.08)"
          strokeWidth={1}
          opacity={ringOpacity * 0.5}
        />
      </svg>

      {/* Center text */}
      <div
        style={{
          position: "absolute",
          left: cx,
          top: cy,
          transform: "translate(-50%, -50%)",
          textAlign: "center",
          opacity: ringOpacity,
        }}
      >
        <div
          style={{
            fontFamily: "Inter, sans-serif",
            fontSize: 14,
            color: "rgba(0, 200, 255, 0.5)",
            letterSpacing: 3,
            textTransform: "uppercase",
          }}
        >
          ven
        </div>
      </div>

      {/* Orb nodes */}
      {languages.map((lang, i) => {
        const pos = orbPositions[i];
        const nodeFrame = 10 + i * 5;
        const nodeProgress = spring({
          frame: Math.max(0, frame - nodeFrame),
          fps: 30,
          config: { damping: 16, stiffness: 150, mass: 0.4 },
        });

        const isActive = i <= activeOrbIndex && activeOrbIndex >= 0;
        const activeGlow = isActive
          ? 0.5 + 0.5 * Math.sin(frame * 0.1 + i)
          : 0;

        return (
          <div
            key={lang}
            style={{
              position: "absolute",
              left: pos.x - 25,
              top: pos.y - 25,
              width: 50,
              height: 50,
              borderRadius: "50%",
              background: isActive
                ? "radial-gradient(circle, rgba(0,200,255,0.3) 0%, rgba(0,200,255,0.05) 100%)"
                : "radial-gradient(circle, rgba(255,255,255,0.08) 0%, rgba(255,255,255,0.02) 100%)",
              border: `1px solid ${
                isActive
                  ? `rgba(0, 200, 255, ${0.3 + activeGlow * 0.5})`
                  : "rgba(255,255,255,0.1)"
              }`,
              transform: `scale(${nodeProgress})`,
              opacity: 0.3 + 0.7 * nodeProgress,
              boxShadow: isActive
                ? `0 0 ${20 + activeGlow * 30}px rgba(0, 200, 255, ${0.1 + activeGlow * 0.3})`
                : "none",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              transition: "none",
            }}
          >
            <div
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: isActive ? "#00c8ff" : "rgba(255,255,255,0.2)",
                opacity: isActive ? 0.8 + 0.2 * activeGlow : 0.3,
              }}
            />
          </div>
        );
      })}

      {/* Language labels */}
      {languages.map((lang, i) => {
        const pos = orbPositions[i];
        const labelFrame = 10 + i * 5;
        const labelProgress = spring({
          frame: Math.max(0, frame - labelFrame),
          fps: 30,
          config: { damping: 20, stiffness: 100, mass: 0.5 },
        });
        const isActive = i <= activeOrbIndex && activeOrbIndex >= 0;

        return (
          <div
            key={`label-${lang}`}
            style={{
              position: "absolute",
              left: pos.x - 40,
              top: pos.y + 35,
              width: 80,
              textAlign: "center",
              fontFamily: "Inter, sans-serif",
              fontSize: 12,
              fontWeight: "500",
              color: isActive ? "rgba(0, 200, 255, 0.8)" : "rgba(255,255,255,0.25)",
              opacity: 0.7 * labelProgress,
              transform: `translateY(${(1 - labelProgress) * 8}px)`,
            }}
          >
            {lang}
          </div>
        );
      })}

      {/* Cursor orbiting */}
      <Cursor
        waypoints={[
          { x: cx, y: cy - ringR - 40, frame: 0 },
          ...cursorWaypoints,
        ]}
        currentFrame={frame}
        showTrail
      />

      {/* Closing text */}
      {frame >= 110 && (
        <div
          style={{
            position: "absolute",
            bottom: 140,
            left: 0,
            right: 0,
            textAlign: "center",
          }}
        >
          <KineticText
            text="8 languages. One interface."
            startFrame={110}
            currentFrame={frame}
            fontSize={28}
            color="rgba(255,255,255,0.6)"
            fontWeight="500"
            staggerDelay={4}
            highlightWords={["8", "One"]}
            highlightColor="rgba(0, 200, 255, 0.15)"
          />
        </div>
      )}
    </AbsoluteFill>
  );
};
