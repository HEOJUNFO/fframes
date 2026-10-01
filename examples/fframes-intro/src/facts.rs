//! Every measured number that appears on screen, in one place. They come
//! from real runs (see render-bench/vs-remotion and this project's CLI) and
//! are refreshed before the final render.

/// Release shown on the opening and closing screens.
pub const RELEASE_LABEL: &str = concat!("v", env!("CARGO_PKG_VERSION"));

/// Lines of Rust in fframes plus its SVG renderer fork, svgr (`wc -l` over
/// the git-tracked `.rs` files of both repositories).
pub const LINES_OF_RUST: u64 = 89_372;
/// Frames in this video.
pub const FRAMES: u64 = 7_650;
/// Measured wall-clock seconds for this video on Apple M5 Max (Skia on Metal,
/// libx264, with audio).
pub const RENDER_SECONDS: f32 = 79.6;

/// Median complete H.264 MP4 export seconds for 30 frames at 1000×1000.
/// Measured on Apple M5 Max with `CpuRenderingBackend`, Skia Metal and Remotion 4.0.529,
/// serially, after a three-frame MP4 warm-up. Encoding and file writes are included.
pub const BENCH_REMOTION_S: f32 = 102.802_53;
pub const BENCH_FFRAMES_S: f32 = 1.435_308;
pub const BENCH_FFRAMES_CPU_S: f32 = 8.372_914;
pub const BENCH_REMOTION_LABEL: &str = "REMOTION 4.0";
pub const BENCH_FFRAMES_LABEL: &str = "FFRAMES · SKIA ON METAL";
pub const BENCH_NOTE: &str = "M5 MAX · REMOTION 4.0.529 · SERIAL · H.264 MP4 · MEDIANS";
