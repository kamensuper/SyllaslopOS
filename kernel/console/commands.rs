use core::arch::x86_64::__cpuid;

use super::state::{Change, Console, Theme, Tone, View};
use crate::bootinfo::{BootInfo, Memory, Video};

pub const VERSION: &str = "0.2.0";

pub struct Cpu {
    vendor: [u8; 12],
    brand: [u8; 48],
    family: u32,
    model: u32,
    stepping: u32,
    nx: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            vendor: [0; 12],
            brand: [0; 48],
            family: 0,
            model: 0,
            stepping: 0,
            nx: false,
        }
    }
}

impl Cpu {
    fn read() -> Self {
        // SAFETY: long mode requires a CPU with CPUID; maximum leaves are
        // checked before querying optional fields.
        unsafe {
            let base = __cpuid(0);
            let ext = __cpuid(0x8000_0000).eax;
            let mut cpu = Self::default();
            cpu.vendor[..4].copy_from_slice(&base.ebx.to_le_bytes());
            cpu.vendor[4..8].copy_from_slice(&base.edx.to_le_bytes());
            cpu.vendor[8..].copy_from_slice(&base.ecx.to_le_bytes());
            if base.eax >= 1 {
                let version = __cpuid(1).eax;
                let family = (version >> 8) & 15;
                cpu.family = family
                    + if family == 15 {
                        (version >> 20) & 255
                    } else {
                        0
                    };
                cpu.model = (version >> 4) & 15;
                if matches!(family, 6 | 15) {
                    cpu.model |= (version >> 12) & 0xf0;
                }
                cpu.stepping = version & 15;
            }
            if ext >= 0x8000_0001 {
                cpu.nx = __cpuid(0x8000_0001).edx & (1 << 20) != 0;
            }
            if ext >= 0x8000_0004 {
                for (i, leaf) in cpu.brand.chunks_exact_mut(16).enumerate() {
                    let r = __cpuid(0x8000_0002 + i as u32);
                    for (chunk, word) in leaf.chunks_exact_mut(4).zip([r.eax, r.ebx, r.ecx, r.edx])
                    {
                        chunk.copy_from_slice(&word.to_le_bytes());
                    }
                }
            }
            cpu
        }
    }

    fn vendor(&self) -> &str {
        cpu_string(&self.vendor)
    }

    fn brand(&self) -> &str {
        cpu_string(&self.brand)
    }
}

fn cpu_string(bytes: &[u8]) -> &str {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    core::str::from_utf8(&bytes[..end])
        .unwrap_or("")
        .trim_ascii()
}

#[derive(Default)]
pub struct Services {
    pub cpu: Cpu,
    pub memory: Option<Memory>,
    pub video: Option<Video>,
    pub loader: Option<&'static str>,
    pub boot_blob: Option<(u64, usize)>,
    pub keyboard: bool,
    boot: Option<BootInfo<'static>>,
    pub frame_bytes: u64,
    pub initial_bytes: u64,
    pub last_draw_bytes: u64,
}

impl Services {
    pub fn collect(boot: Option<&BootInfo<'static>>, video: Option<Video>, keyboard: bool) -> Self {
        let (loader, memory, boot_blob) = boot.map_or((None, None, None), |boot| {
            (
                boot.loader(),
                boot.memory(),
                Some((boot.address(), boot.size())),
            )
        });
        Self {
            cpu: Cpu::read(),
            boot: boot.copied(),
            memory,
            video,
            loader,
            boot_blob,
            keyboard,
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy)]
enum Args {
    None,
    Text,
    Choices(&'static [&'static str]),
}

#[derive(Clone, Copy)]
enum Need {
    Base,
    MemoryMap,
    Graphics,
    Keyboard,
}

pub struct Command {
    pub name: &'static str,
    pub usage: &'static str,
    pub description: &'static str,
    args: Args,
    need: Need,
    handler: fn(&mut Console, &Services, &str) -> Change,
}

const fn cmd(
    name: &'static str,
    usage: &'static str,
    description: &'static str,
    args: Args,
    need: Need,
    handler: fn(&mut Console, &Services, &str) -> Change,
) -> Command {
    Command {
        name,
        usage,
        description,
        args,
        need,
        handler,
    }
}

