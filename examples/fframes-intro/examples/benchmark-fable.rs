use std::f32::consts::TAU;
use std::path::PathBuf;
use std::process::ExitCode;

use fframes::{
    AudioMap, AudioTimestamp, AudioTrack, Color, CombinedMediaProvider, Duration, EncoderOptions,
    FFramesContext, Frame, MediaDirectory, MediaProvider, RenderOptions, StaticMediaProvider, Svgr,
    Video, cli, svgr,
};
use fframes_intro::IntroMedia;
use fframes_skia_renderer::{
    SkiaFFramesRenderer, SkiaPipelineConcurrencyPolicy, SkiaPipelineConfig,
};

const INK: &str = "#17171d";
const PAPER: &str = "#f0eadc";
const RUST: &str = "#ff542b";
const LIME: &str = "#ddf896";

fn unit(t: f32, a: f32, b: f32) -> f32 {
    ((t - a) / (b - a)).clamp(0.0, 1.0)
}

fn ease(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn spring(t: f32) -> f32 {
    if t >= 1.0 {
        1.0
    } else {
        1.0 - (-7.0 * t).exp() * (11.0 * t).cos()
    }
}

fn camera(x: f32, y: f32, scale: f32, angle: f32) -> String {
    format!(
        "translate({x} {y}) scale({scale}) rotate({angle}) translate({} {})",
        -x, -y
    )
}

fn hash(n: usize) -> f32 {
    ((n as f32 * 127.13 + 17.7).sin() * 4357.37).fract().abs()
}

fn tr(x: f32, y: f32, angle: f32, scale: f32) -> String {
    format!("translate({x} {y}) rotate({angle}) scale({scale})")
}

fn word(x: f32, y: f32, size: f32, text: &str, fill: &str) -> Svgr<'static> {
    svgr!(<text x={x} y={y} font-family="Archivo Black" font-size={size}
        letter-spacing={-size * 0.055} fill={fill.to_owned()}>{text.to_owned()}</text>)
}

fn mono(x: f32, y: f32, size: f32, text: &str, fill: &str) -> Svgr<'static> {
    svgr!(<text x={x} y={y} font-family="IBM Plex Mono" font-size={size}
        font-weight="500" fill={fill.to_owned()}>{text.to_owned()}</text>)
}

fn italic(x: f32, y: f32, size: f32, text: &str, fill: &str) -> Svgr<'static> {
    svgr!(<text x={x} y={y} font-family="Instrument Serif" font-style="italic"
        font-size={size} fill={fill.to_owned()}>{text.to_owned()}</text>)
}

fn rays(t: f32, cx: f32, cy: f32, color: &str, strength: f32) -> Svgr<'static> {
    (0..35).map(|i| {
        let angle = i as f32 * TAU / 35.0 + t * 0.05;
        let inner = 270.0 + hash(i) * 280.0;
        let outer = inner + 230.0 + hash(i + 5) * 900.0;
        let (s, c) = angle.sin_cos();
        let d = format!("M{} {} L{} {}", cx + c * inner, cy + s * inner,
            cx + c * outer, cy + s * outer);
        svgr!(<path d={d} fill="none" stroke={color.to_owned()}
            stroke-width={1.0 + hash(i + 90) * 8.0} opacity={strength * (0.2 + hash(i + 20) * 0.7)} />)
    }).collect()
}

fn ground(t: f32, speed: f32, color: &str) -> Svgr<'static> {
    (0..24)
        .map(|i| {
            let x = (i as f32 * 131.0 - t * speed).rem_euclid(2400.0) - 200.0;
            let y = 835.0 + hash(i) * 95.0;
            svgr!(<path d={format!("M{x} {y} l{} -2", 25.0 + hash(i + 3) * 105.0)}
            stroke={color.to_owned()} stroke-width={1.0 + hash(i + 4) * 2.0} opacity="0.4" />)
        })
        .collect()
}

