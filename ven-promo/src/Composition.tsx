import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, Sequence, staticFile } from "remotion";
import { Audio } from "@remotion/media";
import { Beat1Chaos } from "./scenes/Beat1Chaos";
import { Beat2Reframe } from "./scenes/Beat2Reframe";
import { Beat3Graph } from "./scenes/Beat3Graph";
import { Beat4Autoswitch } from "./scenes/Beat4Autoswitch";
import { Beat5Languages } from "./scenes/Beat5Languages";
import { Beat6Ghost } from "./scenes/Beat6Ghost";
import { Beat7CTA } from "./scenes/Beat7CTA";
import { AmbientDrone } from "./components/SoundFX";

const CLAMP = { extrapolateLeft: "clamp" as const, extrapolateRight: "clamp" as const };

const CrossfadeScene: React.FC<{
  children: React.ReactNode;
  sceneDuration: number;
  fadeIn?: boolean;
  fadeOut?: boolean;
}> = ({ children, sceneDuration, fadeIn, fadeOut }) => {
  const frame = useCurrentFrame();
  let opacity = 1;
  if (fadeIn) {
    opacity = Math.min(opacity, interpolate(frame, [0, 20], [0, 1], CLAMP));
  }
  if (fadeOut) {
    opacity = Math.min(opacity, interpolate(frame, [sceneDuration, sceneDuration + 20], [1, 0], CLAMP));
  }
  return <AbsoluteFill style={{ opacity }}>{children}</AbsoluteFill>;
};

export const VenPromo: React.FC = () => {
  const frame = useCurrentFrame();

  const tenseVol = interpolate(frame, [360, 380], [0.5, 0], CLAMP);
  const hopefulVol = Math.min(
    interpolate(frame, [360, 380], [0, 0.6], CLAMP),
    interpolate(frame, [1170, 1190], [0.6, 0], CLAMP),
  );
  const triumphantVol = interpolate(frame, [1170, 1190], [0, 0.65], CLAMP);

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <AmbientDrone />
      <Audio src={staticFile("assets/music-tense.mp3")} volume={tenseVol} />
      <Audio src={staticFile("assets/music-hopeful.mp3")} volume={hopefulVol} />
      <Audio src={staticFile("assets/music-triumphant.mp3")} volume={triumphantVol} />

      <Sequence from={0} durationInFrames={230}>
        <CrossfadeScene sceneDuration={210} fadeOut>
          <Beat1Chaos />
        </CrossfadeScene>
      </Sequence>

      <Sequence from={210} durationInFrames={170}>
        <CrossfadeScene sceneDuration={150} fadeIn fadeOut>
          <Beat2Reframe />
        </CrossfadeScene>
      </Sequence>

      <Sequence from={360} durationInFrames={410}>
        <CrossfadeScene sceneDuration={390} fadeIn fadeOut>
          <Beat3Graph />
        </CrossfadeScene>
      </Sequence>

      <Sequence from={750} durationInFrames={230}>
        <CrossfadeScene sceneDuration={210} fadeIn fadeOut>
          <Beat4Autoswitch />
        </CrossfadeScene>
      </Sequence>

      <Sequence from={960} durationInFrames={230}>
        <CrossfadeScene sceneDuration={210} fadeIn fadeOut>
          <Beat5Languages />
        </CrossfadeScene>
      </Sequence>

      <Sequence from={1170} durationInFrames={260}>
        <CrossfadeScene sceneDuration={240} fadeIn fadeOut>
          <Beat6Ghost />
        </CrossfadeScene>
      </Sequence>

      <Sequence from={1410} durationInFrames={210}>
        <CrossfadeScene sceneDuration={210} fadeIn>
          <Beat7CTA />
        </CrossfadeScene>
      </Sequence>
    </AbsoluteFill>
  );
};
