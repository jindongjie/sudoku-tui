use color_eyre;

use array2d::Array2D;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

#[derive(Debug)]
pub struct App {
    sudoku_array: Array2D<u8>,
    selected_row: usize,
    selected_col: usize,
    status_message: String,
    event_log: Vec<String>,
    event_scroll: Option<usize>,
    event_offset: usize,
    exit: bool,
}

//Default value implementation
impl Default for App {
    fn default() -> Self {
        let status_message = String::from(
            "Use hjkl/arrow to move, numbers to fill, 0 to clear, s to solve (todo), q to quit",
        );
        Self {
            sudoku_array: Array2D::filled_with(0, 9, 9),
            selected_row: 0,
            selected_col: 0,
            event_log: vec![status_message.clone()],
            event_scroll: None,
            event_offset: 0,
            status_message,
            exit: false,
        }
    }
}

//ratatui required fuction implementation
impl App {
    const HINT: &'static str = "Move: hjkl/arrow | Set: 1-9 | Clear: 0 | Solve: s | Quit: q";

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    //layout
    fn draw(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(9), Constraint::Length(4)])
            .split(frame.area());

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(24), Constraint::Min(9)])
            .split(chunks[0]);

        let event_items: Vec<ListItem<'_>> = self
            .event_log
            .iter()
            .map(|msg| ListItem::new(msg.as_str()))
            .collect();
        let event_height = main_chunks[0].height.saturating_sub(2) as usize;
        let max_offset = self.event_log.len().saturating_sub(event_height);
        let offset = self
            .event_scroll
            .map(|offset| offset.min(max_offset))
            .unwrap_or(max_offset);
        self.event_offset = offset;
        let mut event_state = ListState::default();
        *event_state.offset_mut() = offset;
        let event_list = List::new(event_items)
            .block(Block::default().title("Event List").borders(Borders::ALL));
        frame.render_stateful_widget(event_list, main_chunks[0], &mut event_state);

        let grid = Paragraph::new(Text::from(self.grid_lines()))
            .block(Block::default().title("Sudoku").borders(Borders::ALL));
        frame.render_widget(grid, main_chunks[1]);

        let status_text = format!("{}\n{}", Self::HINT, self.status_message);
        let status = Paragraph::new(status_text)
            .block(Block::default().title("Status").borders(Borders::ALL));
        frame.render_widget(status, chunks[1]);
    }

    //sudoku grid
    fn grid_lines(&self) -> Vec<Line<'_>> {
        let mut lines = Vec::new();
        for row in 0..9 {
            if row > 0 && row % 3 == 0 {
                lines.push(Line::from("──────┼───────┼──────"));
            }
            let mut spans: Vec<Span> = Vec::new();
            for col in 0..9 {
                if col > 0 && col % 3 == 0 {
                    spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::raw(" "));
                }
                let value = self.sudoku_array[(row, col)];
                let text = if value == 0 {
                    ".".to_string()
                } else {
                    value.to_string()
                };

                let mut style = Style::default();
                if self.selected_row == row && self.selected_col == col {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                spans.push(Span::styled(text, style));
                if col < 8 {
                    spans.push(Span::raw(" "));
                }
            }
            lines.push(Line::from(spans));
        }
        lines
    }

    fn handle_events(&mut self) -> color_eyre::Result<()> {
        let event = event::read()?;
        if let Event::Key(key_event) = event {
            if key_event.kind != KeyEventKind::Press {
                return Ok(());
            }
            match key_event.code {
                KeyCode::Char('q') => {
                    self.exit = true;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.move_selection(0, -1);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.move_selection(0, 1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_selection(-1, 0);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_selection(1, 0);
                }
                KeyCode::PageUp => {
                    self.scroll_event_log(-1);
                }
                KeyCode::PageDown => {
                    self.scroll_event_log(1);
                }
                KeyCode::Char('0') => {
                    self.update_cell(0);
                    self.set_status(format!(
                        "Cleared cell ({}, {}).",
                        self.selected_row + 1,
                        self.selected_col + 1
                    ));
                }
                KeyCode::Char('s') => {
                    self.set_status("Solve sudoku!".to_string());
                    solve_sudoku(&mut self.sudoku_array);
                }
                KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                    let value = c.to_digit(10).unwrap() as u8;
                    self.update_cell(value);
                    self.set_status(format!(
                        "Set cell ({}, {}) to {}.",
                        self.selected_row + 1,
                        self.selected_col + 1,
                        value
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn move_selection(&mut self, row_delta: i32, col_delta: i32) {
        let new_row = (self.selected_row as i32 + row_delta).clamp(0, 8) as usize;
        let new_col = (self.selected_col as i32 + col_delta).clamp(0, 8) as usize;
        self.selected_row = new_row;
        self.selected_col = new_col;
        self.set_status(format!(
            "Selected cell ({}, {}).",
            self.selected_row + 1,
            self.selected_col + 1
        ));
    }

    fn update_cell(&mut self, value: u8) {
        if let Some(cell) = self
            .sudoku_array
            .get_mut(self.selected_row, self.selected_col)
        {
            *cell = value;
        }
    }

    fn set_status<S: Into<String>>(&mut self, message: S) {
        let message = message.into();
        self.status_message = message.clone();
        self.event_log.push(message);
        self.event_scroll = None;
    }

    fn scroll_event_log(&mut self, delta: i32) {
        let len = self.event_log.len();
        let current = self
            .event_scroll
            .unwrap_or(self.event_offset)
            .min(len.saturating_sub(1)) as i32;
        let new_scroll = (current + delta).clamp(0, len.saturating_sub(1) as i32) as usize;
        self.event_scroll = Some(new_scroll);
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::default().run(terminal))?;
    Ok(())
}

fn solve_sudoku(arr: &mut Array2D<u8>) {
    for row in 0..9 {
        for col in 0..9 {
            for num in 1..10 {
                //check small grid
                //
                // check column
                //check row
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_lines_have_expected_count() {
        let app = App::default();
        let lines = app.grid_lines();
        assert_eq!(lines.len(), 11);
    }

    #[test]
    fn move_selection_clamps_to_board() {
        let mut app = App::default();
        app.move_selection(-1, -1);
        assert_eq!((app.selected_row, app.selected_col), (0, 0));

        app.move_selection(10, 10);
        assert_eq!((app.selected_row, app.selected_col), (8, 8));
    }

    #[test]
    fn update_cell_sets_value() {
        let mut app = App::default();
        app.update_cell(5);
        assert_eq!(app.sudoku_array[(0, 0)], 5);
    }

    #[test]
    fn set_status_updates_event_log() {
        let mut app = App::default();
        let initial_len = app.event_log.len();
        app.set_status("Test event");
        assert_eq!(app.status_message, "Test event");
        assert_eq!(app.event_log.len(), initial_len + 1);
        assert_eq!(app.event_log.last().unwrap(), "Test event");
        assert!(app.event_scroll.is_none());
    }

    #[test]
    #[ignore]
    fn render_snapshot_to_stdout() {
        let app = App::default();
        for line in app.grid_lines() {
            println!("{line}");
        }
    }
}