fn bunny(t: f32, panic: f32, dead: bool) -> Svgr<'static> {
    let tick = (t * 14.0).floor() / 14.0;
    let hop = (tick * 16.0).sin();
    let ear = (tick * 13.0).sin() * (8.0 + panic * 16.0);
    let legs = if dead { 0.0 } else { hop * 46.0 };
    let eyes = if dead {
        svgr!(<g stroke={INK} stroke-width="9" stroke-linecap="round">
            <path d="M65 -106 l26 26 M91 -106 l-26 26 M111 -105 l20 22 M131 -105 l-20 22" />
        </g>)
    } else {
        svgr!(<g>
            <ellipse cx="81" cy="-94" rx={12.0 + panic * 5.0} ry={21.0 + panic * 7.0} fill={INK} />
            <ellipse cx="123" cy="-92" rx="9" ry={16.0 + panic * 7.0} fill={INK} />
            <circle cx={84.0 + panic * 3.0} cy="-103" r="5" fill={PAPER} />
        </g>)
    };
    let mouth = if panic > 0.45 && !dead {
        svgr!(<ellipse cx="122" cy="-34" rx="10" ry={10.0 + panic * 5.0} fill={INK} stroke="none" />)
    } else {
        svgr!(<path d="M133 -34 q-17 12 -26 -4" fill="none" stroke-width="5" />)
    };
    svgr!(<g stroke={INK} stroke-width="7" stroke-linejoin="round">
        <ellipse cx="-50" cy="18" rx="95" ry="106" fill={PAPER} transform="rotate(-18)" />
        <ellipse cx="-131" cy="20" rx="32" ry="31" fill={PAPER} />
        <g transform={tr(60.0, -122.0, -18.0 + ear, 1.0)}>
            <path d="M-22 5 C-73 -111 -72 -214 -36 -213 C0 -215 16 -96 14 8Z" fill={PAPER} />
            <path d="M-22 -24 Q-60 -164 -36 -183 Q-12 -124 -9 -29" fill="#c1b3c4" stroke="none" />
        </g>
        <g transform={tr(95.0, -123.0, 8.0 + ear * 0.6, 1.0)}>
            <path d="M-14 14 C-19 -120 6 -201 29 -188 C58 -170 37 -70 18 14Z" fill={PAPER} />
            <path d="M4 -23 Q1 -128 24 -166 Q31 -106 16 -24" fill="#c1b3c4" stroke="none" />
        </g>
        <path d="M6 -104 C12 -158 103 -157 138 -115 C164 -87 174 -39 128 -26 C74 -2 8 -25 6 -104Z" fill={PAPER} />
        {eyes}
        <path d="M154 -70 l-18 -5 l4 16Z" fill={INK} stroke="none" />
        {mouth}
        <path d="M-24 -18 Q35 33 87 -7" fill="none" stroke-width="24" stroke={INK} />
        <path d="M-24 -18 Q35 33 87 -7" fill="none" stroke-width="14" stroke={PAPER} />
        <g transform={tr(-74.0, 77.0, legs, 1.0)}>
            <path d="M-23 -20 Q-46 13 -6 31 L61 40 Q93 37 74 15 L27 -5Z" fill={PAPER} />
        </g>
        <g transform={tr(5.0, 69.0, -legs, 1.0)}>
            <path d="M-23 -20 Q-46 13 -6 31 L61 40 Q93 37 74 15 L27 -5Z" fill={PAPER} />
        </g>
        <path d="M-85 -34 q-24 47 -8 74 M-70 -12 l-11 30" fill="none" stroke-width="3" opacity="0.24" />
    </g>)
}

