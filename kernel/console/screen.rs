use crate::graphics::{mix, Color, Font, Framebuffer, Ink, Rect, BODY, SMALL, WORDMARK};

use super::commands::Services;
use super::state::{Change, Console, Theme, Tone, View};

const BACK: Color = Color(12, 13, 16);
const PANEL: Color = Color(20, 22, 28);
const PANEL_TOP: Color = Color(25, 27, 34);
const LINE: Color = Color(43, 47, 58);
const TEXT: Color = Color(232, 234, 240);
const MUTED: Color = Color(158, 165, 184);
const GOOD: Color = Color(143, 197, 167);
const ERROR: Color = Color(225, 141, 148);

#[derive(Clone, Copy)]
struct Style {
    accent: Color,
    warm: Color,
    hero: Color,
    prompt: Color,
    mono: bool,
}

impl Style {
    fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::Graphite => Self {
                accent: Color(189, 175, 240),
                warm: Color(228, 154, 116),
                hero: Color(23, 24, 34),
                prompt: Color(30, 27, 43),
                mono: false,
            },
            Theme::Ember => Self {
                accent: Color(242, 176, 137),
                warm: Color(231, 193, 148),
                hero: Color(31, 24, 23),
                prompt: Color(41, 29, 27),
                mono: false,
            },
            Theme::Mono => Self {
                accent: Color(249, 249, 249),
                warm: Color(206, 206, 206),
                hero: Color(20, 21, 24),
                prompt: Color(27, 28, 32),
                mono: true,
            },
        }
    }

    fn tone(self, tone: Tone) -> Color {
        match tone {
            Tone::Text => TEXT,
            Tone::Muted => MUTED,
            Tone::Accent => self.accent,
            Tone::Success if !self.mono => GOOD,
            Tone::Error if !self.mono => ERROR,
            _ => TEXT,
        }
    }
}

struct Layout {
    head: Rect,
    hero: Rect,
    panel: Rect,
    prompt: Rect,
    hints: Rect,
}

impl Layout {
    fn new(bounds: Rect) -> Self {
        let width = (bounds.w - 64).min(1160);
        let x = (bounds.w - width) / 2;
        let compact = bounds.h < 650;
        let top = if compact { 18 } else { 28 };
        let header = Rect::new(x, top, width, 40);
        let hero = Rect::new(x, top + 52, width, if compact { 124 } else { 180 });
        let prompt = Rect::new(x, bounds.h - 105, width, 62);
        let panel_y = hero.y + hero.h + 16;
        Self {
            head: header,
            hero,
            panel: Rect::new(x, panel_y, width, prompt.y - panel_y - 14),
            prompt,
            hints: Rect::new(x, bounds.h - 32, width, 22),
        }
    }
}

pub fn draw(fb: &mut Framebuffer, console: &Console, services: &Services, change: Change) {
    let layout = Layout::new(fb.bounds());
    let style = Style::for_theme(console.theme);
    if change == Change::Full {
        fb.fill(fb.bounds(), BACK);
        header(fb, layout.head, services, style);
        hero(fb, layout.hero, style);
    }
    if !matches!(change, Change::None | Change::Prompt) {
        workspace(fb, layout.panel, console, style);
    }
    if change != Change::None {
        prompt(fb, layout.prompt, layout.hints, console, style);
    }
}

fn words(
    fb: &mut Framebuffer,
    rect: Rect,
    text: &[u8],
    font: &Font,
    scale: u32,
    fore: Color,
    back: Color,
) {
    fb.text(rect, text, font, scale, Ink { fore, back });
}

