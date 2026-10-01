function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
}
export function summarize(records, plan) {
  const native = records.filter(r => r.engine === "fframes");
  const browser = records.filter(r => r.engine === "remotion");
  const gpu = records.filter(r => r.engine === "fframes-gpu");
  const valid = runs =>
    runs.length === plan.rounds &&
    new Set(runs.map(r => r.round)).size === plan.rounds &&
    runs.every(
      r =>
        r.status === "ok" &&
        r.samples?.length === plan.frames &&
        r.samples.every(
          (s, i) =>
            s.frame === plan.warmup + i &&
            Number.isFinite(s.total_ms) &&
            s.total_ms > 0
        ) &&
        Number.isFinite(r.encoding_ms) &&
        r.encoding_ms > 0 &&
        Number.isFinite(r.export_ms) &&
        r.export_ms > 0 &&
        ["libx264", "h264_videotoolbox"].includes(r.encoder)
    );
  const complete = valid(native) && valid(browser);
  const gpuComplete = valid(gpu);
  const gpuSkipped =
    gpu.length === plan.rounds && gpu.every(r => r.status === "skipped");
  const total = r => r.samples.reduce((n, s) => n + s.total_ms, 0);
  const fframes = complete ? median(native.map(total)) : null;
  const remotion = complete ? median(browser.map(total)) : null;
  const gpuMs = gpuComplete ? median(gpu.map(total)) : null;
  return {
    status: complete && (gpuComplete || gpuSkipped) ? "complete" : "incomplete",
    fframes_median_ms: fframes,
    remotion_median_ms: remotion,
    speedup: complete ? remotion / fframes : null,
    gpu_median_ms: gpuMs,
    gpu_speedup: complete && gpuComplete ? remotion / gpuMs : null,
    gpu_status: gpuSkipped
      ? "skipped"
      : gpuComplete
        ? "complete"
        : "incomplete",
    gpu_skip_reason: gpuSkipped ? gpu[0].reason : null,
    encoding_median_ms: {
      cpu: complete ? median(native.map(r => r.encoding_ms)) : null,
      gpu: gpuComplete ? median(gpu.map(r => r.encoding_ms)) : null,
      remotion: complete ? median(browser.map(r => r.encoding_ms)) : null,
    },
    export_median_ms: {
      cpu: complete ? median(native.map(r => r.export_ms)) : null,
      gpu: gpuComplete ? median(gpu.map(r => r.export_ms)) : null,
      remotion: complete ? median(browser.map(r => r.export_ms)) : null,
    },
  };
}
export function markdown(report) {
  const result = report.summary;
  const encoder = engine =>
    report.records.find(r => r.engine === engine && r.status === "ok")
      ?.encoder ?? "n/a";
  return [
    "# fframes + Skia vs Remotion",
    "",
    "100,000 elements in 20 animated panels: 99,000 overlapping 16–28px rectangles with blurred glows, saturation and soft shadows, plus 1,000 changing text digits. Remotion uses an unkeyed list with 12 dependent effect/state updates per element. All render the same 1000×1000 scene with one rendering pipeline.",
    "",
    "Median of 3 rounds, each with 3 warm-up and 30 measured frames. PNG compression is included; startup, warm-up and disk writes are excluded.",
    "",
    "| Renderer | Render-to-PNG, 30 frames | Speedup vs Remotion |",
    "|---|---:|---:|",
    `| fframes + Skia CPU | ${result.fframes_median_ms?.toFixed(2) ?? "incomplete"} ms | ${result.speedup?.toFixed(2) ?? "n/a"}× |`,
    `| fframes + Skia GPU | ${result.gpu_status === "skipped" ? `skipped: ${result.gpu_skip_reason}` : `${result.gpu_median_ms?.toFixed(2) ?? "incomplete"} ms`} | ${result.gpu_speedup?.toFixed(2) ?? "n/a"}× |`,
    `| Remotion | ${result.remotion_median_ms?.toFixed(2) ?? "incomplete"} ms | — |`,
    "",
    "H.264 MP4: 30 frames at 30 fps, 8 Mbps target, GOP 30, yuv420p, one codec thread, no audio. On macOS, Skia GPU and Remotion require hardware VideoToolbox; CPU uses x264 medium. Elsewhere all use x264 medium. Standalone encoding includes RGBA conversion (fframes) or PNG decoding (Remotion), setup, flushing and writes. Complete export measures rendering, encoding and muxing directly; browser launch and bundling are excluded. A matching target bitrate does not guarantee matching image quality.",
    "",
    "| Renderer | Encoder | Standalone encoding | Complete MP4 export |",
    "|---|---|---:|---:|",
    ...[
      ["fframes + Skia CPU", "cpu", "fframes"],
      ["fframes + Skia GPU", "gpu", "fframes-gpu"],
      ["Remotion", "remotion", "remotion"],
    ].map(
      ([label, key, engine]) =>
        `| ${label} | ${encoder(engine)} | ${result.encoding_median_ms[key]?.toFixed(2) ?? "n/a"} ms | ${result.export_median_ms[key]?.toFixed(2) ?? "n/a"} ms |`
    ),
    "",
    ...(report.environment.platform === "darwin"
      ? [
          "```mermaid",
          "flowchart LR",
          "  A[SVG scene on CPU] --> B[Skia Metal on GPU]",
          "  B --> C[RGBA readback and YUV conversion on CPU]",
          "  C --> D[VideoToolbox hardware H.264]",
          "  D --> E[MP4 muxing and writes]",
          "```",
          "",
        ]
      : []),
  ].join("\n");
}
