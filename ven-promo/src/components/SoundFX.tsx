import React, { useMemo } from "react";
import { Audio, Sequence } from "remotion";
import {
  generateTypingClick,
  generateMouseClick,
  generateWhoosh,
  generateWhooshShort,
  generateErrorBuzz,
  generateSuccessDing,
  generateOrbChime,
  generateAmbientDrone,
  generateScanWobble,
} from "../audio/generate";

type SoundType = "typing" | "click" | "whoosh" | "whooshShort" | "error" | "success" | "scanWobble";

interface SoundFXProps {
  type: SoundType;
  startFrame: number;
  volume?: number;
}

const typeDuration: Record<SoundType, number> = {
  typing: 1,
  click: 2,
  whoosh: 12,
  whooshShort: 5,
  error: 9,
  success: 9,
  scanWobble: 9,
};

const genMap: Record<SoundType, () => string> = {
  typing: generateTypingClick,
  click: generateMouseClick,
  whoosh: generateWhoosh,
  whooshShort: generateWhooshShort,
  error: generateErrorBuzz,
  success: generateSuccessDing,
  scanWobble: generateScanWobble,
};

const cache = new Map<string, string>();

function getDataUrl(type: SoundType): string {
  const key = type;
  if (!cache.has(key)) {
    cache.set(key, genMap[type]());
  }
  return cache.get(key)!;
}

export const SoundFX: React.FC<SoundFXProps> = ({ type, startFrame, volume = 1 }) => {
  const src = useMemo(() => getDataUrl(type), [type]);
  const dur = typeDuration[type];
  return (
    <Sequence from={startFrame} durationInFrames={dur}>
      <Audio src={src} volume={volume} />
    </Sequence>
  );
};

export const OrbChime: React.FC<{ startFrame: number; index: number }> = ({
  startFrame,
  index,
}) => {
  const src = useMemo(() => {
    const key = `orb-${index}`;
    if (!cache.has(key)) cache.set(key, generateOrbChime(index));
    return cache.get(key)!;
  }, [index]);
  return (
    <Sequence from={startFrame} durationInFrames={6}>
      <Audio src={src} />
    </Sequence>
  );
};

export const AmbientDrone: React.FC = () => {
  const src = useMemo(() => generateAmbientDrone(1740), []);
  return <Audio src={src} volume={0.03} />;
};