fn crab(t: f32, charge: f32) -> Svgr<'static> {
    let tick = (t * 12.0).floor() / 12.0;
    let walk = (tick * 15.0).sin() * 29.0;
    let limbs: Svgr<'static> = (0..3)
        .flat_map(|i| {
            [-1.0, 1.0].map(move |side| {
        let y = i as f32 * 23.0;
        let d = format!("M{} {} L{} {} L{} {}", side * 89.0, 10.0 + y,
            side * (154.0 + i as f32 * 12.0), 48.0 + y + walk,
            side * (162.0 + i as f32 * 19.0), 109.0 + y - walk);
        svgr!(<path d={d} fill="none" stroke={RUST} stroke-width="18" stroke-linejoin="bevel" />)
    })
        })
        .collect();
    let claws: Svgr<'static> = [-1.0, 1.0]
        .into_iter()
        .map(|side| {
            svgr!(<g transform={format!("scale({side} 1)")}>
            <path d="M102 -18 Q186 -34 178 -124" stroke={RUST} stroke-width="23" fill="none" />
            <g transform={tr(177.0, -137.0, -charge * 28.0 + walk * 0.2, 1.0)}>
                <path d="M-18 42 C-83 5 -61 -65 -33 -80 L-9 -18 L23 -80 C67 -35 50 20 15 43Z"
                    fill={RUST} stroke={INK} stroke-width="6" />
                <path d="M-37 -49 L-15 -10" stroke="#ffb08b" stroke-width="5" />
            </g>
        </g>)
        })
        .collect();
    svgr!(<g>
        {limbs}
        {claws}
        <path d="M-120 14 L-128 -7 L-105 -21 L-99 -55 L-77 -53 L-62 -79 L-38 -68 L-19 -88 L7 -71 L36 -87 L57 -65 L84 -67 L94 -40 L120 -31 L117 -2 L136 15 L115 42 L113 63 L77 88 L-77 88 L-111 64Z"
            fill={RUST} stroke={INK} stroke-width="7" />
        <path d="M-92 10 Q0 -53 99 8" stroke="#ffb793" stroke-width="7" fill="none" opacity="0.8" />
        <path d="M-47 -44 L-54 -105 M44 -44 L53 -107" stroke={RUST} stroke-width="19" />
        <ellipse cx="-54" cy="-110" rx="20" ry="26" fill={PAPER} stroke={INK} stroke-width="5" />
        <ellipse cx="53" cy="-112" rx="20" ry="26" fill={PAPER} stroke={INK} stroke-width="5" />
        <circle cx="-48" cy="-108" r="10" fill={INK} />
        <circle cx="59" cy="-110" r="10" fill={INK} />
        <path d="M-77 -133 l39 12 M34 -122 l42 -17" stroke={INK} stroke-width="9" />
        <path d="M-37 39 Q0 63 44 32" fill="none" stroke={INK} stroke-width="7" />
        <path d="M-12 -6 h25 q21 0 21 16 q0 11 -13 14 l18 19 h-21 l-17 -20 h-2 v20 h-18Z M1 6 v7 h10 q6 0 6 -4 q0 -3 -6 -3Z" fill={INK} />
    </g>)
}

fn wheel(t: f32) -> Svgr<'static> {
    let spokes: Svgr<'static> = (0..36)
        .map(|i| {
            svgr!(<path d="M0 -287 L0 -254" stroke={PAPER} stroke-width="5"
            transform={format!("rotate({})", i as f32 * 10.0 + t * 230.0)} />)
        })
        .collect();
    svgr!(<g>
        <circle r="290" fill="none" stroke={PAPER} stroke-width="13" />
        <circle r="267" fill="none" stroke={PAPER} stroke-width="2" opacity="0.3" />
        {spokes}
        <path d="M-190 315 H190 M-150 315 L-78 284 M150 315 L78 284" stroke={PAPER} stroke-width="10" fill="none" />
    </g>)
}

