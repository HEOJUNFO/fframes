# fframes + Skia vs Remotion

```sh
./render-bench/vs-remotion/run.sh
```

Requires Rust, Node.js, ffprobe and Chromium (`CHROME_PATH` or `--chrome PATH`).
MP4 files and results go to `out/`; use `--out DIR` to choose a directory.

One fixed 1000×1000 scene: 99,000 rectangles and 1,000 changing DM Sans text
digits in 20 panels with blur, glow and shadows. Remotion uses an unkeyed list
with 12 dependent effect/state updates per element. Results describe this workload.
Skia CPU and available hardware GPUs render the same content. Both use
100,000-entry text and geometry caches, with 51.2 MB of geometry per generation.

The only timing is wall-clock time to finish a 30-frame H.264 MP4: scene rendering,
pixel conversion, encoder setup, encoding, draining, muxing and file writes.
The result is the median of three rounds, each after a three-frame MP4 warm-up.
Compilation, media loading, browser launch, bundling and warm-up are excluded.
Completed files are checked for 30 frames and one second of video after timing.

Settings: 30 fps, 8 Mbps target, GOP 30, 8-bit 4:2:0, one rendering pipeline,
one codec thread, no audio. On macOS, Skia GPU and Remotion require hardware
VideoToolbox; CPU uses x264 medium. Elsewhere all use x264 medium. GPU is skipped
when no hardware device is available. Equal bitrate targets do not guarantee
equal image quality.

`results.json` contains timings, output metadata, settings and versions;
`results.md` contains the comparison table. Generated results are ignored.

Apple M5 Max, Remotion 4.0.529, Chrome for Testing 149.0.7790.0:

| Renderer                   | Encoder               | Complete MP4 export | Speedup vs Remotion |
| -------------------------- | --------------------- | ------------------: | ------------------: |
| fframes + Skia CPU         | x264 medium           |             5.708 s |              18.31× |
| fframes + Skia GPU (Metal) | VideoToolbox hardware |             1.526 s |              68.51× |
| Remotion                   | VideoToolbox hardware |           104.540 s |                   — |
