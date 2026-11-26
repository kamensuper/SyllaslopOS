mod framebuffer;

pub use framebuffer::Framebuffer;

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

pub struct Font {
    pub w: u32,
    pub h: u32,
    pub pixels: &'static [u8],
}

pub const BODY: Font = Font {
    w: 9,
    h: 18,
    pixels: include_bytes!("assets/body.mask"),
};
pub const SMALL: Font = Font {
    w: 8,
    h: 16,
    pixels: include_bytes!("assets/small.mask"),
};

#[derive(Clone, Copy)]
pub struct Ink {
    pub fore: Color,
    pub back: Color,
}