fn waiting(t: f32) -> Svgr<'static> {
    let p = ease(unit(t, 0.6, 1.15));
    let jitter = ((t * 12.0).floor() * 2.4).sin();
    let pullback = ease(unit(t, 0.05, 0.95));
    let view = camera(
        1210.0,
        560.0,
        1.0 + 0.65 * (1.0 - pullback) + 0.11 * unit(t, 2.85, 3.55),
        -4.0 * (1.0 - pullback) + jitter * 0.2,
    );
    let bounce = (t * 15.0).sin().abs();
    let pages: Svgr<'static> = (0..21)
        .map(|i| {
            let phase = (i as f32 / 21.0 + t * 0.18) % 1.0;
            let x = 900.0 + (phase * TAU).cos() * (350.0 + hash(i) * 160.0);
            let y = 560.0 + (phase * TAU).sin() * (290.0 + hash(i + 3) * 160.0);
            svgr!(<g transform={tr(x, y, phase * 360.0, 0.5 + hash(i) * 0.5)} opacity="0.25">
            <path d="M0 0 l44 -8 l7 62 l-42 8Z" fill={PAPER} />
            <path d="M8 12 l22 -4 M11 25 l20 -4 M14 39 l19 -4" stroke={INK} stroke-width="3" />
        </g>)
        })
        .collect();
    svgr!(<g>
        <rect width="1920" height="1080" fill={INK} />
        <ellipse cx="1470" cy="175" rx="400" ry="410" fill="#373044" />
        <path d="M0 935 Q510 773 1010 898 T1920 823 V1080 H0Z" fill="#24222c" />
        <g transform={view}>
            {pages}
            <g transform={tr(1330.0, 552.0, (t * 6.0).sin() * 3.0, 1.0)}>{wheel(t * (1.0 + t * 0.18))}</g>
            <ellipse cx="1335" cy="787" rx={158.0 - bounce * 38.0} ry="16" fill="#000000" opacity="0.5" />
            <g transform={format!("translate({} {}) rotate({}) scale({} {})",
                1320.0 + (t * 8.0).sin() * 34.0, 665.0 - bounce * 75.0,
                -12.0 + (t * 7.0).sin() * 13.0, 0.95 - bounce * 0.12, 0.87 + bounce * 0.12)}>
                {bunny(t, unit(t, 2.2, 3.5), false)}
            </g>
            <g opacity={p} transform={tr(-45.0 * (1.0 - p), 0.0, 0.0, 1.0)}>
                {italic(145.0, 220.0, 64.0, "Just one more frame...", PAPER)}
                {mono(152.0, 370.0, 29.0, "REMOTION + MEDIABUNNY", "#b6a9c5")}
                {word(138.0, 510.0, 142.0, "112.274", PAPER)}
                {italic(161.0, 588.0, 59.0, "seconds later.", PAPER)}
                <path d="M163 650 Q405 637 659 653" fill="none" stroke={RUST} stroke-width="6" />
                {mono(157.0, 710.0, 22.0, "100,000 ELEMENTS. ONE MP4.", "#aaa0b1")}
            </g>
        </g>
        {ground(t, 360.0, PAPER)}
    </g>)
}

fn heavy(t: f32) -> Svgr<'static> {
    let q = t - 3.55;
    let jitter = (q * 24.0).sin() * (3.0 + unit(q, 2.0, 2.85) * 10.0);
    let effort = (q * 8.0).sin();
    let x = 465.0 + spring(unit(q, 0.0, 0.9)) * 320.0 + effort * 30.0;
    let rock_roll = (q * 8.0).sin() * 3.0;
    let hatch: Svgr<'static> = (0..12).map(|i| {
        let x = 1100.0 + i as f32 * 35.0;
        svgr!(<path d={format!("M{x} 823 l100 -135")} stroke={INK} stroke-width="2" opacity="0.2" />)
    }).collect();
    svgr!(<g>
        <rect width="1920" height="1080" fill={PAPER} />
        <circle cx="630" cy="508" r="417" fill="#d5cec0" />
        <path d="M0 916 L1920 794 V1080 H0Z" fill="#bcb8aa" />
        <g transform={format!("translate({jitter} {}) {}", jitter * 0.7,
            camera(1080.0, 610.0, 1.0 + 0.14 * (1.0 - ease(unit(q, 0.0, 0.65))), -2.0))}>
            <path d={format!("M{} 648 Q1007 737 1175 673", x + 200.0)} stroke={INK} stroke-width="10" fill="none" />
            <g transform={format!("translate({} {}) rotate({rock_roll} 1370 770)", -effort * 16.0, -effort.abs() * 13.0)}>
            <path d="M1126 460 L1438 419 L1606 577 L1561 817 L1282 864 L1118 712Z" fill="#d7d1c4" stroke={INK} stroke-width="10" />
            {hatch}
            {mono(1205.0, 536.0, 25.0, "FFRAMES / CPU", INK)}
            {word(1190.0, 660.0, 102.0, "8.436", INK)}
            {italic(1230.0, 737.0, 65.0, "seconds", INK)}
            <path d="M1390 425 l-40 90 l42 36 l-69 84 l36 63 l-28 138 M1392 551 l92 6 l42 60"
                fill="none" stroke={RUST} stroke-width="10" opacity={unit(q, 2.1, 2.65)} />
            </g>
            <ellipse cx={x} cy="831" rx="240" ry="26" fill={INK} opacity="0.2" />
            <g transform={format!("translate({x} {}) rotate({}) scale({} {})",
                664.0 - effort.abs() * 38.0, -8.0 + effort * 7.0,
                1.38 + effort.abs() * 0.08, 1.38 - effort.abs() * 0.10)}>{crab(q, 0.0)}</g>
            {italic(160.0, 243.0, 90.0, "Then Rust woke up.", INK)}
            <path d="M160 281 C280 269 625 280 800 269" stroke={RUST} stroke-width="8" fill="none" />
            {mono(164.0, 343.0, 24.0, "SAME JOB. LESS STRUGGLE.", INK)}
        </g>
        {ground(q, 230.0, INK)}
    </g>)
}

