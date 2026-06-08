import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, staticFile } from "remotion";
import { Audio } from "@remotion/media";
import { TransitionSeries, springTiming } from "@remotion/transitions";
import { slide } from "@remotion/transitions/slide";
import { wipe } from "@remotion/transitions/wipe";
import { zoomBlur } from "@remotion/transitions/zoom-blur";
import { linearBlur } from "@remotion/transitions/linear-blur";
import { crossZoom } from "@remotion/transitions/cross-zoom";
import { Beat1Chaos } from "./scenes/Beat1Chaos";
import { Beat2Reframe } from "./scenes/Beat2Reframe";
import { Beat3Graph } from "./scenes/Beat3Graph";
import { Beat4Autoswitch } from "./scenes/Beat4Autoswitch";
import { Beat5Languages } from "./scenes/Beat5Languages";
import { Beat6Ghost } from "./scenes/Beat6Ghost";
import { Beat7CTA } from "./scenes/Beat7CTA";
import { AmbientDrone } from "./components/SoundFX";

const CLAMP = { extrapolateLeft: "clamp" as const, extrapolateRight: "clamp" as const };

export const VenPromo: React.FC = () => {
  const frame = useCurrentFrame();

  const tenseVol = interpolate(frame, [400, 430], [0.5, 0], CLAMP);
  const hopefulVol = Math.min(
    interpolate(frame, [420, 430], [0, 0.6], CLAMP),
    interpolate(frame, [1250, 1270], [0.6, 0], CLAMP),
  );
  const triumphantVol = interpolate(frame, [1255, 1270], [0, 0.65], CLAMP);

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <AmbientDrone />
      <Audio src={staticFile("assets/music-tense.mp3")} volume={tenseVol} />
      <Audio src={staticFile("assets/music-hopeful.mp3")} volume={hopefulVol} />
      <Audio src={staticFile("assets/music-triumphant.mp3")} volume={triumphantVol} />
      <TransitionSeries>
        <TransitionSeries.Sequence durationInFrames={210}>
          <Beat1Chaos />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition
          timing={springTiming({ config: { damping: 30, stiffness: 200 }, durationInFrames: 20 })}
          presentation={wipe({ direction: "from-top" })}
        />

        <TransitionSeries.Sequence durationInFrames={180}>
          <Beat2Reframe />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition
          timing={springTiming({ config: { damping: 30, stiffness: 200 }, durationInFrames: 20 })}
          presentation={slide({ direction: "from-bottom" })}
        />

        <TransitionSeries.Sequence durationInFrames={360}>
          <Beat3Graph />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition
          timing={springTiming({ config: { damping: 30, stiffness: 200 }, durationInFrames: 20 })}
          presentation={wipe({ direction: "from-left" })}
        />

        <TransitionSeries.Sequence durationInFrames={210}>
          <Beat4Autoswitch />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition
          timing={springTiming({ config: { damping: 30, stiffness: 200 }, durationInFrames: 20 })}
          presentation={zoomBlur()}
        />

        <TransitionSeries.Sequence durationInFrames={210}>
          <Beat5Languages />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition
          timing={springTiming({ config: { damping: 30, stiffness: 200 }, durationInFrames: 20 })}
          presentation={linearBlur({ overlayColor: "#131313", kernel: 20 })}
        />

        <TransitionSeries.Sequence durationInFrames={240}>
          <Beat6Ghost />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition
          timing={springTiming({ config: { damping: 30, stiffness: 200 }, durationInFrames: 20 })}
          presentation={crossZoom()}
        />

        <TransitionSeries.Sequence durationInFrames={210}>
          <Beat7CTA />
        </TransitionSeries.Sequence>
      </TransitionSeries>
    </AbsoluteFill>
  );
};