fn header(fb: &mut Framebuffer, r: Rect, services: &Services, style: Style) {
    let mark = [
        0b01110u8, 0b11001, 0b11000, 0b01110, 0b00011, 0b10011, 0b01110,
    ];
    fb.fill(Rect::new(r.x, r.y, 36, 36), PANEL);
    for (row, mask) in mark.into_iter().enumerate() {
        for col in 0..5 {
            if mask & (1 << (4 - col)) != 0 {
                fb.fill(
                    Rect::new(r.x + 9 + col * 4, r.y + 5 + row as u32 * 4, 2, 2),
                    style.accent,
                );
            }
        }
    }
    words(
        fb,
        Rect::new(r.x + 48, r.y + 1, 224, 18),
        b"SYLLASLOPOS",
        &BODY,
        1,
        TEXT,
        BACK,
    );
    words(
        fb,
        Rect::new(r.x + 48, r.y + 20, 224, 16),
        b"NATIVE CONSOLE",
        &SMALL,
        1,
        MUTED,
        BACK,
    );
    let badge = if services.keyboard {
        b"PS/2 + COM1".as_slice()
    } else {
        b"COM1 INPUT".as_slice()
    };
    let width = (badge.len() as u32 + 4) * SMALL.w;
    let pill = Rect::new(r.x + r.w - width, r.y + 6, width, 25);
    fb.fill(pill, PANEL_TOP);
    let pad = Rect::new(pill.x + 16, pill.y + 5, pill.w - 16, 16);
    words(fb, pad, badge, &SMALL, 1, style.accent, PANEL_TOP);
}

fn hero(fb: &mut Framebuffer, r: Rect, style: Style) {
    fb.fill(r, style.hero);
    let right = Rect::new(r.x + r.w / 2, r.y + 1, r.w / 2 - 1, r.h - 1);
    material(fb, right, style);
    fb.fill(Rect::new(r.x, r.y, r.w, 1), LINE);
    fb.fill(Rect::new(r.x, r.y + r.h - 1, r.w, 1), LINE);
    let x = r.x + 24;
    let y = r.y + if r.h < 150 { 9 } else { 23 };
    words(
        fb,
        Rect::new(x + 2, y, 360, 16),
        b"SLC / 0.1.0    x86_64",
        &SMALL,
        1,
        style.warm,
        style.hero,
    );
    let ink = Ink {
        fore: TEXT,
        back: style.hero,
    };
    fb.mask((x, y + 12), (305, 68), WORDMARK, 1, ink, r);
    words(
        fb,
        Rect::new(x + 2, y + 80, 330, 18),
        b"System command environment",
        &BODY,
        1,
        MUTED,
        style.hero,
    );
    if r.h >= 150 {
        let labels = [
            (b"F1", b" Help".as_slice()),
            (b"F2", b" System"),
            (b"F3", b" CPU"),
            (b"F4", b" Memory"),
        ];
        let mut left = x + 2;
        for (key, label) in labels {
            let width = (2 + label.len()) as u32 * SMALL.w;
            words(
                fb,
                Rect::new(left, y + 113, 16, 16),
                key,
                &SMALL,
                1,
                style.accent,
                style.hero,
            );
            words(
                fb,
                Rect::new(left + 16, y + 113, width - 16, 16),
                label,
                &SMALL,
                1,
                MUTED,
                style.hero,
            );
            left += width + 22;
        }
    }
}

/// One static 5 px lattice, lit from the upper left. Drawing is limited to
/// the hero's right half and occurs only on a full redraw.
fn material(fb: &mut Framebuffer, r: Rect, style: Style) {
    let radius = (r.h * 2 / 3) as i32;
    let center_x = (r.x + r.w * 3 / 5) as i32;
    let center_y = (r.y + r.h * 3 / 5) as i32;
    let rr = (radius * radius).max(1);
    let start_x = (center_x - radius).max(r.x as i32) as u32;
    let end_x = (center_x + radius).min((r.x + r.w) as i32) as u32;
    for y in (r.y..r.y + r.h).step_by(5) {
        for x in (start_x..end_x).step_by(5) {
            let dx = x as i32 - center_x;
            let dy = y as i32 - center_y;
            let d2 = dx * dx + dy * dy;
            if d2 >= rr {
                continue;
            }
            let light_x = dx + radius * 2 / 5;
            let light_y = dy + radius / 2;
            let light = (light_x * light_x + light_y * light_y) * 123 / rr;
            let edge = (rr - d2) * 84 / rr;
            let shade = (136 + edge - light).clamp(12, 205) as u8;
            let fore = mix(Color(64, 65, 80), style.accent, shade / 2);
            let side = if shade > 145 { 2 } else { 1 };
            fb.fill(
                Rect::new(x, y, side, side).intersect(r),
                mix(style.hero, fore, shade),
            );
        }
    }
}

