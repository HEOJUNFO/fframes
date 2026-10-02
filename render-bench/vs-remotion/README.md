# fframes vs Remotion + FFmpeg and Remotion + MediaBunny

```sh
./render-bench/vs-remotion/run.sh
```

Requires Rust, Node.js, ffprobe and Chrome with H.264 WebCodecs support
(`CHROME_PATH` or `--chrome PATH`) for MediaBunny. The FFmpeg path uses Remotion's
headless Chrome; override it with `FFMPEG_CHROME_PATH` or `--ffmpeg-chrome PATH`.
MP4 files and results go to `out/`; use `--out DIR` to choose a directory.

One command runs four pipelines: fframes CPU, fframes + Skia GPU,
Remotion + FFmpeg (`renderMedia`), and Remotion + MediaBunny (`renderMediaOnWeb`).
Both Remotion paths import the same component from `scene.jsx`.

One fixed 1000×1000 scene: 99,000 rectangles and 1,000 changing DM Sans text
digits in 20 panels with blur, glow and shadows. Both Remotion paths use an unkeyed list
with 12 dependent effect/state updates per element. Results describe this workload.
fframes uses its built-in `CpuRenderingBackend` (tiny-skia) and Skia GPU when
hardware is available. Both have 100,000-entry text caches. CPU keeps its default
20-layer cache; Skia GPU uses 100,000 geometry entries with 51.2 MB per generation.

The only timing is wall-clock time to finish a 30-frame H.264 MP4: scene rendering,
pixel conversion, encoder setup, encoding, draining, muxing and file writes.
The result is the median of three rounds, each after a three-frame MP4 warm-up.
Compilation, media loading, browser launch, bundling and warm-up are excluded.
Remotion's web renderer uses MediaBunny to write to the browser's origin-private
file system. Copying that file to the host for verification is excluded.
Completed files are checked for 30 frames and one second of video after timing.

Settings: 30 fps, 8 Mbps target, GOP 30, 8-bit 4:2:0, one rendering pipeline,
no audio. Native encoders use one codec thread: CPU uses x264 medium; Skia GPU
and Remotion + FFmpeg require hardware VideoToolbox on macOS and use x264 elsewhere. GPU is skipped
when no hardware device is available. MediaBunny uses WebCodecs H.264 with
`prefer-hardware`; Chrome controls encoder selection and threading. This preference
does not prove which encoder Chrome selected. Equal bitrate targets do not
guarantee equal image quality.

`results.json` contains timings, output metadata, settings and versions;
`results.md` contains all four rows and speedups against each Remotion pipeline.
Browser versions are recorded separately. Generated results are ignored.

Saved M5 Max results, three-run medians from two separate host sessions:

| Session    | fframes CPU | fframes + Skia GPU | Remotion + FFmpeg | Remotion + MediaBunny |
| ---------- | ----------: | -----------------: | ----------------: | --------------------: |
| FFmpeg     |     8.373 s |            1.435 s |         102.803 s |                     — |
| MediaBunny |     8.436 s |            1.514 s |                 — |             112.274 s |

Remotion 4.0.529 in both runs. The FFmpeg run used Chrome headless shell
149.0.7790.0; the MediaBunny run used Chrome 155.0.8059.27 and MediaBunny 1.56.1.
Raw measurements: [FFmpeg](measurements/m5-max-ffmpeg.json),
[MediaBunny](measurements/m5-max-mediabunny.json).
