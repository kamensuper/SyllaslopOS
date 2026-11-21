use crate::graphics::{Color, Font, Framebuffer, Ink, Rect, BODY, SMALL};

use super::state::{Change, Console, Theme, Tone, View};

const BACK: Color = Color(12, 13, 17);
const TEXT: Color = Color(229, 229, 237);
const MUTED: Color = Color(151, 151, 171);
const FIELD: Color = Color(23, 23, 31);
const GHOST: Color = Color(131, 129, 151);
const GOOD: Color = Color(164, 202, 177);
const ERROR: Color = Color(229, 163, 170);
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
    input: Rect,
    input_y: u32,
}

impl Layout {
    fn new(bounds: Rect, idle: bool) -> Self {
        let width = (bounds.w - 64).min(660);
        let column = Rect::new((bounds.w - width) / 2, 0, width, bounds.h);
        let input_y = if idle {
            bounds.h * 45 / 100
        } else {
            (bounds.h / 9).max(56)
        };
        let prompt_width = if idle { 448 } else { width };
        let input = Rect::new((bounds.w - prompt_width) / 2, input_y, prompt_width, 84);
        Self {
            column,
            input,
            input_y,
        }
    }

    fn output(&self, view: View) -> Rect {
        let y = self.input_y
            + match view {
                View::Activity | View::Help | View::History => 90,
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
}

impl Screen {
    pub fn new(frame: &mut Framebuffer, state: &Console) -> Self {
        let mut screen = Self {
            damage: Damage::default(),
            idle: true,
            view: View::Activity,
        };
        frame.fill(frame.bounds(), BACK);
        screen.paint(frame, state, true);
        screen
    }

    pub fn change(&mut self, frame: &mut Framebuffer, state: &Console, change: Change) {
        let idle = state.view == View::Activity && state.rows() == 0;
        let reflowed = idle != self.idle || state.view != self.view;
        self.idle = idle;
        self.view = state.view;
        self.paint(frame, state, reflowed || change != Change::Prompt);
    }

    fn paint(&mut self, frame: &mut Framebuffer, state: &Console, scene: bool) {
        let layout = Layout::new(frame.bounds(), self.idle);
        let style = Style::new(state.theme);
        erase(frame, &mut self.damage.prompt);
        if scene {
            erase(frame, &mut self.damage.wordmark);
            erase(frame, &mut self.damage.output);
            if self.idle {
                let at = Rect::new(layout.input.x + 16, layout.input_y - 60, 162, 36);
                text(frame, at, b"sylla", &BODY, 2, TEXT);
                text(
                    frame,
                    Rect::new(at.x + 90, at.y, 72, at.h),
                    b"slop",
                    &BODY,
                    2,
                    style.accent,
                );
                self.damage.wordmark = Some(at);
            } else {
                self.damage.output = output(frame, &layout, state, &style);
            }
        }
        if state.view == View::Activity || state.view.searchable() {
            prompt(frame, layout.input, state, &style, self.idle);
            self.damage.prompt = Some(layout.input);
        }
    }
}

fn erase(frame: &mut Framebuffer, damage: &mut Option<Rect>) {
    if let Some(rect) = damage.take() {
        frame.fill(rect, BACK);
    }
}

fn text(frame: &mut Framebuffer, at: Rect, bytes: &[u8], font: &Font, scale: u32, fore: Color) {
    frame.text(at, bytes, font, scale, Ink { fore, back: BACK });
}

fn prompt(frame: &mut Framebuffer, at: Rect, state: &Console, style: &Style, idle: bool) {
    let (edit, placeholder, symbol) = match state.view {
        View::Help => (&state.filter, b"find a command".as_slice(), b"/"),
        View::History => (&state.filter, b"find in history".as_slice(), b"/"),
        _ => (&state.prompt, b"type a command".as_slice(), b">"),
    };
    let scale = state.zoom;
    let cell = BODY.w * scale;
    let height = BODY.h * scale;
    let surface = Rect::new(at.x, at.y, at.w, height + 24);
    frame.fill(surface, FIELD);
    let y = at.y + 12;
    let field = Rect::new(at.x + cell + 29, y, at.w - cell - 45, height);
    let count = (field.w / cell) as usize;
    let offset = edit.cursor.saturating_sub(count.saturating_sub(1));
    let typed = &edit.bytes()[offset..];
    let cursor = field.x + (edit.cursor - offset) as u32 * cell;
    let draw = |frame: &mut Framebuffer, rect: Rect, bytes: &[u8], fore| {
        frame.text(rect, bytes, &BODY, scale, Ink { fore, back: FIELD });
    };
    draw(
        frame,
        Rect::new(at.x + 15, y, cell, height),
        symbol,
        style.accent,
    );
    if typed.is_empty() {
        draw(
            frame,
            Rect::new(field.x + 6, y, field.w - 6, height),
            placeholder,
            MUTED,
        );
    } else {
        let end = typed
            .iter()
            .position(|&b| b == b' ')
            .unwrap_or(typed.len())
            .min(count);
        if state.view == View::Activity && offset == 0 {
            draw(frame, field, &typed[..end], style.accent);
            let prefix = end as u32 * cell;
            draw(
                frame,
                Rect::new(field.x + prefix, y, field.w - prefix, height),
                &typed[end..],
                TEXT,
            );
        } else {
            draw(frame, field, typed, TEXT);
        }
    }
    let suggestion = if state.view == View::Activity {
        state.suggestion()
    } else {
        None
    };
    if let Some(suffix) = suggestion {
        let shift = cursor - field.x;
        draw(
            frame,
            Rect::new(cursor, y, field.w - shift, height),
            suffix,
            GHOST,
        );
    }
    frame.fill(Rect::new(cursor, y + 2, 1, height - 4), style.accent);

    let hint: &[u8] = if !state.hint.bytes().is_empty() {
        state.hint.bytes()
    } else if state.view.searchable() {
        b"up/down select   enter stage   esc back"
    } else if suggestion.is_some() {
        b"right accept suggestion"
    } else if idle {
        b"ctrl+p commands    ctrl+r history"
    } else {
        b""
    };
    text(
        frame,
        Rect::new(at.x + 15, surface.y + surface.h + 7, at.w - 30, SMALL.h),
        hint,
        &SMALL,
        1,
        MUTED,
    );
}

fn output(
    frame: &mut Framebuffer,
    layout: &Layout,
    state: &Console,
    style: &Style,
) -> Option<Rect> {
    if state.rows() == 0 {
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
        View::History | View::Help => {
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
        let selected = state.view.searchable() && usize::from(index) == state.selected;
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
                fore,
            );
        }
        if state.view == View::Help {
            let command = 20usize.saturating_sub(offset).min(bytes.len());
            text(
                frame,
                Rect::new(area.x, y, area.w, row_height),
                &bytes[..command],
                &BODY,
                scale,
                fore,
            );
            let indent = command as u32 * BODY.w * scale;
            text(
                frame,
                Rect::new(area.x + indent, y, area.w - indent, row_height),
                &bytes[command..],
                &BODY,
                scale,
                MUTED,
            );
        } else {
            text(
                frame,
                Rect::new(area.x, y, area.w, row_height),
                bytes,
                &BODY,
                scale,
                fore,
            );
        }
        bottom = y + row_height;
    }

    let hint = match state.view {
        View::Activity | View::Help | View::History => b"".as_slice(),
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
            MUTED,
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
            let idle = Layout::new(bounds, true);
            let work = Layout::new(bounds, false);
            assert_eq!(idle.input.intersect(bounds), idle.input);
            assert!(work.input_y < idle.input_y);
            assert_eq!(
                work.output(View::Activity).intersect(bounds),
                work.output(View::Activity)
            );
            assert!(work.output(View::History).w / (BODY.w * 2) >= 29);
            assert!(work.input.y + work.input.h < work.output(View::Help).y);
        }
    }
}