fn workspace(fb: &mut Framebuffer, r: Rect, console: &Console, style: Style) {
    let inner = r.inset(1);
    fb.fill(r, LINE);
    fb.fill(inner, PANEL);
    let bar = Rect::new(inner.x, inner.y, inner.w, 36);
    fb.fill(bar, PANEL_TOP);
    fb.fill(Rect::new(bar.x, bar.y + bar.h, bar.w, 1), LINE);
    let title = Rect::new(r.x + 19, r.y + 10, r.w - 38, 18);
    words(
        fb,
        title,
        console.view.label().as_bytes(),
        &BODY,
        1,
        TEXT,
        PANEL_TOP,
    );
    let badge = if console.view == View::Activity {
        b"SESSION".as_slice()
    } else {
        b"ESC  BACK".as_slice()
    };
    let badge_width = badge.len() as u32 * SMALL.w;
    let status = Rect::new(r.x + r.w - 19 - badge_width, r.y + 11, badge_width, 16);
    words(fb, status, badge, &SMALL, 1, style.accent, PANEL_TOP);
    let area = Rect::new(r.x + 19, r.y + 49, r.w - 38, r.h - 59);
    let height = 23 * console.zoom;
    let visible = (area.h / height).max(1) as usize;
    let cols = (area.w / (BODY.w * console.zoom)).max(1) as usize;

    // A row records the start byte of one visible fragment of a logical line.
    // The largest accepted input is 96 bytes and a compact viewport still fits
    // over 30 characters at 2x. Four fragments cover any accepted line.
    let mut rows = [(0u8, 0u8); 192];
    let mut count = 0;
    for index in 0..console.rows() {
        let len = console.row(index).map_or(0, |line| line.bytes().len());
        for offset in (0..len.max(1)).step_by(cols) {
            rows[count] = (index as u8, offset as u8);
            count += 1;
        }
    }
    if count == 0 {
        words(
            fb,
            area,
            b"Activity cleared",
            &BODY,
            console.zoom,
            MUTED,
            PANEL,
        );
        return;
    }
    let first_fragment = |logical| {
        rows[..count]
            .iter()
            .position(|&(i, _)| usize::from(i) >= logical)
            .unwrap_or(count)
    };
    let end = if console.view == View::Activity {
        first_fragment(console.rows() - console.scroll)
    } else {
        count
    };
    let start = match console.view {
        View::Activity => end.saturating_sub(visible),
        View::History => first_fragment(console.selected).saturating_sub(visible / 2),
        _ => first_fragment(console.scroll),
    };
    for (n, &(idx, offset)) in rows[start..end].iter().take(visible).enumerate() {
        let line = console.row(idx as usize).unwrap();
        let offset = offset as usize;
        let bytes = &line.bytes()[offset..(offset + cols).min(line.bytes().len())];
        let y = area.y + n as u32 * height;
        let selected = console.view == View::History && usize::from(idx) == console.selected;
        let back = if selected { style.prompt } else { PANEL };
        if selected {
            fb.fill(Rect::new(area.x - 8, y - 2, area.w + 8, height - 1), back);
            fb.fill(Rect::new(area.x - 8, y - 2, 2, height - 1), style.accent);
        }
        let row = Rect::new(area.x, y, area.w, height);
        words(
            fb,
            row,
            bytes,
            &BODY,
            console.zoom,
            style.tone(line.tone),
            back,
        );
    }
}

