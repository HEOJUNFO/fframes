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
/// Wall-clock seconds of the final render of this video (Skia on Metal,
/// libx264, with audio).
pub const RENDER_SECONDS: f32 = 40.5;

/// Median render-and-PNG seconds for 30 frames at 1000×1000.
/// Measured on Apple M5 Max with Skia CPU, Metal and Remotion 4.0.529,
/// serially, after three warm-up frames. Video encoding is excluded.
pub const BENCH_REMOTION_S: f32 = 95.842_79;
pub const BENCH_FFRAMES_S: f32 = 2.005_096_2;
pub const BENCH_FFRAMES_CPU_S: f32 = 5.683_136;
pub const BENCH_REMOTION_LABEL: &str = "REMOTION 4.0";
pub const BENCH_FFRAMES_LABEL: &str = "FFRAMES · SKIA ON METAL";
pub const BENCH_NOTE: &str =
    "M5 MAX · REMOTION 4.0.529, SERIAL, PRE-BUNDLED · VIDEO ENCODING NOT MEASURED · MEDIANS";
