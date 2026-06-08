import React from "react";
import {
  AbsoluteFill,
  useCurrentFrame,
  interpolate,
  spring,
  staticFile,
  Img,
} from "remotion";
import { Cursor } from "../components/Cursor";
import { TerminalLine } from "../components/TerminalLine";
import { KineticText } from "../components/KineticText";
import { ParticleBg } from "../components/ParticleBg";

export const Beat7CTA: React.FC = () => {
  const frame = useCurrentFrame();

  const logoScale = spring({
    frame: Math.max(0, frame - 15),
    fps: 30,
    config: { damping: 18, stiffness: 150, mass: 0.4 },
  });

  const buttonSpring = spring({
    frame: Math.max(0, frame - 140),
    fps: 30,
    config: { damping: 14, stiffness: 200, mass: 0.35 },
  });

  const buttonGlow = interpolate(buttonSpring, [0.5, 1], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const taglineSpring = spring({
    frame: Math.max(0, frame - 55),
    fps: 30,
    config: { damping: 16, stiffness: 140, mass: 0.5 },
  });

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <ParticleBg count={35} />

      <div
        style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 30,
        }}
      >
        {/* Logo */}
        <Img
          src={staticFile("Ven_logo.png")}
          style={{
            width: 80 * logoScale,
            height: 80 * logoScale,
            opacity: logoScale,
          }}
        />

        <div
          style={{
            textAlign: "center",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 16,
          }}
        >
          <KineticText
            text="Ready to ship confident?"
            startFrame={50}
            currentFrame={frame}
            fontSize={42}
            color="rgba(255,255,255,0.95)"
            fontWeight="700"
            staggerDelay={5}
            letterSpacing={1}
          />

          {/* Tagline */}
          {frame >= 55 && (
            <div
              style={{
                transform: `translateY(${(1 - taglineSpring) * -12}px)`,
                opacity: taglineSpring,
              }}
            >
              <span
                style={{
                  fontFamily: "Inter, sans-serif",
                  fontSize: 16,
                  color: "rgba(255,255,255,0.35)",
                  letterSpacing: 3,
                  textTransform: "uppercase",
                }}
              >
                ven — environment management, evolved
              </span>
            </div>
          )}

          {/* Terminal line explaining CTA */}
          {frame >= 100 && (
            <div style={{ marginTop: 10 }}>
              <TerminalLine
                text="ven init my-project"
                startFrame={100}
                currentFrame={frame}
                typingDelay={5}
                y={0}
                x={0}
                success
              />
            </div>
          )}

          {/* Action button */}
          {frame >= 130 && (
            <div
              style={{
                marginTop: 10,
                transform: `scale(${buttonSpring})`,
                opacity: buttonSpring,
              }}
            >
              <div
                style={{
                  background: "linear-gradient(135deg, #00c8ff, #0090ff)",
                  color: "#fff",
                  fontFamily: "Inter, sans-serif",
                  fontSize: 20,
                  fontWeight: "600",
                  padding: "16px 48px",
                  borderRadius: 12,
                  border: "none",
                  cursor: "pointer",
                  boxShadow: `0 0 ${40 * buttonGlow}px rgba(0, 200, 255, ${0.4 * buttonGlow})`,
                  letterSpacing: 1,
                }}
              >
                Get Started with ven
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Cursor: idle entrance then exits */}
      <Cursor
        waypoints={[
          { x: 1920 - 100, y: 500, frame: 0 },
          { x: 700, y: 440, frame: 30 },
          { x: 700, y: 440, frame: 60 },
          { x: 480, y: 440, frame: 70 },
          { x: 480, y: 440, frame: 115 },
          { x: 850, y: 580, frame: 145 },
          { x: 850, y: 580, frame: 175 },
          { x: 1920 + 40, y: 600, frame: 195 },
        ]}
        currentFrame={frame}
        clicks={[
          { frame: 65, x: 640, y: 440 },
          { frame: 145, x: 850, y: 580 },
          { frame: 170, x: 860, y: 580 },
        ]}
        showTrail
      />
    </AbsoluteFill>
  );
};
