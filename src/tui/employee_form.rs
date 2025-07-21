use crate::core::{employee::EmployeeService, models::DataPath};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::io;

#[derive(Debug, Clone)]
pub struct EmployeeData {
    pub name: String,
    pub title: String,
}

pub struct EmployeeForm {
    employee: EmployeeData,
    original_name: Option<String>,
    current_field: usize,
    mode: FormMode,
}

#[derive(PartialEq)]
enum FormMode {
    Edit,
    Confirm,
}

impl Default for EmployeeForm {
    fn default() -> Self {
        Self::new()
    }
}

impl EmployeeForm {
    pub fn new() -> Self {
        Self {
            employee: EmployeeData {
                name: String::new(),
                title: String::new(),
            },
            original_name: None,
            current_field: 0,
            mode: FormMode::Edit,
        }
    }

    pub fn new_with_data(name: String, title: String) -> Self {
        Self {
            employee: EmployeeData {
                name: name.clone(),
                title,
            },
            original_name: Some(name),
            current_field: 0,
            mode: FormMode::Edit,
        }
    }

    pub fn run(&mut self, data_path: &DataPath) -> io::Result<Option<EmployeeData>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal, data_path);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_app<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        data_path: &DataPath,
    ) -> io::Result<Option<EmployeeData>> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match self.handle_key_event(key, data_path)? {
                    Some(result) => return Ok(result),
                    None => continue,
                }
            }
        }
    }

    fn handle_key_event(
        &mut self,
        key: KeyEvent,
        data_path: &DataPath,
    ) -> io::Result<Option<Option<EmployeeData>>> {
        match self.mode {
            FormMode::Edit => match key.code {
                KeyCode::Char(c) => {
                    match self.current_field {
                        0 => self.employee.name.push(c),
                        1 => self.employee.title.push(c),
                        _ => {}
                    }
                    Ok(None)
                }
                KeyCode::Backspace => {
                    match self.current_field {
                        0 => {
                            self.employee.name.pop();
                        }
                        1 => {
                            self.employee.title.pop();
                        }
                        _ => {}
                    }
                    Ok(None)
                }
                KeyCode::Tab | KeyCode::Down => {
                    self.current_field = (self.current_field + 1) % 2;
                    Ok(None)
                }
                KeyCode::BackTab | KeyCode::Up => {
                    self.current_field = if self.current_field == 0 {
                        1
                    } else {
                        self.current_field - 1
                    };
                    Ok(None)
                }
                KeyCode::Enter => {
                    if !self.employee.name.trim().is_empty()
                        && !self.employee.title.trim().is_empty()
                    {
                        self.mode = FormMode::Confirm;
                    }
                    Ok(None)
                }
                KeyCode::Esc => Ok(Some(None)),
                _ => Ok(None),
            },
            FormMode::Confirm => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        // Save the employee
                        match &self.original_name {
                            Some(original_name) => {
                                // Update existing employee
                                EmployeeService::update_employee(
                                    data_path,
                                    original_name,
                                    self.employee.name.trim(),
                                    self.employee.title.trim(),
                                )?;
                            }
                            None => {
                                // Create new employee
                                EmployeeService::add_employee_with_data(
                                    data_path,
                                    self.employee.name.trim(),
                                    self.employee.title.trim(),
                                )?;
                            }
                        }
                        Ok(Some(Some(self.employee.clone())))
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.mode = FormMode::Edit;
                        Ok(None)
                    }
                    _ => Ok(None),
                }
            }
        }
    }

    fn ui(&self, f: &mut Frame) {
        let area = f.area();

        match self.mode {
            FormMode::Edit => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(2),
                        Constraint::Min(0),
                    ])
                    .split(area);

                // Name field
                let name_style = if self.current_field == 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                let name_input = Paragraph::new(self.employee.name.as_str())
                    .style(name_style)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Employee Name"),
                    );
                f.render_widget(name_input, chunks[0]);

                // Title field
                let title_style = if self.current_field == 1 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                let title_input = Paragraph::new(self.employee.title.as_str())
                    .style(title_style)
                    .block(Block::default().borders(Borders::ALL).title("Job Title"));
                f.render_widget(title_input, chunks[1]);

                // Instructions
                let instructions = Paragraph::new("Tab: Next field | Enter: Save | Esc: Cancel")
                    .style(Style::default().fg(Color::Gray));
                f.render_widget(instructions, chunks[2]);
            }
            FormMode::Confirm => {
                // Confirmation dialog
                let popup_area = centered_rect(50, 30, area);
                f.render_widget(Clear, popup_area);

                let confirmation_text = format!(
                    "Save employee?\n\nName: {}\nTitle: {}\n\nPress Y to confirm, N to cancel",
                    self.employee.name.trim(),
                    self.employee.title.trim()
                );

                let confirmation = Paragraph::new(confirmation_text)
                    .style(Style::default().fg(Color::White))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Confirm")
                            .style(Style::default().fg(Color::Yellow)),
                    );
                f.render_widget(confirmation, popup_area);
            }
        }
    }
}

// Helper function to create centered popup
fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_employee_form_creation() {
        let form = EmployeeForm::new();
        assert_eq!(form.employee.name, "");
        assert_eq!(form.employee.title, "");
        assert_eq!(form.current_field, 0);
        assert!(form.original_name.is_none());
    }

    #[test]
    fn test_employee_form_with_existing_data() {
        let form = EmployeeForm::new_with_data("John Doe".to_string(), "Engineer".to_string());
        assert_eq!(form.employee.name, "John Doe");
        assert_eq!(form.employee.title, "Engineer");
        assert_eq!(form.original_name, Some("John Doe".to_string()));
    }

    #[test]
    fn test_employee_data_clone() {
        let data = EmployeeData {
            name: "Jane Smith".to_string(),
            title: "Manager".to_string(),
        };
        let cloned = data.clone();
        assert_eq!(data.name, cloned.name);
        assert_eq!(data.title, cloned.title);
    }
}
