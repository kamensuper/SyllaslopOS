use crate::io::{inb, outb};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Char(u8),
    Enter,
    Tab,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Escape,
    PageUp,
    PageDown,
    Clear,
    Palette,
    SearchHistory,
    Erase,
    Function(u8),
}

const STATUS: u16 = 0x64;
const DATA: u16 = 0x60;
const WAIT_LIMIT: usize = 10_000;

pub struct Keyboard {
    decoder: Set1,
}

impl Keyboard {
    /// The boot CPU owns the BIOS i8042 keyboard port. Keep IRQ delivery
    /// masked and leave the auxiliary port's configuration unchanged.
    pub unsafe fn init() -> Option<Self> {
        for _ in 0..32 {
            if inb(STATUS) & 1 == 0 {
                break;
            }
            inb(DATA);
        }
        if !command(0x20) {
            return None;
        }
        let status = (0..WAIT_LIMIT).find_map(|_| {
            let status = inb(STATUS);
            (status & 1 != 0 && status & 0x20 == 0).then_some(status)
        })?;
        let config = inb(DATA);
        if status & 0xc0 != 0 || !command(0x60) || !write_ready() {
            return None;
        }
        outb(DATA, (config | 0x40) & !1);
        if !command(0xae) {
            return None;
        }
        Some(Self {
            decoder: Set1::default(),
        })
    }

    pub fn poll(&mut self) -> Option<Key> {
        // SAFETY: initialization reserves the first i8042 port; CPL0, IRQs off.
        unsafe {
            let status = inb(STATUS);
            if status & 1 == 0 || status == 0xff {
                return None;
            }
            let byte = inb(DATA);
            if status & 0x20 != 0 {
                return None;
            }
            if status & 0xc0 != 0 {
                self.decoder = Set1::default();
                return None;
            }
            self.decoder.feed(byte)
        }
    }

    pub fn reset_machine(&mut self) -> bool {
        // SAFETY: the i8042 command port belongs to this keyboard instance.
        unsafe { command(0xfe) }
    }
}

unsafe fn write_ready() -> bool {
    (0..WAIT_LIMIT).any(|_| inb(STATUS) & 2 == 0)
}

unsafe fn command(value: u8) -> bool {
    if !write_ready() {
        return false;
    }
    outb(STATUS, value);
    true
}

#[derive(Default)]
struct Set1 {
    extended: bool,
    pause_left: u8,
    shifts: u8,
    controls: u8,
    caps: bool,
    caps_down: bool,
}

impl Set1 {
    fn feed(&mut self, byte: u8) -> Option<Key> {
        if self.pause_left != 0 {
            self.pause_left -= 1;
            return None;
        }
        match byte {
            0xe0 => {
                self.extended = true;
                return None;
            }
            0xe1 => {
                self.extended = false;
                self.pause_left = 5;
                return None;
            }
            _ => {}
        }
        let extended = core::mem::take(&mut self.extended);
        let release = byte & 0x80 != 0;
        let code = byte & 0x7f;
        match (extended, code) {
            (false, 0x2a | 0x36) => {
                set_modifier(&mut self.shifts, if code == 0x2a { 1 } else { 2 }, release);
                return None;
            }
            (_, 0x1d) => {
                set_modifier(&mut self.controls, if extended { 2 } else { 1 }, release);
                return None;
            }
            (false, 0x3a) => {
                if !release && !self.caps_down {
                    self.caps = !self.caps;
                }
                self.caps_down = !release;
                return None;
            }
            _ if release => return None,
            _ => {}
        }

        if extended {
            return Some(match code {
                0x1c => Key::Enter,
                0x47 => Key::Home,
                0x48 => Key::Up,
                0x49 => Key::PageUp,
                0x4b => Key::Left,
                0x4d => Key::Right,
                0x4f => Key::End,
                0x50 => Key::Down,
                0x51 => Key::PageDown,
                0x53 => Key::Delete,
                _ => return None,
            });
        }
        let shifted = self.shifts != 0;
        let pair = |start: usize, lower: &[u8], upper: &[u8]| {
            let index = code as usize - start;
            let byte = lower[index];
            let use_upper = shifted ^ (byte.is_ascii_lowercase() && self.caps);
            if use_upper {
                upper[index]
            } else {
                byte
            }
        };
        let ascii = match code {
            0x01 => return Some(Key::Escape),
            0x0e => return Some(Key::Backspace),
            0x0f => return Some(Key::Tab),
            0x1c => return Some(Key::Enter),
            0x3b..=0x3e => return Some(Key::Function(code - 0x3a)),
            0x02..=0x0d => pair(2, b"1234567890-=", b"!@#$%^&*()_+"),
            0x10..=0x1b => pair(0x10, b"qwertyuiop[]", b"QWERTYUIOP{}"),
            0x1e..=0x29 => pair(0x1e, b"asdfghjkl;'\x60", b"ASDFGHJKL:\"~"),
            0x2b => {
                if shifted {
                    b'|'
                } else {
                    b'\\'
                }
            }
            0x2c..=0x35 => pair(0x2c, b"zxcvbnm,./", b"ZXCVBNM<>?"),
            0x39 => b' ',
            _ => return None,
        };
        if self.controls != 0 {
            return match ascii.to_ascii_lowercase() {
                b'l' => Some(Key::Clear),
                b'p' => Some(Key::Palette),
                b'r' => Some(Key::SearchHistory),
                b'u' | b'c' => Some(Key::Erase),
                _ => None,
            };
        }
        Some(Key::Char(ascii))
    }
}

