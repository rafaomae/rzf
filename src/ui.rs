use std::io::{self, Stdout, Write, stdout};

use crossterm::{
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    *,
};

pub struct State {
    candidates: Vec<String>,
    query: String,
    max_quantity: usize,
    results: Vec<(i32, String)>,
    selected: usize,
    scroll_offset: usize,
}

impl State {
    pub fn new(candidates: Vec<String>) -> Self {
        let max_quantity = candidates.len();
        let mut s = Self {
            candidates,
            max_quantity,
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            scroll_offset: 0,
        };

        s.rerank();
        s
    }

    fn move_up(&mut self) {
        let max = self.results.len().saturating_sub(1);
        self.selected = (self.selected + 1).min(max);
    }

    fn move_down(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn push_char(&mut self, c: char) {
        self.query.push(c);
        self.rerank();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    fn pop_char(&mut self) {
        self.query.pop();
        self.rerank();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    fn rerank(&mut self) {
        self.results = self
            .candidates
            .iter()
            .filter_map(|c| crate::algo::score(&self.query, c).map(|s| (s, c.clone())))
            .collect();
        self.results.sort_by_key(|b| std::cmp::Reverse(b.0));
    }

    fn keep_selected_visible(&mut self, visible_count: usize) {
        if visible_count == 0 || self.results.is_empty() {
            self.selected = 0;
            self.scroll_offset = 0;
            return;
        }

        self.selected = self.selected.min(self.results.len() - 1);

        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_count {
            self.scroll_offset = self.selected + 1 - visible_count;
        }

        let max_offset = self.results.len().saturating_sub(visible_count);
        self.scroll_offset = self.scroll_offset.min(max_offset);
    }
}

pub fn run(state: &mut State) -> io::Result<Option<String>> {
    let mut out = stdout();
    terminal::enable_raw_mode()?;
    execute!(out, EnterAlternateScreen)?;

    let layout = Layout::new()?;
    let mut selected_path = None;

    loop {
        render(state, &layout, &mut out)?;

        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                continue;
            }

            match (key_event.code, key_event.modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,

                (KeyCode::Backspace, _) => {
                    state.pop_char();
                }
                (KeyCode::Up, _) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                    state.move_up();
                    state.keep_selected_visible(layout.max_result);
                }
                (KeyCode::Down, _) | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                    state.move_down();
                    state.keep_selected_visible(layout.max_result);
                }
                (KeyCode::Char(c), _) => {
                    state.push_char(c);
                }
                (KeyCode::Enter, _) => {
                    selected_path = state.results.get(state.selected).map(|r| r.1.clone());
                    break;
                }
                _ => {}
            }
        }
    }

    execute!(out, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(selected_path)
}


struct Layout {
    width: u16,
    prompt_row: u16,
    size_row: u16,
    max_result: usize
}

impl Layout {
    pub fn new() -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        let size_row = height.saturating_sub(2);
        let max_row = size_row.saturating_sub(1);
        let prompt_row = height.saturating_sub(1);
        let max_result = max_row as usize;

        Ok(Self { width, prompt_row, size_row, max_result })
    }
}

fn render(state: &mut State, layout: &Layout, out: &mut Stdout) -> io::Result<()> {
    let remaining = state.results.len();
    queue!(out, terminal::Clear(terminal::ClearType::All))?;

    for (screen_i, (result_i, (_, candidate))) in state
        .results
        .iter()
        .enumerate()
        .skip(state.scroll_offset)
        .take(layout.max_result)
        .enumerate()
    {
        let row: u16 = layout.max_result as u16 - screen_i as u16;
        let mut prefix = "  ";
        let mut bg = SetBackgroundColor(style::Color::Reset);

        if result_i == state.selected {
           prefix = "▶ ";
           bg =  SetBackgroundColor(style::Color::DarkGrey);
        }

        queue!(
            out,
            cursor::MoveTo(0, row),
            Print(prefix.to_string()),
            bg,
            Print(candidate.to_string()),
            SetBackgroundColor(style::Color::Reset)
        )?;
    }

    let cursor_x = 2 + state.query.len() as u16;
    let size_format = format!("  {}/{} ", remaining, state.max_quantity);
    let mut line_format = String::new();
    for _ in 1..layout.width.saturating_sub(size_format.len() as u16) {
        line_format.push('-');
    }

    queue!(
        out,
        cursor::MoveTo(0, layout.size_row),
        SetForegroundColor(style::Color::Yellow),
        Print(size_format),
        Print(line_format),
        ResetColor,
        cursor::MoveTo(0, layout.prompt_row),
        Print(format!("> {}", state.query)),
        cursor::MoveTo(cursor_x, layout.prompt_row)
    )?;

    out.flush()?;

    Ok(())
}
