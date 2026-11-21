use core::arch::asm;

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

unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}
