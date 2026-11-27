use core::arch::{asm, x86_64::__cpuid};
use core::cell::UnsafeCell;
use core::ptr::{addr_of_mut, write_volatile};

use crate::bootinfo::{BootInfo, Channel, Video};

use super::{mix, render, Atlas, Color, Font, Ink, Rect, BODY, MAX_GLYPH_BYTES, MAX_SCALE};

const HUGE_PAGE: u64 = 2 * 1024 * 1024;
const WINDOW: u64 = 0xffff_ff00_0000_0000;
const MAX_MAPPING: u64 = 64 * 1024 * 1024;
const MIN_MMIO: u64 = 0xc000_0000;
const MAX_MMIO: u64 = 0x1_0000_0000;

const PAT_MSR: u32 = 0x277;
const PAT_WRITE_COMBINING: u64 = 0x01;
/// PAT entry reprogrammed for write-combining. The boot tables select only
/// entry 0 (PAT, PCD and PWT clear), so no live mapping uses entry 7.
const WC_ENTRY: u64 = 7;

extern "C" {
    static mut mmio_page_directory: [u64; 512];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Caching {
    WriteCombining,
    Uncached,
}

impl Caching {
    pub fn label(self) -> &'static str {
        match self {
            Self::WriteCombining => "write-combining",
            Self::Uncached => "uncached",
        }
    }
}

pub struct Framebuffer {
    base: *mut u8,
    video: Video,
    writes: u64,
    caching: Caching,
    glyphs: Option<&'static mut Atlas>,
}

/// Where the fatal crash path may draw. Written once by `attach`.
struct CrashTarget(UnsafeCell<Option<(usize, Video)>>);

// SAFETY: written once on the boot CPU before any crash can be reported.
unsafe impl Sync for CrashTarget {}

