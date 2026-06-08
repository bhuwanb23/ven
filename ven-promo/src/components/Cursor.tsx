import React from "react";
import { spring } from "remotion";

interface Waypoint {
  x: number;
  y: number;
  frame: number;
}

interface ClickEvent {
  frame: number;
  x: number;
  y: number;
}

interface CursorProps {
  waypoints: Waypoint[];
  currentFrame: number;
  clicks?: ClickEvent[];
  showTrail?: boolean;
}

const getCursorPosition = (waypoints: Waypoint[], frame: number) => {
  if (waypoints.length < 2) {
    return { x: waypoints[0]?.x ?? 0, y: waypoints[0]?.y ?? 0 };
  }
  for (let i = 0; i < waypoints.length - 1; i++) {
    const s = waypoints[i];
    const e = waypoints[i + 1];
    if (frame >= s.frame && frame < e.frame) {
      const dur = e.frame - s.frame;
      const t = spring({
        frame: frame - s.frame,
        fps: 30,
        config: { damping: 22, stiffness: 170, mass: 0.4 },
        durationInFrames: dur,
      });
      const mx = (s.x + e.x) / 2;
      const my = (s.y + e.y) / 2;
      const dx = e.x - s.x;
      const dy = e.y - s.y;
      const dist = Math.sqrt(dx * dx + dy * dy) || 1;
      const arc = Math.min(dist * 0.2, 50);
      const px = -dy / dist;
      const py = dx / dist;
      const cx = mx + px * arc;
      const cy = my + py * arc;
      const u = 1 - t;
      return {
        x: u * u * s.x + 2 * u * t * cx + t * t * e.x,
        y: u * u * s.y + 2 * u * t * cy + t * t * e.y,
      };
    }
  }
  const last = waypoints[waypoints.length - 1];
  return { x: last.x, y: last.y };
};

const CursorSvg: React.FC = () => (
  <svg width="14" height="20" viewBox="0 0 14 20" fill="none">
    <path
      d="M1.5 1.5V16.5L5 11L9 18L11 16.5L7 10L12 10L1.5 1.5Z"
      fill="white"
      stroke="rgba(0,0,0,0.3)"
      strokeWidth="0.5"
    />
  </svg>
);

export const Cursor: React.FC<CursorProps> = ({
  waypoints,
  currentFrame,
  clicks = [],
  showTrail = false,
}) => {
  const pos = getCursorPosition(waypoints, currentFrame);

  const activeClick = clicks.find(
    (c) => currentFrame >= c.frame && currentFrame < c.frame + 15,
  );
  const clickT = activeClick
    ? (currentFrame - activeClick.frame) / 15
    : null;

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        pointerEvents: "none",
        zIndex: 999,
      }}
    >
      {showTrail &&
        [1, 2, 3].map((i) => {
          const tPos = getCursorPosition(
            waypoints,
            Math.max(0, currentFrame - i * 3),
          );
          return (
            <div
              key={i}
              style={{
                position: "absolute",
                left: tPos.x - 7,
                top: tPos.y - 2,
                opacity: 0.15 * (1 - i * 0.28),
                transform: `scale(${1 - i * 0.07})`,
              }}
            >
              <CursorSvg />
            </div>
          );
        })}

      <div
        style={{
          position: "absolute",
          left: pos.x - 7,
          top: pos.y - 2,
        }}
      >
        <CursorSvg />
      </div>

      {activeClick && clickT !== null && clickT <= 1 && (
        <>
          <div
            style={{
              position: "absolute",
              left: activeClick.x - 30 * clickT,
              top: activeClick.y - 30 * clickT,
              width: 60 * clickT,
              height: 60 * clickT,
              borderRadius: "50%",
              border: `2px solid rgba(0, 200, 255, ${1 - clickT})`,
              opacity: 1 - clickT,
            }}
          />
          <div
            style={{
              position: "absolute",
              left: activeClick.x - 4,
              top: activeClick.y - 4,
              width: 8,
              height: 8,
              borderRadius: "50%",
              background: "rgba(0, 200, 255, 0.5)",
              transform: `scale(${1 + 0.5 * (1 - clickT)})`,
              opacity: 1 - clickT,
            }}
          />
        </>
      )}
    </div>
  );
};
