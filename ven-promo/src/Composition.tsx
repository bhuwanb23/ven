import React from "react";
import { AbsoluteFill } from "remotion";
import { TransitionSeries, springTiming } from "@remotion/transitions";
import { fade } from "@remotion/transitions/fade";
import { Beat1Chaos } from "./scenes/Beat1Chaos";
import { Beat2Reframe } from "./scenes/Beat2Reframe";
import { Beat3Graph } from "./scenes/Beat3Graph";
import { Beat4Autoswitch } from "./scenes/Beat4Autoswitch";
import { Beat5Languages } from "./scenes/Beat5Languages";
import { Beat6Ghost } from "./scenes/Beat6Ghost";
import { Beat7CTA } from "./scenes/Beat7CTA";
import { AmbientDrone } from "./components/SoundFX";

const trans = () =>
  springTiming({ config: { damping: 30, stiffness: 200 }, durationInFrames: 15 });

const fadePres = fade({ shouldFadeOutExitingScene: true });

export const VenPromo: React.FC = () => {
  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <AmbientDrone />
      <TransitionSeries>
        <TransitionSeries.Sequence durationInFrames={210}>
          <Beat1Chaos />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition timing={trans()} presentation={fadePres} />

        <TransitionSeries.Sequence durationInFrames={180}>
          <Beat2Reframe />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition timing={trans()} presentation={fadePres} />

        <TransitionSeries.Sequence durationInFrames={360}>
          <Beat3Graph />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition timing={trans()} presentation={fadePres} />

        <TransitionSeries.Sequence durationInFrames={210}>
          <Beat4Autoswitch />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition timing={trans()} presentation={fadePres} />

        <TransitionSeries.Sequence durationInFrames={210}>
          <Beat5Languages />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition timing={trans()} presentation={fadePres} />

        <TransitionSeries.Sequence durationInFrames={240}>
          <Beat6Ghost />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition timing={trans()} presentation={fadePres} />

        <TransitionSeries.Sequence durationInFrames={210}>
          <Beat7CTA />
        </TransitionSeries.Sequence>
      </TransitionSeries>
    </AbsoluteFill>
  );
};
