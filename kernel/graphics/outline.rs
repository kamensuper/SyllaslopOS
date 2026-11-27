//! Scalable glyph outlines and an integer coverage rasterizer.
//!
//! `tools/bake-outlines.py` extracts quadratic TrueType outlines from DejaVu
//! Sans Mono. Glyphs are rasterized at the exact pixel size requested, so
//! every zoom level gets smooth, anti-aliased edges instead of enlarged
//! bitmap pixels. All arithmetic is integer: positions use 1/256 pixel
//! fixed point and coverage uses 16 vertical samples per pixel row with
//! exact horizontal span coverage.

/// The committed outline asset (see `tools/bake-outlines.py`).
pub static MONO: &[u8] = include_bytes!("assets/mono.outline");

const MAGIC: &[u8; 4] = b"SLOF";
const HEADER: usize = 16;
const POINT: usize = 5;
/// Fixed-point subpixel units per pixel.
const UNIT: i64 = 256;
/// Vertical samples per pixel row.
const SUB: i32 = 16;
const MAX_EDGES: usize = 512;
const MAX_CROSSINGS: usize = 64;
/// Widest glyph cell the rasterizer accepts, in pixels.
pub const MAX_WIDTH: u32 = 64;

#[derive(Clone, Copy)]
pub struct Face<'a> {
    data: &'a [u8],
    units: i64,
    first: u16,
    count: u16,
    advance: i64,
}

/// Target cell for one glyph. `size` is the em size in pixels and
/// `baseline` is measured in pixels from the top of the cell.
#[derive(Clone, Copy)]
pub struct Raster {
    pub w: u32,
    pub h: u32,
    pub size: u32,
    pub baseline: u32,
}

impl<'a> Face<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.get(..4)? != MAGIC || u16_at(data, 4)? != 1 {
            return None;
        }
        let units = u16_at(data, 6)?;
        let first = u16_at(data, 8)?;
        let count = u16_at(data, 10)?;
        let advance = u16_at(data, 12)?;
        let table_end = HEADER + 4 * (usize::from(count) + 1);
        if units == 0 || count == 0 || data.len() < table_end {
            return None;
        }
        Some(Self {
            data,
            units: i64::from(units),
            first,
            count,
            advance: i64::from(advance),
        })
    }

    #[allow(dead_code)]
    pub fn count(&self) -> u16 {
        self.count
    }

    fn record(&self, code: u8) -> Option<&'a [u8]> {
        let index = u16::from(code).checked_sub(self.first)?;
        if index >= self.count {
            return None;
        }
        let at = HEADER + 4 * usize::from(index);
        let start = u32_at(self.data, at)? as usize;
        let end = u32_at(self.data, at + 4)? as usize;
        self.data.get(start..end)
    }
}

#[derive(Clone, Copy, Default)]
struct Edge {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

struct Edges {
    list: [Edge; MAX_EDGES],
    len: usize,
}

impl Edges {
    fn line(&mut self, (x0, y0): (i32, i32), (x1, y1): (i32, i32)) {
        // Horizontal edges never cross a sample row.
        if y0 != y1 && self.len < MAX_EDGES {
            self.list[self.len] = Edge { x0, y0, x1, y1 };
            self.len += 1;
        }
    }