static CRASH_TARGET: CrashTarget = CrashTarget(UnsafeCell::new(None));

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
        let (cache, caching) = write_combining_flags();
        let entries = addr_of_mut!(mmio_page_directory).cast::<u64>();
        // Write exclusively to virgin entries. The framebuffer aperture does
        // not alias the initial RAM mapping, so a write-combining leaf here
        // cannot conflict with a cached alias of the same memory.
        for slot in 0..((end - start) / HUGE_PAGE) {
            let physical = start + slot * HUGE_PAGE;
            write_volatile(entries.add(slot as usize), physical | nx | cache | 0x83);
            let virtual_addr = WINDOW + slot * HUGE_PAGE;
            asm!("invlpg [{}]", in(reg) virtual_addr, options(nostack, preserves_flags));
        }

        let base = (WINDOW + video.phys - start) as *mut u8;
        *CRASH_TARGET.0.get() = Some((base as usize, video));
        Some(Self {
            base,
            video,
            writes: 0,
            caching,
            glyphs: Atlas::take(),
        })
    }

    pub fn video(&self) -> Video {
        self.video
    }

    pub fn caching(&self) -> Caching {
        self.caching
    }

    pub fn bytes_written(&self) -> u64 {
        self.writes * u64::from(self.video.bpp / 8)
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.video.width, self.video.height)
    }

    /// Drain write-combining buffers so a finished frame reaches the device.
    pub fn flush(&self) {
        // SAFETY: SFENCE only orders and drains stores. Every x86-64 CPU has it.
        unsafe { asm!("sfence", options(nostack, preserves_flags)) };
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

    /// Draw monospaced text. Each glyph is rasterized from its outline at
    /// the requested scale, so enlarged text keeps smooth edges.
    pub fn text(&mut self, rect: Rect, text: &[u8], font: &Font, scale: u32, ink: Ink) {
        let scale = scale.clamp(1, MAX_SCALE);
        let (w, h) = (font.w * scale, font.h * scale);
        let bytes = (w * h) as usize;
        let mut glyph = [0u8; MAX_GLYPH_BYTES];
        for (index, &byte) in text.iter().take((rect.w / w) as usize).enumerate() {
            if byte == b' ' {
                continue;
            }
            let code = if (32..=126).contains(&byte) {
                byte
            } else {
                b'?'
            };
            match self.glyphs.as_deref_mut() {
                Some(atlas) => glyph[..bytes].copy_from_slice(atlas.glyph(font, scale, code)),
                None => render(font, scale, code, &mut glyph[..bytes]),
            }
            self.mask(
                (rect.x + index as u32 * w, rect.y),
                (w, h),
                &glyph[..bytes],
                ink,
                rect,
            );
        }
    }

    fn mask(&mut self, at: (u32, u32), size: (u32, u32), mask: &[u8], ink: Ink, clip: Rect) {
        let rect = Rect::new(at.0, at.1, size.0, size.1)
            .intersect(clip)
            .intersect(self.bounds());
        for y in rect.y..rect.y + rect.h {
            let row = ((y - at.1) * size.0) as usize;
            for x in rect.x..rect.x + rect.w {
                let alpha = mask[row + (x - at.0) as usize];
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

/// Paint a crash report over the whole display.
///
/// The caller must be the fatal crash path: it never returns control to
/// the code that owns the console framebuffer, so drawing through a second
/// handle cannot be observed by that owner.
pub unsafe fn crash_screen<'a>(lines: impl Iterator<Item = &'a [u8]>) {
    let Some((base, video)) = *CRASH_TARGET.0.get() else {
        return;
    };
    let mut frame = Framebuffer {
        base: base as *mut u8,
        video,
        writes: 0,
        caching: Caching::Uncached,
        glyphs: None,
    };
    let back = Color(38, 12, 16);
    frame.fill(frame.bounds(), back);
    let width = video.width.saturating_sub(64);
    for (row, line) in lines.enumerate() {
        let y = 32 + row as u32 * 22;
        if y + BODY.h > video.height {
            break;
        }
        let fore = if row == 0 {
            Color(255, 186, 186)
        } else {
            Color(236, 228, 230)
        };
        frame.text(Rect::new(32, y, width, BODY.h), line, &BODY, 1, Ink { fore, back });
    }
    frame.flush();
}

/// Select write-combining through PAT entry 7, or fall back to UC when the
/// CPU has no PAT. Requires CPL0 on the boot CPU with interrupts masked.
unsafe fn write_combining_flags() -> (u64, Caching) {
    if __cpuid(1).edx & (1 << 16) == 0 {
        return ((1 << 3) | (1 << 4), Caching::Uncached);
    }
    let low: u32;
    let high: u32;
    asm!("rdmsr", in("ecx") PAT_MSR, out("eax") low, out("edx") high, options(nomem, nostack));
    let pat = u64::from(low) | (u64::from(high) << 32);
    let shift = WC_ENTRY * 8;
    let wanted = (pat & !(0xff << shift)) | (PAT_WRITE_COMBINING << shift);
    if wanted != pat {
        // No live translation selects this entry, so changing it cannot alter
        // the type of mapped memory. Flush caches and TLBs around the update.
        asm!("wbinvd", options(nostack, preserves_flags));
        asm!(
            "wrmsr",
            in("ecx") PAT_MSR,
            in("eax") wanted as u32,
            in("edx") (wanted >> 32) as u32,
            options(nostack, preserves_flags)
        );
        asm!(
            "mov {tmp}, cr3",
            "mov cr3, {tmp}",
            tmp = out(reg) _,
            options(nostack, preserves_flags)
        );
    }
    (pat_flags(WC_ENTRY), Caching::WriteCombining)
}

/// PAT, PCD and PWT form the PAT index. For 2 MiB leaves PAT is bit 12.
fn pat_flags(index: u64) -> u64 {
    ((index & 3) << 3) | ((index & 4) << 10)
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
    fn write_combining_selects_pat_entry_seven() {
        assert_eq!(pat_flags(7), (1 << 3) | (1 << 4) | (1 << 12));
        assert_eq!(pat_flags(0), 0);
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
            caching: Caching::Uncached,
            glyphs: None,
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
    fn text_is_clipped_to_its_rectangle() {
        let mut storage = [0u32; 64 * 40];
        let mut fb = Framebuffer {
            base: storage.as_mut_ptr().cast(),
            video: Video {
                width: 64,
                height: 40,
                pitch: 64 * 4,
                ..video()
            },
            writes: 0,
            caching: Caching::Uncached,
            glyphs: None,
        };
        let ink = Ink {
            fore: Color(255, 255, 255),
            back: Color(0, 0, 0),
        };
        fb.text(Rect::new(0, 0, 18, 18), b"MMMM", &BODY, 1, ink);
        let inked = |x: usize, y: usize| storage[y * 64 + x] != 0;
        assert!((0..18).any(|y| (0..18).any(|x| inked(x, y))));
        assert!((0..40).all(|y| (18..64).all(|x| !inked(x, y))));
    }
}
