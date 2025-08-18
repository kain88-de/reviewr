use crate::core::gerrit::{ChangeInfo, DetailedActivityMetrics};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;

#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    Summary,
    CommitsMerged,
    ChangesCreated,
    ReviewsGiven,
    ReviewsReceived,
}

impl ViewMode {
    fn title(&self) -> &str {
        match self {
            ViewMode::Summary => "📊 Review Activity Summary",
            ViewMode::CommitsMerged => "✅ Commits Merged",
            ViewMode::ChangesCreated => "📝 Changes Created",
            ViewMode::ReviewsGiven => "👀 Reviews Given",
            ViewMode::ReviewsReceived => "📥 Reviews Received",
        }
    }
}

pub struct ReviewBrowser {
    employee_name: String,
    employee_email: String,
    metrics: DetailedActivityMetrics,
    gerrit_base_url: String,
    current_view: ViewMode,
    list_state: ListState,
    show_help: bool,
}

impl ReviewBrowser {
    pub fn new(
        employee_name: String,
        employee_email: String,
        metrics: DetailedActivityMetrics,
        gerrit_base_url: String,
    ) -> Self {
        Self {
            employee_name,
            employee_email,
            metrics,
            gerrit_base_url,
            current_view: ViewMode::Summary,
            list_state: ListState::default(),
            show_help: false,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
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

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()?
                && self.handle_key_event(key)?
            {
                break;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char('h') | KeyCode::F(1) => {
                self.show_help = !self.show_help;
            }
            KeyCode::Char('s') => {
                self.current_view = ViewMode::Summary;
                self.list_state.select(None);
            }
            KeyCode::Char('m') => {
                self.current_view = ViewMode::CommitsMerged;
                self.list_state.select(Some(0));
            }
            KeyCode::Char('c') => {
                self.current_view = ViewMode::ChangesCreated;
                self.list_state.select(Some(0));
            }
            KeyCode::Char('g') => {
                self.current_view = ViewMode::ReviewsGiven;
                self.list_state.select(Some(0));
            }
            KeyCode::Char('r') => {
                self.current_view = ViewMode::ReviewsReceived;
                self.list_state.select(Some(0));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.current_view != ViewMode::Summary {
                    let changes = self.get_current_changes();
                    if !changes.is_empty() {
                        let i = match self.list_state.selected() {
                            Some(i) => {
                                if i >= changes.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.current_view != ViewMode::Summary {
                    let changes = self.get_current_changes();
                    if !changes.is_empty() {
                        let i = match self.list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    changes.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                    }
                }
            }
            KeyCode::Enter => {
                if self.current_view != ViewMode::Summary
                    && let Some(selected) = self.list_state.selected()
                {
                    let changes = self.get_current_changes();
                    if let Some(change) = changes.get(selected) {
                        let url = format!(
                            "{}/c/{}/+/{}",
                            self.gerrit_base_url, change.project, change.number
                        );
                        println!("Opening: {url}");
                        // Try to open the URL in the default browser
                        #[cfg(target_os = "linux")]
                        std::process::Command::new("xdg-open")
                            .arg(&url)
                            .spawn()
                            .ok();
                        #[cfg(target_os = "macos")]
                        std::process::Command::new("open").arg(&url).spawn().ok();
                        #[cfg(target_os = "windows")]
                        std::process::Command::new("cmd")
                            .args(["/c", "start", &url])
                            .spawn()
                            .ok();
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn get_current_changes(&self) -> &Vec<ChangeInfo> {
        static EMPTY_VEC: Vec<ChangeInfo> = Vec::new();
        match self.current_view {
            ViewMode::Summary => &EMPTY_VEC,
            ViewMode::CommitsMerged => &self.metrics.commits_merged,
            ViewMode::ChangesCreated => &self.metrics.changes_created,
            ViewMode::ReviewsGiven => &self.metrics.reviews_given,
            ViewMode::ReviewsReceived => &self.metrics.reviews_received,
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),    // Main content needs minimum space
                Constraint::Length(3), // Navigation bar
            ])
            .margin(0)
            .split(f.area());

        // Main content area
        match self.current_view {
            ViewMode::Summary => self.render_summary(f, main_chunks[0]),
            _ => self.render_change_list(f, main_chunks[0]),
        }

        // Navigation bar
        self.render_navigation(f, main_chunks[1]);

        // Help overlay
        if self.show_help {
            self.render_help(f);
        }
    }

    fn render_summary(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(6), // Header
                Constraint::Length(7), // Metrics
                Constraint::Min(4),    // Instructions
            ])
            .split(area);

        // Header
        let header = Paragraph::new(format!(
            "📊 Review Activity Report\n\
             Employee: {}\n\
             Email: {}\n\
             Period: Last 30 days",
            self.employee_name, self.employee_email
        ))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Summary"));
        f.render_widget(header, chunks[0]);

        // Metrics overview
        let metrics_text = format!(
            "📈 Activity Metrics:\n\
             • [M]erged Commits:   {:>3} | • [G]iven Reviews:    {:>3}\n\
             • [C]reated Changes:  {:>3} | • [R]eceived Reviews: {:>3}\n\
             \n\
             Press the letter keys to explore each category in detail.",
            self.metrics.commits_merged.len(),
            self.metrics.reviews_given.len(),
            self.metrics.changes_created.len(),
            self.metrics.reviews_received.len()
        );

        let metrics = Paragraph::new(metrics_text)
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title("Quick Access"));
        f.render_widget(metrics, chunks[1]);

        // Instructions
        let instructions = Paragraph::new(
            "📋 Navigation:\n\
             • Use letter keys (s/m/c/g/r) to switch views\n\
             • In lists: ↑/↓ or j/k to navigate, Enter to open in browser\n\
             • Press 'h' for detailed help, 'q' to quit",
        )
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Instructions"));
        f.render_widget(instructions, chunks[2]);
    }

    fn render_change_list(&mut self, f: &mut Frame, area: ratatui::layout::Rect) {
        let changes = self.get_current_changes();
        let changes_len = changes.len();
        let selected_index = self.list_state.selected();

        // Get selected change info before borrowing self mutably
        let selected_change = selected_index.and_then(|idx| changes.get(idx).cloned());

        // Split area into list and details sections
        let (list_area, detail_area) = if selected_change.is_some() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(10),   // List takes most space but at least 10 lines
                    Constraint::Length(8), // Details panel fixed height
                ])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None) // Use full area if no selection
        };

        let items: Vec<ListItem> = changes
            .iter()
            .enumerate()
            .map(|(i, change)| {
                let style = if Some(i) == selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let status_color = match change.status.as_str() {
                    "MERGED" => Color::Green,
                    "NEW" => Color::Blue,
                    "ABANDONED" => Color::Red,
                    _ => Color::White,
                };

                // Truncate subject if too long to prevent wrapping
                let max_subject_len = area.width.saturating_sub(30); // Reserve space for other fields
                let truncated_subject = if change.subject.len() > max_subject_len as usize {
                    format!(
                        "{}...",
                        &change.subject[..max_subject_len.saturating_sub(3) as usize]
                    )
                } else {
                    change.subject.clone()
                };

                let content = format!(
                    "{:<8} | {:<10} | {:<20} | {}",
                    change.number,
                    change.status,
                    if change.project.len() > 20 {
                        format!("{}...", &change.project[..17])
                    } else {
                        change.project.clone()
                    },
                    truncated_subject
                );

                ListItem::new(content).style(style.fg(status_color))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "{} ({})",
                self.current_view.title(),
                changes_len
            )))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(list, list_area, &mut self.list_state);

        // Show selected change details if any
        if let (Some(change), Some(detail_area)) = (selected_change, detail_area) {
            let detail_text = format!(
                "📋 {}\n\
                 Project: {}\n\
                 Status: {} | Created: {}\n\
                 Subject: {}\n\
                 URL: {}/c/{}/+/{}\n\
                 Press Enter to open in browser",
                change.change_id,
                change.project,
                change.status,
                change.created.split('T').next().unwrap_or(&change.created),
                change.subject,
                self.gerrit_base_url,
                change.project,
                change.number
            );

            let detail = Paragraph::new(detail_text)
                .style(Style::default().fg(Color::Cyan))
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Selected Change Details"),
                );

            f.render_widget(detail, detail_area);
        }
    }

    fn render_navigation(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let nav_text = format!(
            "[S]ummary | [M]erged ({}) | [C]reated ({}) | [G]iven ({}) | [R]eceived ({}) | [H]elp | [Q]uit",
            self.metrics.commits_merged.len(),
            self.metrics.changes_created.len(),
            self.metrics.reviews_given.len(),
            self.metrics.reviews_received.len()
        );

        let nav = Paragraph::new(nav_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Navigation"));
        f.render_widget(nav, area);
    }

    fn render_help(&self, f: &mut Frame) {
        let help_area = centered_rect(70, 60, f.area());
        f.render_widget(Clear, help_area);

        let help_text = "\
📋 Review Browser Help\n\
====================\n\
\n\
Navigation:\n\
  s       - Show summary view\n\
  m       - Show merged commits\n\
  c       - Show created changes\n\
  g       - Show reviews given\n\
  r       - Show reviews received\n\
\n\
In List Views:\n\
  ↑/↓     - Navigate up/down\n\
  j/k     - Navigate up/down (vim style)\n\
  Enter   - Open change in browser\n\
\n\
General:\n\
  h/F1    - Toggle this help\n\
  q/Esc   - Quit application\n\
\n\
Press any key to close help...";

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .style(Style::default().fg(Color::Yellow)),
            );
        f.render_widget(help, help_area);
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
