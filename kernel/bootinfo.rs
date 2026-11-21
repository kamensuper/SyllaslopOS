//! Bounded views over the GRUB Multiboot2 handoff.
const BOOT_WINDOW: u64 = 32 * 1024 * 1024;
const MAX_INFO_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Channel {
    pub shift: u8,
    pub bits: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Video {
    pub phys: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub channels: [Channel; 3],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Memory {
    pub regions: usize,
    pub usable_regions: usize,
    pub usable_bytes: u64,
}

pub struct BootInfo<'a> {
    phys: u64,
    data: &'a [u8],
}

impl BootInfo<'static> {
    /// The caller must have validated the GRUB handoff magic and entered
    /// with the bootstrap identity map active. GRUB owns the backing bytes,
    /// which this kernel keeps immutable and never reclaims.
    pub unsafe fn read(phys: u64) -> Option<Self> {
        if phys < 0x1000 || phys & 7 != 0 || phys > BOOT_WINDOW - 8 {
            return None;
        }
        let size = (phys as *const u32).read() as usize;
        if !(16..=MAX_INFO_BYTES).contains(&size) || phys + size as u64 > BOOT_WINDOW {
            return None;
        }
        Self::parse(phys, core::slice::from_raw_parts(phys as *const u8, size))
    }
}

impl<'a> BootInfo<'a> {
    pub fn parse(phys: u64, data: &'a [u8]) -> Option<Self> {
        if data.len() < 16 || data.len() % 8 != 0 || word(data, 0)? as usize != data.len() {
            return None;
        }
        let mut remaining = &data[8..];
        loop {
            let kind = word(remaining, 0)?;
            let size = word(remaining, 4)? as usize;
            let aligned = size.checked_add(7)? & !7;
            if size < 8 || aligned > remaining.len() {
                return None;
            }
            if kind == 0 {
                return (size == 8 && remaining.len() == 8).then_some(Self { phys, data });
            }
            remaining = &remaining[aligned..];
        }
    }

    pub fn address(&self) -> u64 {
        self.phys
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    fn tag(&self, kind: u32) -> Option<&'a [u8]> {
        let mut remaining = &self.data[8..];
        while !remaining.is_empty() {
            let code = word(remaining, 0)?;
            let size = word(remaining, 4)? as usize;
            if code == kind {
                return Some(&remaining[8..size]);
            }
            remaining = &remaining[(size + 7) & !7..];
        }
        None
    }

    pub fn loader(&self) -> Option<&'a str> {
        self.text(2)
    }

    fn text(&self, kind: u32) -> Option<&'a str> {
        let raw = self.tag(kind)?;
        let end = raw.iter().position(|&ch| ch == 0)?;
        let text = core::str::from_utf8(&raw[..end]).ok()?;
        text.bytes()
            .all(|b| (b' '..=b'~').contains(&b))
            .then_some(text)
    }

    pub fn serial_only(&self) -> bool {
        self.text(1)
            .is_some_and(|cmdline| cmdline.split_ascii_whitespace().any(|s| s == "serial"))
    }

    pub fn video(&self) -> Option<Video> {
        let raw = self.tag(8)?;
        if raw.get(21) != Some(&1) {
            return None;
        }
        Some(Video {
            phys: quad(raw, 0)?,
            pitch: word(raw, 8)?,
            width: word(raw, 12)?,
            height: word(raw, 16)?,
            bpp: *raw.get(20)?,
            channels: [
                Channel {
                    shift: *raw.get(24)?,
                    bits: *raw.get(25)?,
                },
                Channel {
                    shift: *raw.get(26)?,
                    bits: *raw.get(27)?,
                },
                Channel {
                    shift: *raw.get(28)?,
                    bits: *raw.get(29)?,
                },
            ],
        })
    }

    fn memory_rows(&self) -> Option<impl Iterator<Item = &'a [u8]>> {
        let raw = self.tag(6)?;
        let stride = word(raw, 0)? as usize;
        let version = word(raw, 4)?;
        let entries = raw.get(8..)?;
        if version != 0 || stride < 24 || stride % 8 != 0 || entries.len() % stride != 0 {
            return None;
        }
        Some(entries.chunks_exact(stride))
    }

    pub fn memory(&self) -> Option<Memory> {
        let mut memory = Memory::default();
        for row in self.memory_rows()? {
            memory.regions += 1;
            if word(row, 16)? == 1 {
                let size = quad(row, 8)?;
                quad(row, 0)?.checked_add(size)?;
                memory.usable_bytes = memory.usable_bytes.checked_add(size)?;
                memory.usable_regions += 1;
            }
        }
        Some(memory)
    }

    pub fn clear_of_ram(&self, start: u64, end: u64) -> bool {
        let Some(mut rows) = self.memory_rows() else {
            return false;
        };
        rows.all(|row| {
            let Some(base) = quad(row, 0) else {
                return false;
            };
            let Some(bound) = quad(row, 8).and_then(|size| base.checked_add(size)) else {
                return false;
            };
            // EFI/BIOS gaps and reserved regions can contain a video aperture.
            word(row, 16) == Some(2) || start >= bound || end <= base
        })
    }
}

fn word(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn quad(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_the_tag_walk_and_end_record() {
        let mut bytes = [0u8; 32];
        bytes[0..4].copy_from_slice(&32u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&15u32.to_le_bytes());
        bytes[16..23].copy_from_slice(b"serial\0");
        bytes[28..32].copy_from_slice(&8u32.to_le_bytes());
        let boot = BootInfo::parse(0x1000, &bytes).unwrap();
        assert!(boot.serial_only());
        assert_eq!(boot.address(), 0x1000);
        bytes[12] = 7;
        assert!(BootInfo::parse(0x1000, &bytes).is_none());
        bytes[12] = 15;
        bytes[28] = 16;
        assert!(BootInfo::parse(0x1000, &bytes).is_none());
    }

    #[test]
    fn reports_the_real_usable_region_and_detects_overlap() {
        let mut bytes = [0u8; 56];
        for (offset, value) in [(0, 56u32), (8, 6), (12, 40), (16, 24), (40, 1), (52, 8)] {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[24..32].copy_from_slice(&0x100000u64.to_le_bytes());
        bytes[32..40].copy_from_slice(&0x800000u64.to_le_bytes());
        let boot = BootInfo::parse(0x1000, &bytes).unwrap();
        assert_eq!(boot.memory().unwrap().usable_bytes, 0x800000);
        assert!(!boot.clear_of_ram(0x200000, 0x400000));
        assert!(boot.clear_of_ram(0xfd000000, 0xfd200000));
    }
}
