import React from "react";

const containerStyle: React.CSSProperties = {
  background: "#0e0e0e",
  border: "2px solid #3a494b",
  borderRadius: 8,
  overflow: "hidden",
  boxShadow: "0 24px 80px rgba(0,0,0,0.55)",
  display: "flex",
  flexDirection: "column",
};

const barStyle: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 8,
  padding: "14px 20px",
  background: "#201f1f",
  borderBottom: "2px solid #3a494b",
};

const dotStyle = (color: string): React.CSSProperties => ({
  width: 14,
  height: 14,
  borderRadius: "50%",
  background: color,
});

const titleStyle: React.CSSProperties = {
  fontFamily: "JetBrains Mono, monospace",
  fontSize: 18,
  color: "#f5fafa",
  marginLeft: 8,
};

const bodyStyle: React.CSSProperties = {
  padding: "28px 32px",
  fontFamily: "JetBrains Mono, monospace",
  fontSize: 24,
  lineHeight: 1.65,
  flex: 1,
};

interface TerminalProps {
  title?: string;
  width?: string | number;
  height?: string | number;
  children?: React.ReactNode;
  style?: React.CSSProperties;
}

export const Terminal: React.FC<TerminalProps> = ({
  title = "terminal",
  width = "100%",
  height = "auto",
  children,
  style,
}) => {
  return (
    <div style={{ ...containerStyle, width, height, ...style }}>
      <div style={barStyle}>
        <div style={dotStyle("#ff5f57")} />
        <div style={dotStyle("#febc2e")} />
        <div style={dotStyle("#28c840")} />
        <div style={titleStyle}>{title}</div>
      </div>
      <div style={bodyStyle}>{children}</div>
    </div>
  );
};
