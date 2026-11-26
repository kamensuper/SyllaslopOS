use core::fmt::{self, Write};

use super::commands::{self, Services};
use crate::input::Key;

const INPUT: usize = 96;
const LOG: usize = 48;
const HISTORY: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tone {
    Text,
    Muted,
    Accent,
    Success,
    Error,
}

#[derive(Clone, Copy)]
pub struct Line {
    buf: [u8; INPUT],
    len: usize,
    pub tone: Tone,
}

impl Line {
    pub const EMPTY: Self = Self {
        buf: [0; INPUT],
        len: 0,
        tone: Tone::Text,
    };

    pub fn format(tone: Tone, args: fmt::Arguments<'_>) -> Self {
        let mut line = Self {
            tone,
            ..Self::EMPTY
        };
        let _ = line.write_fmt(args);
        line
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    pub fn text(&self) -> &str {
        core::str::from_utf8(self.bytes()).unwrap_or("")
    }
}

impl Write for Line {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes().take(INPUT - self.len) {
            self.buf[self.len] = if byte.is_ascii_graphic() || byte == b' ' {
                byte
            } else {
                b'?'
            };
            self.len += 1;
        }
        Ok(())
    }
}

pub struct Lines<const N: usize> {
    entries: [Line; N],
    start: usize,
    pub len: usize,
}

