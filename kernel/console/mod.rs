mod commands;
mod screen;
mod state;

use crate::bootinfo::BootInfo;
use crate::graphics::Framebuffer;
use crate::input::{Key, Keyboard, SerialKeys};
use crate::serial::Com1;

use commands::Services;
use state::{Change, Console, Line, Tone, View};

pub fn run(
    mut serial: Com1,
    mut keyboard: Option<Keyboard>,
    mut framebuffer: Option<Framebuffer>,
    boot: Option<&BootInfo<'static>>,
) -> ! {
    let video = framebuffer.as_ref().map(Framebuffer::video);
    let mut services = Services::collect(boot, video, keyboard.is_some());
    let mut console = Console::new();
    console.log(
        Tone::Success,
        format_args!("Kernel initialized / x86_64 long mode"),
    );
    if let Some(video) = video {
        console.log(
            Tone::Text,
            format_args!(
                "Framebuffer {} x {} / {} bpp",
                video.width, video.height, video.bpp
            ),
        );
    } else {
        console.log(Tone::Muted, format_args!("COM1 recovery console"));
    }
    console.log(
        Tone::Muted,
        format_args!("Type help for commands. F1 opens the reference."),
    );
    if let Some(frame) = framebuffer.as_mut() {
        screen::draw(frame, &console, &services, Change::Full);
        services.initial_bytes = frame.bytes_written();
        services.frame_bytes = services.initial_bytes;
        services.last_draw_bytes = services.initial_bytes;
    }
    message(
        &mut serial,
        format_args!(
            "SLC READY {}",
            if video.is_some() {
                "graphics"
            } else {
                "serial"
            }
        ),
    );
    if let Some(video) = video {
        message(
            &mut serial,
            format_args!("SLC VIDEO {}x{}x{}", video.width, video.height, video.bpp),
        );
    }
    report(&mut serial, &console);
    serial_prompt(&mut serial, &console);

    let mut serial_keys = SerialKeys::default();
    loop {
        let ps2 = keyboard.as_mut().and_then(Keyboard::poll);
        let from_serial = serial.read().and_then(|byte| serial_keys.feed(byte));
        if ps2.is_none() && from_serial.is_none() {
            core::hint::spin_loop();
        }
        for (key, serial_input) in [(ps2, false), (from_serial, true)] {
            let Some(key) = key else { continue };
            let was_view = console.view;
            let change = console.event(key, &services);
            if change == Change::None {
                continue;
            }

            if let Some(frame) = framebuffer.as_mut() {
                let before = frame.bytes_written();
                screen::draw(frame, &console, &services, change);
                services.frame_bytes = frame.bytes_written();
                services.last_draw_bytes = services.frame_bytes - before;
            }

            match change {
                Change::Report | Change::Full | Change::Halt | Change::Reboot => {
                    report(&mut serial, &console);
                    serial_prompt(&mut serial, &console);
                }
                Change::Panel if serial_input => {
                    if console.view != View::Activity {
                        report(&mut serial, &console);
                    }
                    serial_prompt(&mut serial, &console);
                }
                Change::Prompt if serial_input => {
                    if key == Key::Tab && !console.hint.bytes().is_empty() {
                        serial.write(b"\r\x1b[2K");
                        serial.write(console.hint.bytes());
                        serial.write(b"\r\n");
                    }
                    serial_prompt(&mut serial, &console);
                }
                _ if was_view != console.view => serial_prompt(&mut serial, &console),
                _ => {}
            }
            match change {
                Change::Halt => crate::halt(),
                Change::Reboot => {
                    if keyboard.as_mut().is_some_and(Keyboard::reset_machine) {
                        crate::halt();
                    }
                    console.log(
                        Tone::Error,
                        format_args!("The i8042 controller rejected the reset request."),
                    );
                    report(&mut serial, &console);
                    if let Some(frame) = framebuffer.as_mut() {
                        screen::draw(frame, &console, &services, Change::Report);
                    }
                }
                _ => {}
            }
        }
    }
}

fn report(serial: &mut Com1, console: &Console) {
    serial.write(b"\r\x1b[2K");
    if console.view == View::Help && !console.filter.bytes().is_empty() {
        message(
            serial,
            format_args!("[{}: {}]", console.view.label(), console.filter.text()),
        );
    } else if console.view != View::Activity {
        message(serial, format_args!("[{}]", console.view.label()));
    } else if console.rows() == 0 {
        message(serial, format_args!("Activity cleared"));
    }
    let range = match console.view {
        View::Activity => console.rows().saturating_sub(console.fresh)..console.rows(),
        View::History | View::Help => {
            let start = if console.view == View::History {
                console.selected
            } else {
                console.scroll
            };
            start..(start + 16).min(console.rows())
        }
        _ => console.scroll..console.rows(),
    };
    for index in range {
        if let Some(line) = console.row(index) {
            serial.write(line.bytes());
            serial.write(b"\r\n");
        }
    }
}

fn serial_prompt(serial: &mut Com1, console: &Console) {
    serial.write(b"\r\x1b[2K");
    let edit = match console.view {
        View::Activity => {
            serial.write(b"slc> ");
            &console.prompt
        }
        View::Help => {
            serial.write(b"find (Ctrl+] back)> ");
            &console.filter
        }
        View::History => {
            serial.write(b"history (Enter recall, Ctrl+] back)> ");
            return;
        }
        _ => {
            serial.write(b"Ctrl+] back> ");
            return;
        }
    };
    let cells = 52;
    let start = edit.cursor.saturating_sub(cells);
    let end = (start + cells).min(edit.bytes().len());
    serial.write(&edit.bytes()[start..end]);
    let left = end - edit.cursor;
    if left != 0 {
        let sequence = Line::format(Tone::Text, format_args!("{left}D"));
        serial.write(b"\x1b[");
        serial.write(sequence.bytes());
    }
}

fn message(serial: &mut Com1, args: core::fmt::Arguments<'_>) {
    let line = Line::format(Tone::Text, args);
    serial.write(line.bytes());
    serial.write(b"\r\n");
}
