import React from "react";
import { Composition, registerRoot } from "remotion";
import { GridVideo } from "./scene.jsx";

registerRoot(() => (
  <Composition
    id="MixedGrid"
    component={GridVideo}
    width={1000}
    height={1000}
    fps={30}
    durationInFrames={33}
    defaultProps={{ fontCss: "" }}
  />
));
