import React from "react";
import { spring } from "remotion";
import { ConnectionLine } from "./ConnectionLine";

interface GraphNode {
  x: number;
  y: number;
  label: string;
  color?: string;
  size?: number;
  pulseColor?: string;
}

interface GraphEdge {
  from: number;
  to: number;
}

interface DependencyGraphProps {
  nodes: GraphNode[];
  edges: GraphEdge[];
  startFrame: number;
  currentFrame: number;
  nodeStagger?: number;
  edgeStagger?: number;
  centerIndex?: number;
}

export const DependencyGraph: React.FC<DependencyGraphProps> = ({
  nodes,
  edges,
  startFrame,
  currentFrame,
  nodeStagger = 8,
  edgeStagger = 4,
  centerIndex = 0,
}) => {
  return (
    <svg
      width="100%"
      height="100%"
      viewBox="0 0 1920 1080"
      style={{ position: "absolute", inset: 0 }}
    >
      {edges.map((edge, i) => {
        const fromNode = nodes[edge.from];
        const toNode = nodes[edge.to];
        if (!fromNode || !toNode) return null;
        return (
          <ConnectionLine
            key={`edge-${i}`}
            startX={fromNode.x}
            startY={fromNode.y}
            endX={toNode.x}
            endY={toNode.y}
            startFrame={startFrame + i * edgeStagger}
            currentFrame={currentFrame}
            color="rgba(0, 200, 255, 0.35)"
            glowColor="rgba(0, 200, 255, 0.08)"
            strokeWidth={2}
          />
        );
      })}
      {nodes.map((node, i) => {
        const nodeFrame = startFrame + i * nodeStagger;
        const progress = spring({
          frame: Math.max(0, currentFrame - nodeFrame),
          fps: 30,
          config: { damping: 14, stiffness: 120, mass: 0.5 },
        });
        const size = node.size ?? 40;
        const color = node.color ?? "#00c8ff";
        const isCenter = i === centerIndex;

        return (
          <g key={`node-${i}`}>
            {isCenter && (
              <>
                <circle
                  cx={node.x}
                  cy={node.y}
                  r={size * 2.5 * progress}
                  fill="none"
                  stroke={color}
                  strokeWidth={1}
                  opacity={0.15 * progress}
                />
                <circle
                  cx={node.x}
                  cy={node.y}
                  r={size * 4 * progress}
                  fill="none"
                  stroke={color}
                  strokeWidth={0.5}
                  opacity={0.08 * progress}
                />
              </>
            )}
            <circle
              cx={node.x}
              cy={node.y}
              r={size * (0.3 + 0.7 * progress)}
              fill={color}
              opacity={0.2 + 0.3 * progress}
            />
            <circle
              cx={node.x}
              cy={node.y}
              r={size * 0.3 * progress}
              fill={isCenter ? "#ffffff" : color}
              opacity={0.5 + 0.5 * progress}
            />
            {isCenter && (
              <circle
                cx={node.x}
                cy={node.y}
                r={size * 0.15 * progress}
                fill="#00c8ff"
                opacity={0.8 * progress}
              />
            )}
            <text
              x={node.x}
              y={node.y + size + 24}
              textAnchor="middle"
              fill={isCenter ? "#ffffff" : "rgba(255,255,255,0.6)"}
              fontSize={isCenter ? 22 : 18}
              fontFamily="Inter, sans-serif"
              fontWeight={isCenter ? "600" : "400"}
              opacity={0.8 * progress}
            >
              {node.label}
            </text>
          </g>
        );
      })}
    </svg>
  );
};
