import { Composition } from "remotion";
import { VenPromo } from "./Composition";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="VenPromo"
        component={VenPromo}
        durationInFrames={1620}
        fps={30}
        width={1920}
        height={1080}
      />
    </>
  );
};
