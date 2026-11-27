mod framebuffer;
mod outline;

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub use framebuffer::{crash_screen, Framebuffer};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub const fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub fn intersect(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self
            .x
            .saturating_add(self.w)
            .min(other.x.saturating_add(other.w));
        let bottom = self
            .y
            .saturating_add(self.h)
            .min(other.y.saturating_add(other.h));
        Self::new(x, y, right.saturating_sub(x), bottom.saturating_sub(y))
    }
}

#[derive(Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

pub fn mix(back: Color, fore: Color, level: u8) -> Color {
    let blend = |b: u8, f: u8| {
        let a = level as u32;
        ((b as u32 * (255 - a) + f as u32 * a + 127) / 255) as u8
    };
    Color(
        blend(back.0, fore.0),
        blend(back.1, fore.1),
        blend(back.2, fore.2),
    )
}

#[derive(Clone, Copy)]
pub struct Ink {
    pub fore: Color,
    pub back: Color,
}

/// A monospaced text style. `w` and `h` are the cell size at scale 1,
/// `size` is the em size in pixels and `baseline` is measured from the top
/// of the cell. Glyphs are rasterized from outlines at `size * scale`.
pub struct Font {
    pub w: u32,
    pub h: u32,
    size: u32,
    baseline: u32,
    slot: usize,
}

pub const BODY: Font = Font {
    w: 9,
    h: 18,
    size: 15,
    baseline: 14,
    slot: 0,
};
pub const SMALL: Font = Font {
    w: 8,
    h: 16,
    size: 13,
    baseline: 12,
    slot: 1,
};

/// Largest supported text scale. Each scale gets its own rasterization.
pub const MAX_SCALE: u32 = 2;

const FONTS: [Font; 2] = [BODY, SMALL];
const GLYPHS: usize = 95;
const SLOTS: usize = FONTS.len() * MAX_SCALE as usize;

impl Font {
    const fn cell(&self, scale: u32) -> (u32, u32) {
        (self.w * scale, self.h * scale)
    }

    const fn glyph_bytes(&self, scale: u32) -> usize {
        let (w, h) = self.cell(scale);
        (w * h) as usize
    }
}

const fn slot_offsets() -> [usize; SLOTS + 1] {
    let mut offsets = [0; SLOTS + 1];
    let mut slot = 0;
    while slot < SLOTS {
        let font = &FONTS[slot / MAX_SCALE as usize];
        let scale = (slot % MAX_SCALE as usize) as u32 + 1;
        offsets[slot + 1] = offsets[slot] + GLYPHS * font.glyph_bytes(scale);
        slot += 1;
    }
    offsets
}

const fn largest_glyph() -> usize {
    let mut largest = 0;
    let mut index = 0;
    while index < FONTS.len() {
        let bytes = FONTS[index].glyph_bytes(MAX_SCALE);
        if bytes > largest {
            largest = bytes;
        }
        index += 1;
    }
    largest
}

const OFFSETS: [usize; SLOTS + 1] = slot_offsets();
const ATLAS_BYTES: usize = OFFSETS[SLOTS];
pub const MAX_GLYPH_BYTES: usize = largest_glyph();

/// Rasterize one printable ASCII glyph for `font` at `scale` into `out`.
pub fn render(font: &Font, scale: u32, code: u8, out: &mut [u8]) {
    let (w, h) = font.cell(scale);
    let spec = outline::Raster {
        w,
        h,
        size: font.size * scale,
        baseline: font.baseline * scale,
    };
    match outline::Face::parse(outline::MONO) {
        Some(face) => {
            outline::rasterize(&face, code, spec, out);
        }
        None => out.fill(0),
    }
}

/// Glyph coverage cache: each glyph is rasterized once per font and scale,
/// the first time it is drawn.
pub struct Atlas {
    ready: [[bool; GLYPHS]; SLOTS],
    pixels: [u8; ATLAS_BYTES],
}

struct AtlasCell(UnsafeCell<Atlas>);

// SAFETY: `Atlas::take` hands out the only reference, at most once.
unsafe impl Sync for AtlasCell {}

static ATLAS: AtlasCell = AtlasCell(UnsafeCell::new(Atlas {
    ready: [[false; GLYPHS]; SLOTS],
    pixels: [0; ATLAS_BYTES],
}));
static ATLAS_TAKEN: AtomicBool = AtomicBool::new(false);

impl Atlas {
    /// Claim the kernel's glyph cache. Only the first call succeeds.
    pub fn take() -> Option<&'static mut Self> {
        if ATLAS_TAKEN.swap(true, Ordering::AcqRel) {
            return None;
        }
        // SAFETY: the flag above guarantees this is the only reference.
        Some(unsafe { &mut *ATLAS.0.get() })
    }

    /// `code` must be printable ASCII and `scale` within `1..=MAX_SCALE`.
    pub fn glyph(&mut self, font: &Font, scale: u32, code: u8) -> &[u8] {
        let slot = font.slot * MAX_SCALE as usize + (scale - 1) as usize;
        let index = usize::from(code - b' ');
        let bytes = font.glyph_bytes(scale);
        let start = OFFSETS[slot] + index * bytes;
        if !self.ready[slot][index] {
            render(font, scale, code, &mut self.pixels[start..start + bytes]);
            self.ready[slot][index] = true;
        }
        &self.pixels[start..start + bytes]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_slots_hold_every_glyph_at_every_scale() {
        assert_eq!(OFFSETS[0], 0);
        assert_eq!(OFFSETS[1], GLYPHS * 9 * 18);
        assert_eq!(OFFSETS[2] - OFFSETS[1], GLYPHS * 18 * 36);
        assert_eq!(MAX_GLYPH_BYTES, 18 * 36);
        let mut cell = [0u8; MAX_GLYPH_BYTES];
        render(&SMALL, 2, b'g', &mut cell[..SMALL.glyph_bytes(2)]);
        assert!(cell.iter().any(|&c| c > 200));
    }
}
