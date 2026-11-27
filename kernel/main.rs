#![no_std]
#![no_main]

mod bootinfo;
mod console;
mod crash;
mod graphics;
mod input;
mod interrupts;
mod io;
mod paging;
mod serial;

use core::arch::asm;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::addr_of;
use serial::Com1;

const MULTIBOOT2_MAGIC: u32 = 0x36d7_6289;

extern "C" {
    static __stack_bottom: u8;
    static __stack_top: u8;
}

#[no_mangle]
pub extern "C" fn kernel_main(magic: u32, info_phys: u64) -> ! {
    // SAFETY: entry runs at CPL0 with interrupts masked. This boot path
    // targets the PC machine's legacy COM1 UART.
    let mut serial = unsafe { Com1::init() };
    if magic != MULTIBOOT2_MAGIC || info_phys == 0 || info_phys & 7 != 0 {
        serial.write(b"SYLLASLOPOS: invalid multiboot2 handoff\r\n");
        halt();
    }

    let stack: usize;
    // SAFETY: this instruction only observes the current stack pointer.
    unsafe { asm!("mov {}, rsp", out(reg) stack, options(nomem, nostack, preserves_flags)) };
    let stack_range = addr_of!(__stack_bottom) as usize..addr_of!(__stack_top) as usize;
    if !stack_range.contains(&stack) {
        serial.write(b"SYLLASLOPOS: invalid kernel stack\r\n");
        halt();
    }

    serial.write(b"SYLLASLOPOS: x86_64 kernel entry ok\r\n");
    // SAFETY: single boot CPU at CPL0, interrupts masked, higher half mapped.
    unsafe { interrupts::install() };
    // SAFETY: as above, with the IDT live so a mistake here is reported.
    let guards = unsafe { paging::protect(info_phys) };
    crash::set_guards(guards);
    let _ = write!(
        serial,
        "SYLLASLOPOS: exceptions armed, null guard below {:#x}, stack guard at {:#x}\r\n",
        guards.null_end, guards.stack_guard
    );

    // SAFETY: the magic and entry stack were checked, GRUB handed off this
    // immutable physical blob, and the identity map above the null guard is
    // still live.
    let boot = unsafe { bootinfo::BootInfo::read(info_phys) };
    let framebuffer = boot
        .as_ref()
        .filter(|info| !info.serial_only())
        .and_then(|info| {
            // SAFETY: single boot CPU, framebuffer unclaimed, no interrupts.
            unsafe { graphics::Framebuffer::attach(info) }
        });
    if let Some(frame) = framebuffer.as_ref() {
        let video = frame.video();
        let _ = write!(
            serial,
            "SYLLASLOPOS: framebuffer {}x{}x{} {}\r\n",
            video.width,
            video.height,
            video.bpp,
            frame.caching().label()
        );
    }
    // SAFETY: the legacy PC i8042 ports are unclaimed and IRQs are masked.
    let keyboard = unsafe { input::Keyboard::init() };
    console::run(serial, keyboard, framebuffer, boot.as_ref())
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    crash::panic(info)
}

fn halt() -> ! {
    loop {
        // SAFETY: the boot CPU remains at CPL0 with maskable interrupts off.
        unsafe { asm!("cli", "hlt", options(nomem, nostack)) };
    }
}
