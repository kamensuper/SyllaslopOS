# Syllaslop Console

SLC is the default kernel-resident console. It provides a graphical
command bar and a concurrent COM1 path using one state machine. The PS/2
decoder handles translated set 1, US printable keys, modifier releases and
extended navigation. COM1 accepts ASCII, common ANSI navigation sequences
and Ctrl+] for returning from a view.

## Commands and boundaries

| Command | Behavior |
| --- | --- |
| `help [filter]` | Search command names and descriptions |
| `history` | Browse this boot's history and return a selection to the editable prompt |
| `sysinfo`, `cpu`, `memory`, `bootinfo`, `devices` | Inspect CPUID, the Multiboot2 handoff, framebuffer and active legacy interfaces |
| `echo`, `clear` | Print ASCII text or reset the activity log |
| `theme [name]`, `zoom [1\|2]` | Change palette or scale framebuffer text |
| `calc <expression>`, `base <expression>` | Compute signed integer expressions or display a result in four bases |
| `memmap [page]` | List physical ranges from the bootloader with exclusive end addresses |
| `keys` | Show the keyboard controls |
| `metrics` | Report actual framebuffer bytes written |
| `version`, `about`, `uptime` | Read version, current scope or clock availability |
| `halt confirm`, `reboot confirm` | Halt the boot CPU or request an i8042 reset |

Command names are case-insensitive. Input is capped at 96 printable ASCII
bytes. The command history retains 16 lines. Views and transcript each
retain at most 48 lines. These buffers and the font masks need no heap.
Files and process commands explain the missing filesystem and scheduler.
Memory values are the firmware map's usable regions, before kernel
reservations. The device view records configured boot interfaces.

## Rendering and focus

The console shell model consumes device-independent key events. Command
handlers receive a typed system snapshot and update bounded output. The
renderer draws from that model, with clipped glyph masks and rectangles.
The CPU and memory views share one focus policy. In history, Enter recalls
a command for editing. In help, typing filters immediately and PgUp/PgDn
moves through matches. Esc returns to the command prompt.

At idle, the near-black framebuffer shows a two-tone wordmark, a command
field, and one shortcut line. Command results use a reading column.
System views occupy that column on demand. Screen changes are immediate
and use the existing clipped framebuffer renderer.

Ctrl+P, F1, and `help` use the same command finder. Exact and prefix
matches appear before subsequence and description matches. Ctrl+R or
`history` filters recent commands from the current boot. Arrow keys change
the selection. Enter stages a selection in the editable prompt, and a
separate Enter submits it. Esc returns to the prior draft. On COM1 use
Ctrl+] for Esc.

In the command field, nonempty input can expose a muted inline suffix.
Suggestions come from this boot's history, then command names. Right
accepts the entire suggestion when the caret is at the end of the field.
Tab still performs command and argument completion.

The calculator accepts decimal, `0x` hex, `0o` octal, and `0b` binary.
Precedence is unary minus, `* / %`, `+ -`, `&`, `^`, then `|`.
Parentheses override precedence. Operations use checked signed 64-bit
integers and report invalid syntax, overflow, or division by zero.

See [CLI references and choices](CLI-RESEARCH.md) and the
[earlier OpenCode source study](OPENCODE-V2-RESEARCH.md).

## Implementation files

| Area | Files |
| --- | --- |
| Boot and handoff | `arch/x86_64/boot.S`, `arch/x86_64/linker.ld`, `boot/grub/grub.cfg`, `kernel/main.rs`, `kernel/bootinfo.rs` |
| Input and services | `kernel/input.rs`, `kernel/io.rs`, `kernel/serial.rs`, `kernel/console/commands.rs` |
| Console model and renderer | `kernel/console/{mod,state,screen,numbers}.rs`, `kernel/graphics/{mod,framebuffer}.rs`, bitmap masks and license under `kernel/graphics/assets/` |
| Builds and tests | `Makefile`, `kernel/host_tests.rs`, `tools/console-check.py`, `tools/bake-fonts.py` |
| Project documentation | `README.md`, `PROGRESS.md`, `docs/{BOOT,CONSOLE,CLI-RESEARCH,OPENCODE-V2-RESEARCH,REFERENCES}.md` |

## Verification procedure

```sh
make -j4
make test
make test-console
qemu-system-x86_64 -machine pc-i440fx-7.2 -accel tcg -cpu qemu64 -m 128M \
  -boot d -cdrom build/syllaslopos.iso -nic none -no-reboot \
  -display gtk -monitor none -serial stdio
```

The QMP tests inject actual PS/2 key presses at both resolutions, drive
COM1 directly on a separate recovery boot, and capture the guest display to
`build/screens*/`. QEMU video screenshots come from the device scanout.
See [the verification record](../PROGRESS.md) for measured results.

## Remaining work

Input uses polled i8042 and COM1 because there is no IDT or IRQ dispatcher.
It can keep a virtual CPU busy while idle. The console has a fixed firmware
display mode, a US keyboard map, ASCII glyphs and boot-local history.
There is no USB HID, hardware device enumeration, filesystem, process
scheduler, protected userspace, calibrated clock, or physical/UEFI
qualification. When IRQs, IPC, input, and display services are available,
keep this recovery front end in the kernel and move the full console into
an unprivileged process backed by those services.
