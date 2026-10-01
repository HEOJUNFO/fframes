use crate::ffmpeg_sys_fframes::AVPixelFormat::{self, *};
use crate::renderer::pix_fmt::fill_yuv420_from_rgba_pixmap_base;
use crate::{
    EncoderInput, EncoderOptions, FramePool, RgbaFrameConverter, VideoEncoderInfo,
    is_hardware_pixel_format,
};
use std::path::Path;

fn random_rgba(width: usize, height: usize) -> Vec<u8> {
    (0..width * height * 4)
        .map(|_| rand::random::<u8>())
        .collect()
}

#[test]
fn converter_produces_the_reference_yuv420() {
    // widths that are and are not a multiple of the SIMD block
    for (width, height) in [(64, 48), (36, 20), (1920, 1080)] {
        let rgba = random_rgba(width, height);
        let mut converter =
            RgbaFrameConverter::new(AV_PIX_FMT_YUV420P, width as i32, height as i32).unwrap();
        let frame = converter.convert(&rgba).unwrap();

        assert_eq!(frame.pixel_format(), AV_PIX_FMT_YUV420P);
        assert_eq!(
            (frame.width(), frame.height()),
            (width as i32, height as i32)
        );

        let (chroma_width, chroma_height) = (width / 2, height / 2);
        let mut luma = vec![0; width * height];
        let mut cb = vec![0; chroma_width * chroma_height];
        let mut cr = vec![0; chroma_width * chroma_height];
        fill_yuv420_from_rgba_pixmap_base(
            width as i32,
            height as i32,
            width as i32,
            chroma_width as i32,
            chroma_width as i32,
            &rgba,
            luma.as_mut_ptr(),
            cb.as_mut_ptr(),
            cr.as_mut_ptr(),
        );

        // The NEON converter rounds chroma to the nearest value, the portable one down.
        let close = |actual: Vec<u8>, expected: &[u8]| {
            actual.len() == expected.len()
                && actual
                    .iter()
                    .zip(expected)
                    .all(|(a, e)| a.abs_diff(*e) <= 1)
        };
        assert!(close(frame.copy_plane(0, width, height), &luma));
        assert!(close(frame.copy_plane(1, chroma_width, chroma_height), &cb));
        assert!(close(frame.copy_plane(2, chroma_width, chroma_height), &cr));
    }
}

#[test]
fn converter_uses_swscale_for_other_formats() {
    let (width, height) = (32, 16);
    let white = vec![255_u8; width * height * 4];

    for format in [AV_PIX_FMT_NV12, AV_PIX_FMT_YUV444P, AV_PIX_FMT_YUVA420P] {
        let mut converter = RgbaFrameConverter::new(format, width as i32, height as i32).unwrap();
        let frame = converter.convert(&white).unwrap();

        assert_eq!(frame.pixel_format(), format);
        assert!(
            frame
                .copy_plane(0, width, height)
                .iter()
                .all(|y| y.abs_diff(235) <= 1),
            "{format:?}: white is 235 in limited range"
        );
    }

    let mut converter = RgbaFrameConverter::new(AV_PIX_FMT_NV12, 32, 16).unwrap();
    let chroma = converter.convert(&white).unwrap().copy_plane(1, 32, 8);
    assert!(chroma.iter().all(|c| c.abs_diff(128) <= 1));
}

#[test]
fn converter_rejects_short_buffers_and_hardware_formats() {
    let mut converter = RgbaFrameConverter::new(AV_PIX_FMT_YUV420P, 16, 16).unwrap();
    assert!(converter.convert(&[0; 16]).is_err());

    assert!(is_hardware_pixel_format(AV_PIX_FMT_VULKAN));
    assert!(is_hardware_pixel_format(AV_PIX_FMT_VIDEOTOOLBOX));
    assert!(!is_hardware_pixel_format(AV_PIX_FMT_NV12));
    assert!(RgbaFrameConverter::new(AV_PIX_FMT_VULKAN, 16, 16).is_err());
}

#[test]
fn frame_pool_recycles_buffers_and_aligns_planes() {
    let pool = FramePool::new(AV_PIX_FMT_YUV420P, 1080, 1920).unwrap();
    let layout = *pool.layout();
    assert!(
        layout.planes[..3]
            .iter()
            .all(|plane| plane.is_some_and(|(_, linesize)| linesize % 64 == 0))
    );
    assert_eq!(layout.planes[3], None);

    let first = pool.get().unwrap();
    let first_pixels = first.plane(0).0;
    assert_eq!(first_pixels as usize % 16, 0);
    assert!(first.plane(0).1 >= 1080 && first.plane(1).1 >= 540);

    // a frame that is still in use keeps its buffer
    let second = pool.get().unwrap();
    assert_ne!(second.plane(0).0, first_pixels);

    drop(first);
    assert_eq!(pool.get().unwrap().plane(0).0, first_pixels);

    // frames outlive the pool they came from
    drop(pool);
    unsafe { second.plane(0).0.write(42) };
}

fn encoder_info<'a>(options: &'a EncoderOptions<'a>) -> VideoEncoderInfo<'a> {
    VideoEncoderInfo::for_output(Path::new("video.mp4"), (64, 64, 30), options).unwrap()
}

#[test]
fn default_negotiation_keeps_the_requested_format() {
    let options = EncoderOptions {
        preferred_encoder: Some("mpeg4"),
        ..Default::default()
    };
    let encoder = encoder_info(&options);
    assert_eq!(encoder.name(), "mpeg4");
    assert!(encoder.supports(AV_PIX_FMT_YUV420P));
    assert!(!encoder.lists(AV_PIX_FMT_VULKAN));

    let input = EncoderInput::requested(&encoder).unwrap();
    assert_eq!(input.pixel_format, AV_PIX_FMT_YUV420P);
    assert_eq!(input.software_format(), AV_PIX_FMT_YUV420P);
    assert!(!input.is_hardware());
    encoder.try_open(&input).unwrap();

    let unsupported = EncoderOptions {
        preferred_encoder: Some("mpeg4"),
        pixel_format: AVPixelFormat::AV_PIX_FMT_YUV444P,
        ..Default::default()
    };
    assert!(EncoderInput::requested(&encoder_info(&unsupported)).is_err());
}

#[test]
fn unknown_containers_and_encoders_are_errors() {
    let options = EncoderOptions::default();
    assert!(
        VideoEncoderInfo::for_output(Path::new("video.not-a-container"), (64, 64, 30), &options)
            .is_err()
    );
}