impl<const N: usize> Lines<N> {
    const fn new() -> Self {
        Self {
            entries: [Line::EMPTY; N],
            start: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, line: Line) {
        self.entries[(self.start + self.len) % N] = line;
        if self.len == N {
            self.start = (self.start + 1) % N;
        } else {
            self.len += 1;
        }
    }

    pub fn get(&self, index: usize) -> Option<&Line> {
        (index < self.len).then(|| &self.entries[(self.start + index) % N])
    }

    fn clear(&mut self) {
        self.len = 0;
        self.start = 0;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Edit {
    bytes: [u8; INPUT],
    len: usize,
    pub cursor: usize,
}

impl Edit {
    pub const EMPTY: Self = Self {
        bytes: [0; INPUT],
        len: 0,
        cursor: 0,
    };

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn text(&self) -> &str {
        core::str::from_utf8(self.bytes()).unwrap_or("")
    }

    pub fn replace(&mut self, bytes: &[u8]) {
        self.len = bytes.len().min(INPUT);
        for (out, &byte) in self.bytes[..self.len].iter_mut().zip(bytes) {
            *out = if (b' '..=b'~').contains(&byte) {
                byte
            } else {
                b'?'
            };
        }
        self.cursor = self.len;
    }

    fn apply(&mut self, key: Key) -> bool {
        match key {
            Key::Char(byte @ b' '..=b'~') if self.len < INPUT => {
                self.bytes
                    .copy_within(self.cursor..self.len, self.cursor + 1);
                self.bytes[self.cursor] = byte;
                self.len += 1;
                self.cursor += 1;
            }
            Key::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.delete();
            }
            Key::Delete if self.cursor < self.len => self.delete(),
            Key::Left if self.cursor > 0 => self.cursor -= 1,
            Key::Right if self.cursor < self.len => self.cursor += 1,
            Key::Home => self.cursor = 0,
            Key::End => self.cursor = self.len,
            Key::Erase => *self = Self::EMPTY,
            _ => return false,
        }
        true
    }

    fn delete(&mut self) {
        self.bytes
            .copy_within(self.cursor + 1..self.len, self.cursor);
        self.len -= 1;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum View {
    Activity,
    Help,
    History,
    System,
    Cpu,
    Memory,
    Boot,
    Devices,
    Map,
}

impl View {
    pub fn searchable(self) -> bool {
        matches!(self, Self::Help | Self::History)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Help => "command reference",
            Self::History => "command history",
            Self::System => "system overview",
            Self::Cpu => "processor",
            Self::Memory => "memory map",
            Self::Boot => "boot handoff",
            Self::Devices => "active interfaces",
            Self::Map => "physical memory map",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Theme {
    Graphite,
    Ember,
    Mono,
}

impl Theme {
    pub fn label(self) -> &'static str {
        match self {
            Self::Graphite => "graphite",
            Self::Ember => "ember",
            Self::Mono => "mono",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Change {
    None,
    Prompt,
    Panel,
    Report,
    Full,
    Halt,
    Reboot,
}

pub struct Console {
    pub prompt: Edit,
    pub filter: Edit,
    pub view: View,
    pub theme: Theme,
    pub zoom: u32,
    pub scroll: usize,
    pub selected: usize,
    pub hint: Line,
    log: Lines<LOG>,
    panel: Lines<LOG>,
    history: Lines<HISTORY>,
    recalled: Option<usize>,
    draft: Edit,
    pub fresh: usize,
}

impl Console {
    pub fn new() -> Self {
        Self {
            prompt: Edit::EMPTY,
            filter: Edit::EMPTY,
            view: View::Activity,
            theme: Theme::Graphite,
            zoom: 1,
            scroll: 0,
            selected: 0,
            hint: Line::EMPTY,
            log: Lines::new(),
            panel: Lines::new(),
            history: Lines::new(),
            recalled: None,
            draft: Edit::EMPTY,
            fresh: 0,
        }
    }

    pub fn rows(&self) -> usize {
        if self.view == View::Activity {
            self.log.len
        } else {
            self.panel.len
        }
    }

    pub fn row(&self, index: usize) -> Option<&Line> {
        if self.view == View::Activity {
            self.log.get(index)
        } else {
            self.panel.get(index)
        }
    }

    pub fn log(&mut self, tone: Tone, args: fmt::Arguments<'_>) {
        self.log.push(Line::format(tone, args));
        self.fresh += 1;
    }

    pub fn detail(&mut self, tone: Tone, args: fmt::Arguments<'_>) {
        self.panel.push(Line::format(tone, args));
    }

    pub fn open(&mut self, view: View) {
        self.view = view;
        self.scroll = 0;
        self.selected = 0;
        self.filter = Edit::EMPTY;
        self.panel.clear();
    }

    pub fn clear(&mut self) -> Change {
        self.open(View::Activity);
        self.log.clear();
        self.fresh = 0;
        Change::Report
    }

    pub fn help(&mut self, query: &str) -> Change {
        self.open(View::Help);
        self.filter.replace(query.as_bytes());
        self.refresh_help();
        Change::Report
    }

    fn refresh_help(&mut self) {
        self.panel.clear();
        for priority in 0..4 {
            for command in commands::REGISTRY {
                let query = self.filter.bytes();
                let name = command.name.as_bytes();
                let rank = if name.eq_ignore_ascii_case(query) {
                    0
                } else if name
                    .get(..query.len())
                    .is_some_and(|head| head.eq_ignore_ascii_case(query))
                {
                    1
                } else if subsequence(name, query) {
                    2
                } else if matches_text(command.description.as_bytes(), query) {
                    3
                } else {
                    continue;
                };
                if rank == priority {
                    self.detail(
                        Tone::Text,
                        format_args!("{:<20} {}", command.usage, command.description),
                    );
                }
            }
        }
        self.finish_search("No matching commands.");
    }

    pub fn history(&mut self) -> Change {
        self.open(View::History);
        self.refresh_history();
        Change::Report
    }

    fn refresh_history(&mut self) {
        self.panel.clear();
        for index in (0..self.history.len).rev() {
            let line = *self.history.get(index).unwrap();
            if matches_text(line.bytes(), self.filter.bytes()) {
                self.panel.push(line);
            }
        }
        self.finish_search("No matching history.");
    }

    fn finish_search(&mut self, empty: &str) {
        if self.panel.len == 0 {
            self.detail(Tone::Muted, format_args!("{empty}"));
        }
        self.selected = 0;
    }

    pub fn suggestion(&self) -> Option<&[u8]> {
        let typed = self.prompt.bytes();
        if typed.is_empty() || self.prompt.cursor != typed.len() {
            return None;
        }
        let starts = |candidate: &[u8]| {
            candidate.len() > typed.len() && candidate[..typed.len()].eq_ignore_ascii_case(typed)
        };
        (0..self.history.len)
            .rev()
            .filter_map(|index| self.history.get(index).map(Line::bytes))
            .chain(
                commands::REGISTRY
                    .iter()
                    .map(|command| command.name.as_bytes()),
            )
            .find(|candidate| starts(candidate))
            .map(|candidate| &candidate[typed.len()..])
    }

    fn recall(&mut self, older: bool) {
        let next = match (older, self.recalled) {
            (true, Some(index)) => index.checked_sub(1).or(Some(0)),
            (true, None) => {
                self.draft = self.prompt;
                self.history.len.checked_sub(1)
            }
            (false, Some(index)) if index + 1 < self.history.len => Some(index + 1),
            (false, Some(_)) => {
                self.prompt = self.draft;
                None
            }
            (false, None) => None,
        };
        self.recalled = next;
        if let Some(line) = next.and_then(|index| self.history.get(index)) {
            self.prompt.replace(line.bytes());
        }
    }

    fn submit(&mut self, services: &Services) -> Change {
        let input = core::mem::replace(&mut self.prompt, Edit::EMPTY);
        let text = input.text().trim_ascii();
        self.recalled = None;
        self.scroll = 0;
        if text.is_empty() {
            return Change::Prompt;
        }
        if self
            .history
            .len
            .checked_sub(1)
            .and_then(|i| self.history.get(i))
            .map(Line::text)
            != Some(text)
        {
            self.history
                .push(Line::format(Tone::Text, format_args!("{text}")));
        }
        self.log(Tone::Accent, format_args!("> {text}"));
        commands::run(self, services, text)
    }

    fn complete(&mut self) {
        let mut edit = self.prompt;
        if edit.cursor != edit.len {
            return;
        }
        let mut words = edit.text().splitn(2, ' ');
        let first = words.next().unwrap_or("");
        let rest = words.next();
        let mut names: [&str; 24] = [""; 24];
        let mut count = 0;
        commands::complete(first, rest, |match_name| {
            if count < names.len() {
                names[count] = match_name;
                count += 1;
            }
        });
        if count == 0 {
            self.hint = Line::format(Tone::Muted, format_args!("No completion."));
            return;
        }
        let command_name = rest.is_none();
        let begin = rest.map_or(0, |_| first.len() + 1);
        let common = names[..count].iter().fold(names[0].len(), |length, name| {
            names[0]
                .bytes()
                .zip(name.bytes())
                .take(length)
                .take_while(|(a, b)| a == b)
                .count()
        });
        edit.cursor = begin + common.min(INPUT - begin);
        edit.len = edit.cursor;
        edit.bytes[begin..edit.len].copy_from_slice(&names[0].as_bytes()[..edit.len - begin]);
        if command_name && count == 1 && edit.len < INPUT {
            edit.apply(Key::Char(b' '));
        }
        self.prompt = edit;
        if count > 1 {
            let mut hint = Line::EMPTY;
            hint.tone = Tone::Accent;
            for (index, name) in names[..count].iter().enumerate() {
                if index != 0 {
                    let _ = hint.write_str("  /  ");
                }
                let _ = hint.write_str(name);
            }
            self.hint = hint;
        }
    }

    pub fn event(&mut self, key: Key, services: &Services) -> Change {
        self.fresh = 0;
        self.hint = Line::EMPTY;
        match key {
            Key::Palette => return self.help(""),
            Key::SearchHistory => return self.history(),
            _ => {}
        }
        if let Key::Function(number) = key {
            return match number {
                1 => self.help(""),
                2 => commands::run(self, services, "sysinfo"),
                3 => commands::run(self, services, "cpu"),
                4 => commands::run(self, services, "memory"),
                _ => Change::None,
            };
        }
        if key == Key::Clear {
            return self.clear();
        }
        if self.view != View::Activity {
            return self.view_event(key);
        }
        match key {
            Key::Enter => self.submit(services),
            Key::Right if self.suggestion().is_some() => {
                let mut edit = self.prompt;
                for &byte in self.suggestion().unwrap() {
                    edit.apply(Key::Char(byte));
                }
                self.prompt = edit;
                self.recalled = None;
                Change::Prompt
            }
            Key::Tab => {
                self.complete();
                Change::Prompt
            }
            Key::Up | Key::Down => {
                self.recall(key == Key::Up);
                Change::Prompt
            }
            Key::PageUp | Key::PageDown => {
                self.scroll = scroll(self.scroll, self.rows(), key == Key::PageUp, 5);
                Change::Panel
            }
            Key::Escape => {
                self.prompt = Edit::EMPTY;
                self.recalled = None;
                Change::Prompt
            }
            key => {
                if self.prompt.apply(key) {
                    Change::Prompt
                } else {
                    Change::None
                }
            }
        }
    }

    fn view_event(&mut self, key: Key) -> Change {
        if key == Key::Escape {
            self.open(View::Activity);
            return Change::Panel;
        }
        if self.view.searchable() {
            match key {
                Key::Enter => {
                    let Some(line) = self.panel.get(self.selected) else {
                        return Change::None;
                    };
                    if line.tone == Tone::Muted {
                        return Change::None;
                    }
                    let bytes = if self.view == View::Help {
                        line.text()
                            .split_ascii_whitespace()
                            .next()
                            .unwrap_or("")
                            .as_bytes()
                    } else {
                        line.bytes()
                    };
                    self.prompt.replace(bytes);
                    if self.view == View::Help {
                        self.prompt.apply(Key::Char(b' '));
                    }
                    self.recalled = None;
                    self.open(View::Activity);
                    return Change::Panel;
                }
                Key::Up | Key::Down | Key::PageUp | Key::PageDown => {
                    self.selected = scroll(
                        self.selected,
                        self.rows(),
                        matches!(key, Key::Down | Key::PageDown),
                        if matches!(key, Key::PageUp | Key::PageDown) {
                            5
                        } else {
                            1
                        },
                    );
                    return Change::Panel;
                }
                key if self.filter.apply(key) => {
                    if self.view == View::Help {
                        self.refresh_help();
                    } else {
                        self.refresh_history();
                    }
                    return Change::Panel;
                }
                _ => return Change::None,
            }
        }
        match key {
            Key::Up | Key::Down | Key::PageUp | Key::PageDown => {
                self.scroll = scroll(
                    self.scroll,
                    self.rows(),
                    matches!(key, Key::Down | Key::PageDown),
                    if matches!(key, Key::PageUp | Key::PageDown) {
                        5
                    } else {
                        1
                    },
                );
                Change::Panel
            }
            _ => Change::None,
        }
    }
}

fn scroll(offset: usize, len: usize, forward: bool, step: usize) -> usize {
    if forward {
        (offset + step).min(len.saturating_sub(1))
    } else {
        offset.saturating_sub(step)
    }
}

fn matches_text(text: &[u8], query: &[u8]) -> bool {
    query.is_empty()
        || text
            .windows(query.len())
            .any(|part| part.eq_ignore_ascii_case(query))
}

fn subsequence(text: &[u8], query: &[u8]) -> bool {
    let mut chars = text.iter();
    query
        .iter()
        .all(|needle| chars.by_ref().any(|ch| ch.eq_ignore_ascii_case(needle)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_respects_limits_and_insertion_point() {
        let mut edit = Edit::EMPTY;
        for _ in 0..INPUT {
            assert!(edit.apply(Key::Char(b'a')));
        }
        assert!(!edit.apply(Key::Char(b'b')));
        edit.apply(Key::Home);
        edit.apply(Key::Delete);
        edit.apply(Key::Char(b'b'));
        assert_eq!(edit.bytes()[0], b'b');
        assert_eq!(edit.bytes().len(), INPUT);
        edit.apply(Key::Backspace);
        assert_eq!(edit.bytes()[0], b'a');
    }

    #[test]
    fn history_preserves_the_draft() {
        let mut model = Console::new();
        let system = Services::default();
        model.prompt.replace(b"echo first");
        assert_eq!(model.event(Key::Enter, &system), Change::Report);
        model.prompt.replace(b"unfinished");
        model.event(Key::Up, &system);
        assert_eq!(model.prompt.bytes(), b"echo first");
        model.event(Key::Down, &system);
        assert_eq!(model.prompt.bytes(), b"unfinished");
    }

    #[test]
    fn deterministic_completion_and_empty_input() {
        let system = Services::default();
        let mut model = Console::new();
        assert_eq!(model.event(Key::Enter, &system), Change::Prompt);
        assert_eq!(model.rows(), 0);
        model.prompt.replace(b"sys");
        model.event(Key::Tab, &system);
        assert_eq!(model.prompt.bytes(), b"sysinfo ");
        model.prompt.replace(b"theme e");
        model.event(Key::Tab, &system);
        assert_eq!(model.prompt.bytes(), b"theme ember");
    }

    #[test]
    fn palette_stages_confirmation_commands_without_running_them() {
        let mut state = Console::new();
        let services = Services::default();
        state.help("hlt");
        assert!(state.row(0).unwrap().text().starts_with("halt "));
        assert_eq!(state.event(Key::Enter, &services), Change::Panel);
        assert_eq!(state.prompt.bytes(), b"halt ");
        assert_eq!(state.rows(), 0);
        assert_eq!(state.event(Key::Enter, &services), Change::Report);
    }

    #[test]
    fn palette_ranks_names_before_description_matches() {
        let mut state = Console::new();
        state.help("memory");
        assert!(state.row(0).unwrap().text().starts_with("memory "));
    }

    #[test]
    fn log_overwrites_the_oldest_entry() {
        let mut log = Lines::<2>::new();
        for n in 1..4 {
            log.push(Line::format(Tone::Text, format_args!("{n}")));
        }
        assert_eq!(log.get(0).unwrap().text(), "2");
        assert_eq!(log.get(1).unwrap().text(), "3");
        assert!(log.get(2).is_none());
    }
}
