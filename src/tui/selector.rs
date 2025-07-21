use crate::core::{employee::EmployeeService, models::DataPath};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use nucleo::{Config, Matcher, Utf32Str};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

pub struct EmployeeSelector {
    employees: Vec<String>,
    filtered_employees: Vec<(String, u32)>,
    list_state: ListState,
    input: String,
    matcher: Matcher,
}

impl EmployeeSelector {
    pub fn new(data_path: &DataPath) -> io::Result<Self> {
        let employees = EmployeeService::list_employees(data_path)?;
        let filtered_employees: Vec<(String, u32)> =
            employees.iter().map(|e| (e.clone(), 0)).collect();

        let mut list_state = ListState::default();
        if !filtered_employees.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            employees,
            filtered_employees,
            list_state,
            input: String::new(),
            matcher: Matcher::new(Config::DEFAULT),
        })
    }

    pub fn run(&mut self) -> io::Result<Option<String>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<Option<String>> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match self.handle_key_event(key) {
                    Some(result) => return Ok(result),
                    None => continue,
                }
            }
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Option<String>> {
        match key.code {
            KeyCode::Char(c) => {
                self.input.push(c);
                self.filter_employees();
                None
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.filter_employees();
                None
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.filtered_employees.len() {
                        return Some(Some(self.filtered_employees[selected].0.clone()));
                    }
                }
                None
            }
            KeyCode::Up => {
                if let Some(selected) = self.list_state.selected() {
                    if selected > 0 {
                        self.list_state.select(Some(selected - 1));
                    }
                }
                None
            }
            KeyCode::Down => {
                if let Some(selected) = self.list_state.selected() {
                    if selected + 1 < self.filtered_employees.len() {
                        self.list_state.select(Some(selected + 1));
                    }
                } else if !self.filtered_employees.is_empty() {
                    self.list_state.select(Some(0));
                }
                None
            }
            KeyCode::Esc => Some(None),
            _ => None,
        }
    }

    fn filter_employees(&mut self) {
        if self.input.is_empty() {
            self.filtered_employees = self.employees.iter().map(|e| (e.clone(), 0)).collect();
        } else {
            let mut matches = Vec::new();
            for employee in &self.employees {
                let mut haystack_buf = Vec::new();
                let mut needle_buf = Vec::new();
                let haystack = Utf32Str::new(employee, &mut haystack_buf);
                let needle = Utf32Str::new(&self.input, &mut needle_buf);
                if let Some(score) = self.matcher.fuzzy_match(haystack, needle) {
                    matches.push((employee.clone(), score as u32));
                }
            }
            matches.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_employees = matches;
        }

        self.list_state
            .select(if self.filtered_employees.is_empty() {
                None
            } else {
                Some(0)
            });
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(f.area());

        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Search"));
        f.render_widget(input, chunks[0]);

        let items: Vec<ListItem> = self
            .filtered_employees
            .iter()
            .map(|(name, score)| {
                let content = if *score > 0 {
                    Line::from(vec![
                        Span::raw(name.clone()),
                        Span::styled(format!(" ({score})"), Style::default().fg(Color::Gray)),
                    ])
                } else {
                    Line::from(Span::raw(name.clone()))
                };
                ListItem::new(content)
            })
            .collect();

        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Employees"))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(items, chunks[1], &mut self.list_state);
    }
}