fn launch(t: f32) -> Svgr<'static> {
    let q = t - 6.4;
    let accelerate = ease(unit(q, 0.0, 0.65));
    let trails: Svgr<'static> = (0..65).map(|i| {
        let y = hash(i * 2 + 3) * 1080.0;
        let x = (hash(i * 2 + 1) * 2500.0 - q * (1200.0 + hash(i) * 2300.0)).rem_euclid(2600.0) - 400.0;
        let len = 40.0 + hash(i + 8) * 470.0;
        svgr!(<path d={format!("M{x} {y} l{len} -30")} stroke={if i % 4 == 0 { LIME } else { "#fba080" }}
            stroke-width={2.0 + hash(i + 5) * 12.0} opacity={0.2 + hash(i + 9) * 0.45} />)
    }).collect();
    let lunge = unit(q, 1.50, 2.18).powi(3);
    let warp = 1.0 + accelerate * 0.35 + lunge * 0.55;
    let x = 370.0 + accelerate * 280.0 + (q * 15.0).sin() * 35.0 + lunge * 630.0;
    let bob = (q * 16.0).sin();
    let lean = -5.0 - (q * 2.5).sin() * 4.5;
    svgr!(<g>
        <rect width="1920" height="1080" fill={RUST} />
        <path d="M-100 793 Q701 671 2040 251 L2040 515 Q900 840 -100 974Z" fill="#ffbf90" opacity="0.65" />
        {trails}
        <g transform={format!("rotate({lean} 960 540)")}>
            <path d={format!("M-300 705 L{} {} L{} {} L-230 843Z", x + 54.0, 611.0 + bob * 15.0, x + 180.0, 704.0 + bob * 15.0)} fill={LIME} />
            <path d={format!("M-350 743 L{} 643 L{} 689 L-300 801Z", x + 61.0, x + 133.0)} fill={PAPER} />
            <g transform={format!("translate({x} {}) rotate({}) scale({warp} {})", 678.0 + bob * 27.0, bob * 4.0, 0.85 - lunge * 0.15)}>
                {crab(q * 2.0, 1.0)}
            </g>
            <g transform={format!("translate({} {}) rotate({}) scale({} {})",
                1470.0 + (q * 12.0).sin() * 30.0, 668.0 - (q * 18.0).sin().abs() * 90.0,
                -21.0 + bob * 10.0, 0.95 + bob * 0.12, 0.9 - bob * 0.1)}>{bunny(q * 2.0, 1.0, false)}</g>
        </g>
        <g transform={format!("translate({} {}) {}", 160.0 * (1.0 - accelerate), -70.0 * (1.0 - accelerate),
            camera(580.0, 300.0, 1.0 + 0.18 * (1.0 - spring(unit(q, 0.0, 0.7))), -3.0))} opacity={accelerate}>
            {mono(209.0, 206.0, 29.0, "FFRAMES + SKIA GPU", INK)}
            {word(187.0, 404.0, 190.0, "1.514 s", INK)}
            {italic(990.0, 295.0, 75.0, "Oh.", INK)}
            {italic(1125.0, 406.0, 91.0, "Oh no.", INK)}
        </g>
    </g>)
}

