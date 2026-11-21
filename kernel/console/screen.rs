use crate::graphics::{mix, Color, Font, Framebuffer, Ink, Rect, BODY, SMALL};

use super::state::{Change, Console, Theme, Tone, View};

const BACK: Color = Color(12, 13, 17);
const TEXT: Color = Color(229, 229, 237);
const MUTED: Color = Color(151, 151, 171);
const WORDMARK: Color = Color(189, 182, 204);
const GOOD: Color = Color(164, 202, 177);
const ERROR: Color = Color(229, 163, 170);
const FRAME_MS: u64 = 24;
const REVEAL_MS: u64 = 180;
const SETTLE: u32 = 6;

struct Style {
    accent: Color,
    mono: bool,
}

impl Style {
    fn new(theme: Theme) -> Self {
        let accent = match theme {
            Theme::Graphite => Color(193, 181, 237),
            Theme::Ember => Color(239, 177, 148),
            Theme::Mono => TEXT,
        };
        Self {
            accent,
            mono: theme == Theme::Mono,
        }
    }

    fn tone(&self, tone: Tone) -> Color {
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
    column: Rect,
    anchor_x: u32,
    input_y: u32,
    idle: bool,
}

impl Layout {
    fn new(bounds: Rect, idle: bool, lift: u32) -> Self {
        let width = (bounds.w - 64).min(660);
        let column = Rect::new((bounds.w - width) / 2, 0, width, bounds.h);
        let (anchor_x, y) = if idle {
            (bounds.w / 2 - 89, bounds.h * 44 / 100)
        } else {
            (column.x, (bounds.h / 9).max(56))
        };
        Self {
            column,
            anchor_x,
            input_y: y + lift,
            idle,
        }
    }

    fn prompt(&self) -> Rect {
        Rect::new(
            self.anchor_x,
            self.input_y,
            if self.idle {
                self.column.w - (self.anchor_x - self.column.x)
            } else {
                self.column.w
            },
            64,
        )
    }

    fn output(&self, view: View) -> Rect {
        let y = self.input_y
            + match view {
                View::Activity => 69,
                View::Help => 69,
                _ => 0,
            };
        Rect::new(
            self.column.x,
            y,
            self.column.w,
            self.column.h.saturating_sub(y + 46),
        )
    }
}

#[derive(Default)]
struct Damage {
    prompt: Option<Rect>,
    wordmark: Option<Rect>,
    output: Option<Rect>,
}

pub struct Screen {
    damage: Damage,
    idle: bool,
    view: View,
    reveal_start: Option<u64>,
    next_frame: u64,
}

impl Screen {
    pub fn new(frame: &mut Framebuffer, state: &Console, now: Option<u64>) -> Self {
        let mut screen = Self {
            damage: Damage::default(),
            idle: true,
            view: View::Activity,
            reveal_start: now,
            next_frame: now.unwrap_or(0) + FRAME_MS,
        };
        frame.fill(frame.bounds(), BACK);
        screen.paint(frame, state, now, true);
        screen
    }

    pub fn change(
        &mut self,
        frame: &mut Framebuffer,
        state: &Console,
        change: Change,
        now: Option<u64>,
    ) {
        let idle = state.view == View::Activity && state.rows() == 0;
        let reflowed = idle != self.idle || state.view != self.view;
        self.idle = idle;
        self.view = state.view;
        if !state.motion || now.is_none() {
            self.reveal_start = None;
        } else if reflowed {
            self.reveal_start = now;
            self.next_frame = now.unwrap_or(0) + FRAME_MS;
        }
        self.paint(frame, state, now, reflowed || change != Change::Prompt);
    }

    pub fn animating(&self) -> bool {
        self.reveal_start.is_some()
    }

    pub fn tick(&mut self, frame: &mut Framebuffer, state: &Console, now: u64) {
        if self.reveal_start.is_none() || now < self.next_frame {
            return;
        }
        self.next_frame = now + FRAME_MS;
        self.paint(frame, state, Some(now), true);
    }

