//! Replace the first 2 MiB boot leaf with 4 KiB pages so that null pointer
//! accesses and kernel stack overflows fault instead of touching memory.
//!
//! The boot stub maps physical 0-32 MiB with 2 MiB leaves at VA 0 and at
//! the kernel base, and both aliases share one page directory. Splitting
//! entry 0 therefore changes both aliases identically.

#[cfg(not(test))]
use core::arch::asm;
#[cfg(not(test))]
use core::cell::UnsafeCell;
#[cfg(not(test))]
use core::ptr::{addr_of, addr_of_mut, write_volatile};

pub const KERNEL_BASE: u64 = 0xffff_ffff_8000_0000;
const PAGE: u64 = 4096;
/// Unmap low memory up to this address, unless boot data lives below it.
const NULL_GUARD: u64 = 64 * 1024;
const PRESENT_WRITABLE: u64 = 0x3;
#[cfg(not(test))]
const ENTRIES: usize = 512;

#[cfg(not(test))]
extern "C" {
    static mut page_directory: [u64; ENTRIES];
    static __stack_guard: u8;
}

#[cfg(not(test))]
#[repr(C, align(4096))]
struct Table(UnsafeCell<[u64; ENTRIES]>);

// SAFETY: `protect` writes the table once before the CPU can walk it.
#[cfg(not(test))]
unsafe impl Sync for Table {}

#[cfg(not(test))]
static LOW_TABLE: Table = Table(UnsafeCell::new([0; ENTRIES]));

#[derive(Clone, Copy)]
pub struct Guards {
    /// Accesses below this address fault.
    pub null_end: u64,
    /// Higher-half address of the unmapped page below the kernel stack.
    pub stack_guard: u64,
}

/// Which pages of the first 2 MiB stay mapped.
fn plan(null_end: u64, stack_guard_phys: u64) -> impl Fn(u64) -> bool {
    move |phys| phys >= null_end && phys != stack_guard_phys
}

/// Keep at least page 0 unmapped, and keep the page holding the boot
/// information blob mapped.
fn null_end(info_phys: u64) -> u64 {
    (info_phys & !(PAGE - 1)).clamp(PAGE, NULL_GUARD)
}

/// Call once on the boot CPU with interrupts masked, the entry stub's page
/// tables active and the kernel IDT installed.
#[cfg(not(test))]
pub unsafe fn protect(info_phys: u64) -> Guards {
    let stack_guard = addr_of!(__stack_guard) as u64;
    let guards = Guards {
        null_end: null_end(info_phys),
        stack_guard,
    };
    let mapped = plan(guards.null_end, stack_guard - KERNEL_BASE);
    let table = &mut *LOW_TABLE.0.get();
    for (index, entry) in table.iter_mut().enumerate() {
        let phys = index as u64 * PAGE;
        *entry = if mapped(phys) {
            phys | PRESENT_WRITABLE
        } else {
            0
        };
    }
    // The new table maps every page the old 2 MiB leaf mapped, except the
    // guards, so the single entry swap never unmaps live code or data.
    let table_phys = LOW_TABLE.0.get() as u64 - KERNEL_BASE;
    write_volatile(
        addr_of_mut!(page_directory).cast::<u64>(),
        table_phys | PRESENT_WRITABLE,
    );
    asm!(
        "mov {tmp}, cr3",
        "mov cr3, {tmp}",
        tmp = out(reg) _,
        options(nostack, preserves_flags)
    );
    guards
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guards_cover_low_memory_without_hiding_boot_data() {
        assert_eq!(null_end(0x10_b000), NULL_GUARD);
        assert_eq!(null_end(0x9000), 0x9000);
        assert_eq!(null_end(0x9abc), 0x9000);
        assert_eq!(null_end(0), PAGE);
        let mapped = plan(NULL_GUARD, 0x1a_0000);
        assert!(!mapped(0));
        assert!(!mapped(0xf000));
        assert!(mapped(0x1_0000));
        assert!(mapped(0x10_0000));
        assert!(!mapped(0x1a_0000));
        assert!(mapped(0x1a_1000));
    }
}