fn fragments(t: f32, color: &str) -> Svgr<'static> {
    (0..190).map(|i| {
        let a = hash(i * 2 + 10) * TAU;
        let speed = 180.0 + hash(i * 2 + 2) * 1200.0;
        let x = 1270.0 + a.cos() * speed * t;
        let y = 535.0 + a.sin() * speed * t + 150.0 * t * t;
        let size = (4.0 + hash(i + 45) * 25.0) * (1.0 - unit(t, 1.6, 3.5));
        let path = format!("M{} 0 L{} {} L{} {} Z", -size, size * 0.7, -size * 0.6, size * 0.3, size);
        svgr!(<path d={path} fill={if i % 6 == 0 { RUST.to_owned() } else { color.to_owned() }}
            transform={tr(x, y, t * (hash(i + 9) - 0.5) * 900.0, 1.0)} opacity={1.0 - unit(t, 2.0, 3.5)} />)
    }).collect()
}

fn impact(t: f32) -> Svgr<'static> {
    let q = t - 8.65;
    let jolt = (q * 95.0).sin() * 26.0 * (-q * 4.0).exp();
    let flash = q < 0.06;
    let flight = unit(q, 0.04, 1.1);
    let bunny_x = 1380.0 - ease(flight) * 100.0;
    let bunny_y = 575.0 + 290.0 * flight - 280.0 * (flight * std::f32::consts::PI).sin();
    let impact_zoom = 1.0 + 0.45 * (1.0 - ease(unit(q, 0.02, 0.68)));
    let blast: Svgr<'static> = (0..12)
        .map(|i| {
            let a = i as f32 * TAU / 12.0;
            let size = 180.0 + q * 1700.0;
            let tip_x = 1270.0 + a.cos() * size;
            let tip_y = 535.0 + a.sin() * size;
            let d = format!(
                "M{} {} L{tip_x} {tip_y} L{} {} Z",
                1270.0 + (a - 0.18).cos() * size * 0.4,
                535.0 + (a - 0.18).sin() * size * 0.4,
                1270.0 + (a + 0.18).cos() * size * 0.4,
                535.0 + (a + 0.18).sin() * size * 0.4
            );
            svgr!(<path d={d} fill={LIME} opacity={1.0 - unit(q, 0.05, 0.24)} />)
        })
        .collect();
    svgr!(<g>
        <rect width="1920" height="1080" fill={if flash { PAPER } else { INK }} />
        <g transform={format!("translate({jolt} {}) {}", jolt * 0.4,
            camera(1210.0, 570.0, impact_zoom, 8.0 * (1.0 - ease(unit(q, 0.0, 0.7)))))}>
            {blast}
            {rays(q, 1270.0, 535.0, RUST, (1.0 - q * 0.55).max(0.0))}
            <circle cx="1270" cy="535" r={20.0 + q * 1400.0} fill="none" stroke={PAPER}
                stroke-width={35.0 * (1.0 - unit(q, 0.0, 0.6))} opacity={1.0 - unit(q, 0.0, 0.7)} />
            <path d="M-150 935 L1780 210 L1890 354 L-150 1045Z" fill={RUST} opacity={1.0 - unit(q, 0.12, 0.7)} />
            {fragments(q, PAPER)}
            <g transform={tr(bunny_x, bunny_y, -280.0 + ease(flight) * 360.0, 0.82 - ease(flight) * 0.20)} opacity={unit(q, 0.06, 0.14)}>
                {bunny(0.0, 0.0, true)}
            </g>
            <g transform={tr(1020.0 - ease(unit(q, 0.0, 0.75)) * 150.0,
                690.0 - 105.0 * (unit(q, 0.0, 0.85) * std::f32::consts::PI).sin(),
                -22.0 + ease(unit(q, 0.0, 0.8)) * 17.0, 1.5)}>{crab(q * 0.15, 0.8)}</g>
            <g transform={tr(1040.0, 402.0 - q * 170.0, -13.0, 0.75)} opacity={1.0 - unit(q, 0.65, 1.5)}>
                {italic(0.0, 0.0, 140.0, "...pop.", PAPER)}
            </g>
        </g>
    </g>)
}