pub static REGISTRY: &[Command] = &[
    cmd(
        "help",
        "help [filter]",
        "Search the command reference",
        Args::Text,
        Need::Base,
        help,
    ),
    cmd(
        "sysinfo",
        "sysinfo",
        "Open the system overview",
        Args::None,
        Need::Base,
        sysinfo,
    ),
    cmd(
        "cpu",
        "cpu",
        "Read processor information",
        Args::None,
        Need::Base,
        cpu,
    ),
    cmd(
        "memory",
        "memory",
        "Summarize the firmware memory map",
        Args::None,
        Need::MemoryMap,
        memory,
    ),
    cmd(
        "bootinfo",
        "bootinfo",
        "Inspect the Multiboot2 handoff",
        Args::None,
        Need::Base,
        bootinfo,
    ),
    cmd(
        "devices",
        "devices",
        "Show active boot interfaces",
        Args::None,
        Need::Base,
        devices,
    ),
    cmd(
        "metrics",
        "metrics",
        "Inspect framebuffer write counters",
        Args::None,
        Need::Graphics,
        metrics,
    ),
    cmd(
        "history",
        "history",
        "Recall a command from this boot",
        Args::None,
        Need::Base,
        history,
    ),
    cmd(
        "echo",
        "echo [text]",
        "Print text to the activity log",
        Args::Text,
        Need::Base,
        echo,
    ),
    cmd(
        "clear",
        "clear",
        "Clear the activity log",
        Args::None,
        Need::Base,
        clear,
    ),
    cmd(
        "theme",
        "theme [name]",
        "Select graphite, ember or mono",
        Args::Choices(&["graphite", "ember", "mono"]),
        Need::Base,
        theme,
    ),
    cmd(
        "zoom",
        "zoom [1|2]",
        "Set the text scale",
        Args::Choices(&["1", "2"]),
        Need::Graphics,
        zoom,
    ),
    cmd(
        "calc",
        "calc <expression>",
        "Compute integer arithmetic and bitwise expressions",
        Args::Text,
        Need::Base,
        calc,
    ),
    cmd(
        "base",
        "base <expression>",
        "Show decimal, hex, octal and binary",
        Args::Text,
        Need::Base,
        base,
    ),
    cmd(
        "memmap",
        "memmap [page]",
        "List physical ranges from the bootloader",
        Args::Text,
        Need::MemoryMap,
        memmap,
    ),
    cmd(
        "keys",
        "keys",
        "Show console keyboard shortcuts",
        Args::None,
        Need::Base,
        keys,
    ),
    cmd(
        "version",
        "version",
        "Show the console version",
        Args::None,
        Need::Base,
        version,
    ),
    cmd(
        "about",
        "about",
        "Describe this build of SyllaslopOS",
        Args::None,
        Need::Base,
        about,
    ),
    cmd(
        "uptime",
        "uptime",
        "Check monotonic clock availability",
        Args::None,
        Need::Base,
        uptime,
    ),
    cmd(
        "reboot",
        "reboot confirm",
        "Reset this machine through i8042",
        Args::Choices(&["confirm"]),
        Need::Keyboard,
        reboot,
    ),
    cmd(
        "halt",
        "halt confirm",
        "Stop the boot CPU",
        Args::Choices(&["confirm"]),
        Need::Base,
        halt,
    ),
];

pub fn run(console: &mut Console, services: &Services, text: &str) -> Change {
    let end = text.find(' ').unwrap_or(text.len());
    let (name, rest) = text.split_at(end);
    let argument = rest.trim_ascii();
    let Some(command) = REGISTRY.iter().find(|c| c.name.eq_ignore_ascii_case(name)) else {
        let detail = match name {
            "ls" | "cd" | "cat" | "pwd" | "mkdir" => {
                "Filesystem commands need a mounted filesystem."
            }
            "ps" | "kill" | "tasks" => "Process commands need a scheduler and userspace.",
            _ => "Use help to see available commands.",
        };
        console.log(Tone::Error, format_args!("Unknown command: {name}"));
        console.log(Tone::Muted, format_args!("{detail}"));
        return Change::Report;
    };
    let valid = match command.args {
        Args::None => argument.is_empty(),
        Args::Text => true,
        Args::Choices(choices) => argument.is_empty() || choices.contains(&argument),
    };
    if !valid {
        console.log(Tone::Error, format_args!("Usage: {}", command.usage));
        return Change::Report;
    }
    let missing = match command.need {
        Need::MemoryMap if services.memory.is_none() => {
            Some("The bootloader memory map is unavailable.")
        }
        Need::Graphics if services.video.is_none() => {
            Some("This command needs an active framebuffer.")
        }
        Need::Keyboard if !services.keyboard => Some("The i8042 controller is unavailable."),
        _ => None,
    };
    if let Some(reason) = missing {
        console.log(Tone::Error, format_args!("{reason}"));
        return Change::Report;
    }
    (command.handler)(console, services, argument)
}