fn prompt(fb: &mut Framebuffer, r: Rect, hints: Rect, console: &Console, style: Style) {
    let editing = console.view == View::Activity || console.view == View::Help;
    let back = if editing { style.prompt } else { PANEL };
    fb.fill(r, back);
    fb.fill(Rect::new(r.x, r.y, r.w, 1), LINE);
    fb.fill(Rect::new(r.x, r.y + r.h - 1, r.w, 1), LINE);
    fb.fill(
        Rect::new(r.x, r.y + 1, 2, r.h - 2),
        if editing { style.accent } else { LINE },
    );
    let tag = if console.view == View::Help {
        b"FIND".as_slice()
    } else {
        b"SLC".as_slice()
    };
    let y = r.y + (r.h - BODY.h * console.zoom) / 2;
    words(
        fb,
        Rect::new(r.x + 19, r.y + 23, 42, 16),
        tag,
        &SMALL,
        1,
        style.accent,
        back,
    );
    fb.fill(Rect::new(r.x + 65, r.y + 16, 1, r.h - 32), LINE);
    let area = Rect::new(r.x + 82, y, r.w - 102, BODY.h * console.zoom);
    let (entry, placeholder) = match console.view {
        View::Activity => (&console.prompt, b"Type a command".as_slice()),
        View::Help => (&console.filter, b"Filter by name or description".as_slice()),
        View::History => (
            &console.prompt,
            b"Up / Down to select, Enter to recall".as_slice(),
        ),
        _ => (
            &console.prompt,
            b"Esc to return to the command prompt".as_slice(),
        ),
    };
    let cell = BODY.w * console.zoom;
    let cells = (area.w / cell) as usize;
    let offset = if editing {
        entry.cursor.saturating_sub(cells.saturating_sub(1))
    } else {
        0
    };
    let data = if editing {
        &entry.bytes()[offset..]
    } else {
        b""
    };
    let show_hint = data.is_empty();
    let start = if show_hint && editing { cell + 3 } else { 0 };
    let pos = Rect::new(area.x + start, area.y, area.w - start, area.h);
    words(
        fb,
        pos,
        if show_hint { placeholder } else { data },
        &BODY,
        console.zoom,
        if show_hint { MUTED } else { TEXT },
        back,
    );

    if editing {
        let cursor = area.x + (entry.cursor - offset) as u32 * cell;
        fb.fill(Rect::new(cursor, area.y + 1, 2, area.h - 2), style.accent);
    }

    fb.fill(hints, BACK);
    let tips = match console.view {
        View::Activity => {
            b"Tab complete    Up/Down history    Ctrl+L clear    F1 help    PgUp/PgDn scroll"
                .as_slice()
        }
        View::Help => b"Type to filter    PgUp/PgDn scroll    Esc back",
        View::History => b"Up/Down select    Enter recall    Esc back",
        _ => b"PgUp/PgDn scroll    Esc back",
    };
    let custom = console.hint.bytes();
    words(
        fb,
        hints,
        if custom.is_empty() { tips } else { custom },
        &SMALL,
        1,
        if custom.is_empty() {
            MUTED
        } else {
            style.accent
        },
        BACK,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layouts_fit_compact_and_extended_modes() {
        for (width, height) in [
            (640, 480),
            (800, 600),
            (1024, 768),
            (1920, 1080),
            (4096, 2304),
        ] {
            let bounds = Rect::new(0, 0, width, height);
            let layout = Layout::new(bounds);
            let blocks = [
                layout.head,
                layout.hero,
                layout.panel,
                layout.prompt,
                layout.hints,
            ];
            for area in blocks {
                assert_eq!(area.intersect(bounds), area);
            }
            assert!(blocks.windows(2).all(|w| w[0].y + w[0].h < w[1].y));
            assert!((layout.panel.w - 38) / (BODY.w * 2) >= 29);
        }
    }
}