fn aftermath(t: f32) -> Svgr<'static> {
    let q = t - 10.15;
    let reveal = ease(unit(q, 0.05, 0.8));
    let float = (q * 2.6).sin() * 35.0;
    let jump = unit(q, 0.0, 0.82);
    let land = (q - 0.82).max(0.0);
    let squash = if q >= 0.82 {
        (land * 17.0).sin() * (-land * 7.0).exp()
    } else {
        0.0
    };
    let hero_x = 870.0 + ease(jump) * 585.0;
    let hero_y = 690.0 - 6.0 * jump - 250.0 * (jump * std::f32::consts::PI).sin();
    svgr!(<g>
        <rect width="1920" height="1080" fill={INK} />
        <circle cx="1500" cy="541" r={320.0 + q * 8.0} fill="#2b2628" />
        <path d="M0 888 Q700 809 1920 920 V1080 H0Z" fill="#26232a" />
        <g transform={tr(0.0, 90.0 * (1.0 - reveal), -2.0, 1.0)} opacity={reveal}>
            {italic(148.0, 219.0, 71.0, "Less waiting. More living.", PAPER)}
            <g transform={camera(540.0, 435.0,
                0.72 + 0.28 * spring(unit(q, 0.25, 1.1)),
                -9.0 * (1.0 - ease(unit(q, 0.25, 1.1))))}>
                {word(129.0, 497.0, 236.0, "74.14×", RUST)}
            </g>
            {mono(151.0, 572.0, 24.0, "GPU EXPORT SPEEDUP VS REMOTION + MEDIABUNNY", PAPER)}
            <path d="M148 620 Q470 604 910 623" stroke={RUST} stroke-width="6" fill="none" />
            {mono(151.0, 686.0, 22.0, "112.274s → 1.514s / COMPLETE H.264 MP4", "#b8b0ad")}
            {mono(151.0, 774.0, 22.0, "OPEN DATA / BENCHMARK", LIME)}
            {mono(151.0, 813.0, 24.0, "github.com/dmtrKovalenko/fframes", PAPER)}
            {mono(151.0, 846.0, 20.0, "/tree/bench/mediabunny-m5/render-bench/vs-remotion", PAPER)}
        </g>
        <g transform={format!("translate({hero_x} {hero_y}) rotate({}) scale({} {})",
            -5.0 + jump - 22.0 * (jump * std::f32::consts::PI).sin(),
            1.5 - 0.24 * jump + squash * 0.20, 1.5 - 0.24 * jump - squash * 0.25)}>{crab(q * 0.4, 0.6)}</g>
        <g transform={tr(1280.0 + ease(jump) * 44.0, 865.0, 80.0 + jump * 10.0, 0.62 - jump * 0.10)}>{bunny(0.0, 0.0, true)}</g>
        <g transform={tr(1520.0 + ease(unit(q, 0.82, 2.0)) * 105.0 + float,
            730.0 - ease(unit(q, 0.82, 2.0)) * 319.0 - q * 25.0,
            -12.0 + (q * 3.0).sin() * 15.0, 0.34)} opacity={(0.75 - q * 0.05) * unit(q, 0.82, 1.05)}>
            <path d="M-100 120 C-140 -20 -79 -110 10 -99 C104 -101 165 -8 132 118 L87 99 L49 127 L9 101 L-41 128Z" fill={PAPER} />
            <path d="M-9 -84 C-84 -217 -46 -266 -12 -205 L31 -103 M48 -95 C65 -249 131 -246 104 -128 L83 -91" fill={PAPER} />
            <ellipse cx="16" cy="-15" rx="11" ry="25" fill={INK} />
            <ellipse cx="74" cy="-15" rx="11" ry="25" fill={INK} />
            <ellipse cx="53" cy="39" rx="14" ry="18" fill={INK} />
            <ellipse cx="36" cy="-248" rx="96" ry="19" fill="none" stroke={LIME} stroke-width="9" />
        </g>
        <g opacity={unit(q, 0.5, 1.0)}>
            {mono(152.0, 949.0, 19.0, "M5 MAX · 100K-ELEMENT STRESS SCENE · 30 FRAMES · ONE EXPORT PIPELINE · 3-RUN MEDIANS", "#b8b0ad")}
            {mono(152.0, 982.0, 18.0, "GPU: VIDEOTOOLBOX / CPU: X264 / REMOTION + MEDIABUNNY: WEBCODECS (PREFER-HARDWARE)", "#9a9291")}
            {mono(1534.0, 917.0, 26.0, "fframes", RUST)}
        </g>
    </g>)
}