    fn quad(&mut self, p0: (i32, i32), control: (i32, i32), p1: (i32, i32)) {
        let (x0, y0) = (i64::from(p0.0), i64::from(p0.1));
        let (cx, cy) = (i64::from(control.0), i64::from(control.1));
        let (x1, y1) = (i64::from(p1.0), i64::from(p1.1));
        // Flattening n segments leaves a deviation of about |p0 - 2c + p1| / (4 n^2).
        // Keep it below 1/16 pixel.
        let deviation = (x0 - 2 * cx + x1).abs().max((y0 - 2 * cy + y1).abs());
        let steps = (isqrt(deviation / 64) + 1).clamp(1, 16);
        let squared = steps * steps;
        let mut previous = p0;
        for k in 1..=steps {
            let a = (steps - k) * (steps - k);
            let b = 2 * k * (steps - k);
            let c = k * k;
            let next = (
                ((a * x0 + b * cx + c * x1) / squared) as i32,
                ((a * y0 + b * cy + c * y1) / squared) as i32,
            );
            self.line(previous, next);
            previous = next;
        }
    }
}

struct Transform {
    origin: i64,
    baseline: i64,
    size: i64,
    units: i64,
}

impl Transform {
    fn point(&self, x: i16, y: i16) -> (i32, i32) {
        let scale = |v: i16| i64::from(v) * self.size * UNIT / self.units;
        (
            (self.origin + scale(x)) as i32,
            (self.baseline - scale(y)) as i32,
        )
    }
}

/// Rasterize `code` into `out` (row-major, `spec.w * spec.h` coverage
/// bytes). Returns false and leaves `out` cleared when the glyph or cell is
/// unsupported.
pub fn rasterize(face: &Face<'_>, code: u8, spec: Raster, out: &mut [u8]) -> bool {
    let cells = spec.w as usize * spec.h as usize;
    let Some(out) = out.get_mut(..cells) else {
        return false;
    };
    out.fill(0);
    if spec.w == 0 || spec.w > MAX_WIDTH || spec.h == 0 {
        return false;
    }
    let Some(record) = face.record(code) else {
        return false;
    };
    let size = i64::from(spec.size);
    let transform = Transform {
        origin: (i64::from(spec.w) * UNIT - face.advance * size * UNIT / face.units) / 2,
        baseline: i64::from(spec.baseline) * UNIT,
        size,
        units: face.units,
    };
    let mut edges = Edges {
        list: [Edge::default(); MAX_EDGES],
        len: 0,
    };
    if collect(record, &transform, &mut edges).is_none() {
        return false;
    }
    scan(&edges.list[..edges.len], spec.w as usize, spec.h as usize, out);
    true
}

fn collect(record: &[u8], transform: &Transform, edges: &mut Edges) -> Option<()> {
    let contours = u16_at(record, 0)?;
    let mut at = 2;
    for _ in 0..contours {
        let count = usize::from(u16_at(record, at)?);
        at += 2;
        let points = record.get(at..at + count * POINT)?;
        at += count * POINT;
        if count < 2 {
            continue;
        }
        let read = |index: usize| -> Option<((i32, i32), bool)> {
            let base = index * POINT;
            let x = i16_at(points, base)?;
            let y = i16_at(points, base + 2)?;
            Some((transform.point(x, y), *points.get(base + 4)? != 0))
        };
        let (start, on) = read(0)?;
        if !on {
            return None;
        }
        let mut current = start;
        let mut index = 1;
        while index < count {
            let (point, on) = read(index)?;
            if on {
                edges.line(current, point);
                current = point;
                index += 1;
            } else {
                let end = if index + 1 < count {
                    read(index + 1)?.0
                } else {
                    start
                };
                edges.quad(current, point, end);
                current = end;
                index += 2;
            }
        }
        if current != start {
            edges.line(current, start);
        }
    }
    Some(())
}

fn scan(edges: &[Edge], w: usize, h: usize, out: &mut [u8]) {
    let right = (w as i64 * UNIT) as i32;
    let step = (UNIT as i32) / SUB;
    for row in 0..h {
        let mut coverage = [0u32; MAX_WIDTH as usize];
        for sample in 0..SUB {
            let y = row as i32 * UNIT as i32 + sample * step + step / 2;
            let mut crossings = [(0i32, 0i8); MAX_CROSSINGS];
            let mut count = 0;
            for edge in edges {
                let (low, high, direction) = if edge.y0 < edge.y1 {
                    (edge.y0, edge.y1, 1)
                } else {
                    (edge.y1, edge.y0, -1)
                };
                if y < low || y >= high {
                    continue;
                }
                if count == MAX_CROSSINGS {
                    break;
                }
                let dy = i64::from(edge.y1 - edge.y0);
                let x = i64::from(edge.x0)
                    + i64::from(y - edge.y0) * i64::from(edge.x1 - edge.x0) / dy;
                crossings[count] = (x as i32, direction);
                count += 1;
            }
            let crossings = &mut crossings[..count];
            // Stable insertion sort: a sample row crosses only a few edges.
            for i in 1..crossings.len() {
                let mut j = i;
                while j > 0 && crossings[j - 1].0 > crossings[j].0 {
                    crossings.swap(j - 1, j);
                    j -= 1;
                }
            }
            let mut winding = 0i32;
            for i in 0..crossings.len() {
                winding += i32::from(crossings[i].1);
                if winding != 0 && i + 1 < crossings.len() {
                    let start = crossings[i].0.clamp(0, right);
                    let end = crossings[i + 1].0.clamp(0, right);
                    span(&mut coverage[..w], start, end);
                }
            }
        }
        for (pixel, &sum) in out[row * w..(row + 1) * w].iter_mut().zip(&coverage[..w]) {
            let linear = (sum * 255 / (UNIT as u32 * SUB as u32)).min(255);
            // Framebuffer blending happens in sRGB space, which thins light
            // text on dark backgrounds. Lift partial coverage to compensate.
            *pixel = ((linear + linear * (510 - linear) / 255) / 2) as u8;
        }
    }
}

/// Add exact horizontal coverage for `[start, end)` in subpixel units.
fn span(coverage: &mut [u32], start: i32, end: i32) {
    if start >= end {
        return;
    }
    let (start, end) = (start as u32, end as u32);
    let unit = UNIT as u32;
    let (first, last) = ((start / unit) as usize, (end / unit) as usize);
    if first == last {
        coverage[first] += end - start;
        return;
    }
    coverage[first] += unit - start % unit;
    for cell in &mut coverage[first + 1..last] {
        *cell += unit;
    }
    if let Some(cell) = coverage.get_mut(last) {
        *cell += end % unit;
    }
}

fn isqrt(value: i64) -> i64 {
    let mut root = 0;
    while root < 16 && (root + 1) * (root + 1) <= value {
        root += 1;
    }
    root
}

fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

fn i16_at(bytes: &[u8], at: usize) -> Option<i16> {
    Some(i16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: Raster = Raster {
        w: 9,
        h: 18,
        size: 15,
        baseline: 14,
    };

    fn square(units: u16, low: i16, high: i16) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(MAGIC);
        for value in [1, units, 65, 1, units, 0] {
            data.extend_from_slice(&value.to_le_bytes());
        }
        let record = HEADER as u32 + 8;
        data.extend_from_slice(&record.to_le_bytes());
        data.extend_from_slice(&(record + 4 + 4 * POINT as u32).to_le_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&4u16.to_le_bytes());
        for (x, y) in [(low, low), (high, low), (high, high), (low, high)] {
            data.extend_from_slice(&x.to_le_bytes());
            data.extend_from_slice(&y.to_le_bytes());
            data.push(1);
        }
        data
    }

    #[test]
    fn bundled_face_covers_printable_ascii() {
        let face = Face::parse(MONO).unwrap();
        assert_eq!(face.count(), 95);
        let mut cell = [0u8; 9 * 18];
        assert!(rasterize(&face, b' ', BODY, &mut cell));
        assert!(cell.iter().all(|&c| c == 0));
        for code in b'!'..=b'~' {
            assert!(rasterize(&face, code, BODY, &mut cell), "{}", code as char);
            assert!(cell.iter().any(|&c| c > 128), "{} has no ink", code as char);
        }
        assert!(!rasterize(&face, 0x7f, BODY, &mut cell));
    }

    #[test]
    fn interiors_are_solid_and_edges_are_fractional() {
        let spec = Raster {
            w: 16,
            h: 16,
            size: 16,
            baseline: 16,
        };
        let mut cell = [0u8; 256];
        let whole = square(16, 4, 12);
        assert!(rasterize(&Face::parse(&whole).unwrap(), b'A', spec, &mut cell));
        assert_eq!(cell[8 * 16 + 8], 255);
        assert_eq!(cell[4 * 16 + 4], 255);
        assert_eq!(cell[3 * 16 + 8], 0);
        assert_eq!(cell[8 * 16 + 12], 0);

        // Two units per pixel puts each edge halfway through a pixel.
        let half = square(32, 9, 23);
        assert!(rasterize(&Face::parse(&half).unwrap(), b'A', spec, &mut cell));
        assert_eq!(cell[8 * 16 + 8], 255);
        let edge = cell[8 * 16 + 4];
        assert!(edge > 64 && edge < 255, "edge coverage {edge}");
    }

    #[test]
    fn zoom_renders_outlines_instead_of_copying_pixels() {
        let face = Face::parse(MONO).unwrap();
        let mut small = [0u8; 9 * 18];
        let mut large = [0u8; 18 * 36];
        let double = Raster {
            w: 18,
            h: 36,
            size: 30,
            baseline: 28,
        };
        assert!(rasterize(&face, b'o', BODY, &mut small));
        assert!(rasterize(&face, b'o', double, &mut large));
        let copied = (0..18 * 36).all(|i| large[i] == small[(i / 18 / 2) * 9 + (i % 18) / 2]);
        assert!(!copied, "2x text must not be a nearest-neighbour enlargement");
        let soft = large.iter().filter(|&&c| c > 0 && c < 255).count();
        assert!(soft > 20, "expected anti-aliased edges, found {soft}");
    }

    #[test]
    fn malformed_assets_are_rejected() {
        assert!(Face::parse(b"SLOF").is_none());
        let mut data = square(16, 4, 12);
        data.truncate(data.len() - 1);
        let face = Face::parse(&data).unwrap();
        let mut cell = [0u8; 256];
        let spec = Raster {
            w: 16,
            h: 16,
            size: 16,
            baseline: 16,
        };
        assert!(!rasterize(&face, b'A', spec, &mut cell));
    }
}
