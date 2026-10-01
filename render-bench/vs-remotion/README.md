# fframes + Skia vs Remotion

```sh
./render-bench/vs-remotion/run.sh
```

Requires Rust, Node.js and Chromium (`CHROME_PATH` or `--chrome PATH`).
Results and frame PNGs go to `out/`; use `--out DIR` to choose a directory.

One fixed 1000×1000 scene: 20 animated panels containing 99,000 overlapping
16–28px rectangles and 1,000 changing text digits. Each panel has a blurred
glow, color saturation and a soft drop shadow; text is painted last.
Both use DM Sans Regular from `examples/beta/media/DMSans-Regular.ttf`.
Remotion uses an unkeyed list with 12 dependent effect/state updates per element.
fframes computes the same content directly with Skia CPU and, when available,
Skia GPU (Metal on macOS, Vulkan elsewhere). GPU is skipped without a hardware device.
Results apply to this React workload with many effects.

All render serially without a video encoder. Times are medians of three 30-frame
runs after three warm-up frames. PNG compression is included; startup, warm-up
and disk writes are excluded.

The runner writes timings and versions to `results.json`, with a summary in
`results.md`. Generated results and dependency lockfiles are ignored.

Measured on Apple M5 Max, with Remotion 4.0.529:

| Renderer                   | Median for 30 frames | Speedup vs Remotion |
| -------------------------- | -------------------: | ------------------: |
| fframes + Skia CPU         |              5.683 s |              16.86× |
| fframes + Skia GPU (Metal) |              2.005 s |              47.80× |
| Remotion                   |             95.843 s |                   — |
