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
        r.video?.frames === plan.frames &&
        Number.isFinite(r.export_ms) &&
        r.export_ms > 0 &&
        ["libx264", "h264_videotoolbox"].includes(r.encoder)
    );
  const cpuComplete = valid(native);
  const remotionComplete = valid(browser);
  const gpuComplete = valid(gpu);
  const gpuSkipped =
    gpu.length === plan.rounds && gpu.every(r => r.status === "skipped");
  const cpuMs = cpuComplete ? median(native.map(r => r.export_ms)) : null;
  const remotionMs = remotionComplete
    ? median(browser.map(r => r.export_ms))
    : null;
  const gpuMs = gpuComplete ? median(gpu.map(r => r.export_ms)) : null;
  return {
    status:
      cpuComplete && remotionComplete && (gpuComplete || gpuSkipped)
        ? "complete"
        : "incomplete",
    export_median_ms: { cpu: cpuMs, gpu: gpuMs, remotion: remotionMs },
    speedup: cpuComplete && remotionComplete ? remotionMs / cpuMs : null,
    gpu_speedup: gpuComplete && remotionComplete ? remotionMs / gpuMs : null,
    gpu_status: gpuSkipped
      ? "skipped"
      : gpuComplete
        ? "complete"
        : "incomplete",
    gpu_skip_reason: gpuSkipped ? gpu[0].reason : null,
  };
}
export function markdown(report) {
  const result = report.summary;
  const encoder = engine =>
    report.records.find(r => r.engine === engine && r.status === "ok")
      ?.encoder ?? "n/a";
  const seconds = value =>
    value == null ? "incomplete" : `${(value / 1000).toFixed(3)} s`;
  return [
    "# fframes vs Remotion",
    "",
    "One 1000×1000 scene: 99,000 rectangles and 1,000 changing text digits in 20 panels with blur, glow and shadows. Remotion uses an unkeyed list with 12 dependent effect/state updates per element. Results describe this workload.",
    "",
    "Median of three complete 30-frame H.264 MP4 exports, each after a three-frame MP4 warm-up. Includes rendering, pixel conversion, encoder setup, encoding, draining, muxing and file writes. Compilation, media loading, browser launch, bundling and warm-up are excluded. Output frame counts and durations are verified after timing.",
    "",
    "| Renderer | Encoder | Complete MP4 export | Speedup vs Remotion |",
    "|---|---|---:|---:|",
    `| fframes CPU (tiny-skia) | ${encoder("fframes")} | ${seconds(result.export_median_ms.cpu)} | ${result.speedup?.toFixed(2) ?? "n/a"}× |`,
    `| fframes + Skia GPU | ${encoder("fframes-gpu")} | ${result.gpu_status === "skipped" ? `skipped: ${result.gpu_skip_reason}` : seconds(result.export_median_ms.gpu)} | ${result.gpu_speedup?.toFixed(2) ?? "n/a"}× |`,
    `| Remotion | ${encoder("remotion")} | ${seconds(result.export_median_ms.remotion)} | — |`,
    "",
    "30 fps, 8 Mbps target, GOP 30, 8-bit 4:2:0, one rendering pipeline, one codec thread, no audio. On macOS, Skia GPU and Remotion require hardware VideoToolbox; CPU uses x264 medium. Elsewhere all use x264 medium. Equal bitrate targets do not guarantee equal image quality.",
    "",
  ].join("\n");
}
