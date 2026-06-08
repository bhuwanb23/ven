import React, { useMemo } from "react";
import { interpolate, spring } from "remotion";

interface KineticTextProps {
  text: string;
  startFrame: number;
  currentFrame: number;
  mode?: "word" | "char";
  color?: string;
  fontSize?: number;
  fontFamily?: string;
  fontWeight?: string;
  highlightWords?: string[];
  highlightColor?: string;
  letterSpacing?: number;
  staggerDelay?: number;
  style?: React.CSSProperties;
}

export const KineticText: React.FC<KineticTextProps> = ({
  text,
  startFrame,
  currentFrame,
  mode = "word",
  color = "#ffffff",
  fontSize = 48,
  fontFamily = "Inter, Inter Display, sans-serif",
  fontWeight = "600",
  highlightWords = [],
  highlightColor = "rgba(0, 200, 255, 0.25)",
  letterSpacing = -1,
  staggerDelay = 3,
  style,
}) => {
  const items = useMemo(
    () =>
      mode === "word" ? text.split(" ") : [...text],
    [text, mode],
  );

  const isHighlighted = (item: string) =>
    highlightWords.some(
      (hw) => item.replace(/[^a-zA-Z0-9-]/g, "") === hw,
    );

  return (
    <div
      style={{
        display: "flex",
        flexWrap: "wrap",
        justifyContent: "center",
        alignItems: "center",
        gap: "0.15em",
        color,
        fontSize,
        fontFamily,
        fontWeight,
        letterSpacing,
        lineHeight: 1.3,
        ...style,
      }}
    >
      {items.map((item, i) => {
        const itemFrame = startFrame + i * staggerDelay;
        const progress = Math.max(
          0,
          Math.min(1, (currentFrame - itemFrame) / 12),
        );

        const scale = spring({
          frame: Math.max(0, currentFrame - itemFrame),
          fps: 30,
          config: { damping: 16, stiffness: 200, mass: 0.3 },
        });

        const opacity = interpolate(progress, [0, 0.4, 1], [0, 0, 1], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        });

        const translateY = interpolate(
          progress,
          [0, 0.3, 1],
          [20, 10, 0],
          {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
          },
        );

        const highlighted = isHighlighted(item);

        return (
          <span
            key={`${item}-${i}`}
            style={{
              display: "inline-block",
              position: "relative",
              opacity,
              transform: `translateY(${translateY}px) scale(${scale})`,
              whiteSpace: mode === "word" ? "nowrap" : "normal",
            }}
          >
            {highlighted && (
              <span
                style={{
                  position: "absolute",
                  inset: 0,
                  background: highlightColor,
                  borderRadius: 4,
                  transform: `scaleX(${interpolate(progress, [0.2, 0.6], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })})`,
                  transformOrigin: "left center",
                  opacity: 0.6,
                }}
              />
            )}
            {mode === "char" ? item : item + "\u00A0"}
          </span>
        );
      })}
    </div>
  );
};
