import React, { useEffect, useLayoutEffect, useMemo, useState } from "react";
import { staticFile, useCurrentFrame, useDelayRender } from "remotion";

const nodes = 100000;
const panels = 20;
const perPanel = nodes / panels;

const color = (id, frame) =>
  `rgb(${(id * 13 + frame * 17) % 256},${(id * 7 + frame * 29) % 256},${(id * 3 + frame * 43) % 256})`;
function useDerivedFrame(input) {
  const [value, setValue] = useState(-1);
  useEffect(() => setValue(input), [input]);
  return value;
}
// Twelve dependent effect/commit passes update each element’s visible content.
function EffectCell({ id, slot, frame, ready }) {
  const a = useDerivedFrame(frame);
  const b = useDerivedFrame(a);
  const c = useDerivedFrame(b);
  const d = useDerivedFrame(c);
  const e = useDerivedFrame(d);
  const f = useDerivedFrame(e);
  const g = useDerivedFrame(f);
  const h = useDerivedFrame(g);
  const i = useDerivedFrame(h);
  const j = useDerivedFrame(i);
  const k = useDerivedFrame(j);
  const l = useDerivedFrame(k);
  useLayoutEffect(() => {
    if (l === frame) ready(frame);
  }, [l, frame, ready]);
  return cell(id, slot, Math.max(0, l));
}
function cell(id, slot, frame) {
  if (slot % 100 === 0) {
    const panel = Math.floor(slot / perPanel);
    const index = (slot % perPanel) / 100;
    return (
      <text
        x={(panel % 5) * 200 + 24 + (index % 10) * 16}
        y={Math.floor(panel / 5) * 250 + 40 + Math.floor(index / 10) * 40}
        fontFamily="DM Sans"
        fontSize="16"
        fill="#fff"
      >
        {(id + frame) % 10}
      </text>
    );
  }
  return (
    <rect
      x={24 + ((slot * 13 + frame * 3) % 128)}
      y={24 + ((slot * 17 + frame * 5) % 176)}
      width={16 + (id % 4) * 4}
      height={16 + (Math.floor(id / 4) % 4) * 4}
      fill={color(id, frame)}
    />
  );
}
export function GridVideo({ fontCss = "" }) {
  const frame = useCurrentFrame();
  const { delayRender, continueRender, cancelRender } = useDelayRender();
  const [fontHandle] = useState(() =>
    delayRender("benchmark font", { retries: 0 })
  );
  useEffect(() => {
    const font = new FontFace(
      "DM Sans",
      `url(${staticFile("DMSans-Regular.ttf")})`
    );
    font
      .load()
      .then(loaded => {
        document.fonts.add(loaded);
        continueRender(fontHandle);
      })
      .catch(cancelRender);
  }, [fontHandle, continueRender, cancelRender]);
  const ready = useMemo(() => {
    const handle = delayRender(`frame ${frame} effects`, { retries: 0 });
    let remaining = nodes;
    return doneFrame => {
      if (doneFrame === frame && --remaining === 0) continueRender(handle);
    };
  }, [frame, delayRender, continueRender]);
  const cellFor = slot => {
    const id = (slot + frame * 37) % nodes;
    return <EffectCell id={id} slot={slot} frame={frame} ready={ready} />;
  };
  const layers = [];
  for (let panel = 0; panel < panels; panel++) {
    const cells = [];
    for (let slot = panel * perPanel; slot < (panel + 1) * perPanel; slot++)
      if (slot % 100 !== 0) cells.push(cellFor(slot));
    layers.push(
      <g
        transform={`translate(${(panel % 5) * 200} ${Math.floor(panel / 5) * 250})`}
        filter="url(#panel-effects)"
      >
        {cells}
      </g>
    );
  }
  const texts = [];
  for (let slot = 0; slot < nodes; slot += 100) texts.push(cellFor(slot));
  return (
    <svg
      width="1000"
      height="1000"
      style={{ display: "block", background: "#18202c" }}
    >
      <defs>
        <style>{fontCss}</style>
        <filter
          id="panel-effects"
          x="-40%"
          y="-40%"
          width="180%"
          height="180%"
          colorInterpolationFilters="sRGB"
        >
          <feGaussianBlur stdDeviation="8" result="glow" />
          <feColorMatrix
            in="glow"
            type="saturate"
            values="1.8"
            result="bright"
          />
          <feMerge>
            <feMergeNode in="bright" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
          <feDropShadow
            dx="4"
            dy="8"
            stdDeviation="12"
            floodColor="#000"
            floodOpacity="0.7"
          />
        </filter>
      </defs>
      {layers}
      {texts}
    </svg>
  );
}
