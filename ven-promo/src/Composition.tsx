import React, { useEffect, useState } from "react";
import { AbsoluteFill, Sequence, continueRender, delayRender, staticFile } from "remotion";
import { Audio } from "@remotion/media";
import { loadFont } from "@remotion/fonts";
import { Scene1Problem } from "./scenes/Scene1Problem";
import { Scene2Solution } from "./scenes/Scene2Solution";
import { Scene3Features } from "./scenes/Scene3Features";
import { Scene4CTA } from "./scenes/Scene4CTA";

export const VenPromo: React.FC = () => {
  const [handle] = useState(() => delayRender("LoadingFonts"));
  const [fontsLoaded, setFontsLoaded] = useState(false);

  useEffect(() => {
    Promise.all([
      loadFont({
        family: "Geist",
        url: staticFile("fonts/Geist-Regular.woff2"),
        weight: "400",
      }),
      loadFont({
        family: "Geist",
        url: staticFile("fonts/Geist-SemiBold.woff2"),
        weight: "600",
      }),
      loadFont({
        family: "Geist",
        url: staticFile("fonts/Geist-Bold.woff2"),
        weight: "700",
      }),
      loadFont({
        family: "JetBrains Mono",
        url: staticFile("fonts/JetBrainsMono-Regular.ttf"),
        weight: "400",
      }),
    ]).then(() => {
      setFontsLoaded(true);
      continueRender(handle);
    });
  }, [handle]);

  if (!fontsLoaded) {
    return null;
  }

  return (
    <AbsoluteFill style={{ background: "#131313" }}>
      <Sequence from={0} durationInFrames={180}>
        <Scene1Problem />
      </Sequence>
      <Sequence from={180} durationInFrames={270}>
        <Scene2Solution />
      </Sequence>
      <Sequence from={450} durationInFrames={300}>
        <Scene3Features />
      </Sequence>
      <Sequence from={750} durationInFrames={150}>
        <Scene4CTA />
      </Sequence>

      {/* Scene 1: tense music */}
      <Sequence from={0} durationInFrames={180}>
        <Audio src={staticFile("assets/music-tense.mp3")} volume={0.35} />
      </Sequence>
      {/* Scene 2+3: hopeful music */}
      <Sequence from={180} durationInFrames={570}>
        <Audio src={staticFile("assets/music-hopeful.mp3")} volume={0.35} />
      </Sequence>
      {/* Scene 4: triumphant overlay */}
      <Sequence from={750} durationInFrames={150}>
        <Audio src={staticFile("assets/music-triumphant.mp3")} volume={0.45} />
      </Sequence>
    </AbsoluteFill>
  );
};
