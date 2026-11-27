use core::arch::{asm, x86_64::__cpuid};
use core::ptr::{addr_of_mut, write_volatile};

use crate::bootinfo::{BootInfo, Channel, Video};

use super::{mix, Color, Font, Ink, Rect};

const HUGE_PAGE: u64 = 2 * 1024 * 1024;
const WINDOW: u64 = 0xffff_ff00_0000_0000;
const MAX_MAPPING: u64 = 64 * 1024 * 1024;
const MIN_MMIO: u64 = 0xc000_0000;
const MAX_MMIO: u64 = 0x1_0000_0000;

extern "C" {
    static mut mmio_page_directory: [u64; 512];
}

pub struct Framebuffer {
    base: *mut u8,
    video: Video,
    writes: u64,
}

impl Framebuffer {
    /// Call once, on the boot CPU with interrupts masked and the entry
    /// stub's page tables active. The GRUB framebuffer must still be owned.
    pub unsafe fn attach(boot: &BootInfo<'_>) -> Option<Self> {
        let video = boot.video()?;
        let (start, end) = aperture(video)?;
        if !boot.clear_of_ram(start, end) {
            return None;
        }
        let nx = if __cpuid(0x8000_0001).edx & (1 << 20) != 0 {
            1 << 63
        } else {
            0
        };
        let cache = uncached_flags()?;
        let entries = addr_of_mut!(mmio_page_directory).cast::<u64>();
        // Write exclusively to virgin entries. The framebuffer aperture does
        // not alias the initial RAM mapping. Select the active UC PAT entry.
        for slot in 0..((end - start) / HUGE_PAGE) {
            let physical = start + slot * HUGE_PAGE;
            write_volatile(entries.add(slot as usize), physical | nx | cache | 0x83);
            let virtual_addr = WINDOW + slot * HUGE_PAGE;
            asm!("invlpg [{}]", in(reg) virtual_addr, options(nostack, preserves_flags));
        }

        let base = (WINDOW + video.phys - start) as *mut u8;
        Some(Self {
            base,
            video,
            writes: 0,
        })
    }

    pub fn video(&self) -> Video {
        self.video
    }

    pub fn bytes_written(&self) -> u64 {
        self.writes * u64::from(self.video.bpp / 8)
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.video.width, self.video.height)
    }

    pub fn fill(&mut self, rect: Rect, color: Color) {
        let rect = rect.intersect(self.bounds());
        let value = packed(color, self.video.channels);
        for y in rect.y..rect.y + rect.h {
            for x in rect.x..rect.x + rect.w {
                self.store(x, y, value);
            }
        }
    }

    /// A zero scale draws nothing.
    pub fn text(&mut self, rect: Rect, text: &[u8], font: &Font, scale: u32, ink: Ink) {
        if scale == 0 {
            return;
        }
        let step = font.w * scale;
        let area = (font.w * font.h) as usize;
        for (index, &byte) in text.iter().take((rect.w / step) as usize).enumerate() {
            if byte == b' ' {
                continue;
            }
            let glyph = if (32..=126).contains(&byte) {
                byte
            } else {
                b'?'
            };
            let offset = (glyph - 32) as usize * area;
            self.mask(
                (rect.x + index as u32 * step, rect.y),
                (font.w, font.h),
                &font.pixels[offset..offset + area],
                scale,
                ink,
                rect,
            );
        }
    }

    pub fn mask(
        &mut self,
        at: (u32, u32),
        size: (u32, u32),
        mask: &[u8],
        scale: u32,
        ink: Ink,
        clip: Rect,
    ) {
        if scale == 0 {
            return;
        }
        let rect = Rect::new(at.0, at.1, size.0 * scale, size.1 * scale)
            .intersect(clip)
            .intersect(self.bounds());
        for y in rect.y..rect.y + rect.h {
            let row = ((y - at.1) / scale * size.0) as usize;
            for x in rect.x..rect.x + rect.w {
                let alpha = mask[row + ((x - at.0) / scale) as usize];
                if alpha != 0 {
                    self.store(
                        x,
                        y,
                        packed(mix(ink.back, ink.fore, alpha), self.video.channels),
                    );
                }
            }
        }
    }

    fn store(&mut self, x: u32, y: u32, value: u32) {
        let bytes = usize::from(self.video.bpp / 8);
        let offset = y as usize * self.video.pitch as usize + x as usize * bytes;
        // SAFETY: all callers clip to validated dimensions. attach checked the
        // full padded extent and, for a u32 store, pixel and row alignment.
        unsafe {
            let pixel = self.base.add(offset);
            if bytes == 4 {
                write_volatile(pixel.cast::<u32>(), value);
            } else {
                for (index, byte) in value.to_le_bytes()[..3].iter().enumerate() {
                    write_volatile(pixel.add(index), *byte);
                }
            }
        }
        self.writes += 1;
    }
}