fn set_modifier(bits: &mut u8, mask: u8, release: bool) {
    if release {
        *bits &= !mask;
    } else {
        *bits |= mask;
    }
}

#[derive(Default)]
pub struct SerialKeys {
    state: u8,
    parameter: u8,
    cr: bool,
}

impl SerialKeys {
    pub fn feed(&mut self, byte: u8) -> Option<Key> {
        let after_cr = core::mem::replace(&mut self.cr, byte == b'\r');
        if byte == 0x1b {
            self.state = 1;
            self.parameter = 0;
            return None;
        }
        match self.state {
            1 if matches!(byte, b'[' | b'O') => {
                self.state = 2;
                return None;
            }
            2 if byte.is_ascii_digit() => {
                self.parameter = self
                    .parameter
                    .saturating_mul(10)
                    .saturating_add(byte - b'0');
                return None;
            }
            2 => {
                self.state = 0;
                return match (byte, self.parameter) {
                    (b'A', 0) => Some(Key::Up),
                    (b'B', 0) => Some(Key::Down),
                    (b'C', 0) => Some(Key::Right),
                    (b'D', 0) => Some(Key::Left),
                    (b'H', 0) | (b'~', 1 | 7) => Some(Key::Home),
                    (b'F', 0) | (b'~', 4 | 8) => Some(Key::End),
                    (b'~', 3) => Some(Key::Delete),
                    (b'~', 5) => Some(Key::PageUp),
                    (b'~', 6) => Some(Key::PageDown),
                    (b'P'..=b'S', 0) => Some(Key::Function(byte - b'P' + 1)),
                    (b'~', 11..=14) => Some(Key::Function(self.parameter - 10)),
                    _ => None,
                };
            }
            _ => self.state = 0,
        }
        match byte {
            b'\r' => Some(Key::Enter),
            b'\n' if !after_cr => Some(Key::Enter),
            0x08 | 0x7f => Some(Key::Backspace),
            b'\t' => Some(Key::Tab),
            0x0c => Some(Key::Clear),
            0x10 => Some(Key::Palette),
            0x12 => Some(Key::SearchHistory),
            0x03 | 0x15 => Some(Key::Erase),
            0x1d => Some(Key::Escape),
            b' '..=b'~' => Some(Key::Char(byte)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_track_each_modifier_and_extended_release() {
        let mut keys = Set1::default();
        for b in [0x2a, 0x36, 0xaa] {
            assert_eq!(keys.feed(b), None);
        }
        assert_eq!(keys.feed(0x10), Some(Key::Char(b'Q')));
        assert_eq!(keys.feed(0xb6), None);
        assert_eq!(keys.feed(0x10), Some(Key::Char(b'q')));
        assert_eq!(keys.feed(0xe0), None);
        assert_eq!(keys.feed(0x48), Some(Key::Up));
        assert_eq!(keys.feed(0xe0), None);
        assert_eq!(keys.feed(0xc8), None);
        for b in [0x1d, 0xe0, 0x1d, 0x9d] {
            assert_eq!(keys.feed(b), None);
        }
        assert_eq!(keys.feed(0x26), Some(Key::Clear));
    }

    #[test]
    fn serial_handles_split_sequences_and_crlf() {
        let mut keys = SerialKeys::default();
        for b in [0x1b, b'[', b'3'] {
            assert_eq!(keys.feed(b), None);
        }
        assert_eq!(keys.feed(b'~'), Some(Key::Delete));
        assert_eq!(keys.feed(b'\r'), Some(Key::Enter));
        assert_eq!(keys.feed(b'\n'), None);
        assert_eq!(keys.feed(0x1d), Some(Key::Escape));
    }
}
