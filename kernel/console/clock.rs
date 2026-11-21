use core::arch::x86_64::{__cpuid, _rdtsc};

use crate::io::{inb, outb};

const SPEAKER: u16 = 0x61;
const PIT_CONTROL: u16 = 0x43;
const PIT_CHANNEL_2: u16 = 0x42;
const SAMPLE_MS: u64 = 12;
const SAMPLE_TICKS: u16 = 14_318;
const POLL_LIMIT: usize = 300_000;

pub struct Clock {
    epoch: u64,
    cycles_per_ms: u64,
}

impl Clock {
    /// The BIOS boot CPU must own PIT channel 2 and the speaker port.
    /// Used for short presentation intervals on one CPU.
    pub unsafe fn calibrate() -> Option<Self> {
        if __cpuid(1).edx & (1 << 4) == 0 {
            return None;
        }
        let previous = inb(SPEAKER);
        if previous == 0xff {
            return None;
        }
        let quiet = previous & !3;
        outb(SPEAKER, quiet);
        outb(PIT_CONTROL, 0xb0); // Channel 2, LSB/MSB, mode 0.
        for byte in SAMPLE_TICKS.to_le_bytes() {
            outb(PIT_CHANNEL_2, byte);
        }
        let start = _rdtsc();
        outb(SPEAKER, quiet | 1);
        let stopped = (0..POLL_LIMIT).any(|_| inb(SPEAKER) & 0x20 != 0);
        let end = _rdtsc();
        outb(SPEAKER, previous);
        let cycles_per_ms = end.saturating_sub(start) / SAMPLE_MS;
        (stopped && (1_000..=10_000_000).contains(&cycles_per_ms)).then_some(Self {
            epoch: end,
            cycles_per_ms,
        })
    }

    pub fn ms(&self) -> u64 {
        // SAFETY: only calibration constructs a clock, after checking CPUID.
        unsafe { _rdtsc() }.saturating_sub(self.epoch) / self.cycles_per_ms
    }
}
