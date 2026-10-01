use fframes::usvgr;
use skia_safe::{BlendMode, Canvas, Color, Matrix, Paint, Point, Rect, vertices};

pub(super) const MIN_RECTS: usize = 32;

// Vertices do not antialias edges. Only batch opaque rectangles whose device
// edges fall on whole pixels, where a filled path has the same coverage.
pub(super) fn draw_batch(nodes: &[usvgr::Node], canvas: &Canvas) -> usize {
    if nodes.len() < MIN_RECTS {
        return 0;
    }
    let matrix = canvas.local_to_device_as_3x3();
    if !matrix.is_scale_translate() {
        return 0;
    }
    let rectangles: Vec<_> = nodes
        .iter()
        .take(16_384)
        .map_while(|node| rectangle(node, &matrix))
        .collect();
    if rectangles.len() < MIN_RECTS {
        return 0;
    }
    let mut builder = vertices::Builder::new(
        vertices::VertexMode::Triangles,
        rectangles.len() * 6,
        0,
        vertices::BuilderFlags::HAS_COLORS,
    );
    for (points, (rect, _)) in builder
        .positions()
        .as_chunks_mut::<6>()
        .0
        .iter_mut()
        .zip(&rectangles)
    {
        let tl = Point::new(rect.left, rect.top);
        let tr = Point::new(rect.right, rect.top);
        let bl = Point::new(rect.left, rect.bottom);
        let br = Point::new(rect.right, rect.bottom);
        points.copy_from_slice(&[tl, tr, bl, tr, br, bl]);
    }
    for (colors, (_, color)) in builder
        .colors()
        .unwrap()
        .as_chunks_mut::<6>()
        .0
        .iter_mut()
        .zip(&rectangles)
    {
        colors.fill(*color);
    }
    canvas.draw_vertices(&builder.detach(), BlendMode::Dst, &Paint::default());
    rectangles.len()
}

fn rectangle(node: &usvgr::Node, matrix: &Matrix) -> Option<(Rect, Color)> {
    use usvgr::tiny_skia_path::PathVerb::{Close, Line, Move};
    let usvgr::Node::Path(path) = node else {
        return None;
    };
    if path.visibility() != usvgr::Visibility::Visible || path.stroke().is_some() {
        return None;
    }
    let fill = path.fill()?;
    let usvgr::Paint::Color(color) = fill.paint() else {
        return None;
    };
    if fill.opacity().get() != 1.0 || path.data().verbs() != [Move, Line, Line, Line, Close] {
        return None;
    }
    let bounds = path.data().bounds();
    let rect = Rect::new(bounds.left(), bounds.top(), bounds.right(), bounds.bottom());
    let points = path.data().points();
    if points.len() != 4
        || points.iter().enumerate().any(|(i, p)| {
            let expected = [
                (rect.left, rect.top),
                (rect.right, rect.top),
                (rect.right, rect.bottom),
                (rect.left, rect.bottom),
            ][i];
            (p.x, p.y) != expected
        })
    {
        return None;
    }
    let device = matrix.map_rect(rect).0;
    if [device.left, device.top, device.right, device.bottom]
        .iter()
        .any(|v| !v.is_finite() || v.fract() != 0.0)
    {
        return None;
    }
    Some((rect, Color::from_rgb(color.red, color.green, color.blue)))
}
