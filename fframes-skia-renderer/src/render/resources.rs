//! Reuse resolved resources without relying on compile-time static hashes.

use std::collections::HashMap;

use fframes::usvgr;

struct Entry<T> {
    value: T,
    bytes: usize,
}

/// Two generations with an estimated resource-byte budget per generation.
/// Hashes locate candidates; callers check source equality before reuse.
/// One-shot animated content
/// can be drawn without retaining an unbounded collection of converted objects.
pub(super) struct ResourceCache<T, const BUDGET: usize> {
    current: HashMap<u64, Entry<T>>,
    previous: HashMap<u64, Entry<T>>,
    bytes: usize,
}

impl<T, const BUDGET: usize> Default for ResourceCache<T, BUDGET> {
    fn default() -> Self {
        Self {
            current: HashMap::new(),
            previous: HashMap::new(),
            bytes: 0,
        }
    }
}

impl<T, const BUDGET: usize> ResourceCache<T, BUDGET> {
    pub(super) fn begin_frame(&mut self) {
        std::mem::swap(&mut self.current, &mut self.previous);
        self.current.clear();
        self.bytes = 0;
    }

    pub(super) fn get(&mut self, key: u64) -> Option<&T> {
        if !self.current.contains_key(&key)
            && self
                .previous
                .get(&key)
                .is_some_and(|entry| self.bytes + entry.bytes <= BUDGET)
        {
            let entry = self.previous.remove(&key)?;
            self.bytes += entry.bytes;
            self.current.insert(key, entry);
        }
        self.current
            .get(&key)
            .or_else(|| self.previous.get(&key))
            .map(|entry| &entry.value)
    }

    pub(super) fn insert_with(&mut self, key: u64, bytes: usize, build: impl FnOnce() -> T) {
        let old = self.current.get(&key).map_or(0, |entry| entry.bytes);
        if self.bytes - old + bytes <= BUDGET {
            self.bytes = self.bytes - old + bytes;
            self.current.insert(
                key,
                Entry {
                    value: build(),
                    bytes,
                },
            );
        }
    }
}

pub(super) struct Geometry {
    pub source: usvgr::tiny_skia_path::Path,
    pub path: skia_safe::Path,
}

pub(super) struct Fill {
    pub source: usvgr::Fill,
    pub anti_alias: bool,
    pub paint: skia_safe::Paint,
}

pub(super) struct Stroke {
    pub source: usvgr::Stroke,
    pub anti_alias: bool,
    pub paint: skia_safe::Paint,
}

pub(super) fn same_paint(a: &usvgr::Paint, b: &usvgr::Paint) -> bool {
    match (a, b) {
        (usvgr::Paint::Color(a), usvgr::Paint::Color(b)) => a == b,
        (usvgr::Paint::LinearGradient(a), usvgr::Paint::LinearGradient(b)) => {
            (a.x1(), a.y1(), a.x2(), a.y2()) == (b.x1(), b.y1(), b.x2(), b.y2())
                && same_gradient(a, b)
        }
        (usvgr::Paint::RadialGradient(a), usvgr::Paint::RadialGradient(b)) => {
            (a.cx(), a.cy(), a.r(), a.fx(), a.fy()) == (b.cx(), b.cy(), b.r(), b.fx(), b.fy())
                && same_gradient(a, b)
        }
        _ => false, // Pattern pictures may contain time-dependent media or shaders.
    }
}

fn same_gradient(a: &usvgr::BaseGradient, b: &usvgr::BaseGradient) -> bool {
    a.transform() == b.transform()
        && a.spread_method() == b.spread_method()
        && a.stops().len() == b.stops().len()
        && a.stops().iter().zip(b.stops()).all(|(a, b)| {
            (a.offset(), a.opacity(), a.color()) == (b.offset(), b.opacity(), b.color())
        })
}

impl Fill {
    pub(super) fn matches(&self, fill: &usvgr::Fill, anti_alias: bool) -> bool {
        self.anti_alias == anti_alias
            && self.source.opacity() == fill.opacity()
            && self.source.rule() == fill.rule()
            && same_paint(self.source.paint(), fill.paint())
    }
}

impl Stroke {
    pub(super) fn matches(&self, stroke: &usvgr::Stroke, anti_alias: bool) -> bool {
        self.anti_alias == anti_alias
            && self.source.opacity() == stroke.opacity()
            && self.source.width() == stroke.width()
            && self.source.miterlimit() == stroke.miterlimit()
            && self.source.dashoffset() == stroke.dashoffset()
            && self.source.linecap() == stroke.linecap()
            && self.source.linejoin() == stroke.linejoin()
            && self.source.dasharray() == stroke.dasharray()
            && same_paint(self.source.paint(), stroke.paint())
    }
}

pub(super) fn paint_bytes(paint: &usvgr::Paint) -> usize {
    let stops = match paint {
        usvgr::Paint::LinearGradient(gradient) => gradient.stops().len(),
        usvgr::Paint::RadialGradient(gradient) => gradient.stops().len(),
        _ => 0,
    };
    256 + stops * std::mem::size_of::<usvgr::Stop>() * 2
}
