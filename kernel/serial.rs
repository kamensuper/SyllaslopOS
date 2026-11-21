use crate::io::{inb, outb};

const COM1: u16 = 0x3f8;
const LINE_CONTROL: u16 = COM1 + 3;
const LINE_STATUS: u16 = COM1 + 5;
const TX_READY: u8 = 1 << 5;
const POLL_LIMIT: usize = 100_000;

pub struct Com1(());

impl Com1 {
    /// The caller must own COM1 and have port I/O privileges.
    pub unsafe fn init() -> Self {
        outb(LINE_CONTROL, 0x03); // 8N1, divisor latch closed
        outb(COM1 + 1, 0); // UART interrupts disabled
        outb(LINE_CONTROL, 0x80); // divisor latch access
        outb(COM1, 1); // 115200 baud at the standard input clock
        outb(COM1 + 1, 0);
        outb(LINE_CONTROL, 0x03);
        outb(COM1 + 2, 0x07); // enable and clear FIFOs
        outb(COM1 + 4, 0x03); // assert DTR/RTS
        Self(())
    }

    /// Take the already initialized port for a final report. The caller
    /// must never return to the code that owns the original handle.
    pub unsafe fn steal() -> Self {
        Self(())
    }

    pub fn read(&mut self) -> Option<u8> {
        // SAFETY: COM1 is owned by this instance and the kernel runs at CPL0.
        let status = unsafe { inb(LINE_STATUS) };
        if status & 1 == 0 || status == 0xff {
            return None;
        }
        let byte = unsafe { inb(COM1) };
        (status & 0x1e == 0).then_some(byte)
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            // SAFETY: constructing Com1 establishes exclusive, privileged I/O.
            let ready = (0..POLL_LIMIT).any(|_| unsafe { inb(LINE_STATUS) } & TX_READY != 0);
            if !ready {
                return;
            }
            unsafe { outb(COM1, byte) };
        }
    }
}

impl core::fmt::Write for Com1 {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        self.write(text.as_bytes());
        Ok(())
    }
}