fn transition(t: f32) -> Svgr<'static> {
    let cuts = [(3.55, PAPER), (6.4, RUST)];
    for (cut, color) in cuts {
        if (cut - 0.15..cut + 0.15).contains(&t) {
            let p = unit(t, cut - 0.15, cut + 0.15);
            let x = -2700.0 + p * 5300.0;
            return svgr!(<path d={format!("M{x} -100 l2050 0 l-700 1280 l-2050 0Z")} fill={color} />);
        }
    }
    Svgr::empty()
}

#[derive(Debug)]
struct Fable;

impl Video for Fable {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    const FPS: usize = 60;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(14.5)
    }

    fn audio(&self) -> AudioMap<'_> {
        AudioMap::from([AudioTrack::new(
            "fable.wav",
            AudioTimestamp::Second(0.0)..AudioTimestamp::Eof,
        )])
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let artwork = if t < 3.55 {
            waiting(t)
        } else if t < 6.4 {
            heavy(t)
        } else if t < 8.65 {
            launch(t)
        } else if t < 10.15 {
            impact(t)
        } else {
            aftermath(t)
        };
        svgr!(<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080" viewBox="0 0 1920 1080">
            {artwork}
            {transition(t)}
            <rect width="1920" height="1080" fill={INK} opacity={1.0 - unit(t, 0.0, 0.22)} />
        </svg>)
    }
}

fn main() -> ExitCode {
    let media = IntroMedia::prepare().expect("intro fonts");
    let audio_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/benchmark-fable-audio");
    let folder = MediaDirectory::read_folder(audio_dir).expect("fable audio folder");
    let audio = folder.process_media_source().expect("fable sound design");
    let all = CombinedMediaProvider::from([&media as &dyn MediaProvider, &audio]);
    let pipeline = SkiaPipelineConfig {
        concurrency_policy: SkiaPipelineConcurrencyPolicy::MaxPerformance,
        ..Default::default()
    };
    #[cfg(target_os = "macos")]
    let gpu = fframes_skia_renderer::metal::SkiaMetalCtx::new(1920, 1080).expect("Metal device");
    #[cfg(not(target_os = "macos"))]
    let gpu = fframes_skia_renderer::vulkan::SkiaVulkanCtx::new(1920, 1080).expect("Vulkan device");
    #[cfg(target_os = "macos")]
    let backend = SkiaFFramesRenderer::new_metal(&gpu, pipeline);
    #[cfg(not(target_os = "macos"))]
    let backend = SkiaFFramesRenderer::new_vulkan(&gpu, pipeline);
    cli::new(
        &Fable,
        RenderOptions {
            media: Some(&all),
            video_encoder_options: EncoderOptions {
                preferred_encoder: Some("libx264"),
                codec_params: Some(&[("crf", "16"), ("preset", "medium")]),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .backend(backend.expect("Skia renderer"))
    .default_output("benchmark-fable.mp4")
    .run()
}
