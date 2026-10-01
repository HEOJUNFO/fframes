//! Render-only twin of browser.jsx. No video encoder is instantiated.
use std::{hint::black_box, path::Path, time::Instant};

use fframes::{
    AudioMap, Color, Duration, FFramesContext, Frame, Previewer, RenderOptions,
    StaticMediaProvider, Svgr, Transform, Video,
};
use fframes_skia_renderer::{SkiaBackend, SkiaCpuCtx, SkiaFrameRenderer};

fframes::include_media_dir!(struct BenchMedia, "render-bench/vs-remotion/media");

const SIDE: usize = 1000;
const NODES: usize = 100_000;
const PANELS: usize = 20;
const PER_PANEL: usize = NODES / PANELS;
const FRAMES: usize = 30;
const WARMUP: usize = 3;

struct Grid;

impl Video for Grid {
    const FPS: usize = 30;
    const WIDTH: usize = SIDE;
    const HEIGHT: usize = SIDE;
    const BACKGROUND_COLOR: Color = Color::hex("#18202c");
    fn duration(&self) -> Duration<'_> {
        Duration::Frames(FRAMES + WARMUP)
    }
    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }
    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let panels: Vec<_> = (0..PANELS).map(|panel| {
            let rectangles: Vec<_> = (panel * PER_PANEL..(panel + 1) * PER_PANEL)
                .filter(|slot| slot % 100 != 0).map(|slot| {
                    let id = (slot + frame.index * 37) % NODES;
                    let fill = Color::rgba(
                        ((id * 13 + frame.index * 17) % 256) as u8,
                        ((id * 7 + frame.index * 29) % 256) as u8,
                        ((id * 3 + frame.index * 43) % 256) as u8, 255);
                    fframes::svgr!(<rect x={24 + (slot * 13 + frame.index * 3) % 128}
                        y={24 + (slot * 17 + frame.index * 5) % 176}
                        width={16 + id % 4 * 4} height={16 + id / 4 % 4 * 4} fill={fill} />)
                }).collect();
            let transform = Transform::translate((panel % 5 * 200) as f64, (panel / 5 * 250) as f64);
            fframes::svgr!(<g transform={transform} filter="url(#panel-effects)">{rectangles}</g>)
        }).collect();
        let texts: Vec<_> = (0..NODES).step_by(100).map(|slot| {
            let panel = slot / PER_PANEL;
            let index = slot % PER_PANEL / 100;
            let id = (slot + frame.index * 37) % NODES;
            fframes::svgr!(<text x={panel % 5 * 200 + 24 + index % 10 * 16}
                y={panel / 5 * 250 + 40 + index / 10 * 40}
                font-family="DM Sans" font-size="16" fill="#fff">{((id + frame.index) % 10).to_string()}</text>)
        }).collect();
        fframes::svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" width="1000" height="1000">
                <defs>
                    <filter id="panel-effects" x="-40%" y="-40%" width="180%" height="180%" color-interpolation-filters="sRGB">
                        <feGaussianBlur stdDeviation="8" result="glow" />
                        <feColorMatrix in="glow" type="saturate" values="1.8" result="bright" />
                        <feMerge><feMergeNode in="bright" /><feMergeNode in="SourceGraphic" /></feMerge>
                        <feDropShadow dx="4" dy="8" stdDeviation="12" flood-color="#000" flood-opacity="0.7" />
                    </filter>
                </defs>
                {panels}
                {texts}
            </svg>
        )
    }
}

#[cfg(target_os = "macos")]
fn gpu_backend() -> Result<fframes_skia_renderer::metal::SkiaMetalCtx, Box<dyn std::error::Error>> {
    Ok(fframes_skia_renderer::metal::SkiaMetalCtx::new(SIDE, SIDE)?)
}

#[cfg(not(target_os = "macos"))]
fn gpu_backend() -> Result<fframes_skia_renderer::vulkan::SkiaVulkanCtx, Box<dyn std::error::Error>>
{
    use ash::vk;

    // Select hardware explicitly; lavapipe/llvmpipe is a CPU renderer.
    let entry = unsafe { ash::Entry::load()? };
    let app = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_1);
    let instance = unsafe {
        entry.create_instance(
            &vk::InstanceCreateInfo::default().application_info(&app),
            None,
        )?
    };
    let devices = unsafe { instance.enumerate_physical_devices()? };
    let device = devices.into_iter().find(|&device| {
        unsafe { instance.get_physical_device_properties(device) }.device_type
            != vk::PhysicalDeviceType::CPU
    });
    let Some(device) = device else {
        unsafe { instance.destroy_instance(None) };
        return Err("no hardware Vulkan GPU available".into());
    };
    Ok(
        fframes_skia_renderer::vulkan::SkiaVulkanCtx::new_with_device(
            entry, instance, device, SIDE, SIDE,
        )?,
    )
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "usage: vs-remotion-bench cpu|gpu OUTPUT_DIR");
    let out = Path::new(&args[2]);
    std::fs::create_dir_all(out).expect("output directory");
    match args[1].as_str() {
        "cpu" => run(&SkiaCpuCtx::new(SIDE, SIDE), "skia-cpu", out),
        "gpu" => match gpu_backend().and_then(|backend| {
            backend.create_skia_surface()?;
            Ok(backend)
        }) {
            Ok(backend) => run(
                &backend,
                if cfg!(target_os = "macos") {
                    "skia-metal"
                } else {
                    "skia-vulkan"
                },
                out,
            ),
            Err(error) => println!(
                "{}",
                serde_json::json!({"status": "skipped", "reason": error.to_string()})
            ),
        },
        _ => panic!("expected cpu or gpu"),
    }
}

fn run(backend: &impl SkiaBackend, name: &str, out: &Path) {
    let video = Grid;
    let media = BenchMedia::prepare().expect("benchmark font");
    let options = RenderOptions {
        media: Some(&media),
        load_system_fonts: false,
        ..Default::default()
    };
    let mut preview = Previewer::new(&video, &options).expect("preview initialization");
    let mut renderer = SkiaFrameRenderer::new(backend);
    let mut samples = Vec::new();
    for frame in 0..FRAMES + WARMUP {
        let start = Instant::now();
        // Includes scene generation, SVG conversion, and rasterization.
        let image = preview.render(frame, &mut renderer).expect("render frame");
        let render_ms = start.elapsed().as_secs_f64() * 1000.;
        // Chrome exposes screenshots as PNGs. Produce the same artifact in memory;
        // record image compression separately and include it in the comparable total.
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, SIDE as u32, SIDE as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_compression(png::Compression::Fast);
            encoder
                .write_header()
                .expect("PNG header")
                .write_image_data(&image.pixels)
                .expect("PNG pixels");
        }
        let total_ms = start.elapsed().as_secs_f64() * 1000.;
        black_box(&bytes);
        if frame >= WARMUP {
            samples.push(serde_json::json!({"frame": frame, "render_ms": render_ms, "png_ms": total_ms-render_ms, "total_ms": total_ms}));
            // Disk writes and correctness checks are excluded on both sides.
            std::fs::write(out.join(format!("{frame}.png")), &bytes)
                .expect("save verification PNG");
        }
    }
    println!(
        "{}",
        serde_json::json!({"backend": name, "nodes": NODES, "samples": samples})
    );
}