// CPL0 and CPUID capability checks are required before the PAT MSR read.
unsafe fn uncached_flags() -> Option<u64> {
    if __cpuid(1).edx & (1 << 16) == 0 {
        return Some((1 << 3) | (1 << 4));
    }
    let low: u32;
    let high: u32;
    asm!("rdmsr", in("ecx") 0x277u32, out("eax") low, out("edx") high, options(nomem, nostack));
    let pat = u64::from(low) | (u64::from(high) << 32);
    let index = (0..8u64).find(|&index| (pat >> (index * 8)) & 0xff == 0)?;
    // PAT, PCD and PWT form the index. For 2 MiB leaves PAT is bit 12.
    Some(((index & 3) << 3) | ((index & 4) << 10))
}

fn packed(color: Color, channels: [Channel; 3]) -> u32 {
    let [r, g, b] = channels;
    let sample =
        |value: u8, channel: Channel| (u32::from(value) >> (8 - channel.bits)) << channel.shift;
    sample(color.0, r) | sample(color.1, g) | sample(color.2, b)
}

fn aperture(video: Video) -> Option<(u64, u64)> {
    if !(640..=4096).contains(&video.width)
        || !(480..=2304).contains(&video.height)
        || !matches!(video.bpp, 24 | 32)
        || video.pitch < video.width * u32::from(video.bpp / 8)
        || (video.bpp == 32 && (video.pitch % 4 != 0 || video.phys % 4 != 0))
        || !(MIN_MMIO..MAX_MMIO).contains(&video.phys)
    {
        return None;
    }
    let mut used = 0u32;
    for channel in video.channels {
        if !(1..=8).contains(&channel.bits)
            || u16::from(channel.bits) + u16::from(channel.shift) > u16::from(video.bpp)
        {
            return None;
        }
        let mask = ((1u32 << channel.bits) - 1) << channel.shift;
        if used & mask != 0 {
            return None;
        }
        used |= mask;
    }
    let upper = video
        .phys
        .checked_add(u64::from(video.pitch) * u64::from(video.height))?;
    let start = video.phys & !(HUGE_PAGE - 1);
    let end = upper.checked_add(HUGE_PAGE - 1)? & !(HUGE_PAGE - 1);
    (end <= MAX_MMIO && end - start <= MAX_MAPPING).then_some((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn video() -> Video {
        Video {
            phys: 0xfd00_0000,
            pitch: 640 * 4 + 16,
            width: 640,
            height: 480,
            bpp: 32,
            channels: [
                Channel { shift: 16, bits: 8 },
                Channel { shift: 8, bits: 8 },
                Channel { shift: 0, bits: 8 },
            ],
        }
    }

    #[test]
    fn validates_pitch_and_channel_overlaps() {
        let mut fb = video();
        assert_eq!(aperture(fb), Some((0xfd00_0000, 0xfd20_0000)));
        fb.channels[0].shift = 8;
        assert!(aperture(fb).is_none());
        fb = video();
        fb.pitch = 100;
        assert!(aperture(fb).is_none());
        fb = video();
        fb.phys = 0xffff_fffc;
        assert!(aperture(fb).is_none());
    }

    #[test]
    fn clipped_spans_leave_padding_alone() {
        let mut storage = [0x5a5a_5a5au32; 8];
        let mut fb = Framebuffer {
            base: storage.as_mut_ptr().cast(),
            video: Video {
                width: 3,
                height: 2,
                pitch: 16,
                ..video()
            },
            writes: 0,
        };
        fb.fill(Rect::new(2, 0, u32::MAX, 4), Color(1, 2, 3));
        assert_eq!(
            storage,
            [
                0x5a5a_5a5a,
                0x5a5a_5a5a,
                0x010203,
                0x5a5a_5a5a,
                0x5a5a_5a5a,
                0x5a5a_5a5a,
                0x010203,
                0x5a5a_5a5a
            ]
        );
        assert_eq!(fb.bytes_written(), 8);
    }

    #[test]
    fn zero_scale_draws_nothing() {
        let mut storage = [0x5a5a_5a5au32; 8];
        let mut fb = Framebuffer {
            base: storage.as_mut_ptr().cast(),
            video: Video {
                width: 4,
                height: 2,
                pitch: 16,
                ..video()
            },
            writes: 0,
        };
        let ink = Ink {
            fore: Color(1, 2, 3),
            back: Color(4, 5, 6),
        };
        fb.text(fb.bounds(), b"A", &crate::graphics::SMALL, 0, ink);
        fb.mask((0, 0), (2, 2), &[255; 4], 0, ink, fb.bounds());
        assert_eq!(storage, [0x5a5a_5a5a; 8]);
        assert_eq!(fb.bytes_written(), 0);
    }
}