    fn paint(&mut self, frame: &mut Framebuffer, state: &Console, now: Option<u64>, scene: bool) {
        let (alpha, lift) = self
            .reveal_start
            .zip(now)
            .map_or((255, 0), |(start, now)| reveal(now.saturating_sub(start)));
        let transitioning = self.reveal_start.is_some();
        let scene = scene || transitioning;
        if alpha == 255 && lift == 0 {
            self.reveal_start = None;
        }
        let layout = Layout::new(frame.bounds(), self.idle, lift);
        let style = Style::new(state.theme);
        erase(frame, &mut self.damage.prompt);
        if scene {
            erase(frame, &mut self.damage.wordmark);
            erase(frame, &mut self.damage.output);
            if self.idle {
                let mark = b"syllaslop";
                let width = mark.len() as u32 * SMALL.w;
                let at = Rect::new(
                    (frame.bounds().w - width) / 2,
                    layout.input_y - 44,
                    width,
                    SMALL.h,
                );
                text(frame, at, mark, &SMALL, 1, mix(BACK, WORDMARK, alpha));
                self.damage.wordmark = Some(at);
            } else {
                self.damage.output = output(frame, &layout, state, &style, alpha);
            }
        }
        if matches!(state.view, View::Activity | View::Help) {
            let rect = layout.prompt();
            self.damage.prompt = Some(prompt(frame, rect, state, &style));
        }
    }
}

fn erase(frame: &mut Framebuffer, damage: &mut Option<Rect>) {
    if let Some(rect) = damage.take() {
        frame.fill(rect, BACK);
    }
}

fn reveal(elapsed: u64) -> (u8, u32) {
    let left = REVEAL_MS.saturating_sub(elapsed);
    let remaining = left * left * 255 / (REVEAL_MS * REVEAL_MS);
    (
        (255 - remaining) as u8,
        (u64::from(SETTLE) * remaining).div_ceil(255) as u32,
    )
}

fn text(frame: &mut Framebuffer, at: Rect, bytes: &[u8], font: &Font, scale: u32, fore: Color) {
    frame.text(at, bytes, font, scale, Ink { fore, back: BACK });
}

fn prompt(frame: &mut Framebuffer, at: Rect, state: &Console, style: &Style) -> Rect {
    let (edit, placeholder) = if state.view == View::Help {
        (&state.filter, b"filter commands".as_slice())
    } else {
        (&state.prompt, b"type a command".as_slice())
    };
    let scale = state.zoom;
    let cell = BODY.w * scale;
    let height = BODY.h * scale;
    let field = Rect::new(at.x + cell + 14, at.y, at.w - cell - 14, height);
    let visible = (field.w / cell) as usize;
    let offset = edit.cursor.saturating_sub(visible.saturating_sub(1));
    let bytes = &edit.bytes()[offset..];
    let count = (field.w / cell) as usize;

    text(
        frame,
        Rect::new(at.x, at.y, cell, height),
        b">",
        &BODY,
        scale,
        style.accent,
    );
    if bytes.is_empty() {
        text(
            frame,
            Rect::new(field.x + 7, field.y, field.w - 7, height),
            placeholder,
            &BODY,
            scale,
            MUTED,
        );
    } else {
        text(
            frame,
            field,
            &bytes[..bytes.len().min(count)],
            &BODY,
            scale,
            TEXT,
        );
    }
    let caret = field.x + (edit.cursor - offset) as u32 * cell;
    frame.fill(Rect::new(caret, at.y + 2, 2, height - 4), style.accent);

    let hint: &[u8] = if !state.hint.bytes().is_empty() {
        state.hint.bytes()
    } else if state.view == View::Help {
        b"esc  back     pgup / pgdn  scroll"
    } else {
        b""
    };
    let hint_y = at.y + height + 10;
    text(
        frame,
        Rect::new(field.x, hint_y, field.w, SMALL.h),
        hint,
        &SMALL,
        1,
        MUTED,
    );
    let text_width = if bytes.is_empty() {
        7 + (placeholder.len() as u32 * cell).min(field.w - 7)
    } else {
        bytes.len().min(count) as u32 * cell
    };
    let hint_width = (hint.len() as u32 * SMALL.w).min(field.w);
    let occupied_width = text_width.max(hint_width).max(caret - field.x + 2);
    let occupied_height = if hint.is_empty() {
        height
    } else {
        hint_y + SMALL.h - at.y
    };
    Rect::new(at.x, at.y, field.x - at.x + occupied_width, occupied_height)
}

fn output(
    frame: &mut Framebuffer,
    layout: &Layout,
    state: &Console,
    style: &Style,
    alpha: u8,
) -> Option<Rect> {
    if state.rows() == 0 || alpha == 0 {
        return None;
    }
    let area = layout.output(state.view);
    let scale = state.zoom;
    let row_height = 23 * scale;
    let capacity = (area.h / row_height).max(1) as usize;
    let cols = (area.w / (BODY.w * scale)).max(1) as usize;
    // A 96-byte logical line spans at most four rows at the narrowest
    // supported display and 2x text. The state keeps at most 48 lines.
    let mut rows = [(0u8, 0u8); 192];
    let mut count = 0;
    for index in 0..state.rows() {
        let length = state.row(index).map_or(0, |line| line.bytes().len());
        for offset in (0..length.max(1)).step_by(cols) {
            rows[count] = (index as u8, offset as u8);
            count += 1;
        }
    }
    let first = |logical| {
        rows[..count]
            .iter()
            .position(|&(index, _)| usize::from(index) >= logical)
            .unwrap_or(count)
    };
    let (start, end) = match state.view {
        View::Activity => {
            let end = first(state.rows().saturating_sub(state.scroll));
            (end.saturating_sub(capacity), end)
        }
        View::History => {
            let selected = first(state.selected);
            (
                selected
                    .saturating_sub(capacity / 2)
                    .min(count.saturating_sub(capacity)),
                count,
            )
        }
        _ => (first(state.scroll), count),
    };
    let display = &rows[start..end][..(end - start).min(capacity)];
    let mut bottom = area.y;
    for (line_number, &(index, offset)) in display.iter().enumerate() {
        let line = state.row(usize::from(index)).unwrap();
        let offset = usize::from(offset);
        let bytes = &line.bytes()[offset..(offset + cols).min(line.bytes().len())];
        let selected = state.view == View::History && usize::from(index) == state.selected;
        let y = area.y + line_number as u32 * row_height;
        let fore = if selected {
            style.accent
        } else {
            style.tone(line.tone)
        };
        if selected && offset == 0 {
            text(
                frame,
                Rect::new(area.x - 15, y, SMALL.w, SMALL.h),
                b">",
                &SMALL,
                1,
                mix(BACK, fore, alpha),
            );
        }
        text(
            frame,
            Rect::new(area.x, y, area.w, row_height),
            bytes,
            &BODY,
            scale,
            mix(BACK, fore, alpha),
        );
        bottom = y + row_height;
    }

    let hint = match state.view {
        View::Activity | View::Help => b"".as_slice(),
        View::History => b"up / down  select     enter  recall     esc  back",
        _ => b"esc  back",
    };
    if !hint.is_empty() {
        let y = bottom + 12;
        text(
            frame,
            Rect::new(area.x, y, area.w, SMALL.h),
            hint,
            &SMALL,
            1,
            mix(BACK, MUTED, alpha),
        );
        bottom = y + SMALL.h;
    }
    Some(Rect::new(area.x - 15, area.y, area.w + 15, bottom - area.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_and_active_layouts_fit_both_resolutions_and_zoom() {
        for (w, h) in [(640, 480), (800, 600), (1024, 768), (4096, 2304)] {
            let bounds = Rect::new(0, 0, w, h);
            let idle = Layout::new(bounds, true, SETTLE);
            let work = Layout::new(bounds, false, SETTLE);
            assert_eq!(idle.prompt().intersect(bounds), idle.prompt());
            assert!(work.input_y < idle.input_y);
            assert_eq!(
                work.output(View::Activity).intersect(bounds),
                work.output(View::Activity)
            );
            assert!(work.output(View::History).w / (BODY.w * 2) >= 29);
            assert!(work.prompt().y + work.prompt().h < work.output(View::Help).y);
        }
    }

    #[test]
    fn reveal_starts_softly_and_settles_once() {
        assert_eq!(reveal(0), (0, SETTLE));
        let (alpha, lift) = reveal(REVEAL_MS / 2);
        assert!(alpha > 128 && alpha < 255);
        assert!(lift < SETTLE);
        assert_eq!(reveal(REVEAL_MS), (255, 0));
        assert_eq!(reveal(REVEAL_MS + 1_000), (255, 0));
    }
}