/// Completion matches without regard to case, as command lookup does.
pub fn complete(first: &str, rest: Option<&str>, mut emit: impl FnMut(&'static str)) {
    let begins = |name: &str, typed: &str| {
        name.as_bytes()
            .get(..typed.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(typed.as_bytes()))
    };
    if let Some(rest) = rest {
        let Some(command) = REGISTRY.iter().find(|c| c.name.eq_ignore_ascii_case(first)) else {
            return;
        };
        if command.name == "help" {
            for next in REGISTRY.iter().filter(|c| begins(c.name, rest)) {
                emit(next.name);
            }
        } else if let Args::Choices(choices) = command.args {
            for &choice in choices.iter().filter(|choice| begins(choice, rest)) {
                emit(choice);
            }
        }
    } else {
        for command in REGISTRY.iter().filter(|c| begins(c.name, first)) {
            emit(command.name);
        }
    }
}

fn help(console: &mut Console, _: &Services, args: &str) -> Change {
    console.help(args)
}
fn history(console: &mut Console, _: &Services, _: &str) -> Change {
    console.history()
}
fn clear(console: &mut Console, _: &Services, _: &str) -> Change {
    console.clear()
}

fn echo(console: &mut Console, _: &Services, args: &str) -> Change {
    console.log(Tone::Text, format_args!("{args}"));
    Change::Report
}

fn version(console: &mut Console, _: &Services, _: &str) -> Change {
    console.log(
        Tone::Text,
        format_args!("Syllaslop Console {VERSION} / x86_64"),
    );
    Change::Report
}

fn about(console: &mut Console, _: &Services, _: &str) -> Change {
    console.log(
        Tone::Text,
        format_args!("SyllaslopOS is an independent x86-64 successor to Slopix."),
    );
    console.log(
        Tone::Muted,
        format_args!("This recovery console runs inside the freestanding Rust kernel."),
    );
    Change::Report
}

fn sysinfo(console: &mut Console, services: &Services, _: &str) -> Change {
    console.open(View::System);
    console.detail(Tone::Accent, format_args!("SYSTEM OVERVIEW"));
    console.detail(
        Tone::Text,
        format_args!("Kernel      SyllaslopOS / x86_64 long mode"),
    );
    console.detail(
        Tone::Text,
        format_args!("Console     slc {VERSION} / kernel mode"),
    );
    console.detail(
        Tone::Text,
        format_args!("Boot        {}", services.loader.unwrap_or("Multiboot2")),
    );
    console.detail(
        Tone::Text,
        format_args!("Processor   {}", services.cpu.vendor()),
    );
    if let Some(memory) = services.memory {
        console.detail(
            Tone::Text,
            format_args!(
                "Usable RAM  {} MiB reported by firmware",
                memory.usable_bytes / 1_048_576
            ),
        );
    }
    if let Some(video) = services.video {
        console.detail(
            Tone::Text,
            format_args!(
                "Display     {} x {} / {} bpp RGB",
                video.width, video.height, video.bpp
            ),
        );
    } else {
        console.detail(Tone::Text, format_args!("Display     COM1 recovery"));
    }
    console.detail(
        Tone::Muted,
        format_args!("Execution   bootstrap CPU / interrupts masked"),
    );
    Change::Report
}

fn cpu(console: &mut Console, services: &Services, _: &str) -> Change {
    let cpu = &services.cpu;
    console.open(View::Cpu);
    console.detail(Tone::Accent, format_args!("PROCESSOR / CPUID"));
    console.detail(Tone::Text, format_args!("Vendor      {}", cpu.vendor()));
    if !cpu.brand().is_empty() {
        console.detail(Tone::Text, format_args!("Brand       {}", cpu.brand()));
    }
    console.detail(
        Tone::Text,
        format_args!(
            "Signature   family {} / model {} / stepping {}",
            cpu.family, cpu.model, cpu.stepping
        ),
    );
    console.detail(Tone::Text, format_args!("Mode        x86-64 long mode"));
    console.detail(
        Tone::Text,
        format_args!(
            "Execute disable   {}",
            if cpu.nx { "available" } else { "unavailable" }
        ),
    );
    Change::Report
}

fn memory(console: &mut Console, services: &Services, _: &str) -> Change {
    let Some(memory) = services.memory else {
        return Change::Report;
    };
    console.open(View::Memory);
    console.detail(Tone::Accent, format_args!("FIRMWARE MEMORY MAP"));
    console.detail(
        Tone::Text,
        format_args!("Map entries      {}", memory.regions),
    );
    console.detail(
        Tone::Text,
        format_args!("Usable regions   {}", memory.usable_regions),
    );
    console.detail(
        Tone::Text,
        format_args!(
            "Usable bytes     {} ({} MiB)",
            memory.usable_bytes,
            memory.usable_bytes / 1_048_576
        ),
    );
    console.detail(Tone::Muted, format_args!("Allocator        unavailable"));
    Change::Report
}

fn bootinfo(console: &mut Console, services: &Services, _: &str) -> Change {
    console.open(View::Boot);
    console.detail(Tone::Accent, format_args!("MULTIBOOT2 HANDOFF"));
    console.detail(
        Tone::Text,
        format_args!("Loader        {}", services.loader.unwrap_or("Unavailable")),
    );
    if let Some((address, size)) = services.boot_blob {
        console.detail(
            Tone::Text,
            format_args!("Info address  0x{address:08x} physical / {size} bytes"),
        );
    } else {
        console.detail(
            Tone::Muted,
            format_args!("Boot information tags unavailable."),
        );
    }
    if let Some(video) = services.video {
        console.detail(
            Tone::Text,
            format_args!(
                "Framebuffer   0x{:08x} physical / pitch {} bytes",
                video.phys, video.pitch
            ),
        );
    }
    Change::Report
}

fn devices(console: &mut Console, services: &Services, _: &str) -> Change {
    console.open(View::Devices);
    console.detail(Tone::Accent, format_args!("ACTIVE BOOT INTERFACES"));
    console.detail(Tone::Text, format_args!("COM1       0x3f8 / 115200 8N1"));
    if services.keyboard {
        console.detail(
            Tone::Text,
            format_args!("i8042      first port / translated US set 1"),
        );
    }
    if services.video.is_some() {
        console.detail(
            Tone::Text,
            format_args!("Framebuffer  Multiboot2 direct RGB"),
        );
    }
    console.detail(
        Tone::Muted,
        format_args!("Device enumeration is unavailable."),
    );
    Change::Report
}

fn metrics(console: &mut Console, services: &Services, _: &str) -> Change {
    console.log(
        Tone::Text,
        format_args!(
            "Framebuffer writes: {} bytes since boot",
            services.frame_bytes
        ),
    );
    console.log(
        Tone::Text,
        format_args!(
            "First frame: {} bytes / last update: {} bytes",
            services.initial_bytes, services.last_draw_bytes
        ),
    );
    Change::Report
}

fn theme(console: &mut Console, _: &Services, args: &str) -> Change {
    console.theme = match args {
        "graphite" => Theme::Graphite,
        "ember" => Theme::Ember,
        "mono" => Theme::Mono,
        _ => {
            console.log(
                Tone::Muted,
                format_args!(
                    "Theme: {}. Choices: graphite, ember, mono.",
                    console.theme.label()
                ),
            );
            return Change::Report;
        }
    };
    console.log(
        Tone::Success,
        format_args!("Theme set to {}.", console.theme.label()),
    );
    Change::Full
}

fn zoom(console: &mut Console, _: &Services, args: &str) -> Change {
    let scale = match args {
        "1" => 1,
        "2" => 2,
        _ => {
            let current = console.zoom;
            console.log(
                Tone::Muted,
                format_args!("Text scale: {current}. Choices: 1, 2."),
            );
            return Change::Report;
        }
    };
    console.zoom = scale;
    console.log(Tone::Success, format_args!("Text scale set to {scale}."));
    Change::Full
}

fn calc(console: &mut Console, _: &Services, args: &str) -> Change {
    print_number(console, args, false)
}

fn base(console: &mut Console, _: &Services, args: &str) -> Change {
    print_number(console, args, true)
}

fn print_number(console: &mut Console, args: &str, all_bases: bool) -> Change {
    match super::numbers::evaluate(args) {
        Ok(value) if all_bases => {
            let sign = if value < 0 { "-" } else { "" };
            let magnitude = value.unsigned_abs();
            console.log(Tone::Text, format_args!("dec  {value}"));
            console.log(Tone::Text, format_args!("hex  {sign}0x{magnitude:x}"));
            console.log(Tone::Text, format_args!("oct  {sign}0o{magnitude:o}"));
            console.log(Tone::Text, format_args!("bin  {sign}0b{magnitude:b}"));
        }
        Ok(value) => console.log(Tone::Success, format_args!("= {value}")),
        Err(reason) => console.log(Tone::Error, format_args!("{reason}")),
    }
    Change::Report
}

fn memmap(console: &mut Console, services: &Services, arg: &str) -> Change {
    const PAGE: usize = 24;
    let Some(regions) = services.boot.as_ref().and_then(BootInfo::regions) else {
        console.log(
            Tone::Error,
            format_args!("The bootloader memory map is unavailable."),
        );
        return Change::Report;
    };
    let count = services.memory.map_or(0, |memory| memory.regions);
    let pages = count.div_ceil(PAGE).max(1);
    let page = if arg.is_empty() {
        Some(1)
    } else {
        arg.parse::<usize>().ok()
    };
    let Some(page) = page.filter(|page| (1..=pages).contains(page)) else {
        console.log(
            Tone::Error,
            format_args!("Use memmap <page> with a page from 1 through {pages}."),
        );
        return Change::Report;
    };
    console.open(View::Map);
    console.detail(
        Tone::Accent,
        format_args!("PHYSICAL MEMORY MAP  /  {page} of {pages}"),
    );
    console.detail(
        Tone::Muted,
        format_args!("START                END (EXCLUSIVE)      TYPE"),
    );
    for range in regions.skip((page - 1) * PAGE).take(PAGE).flatten() {
        let kind = match range.kind {
            1 => "usable",
            2 => "reserved",
            3 => "ACPI reclaimable",
            4 => "ACPI NVS",
            5 => "bad RAM",
            _ => "other",
        };
        console.detail(
            Tone::Text,
            format_args!("{:016x}     {:016x}     {kind}", range.start, range.end),
        );
    }
    Change::Report
}

fn keys(console: &mut Console, _: &Services, _: &str) -> Change {
    console.open(View::System);
    console.detail(Tone::Accent, format_args!("KEYBOARD"));
    for line in [
        "Ctrl+P / F1    find a command",
        "Ctrl+R         search this boot's history",
        "Right          accept inline suggestion at end",
        "Tab            complete command or argument",
        "Up / Down      recall history or select a match",
        "Ctrl+A / E     move to the start or end of the line",
        "Ctrl+Left/Right  move by word (COM1: Alt+B / Alt+F)",
        "Ctrl+W / K     erase the previous word or to the end",
        "Ctrl+U         erase the line",
        "PgUp / PgDn    scroll output or page through matches",
        "Ctrl+L         clear output",
        "Esc            return to prompt (COM1: Ctrl+])",
    ] {
        console.detail(Tone::Text, format_args!("{line}"));
    }
    Change::Report
}

fn uptime(console: &mut Console, _: &Services, _: &str) -> Change {
    console.log(
        Tone::Muted,
        format_args!("Uptime needs a calibrated monotonic clock."),
    );
    Change::Report
}

fn reboot(console: &mut Console, _: &Services, args: &str) -> Change {
    power(console, args, "reboot", Change::Reboot)
}

fn halt(console: &mut Console, _: &Services, args: &str) -> Change {
    power(console, args, "halt", Change::Halt)
}

fn power(console: &mut Console, args: &str, name: &str, change: Change) -> Change {
    if args != "confirm" {
        console.log(Tone::Muted, format_args!("Type {name} confirm to proceed."));
        return Change::Report;
    }
    console.log(Tone::Success, format_args!("Confirmed: {name}."));
    change
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_registry_entries_and_real_capability_checks() {
        for (i, command) in REGISTRY.iter().enumerate() {
            assert!(!REGISTRY[..i].iter().any(|c| c.name == command.name));
            assert!(command.usage.starts_with(command.name));
        }
        let mut console = Console::new();
        let system = Services::default();
        assert_eq!(run(&mut console, &system, "ls"), Change::Report);
        assert!(console
            .row(1)
            .unwrap()
            .text()
            .contains("mounted filesystem"));
        run(&mut console, &system, "cpu extra");
        assert_eq!(console.row(2).unwrap().text(), "Usage: cpu");
        run(&mut console, &system, "memory");
        assert!(console.row(3).unwrap().text().contains("unavailable"));
        run(&mut console, &system, "echo two  spaces");
        assert_eq!(console.row(4).unwrap().text(), "two  spaces");
    }
}
