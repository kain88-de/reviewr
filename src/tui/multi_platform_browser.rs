use crate::core::platform::{ActivityCategory, ActivityItem, DetailedActivities, PlatformRegistry};
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
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};
use std::collections::HashMap;
use std::io;

#[derive(Clone, PartialEq)]
enum ViewMode {
    Summary,
    PlatformView {
        platform_id: String,
    },
    CategoryView {
        platform_id: String,
        category: ActivityCategory,
    },
}

impl ViewMode {
    fn title(&self, platforms: &HashMap<String, String>) -> String {
        match self {
            ViewMode::Summary => "📊 Multi-Platform Activity Summary".to_string(),
            ViewMode::PlatformView { platform_id } => {
                let platform_name = platforms.get(platform_id).unwrap_or(platform_id);
                format!("🏢 {platform_name} Activity")
            }
            ViewMode::CategoryView {
                platform_id,
                category,
            } => {
                let platform_name = platforms.get(platform_id).unwrap_or(platform_id);
                format!("📋 {} - {}", platform_name, category.display_name())
            }
        }
    }
}

pub struct MultiPlatformBrowser {
    employee_name: String,
    employee_email: String,
    platform_activities: HashMap<String, DetailedActivities>,
    platform_names: HashMap<String, String>, // platform_id -> display name
    platform_icons: HashMap<String, String>, // platform_id -> icon
    current_view: ViewMode,
    selected_platform_index: usize,
    selected_category_index: usize,
    list_state: ListState,
    show_help: bool,
    platform_order: Vec<String>, // Order of platforms for navigation
}

impl MultiPlatformBrowser {
    pub fn new(employee_name: String, employee_email: String, registry: &PlatformRegistry) -> Self {
        let mut platform_names = HashMap::new();
        let mut platform_icons = HashMap::new();
        let mut platform_order = Vec::new();

        // Initialize platform metadata
        for platform in registry.get_configured_platforms() {
            let id = platform.get_platform_id().to_string();
            platform_names.insert(id.clone(), platform.get_platform_name().to_string());
            platform_icons.insert(id.clone(), platform.get_platform_icon().to_string());
            platform_order.push(id);
        }

        Self {
            employee_name,
            employee_email,
            platform_activities: HashMap::new(),
            platform_names,
            platform_icons,
            current_view: ViewMode::Summary,
            selected_platform_index: 0,
            selected_category_index: 0,
            list_state: ListState::default(),
            show_help: false,
            platform_order,
        }
    }

    pub async fn load_data(&mut self, registry: &PlatformRegistry) -> io::Result<()> {
        for platform in registry.get_configured_platforms() {
            let platform_id = platform.get_platform_id();
            match platform
                .get_detailed_activities(&self.employee_email, 30)
                .await
            {
                Ok(activities) => {
                    self.platform_activities
                        .insert(platform_id.to_string(), activities);
                }
                Err(e) => {
                    // Log error but continue with other platforms
                    log::warn!("Failed to load data from {platform_id}: {e}");
                }
            }
        }
        Ok(())
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

            if let Event::Key(key) = event::read()? {
                if self.handle_key_event(key)? {
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> io::Result<bool> {
        if self.show_help {
            if matches!(
                key.code,
                KeyCode::Char('h') | KeyCode::Esc | KeyCode::Char('?')
            ) {
                self.show_help = false;
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char('h') | KeyCode::Char('?') => {
                self.show_help = true;
            }
            KeyCode::Char('s') => {
                self.current_view = ViewMode::Summary;
                self.list_state.select(None);
            }
            KeyCode::Tab => {
                self.next_platform();
            }
            KeyCode::BackTab => {
                self.prev_platform();
            }
            KeyCode::Enter => {
                if let ViewMode::Summary = self.current_view {
                    if !self.platform_order.is_empty() {
                        let platform_id = self.platform_order[self.selected_platform_index].clone();
                        self.current_view = ViewMode::PlatformView { platform_id };
                        self.list_state.select(Some(0));
                        self.selected_category_index = 0;
                    }
                } else if let ViewMode::PlatformView { platform_id } = &self.current_view {
                    if let Some(selected) = self.list_state.selected() {
                        let categories = self.get_available_categories(platform_id);
                        if selected < categories.len() {
                            let category = categories[selected].clone();
                            self.current_view = ViewMode::CategoryView {
                                platform_id: platform_id.clone(),
                                category,
                            };
                            self.list_state.select(Some(0));
                        }
                    }
                } else if let ViewMode::CategoryView { .. } = &self.current_view {
                    if let Some(selected) = self.list_state.selected() {
                        self.open_item_in_browser(selected)?;
                    }
                }
            }
            KeyCode::Backspace => match &self.current_view {
                ViewMode::CategoryView { platform_id, .. } => {
                    self.current_view = ViewMode::PlatformView {
                        platform_id: platform_id.clone(),
                    };
                    self.list_state.select(Some(self.selected_category_index));
                }
                ViewMode::PlatformView { .. } => {
                    self.current_view = ViewMode::Summary;
                    self.list_state.select(Some(self.selected_platform_index));
                }
                ViewMode::Summary => {}
            },
            KeyCode::Up => {
                self.previous_item();
            }
            KeyCode::Down => {
                self.next_item();
            }
            _ => {}
        }
        Ok(false)
    }

    fn next_platform(&mut self) {
        if !self.platform_order.is_empty() {
            self.selected_platform_index =
                (self.selected_platform_index + 1) % self.platform_order.len();
            if let ViewMode::Summary = self.current_view {
                self.list_state.select(Some(self.selected_platform_index));
            }
        }
    }

    fn prev_platform(&mut self) {
        if !self.platform_order.is_empty() {
            self.selected_platform_index = if self.selected_platform_index == 0 {
                self.platform_order.len() - 1
            } else {
                self.selected_platform_index - 1
            };
            if let ViewMode::Summary = self.current_view {
                self.list_state.select(Some(self.selected_platform_index));
            }
        }
    }

    fn next_item(&mut self) {
        let max_items = match &self.current_view {
            ViewMode::Summary => self.platform_order.len(),
            ViewMode::PlatformView { platform_id } => {
                self.get_available_categories(platform_id).len()
            }
            ViewMode::CategoryView {
                platform_id,
                category,
            } => self.get_category_items(platform_id, category).len(),
        };

        if max_items > 0 {
            let selected = self.list_state.selected().unwrap_or(0);
            let next = if selected >= max_items.saturating_sub(1) {
                0
            } else {
                selected + 1
            };
            self.list_state.select(Some(next));

            // Update indices for navigation
            match &self.current_view {
                ViewMode::Summary => self.selected_platform_index = next,
                ViewMode::PlatformView { .. } => self.selected_category_index = next,
                ViewMode::CategoryView { .. } => {}
            }
        }
    }

    fn previous_item(&mut self) {
        let max_items = match &self.current_view {
            ViewMode::Summary => self.platform_order.len(),
            ViewMode::PlatformView { platform_id } => {
                self.get_available_categories(platform_id).len()
            }
            ViewMode::CategoryView {
                platform_id,
                category,
            } => self.get_category_items(platform_id, category).len(),
        };

        if max_items > 0 {
            let selected = self.list_state.selected().unwrap_or(0);
            let prev = if selected == 0 {
                max_items - 1
            } else {
                selected - 1
            };
            self.list_state.select(Some(prev));

            // Update indices for navigation
            match &self.current_view {
                ViewMode::Summary => self.selected_platform_index = prev,
                ViewMode::PlatformView { .. } => self.selected_category_index = prev,
                ViewMode::CategoryView { .. } => {}
            }
        }
    }

    fn get_available_categories(&self, platform_id: &str) -> Vec<ActivityCategory> {
        if let Some(activities) = self.platform_activities.get(platform_id) {
            activities.items_by_category.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    fn get_category_items(
        &self,
        platform_id: &str,
        category: &ActivityCategory,
    ) -> Vec<ActivityItem> {
        if let Some(activities) = self.platform_activities.get(platform_id) {
            activities
                .items_by_category
                .get(category)
                .cloned()
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn open_item_in_browser(&self, selected_index: usize) -> io::Result<()> {
        if let ViewMode::CategoryView {
            platform_id,
            category,
        } = &self.current_view
        {
            let items = self.get_category_items(platform_id, category);
            if let Some(item) = items.get(selected_index) {
                if let Err(e) = webbrowser::open(&item.url) {
                    log::warn!("Failed to open URL in browser: {e}");
                }
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let size = f.area();

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(size);

        // Header
        let header = Paragraph::new(format!(
            "📋 {} ({}) - {}",
            self.employee_name,
            self.employee_email,
            self.current_view.title(&self.platform_names)
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Employee Review Dashboard"),
        )
        .wrap(Wrap { trim: true });
        f.render_widget(header, chunks[0]);

        // Main content
        let current_view = self.current_view.clone();
        match current_view {
            ViewMode::Summary => self.render_summary(f, chunks[1]),
            ViewMode::PlatformView { platform_id } => {
                self.render_platform_view(f, chunks[1], &platform_id)
            }
            ViewMode::CategoryView {
                platform_id,
                category,
            } => self.render_category_view(f, chunks[1], &platform_id, &category),
        }

        // Footer
        let footer_text = match &self.current_view {
            ViewMode::Summary => {
                "Tab/Shift+Tab: Switch Platform | Enter: View Platform | h: Help | q: Quit"
            }
            ViewMode::PlatformView { .. } => {
                "↑/↓: Navigate | Enter: View Category | Backspace: Back | h: Help | q: Quit"
            }
            ViewMode::CategoryView { .. } => {
                "↑/↓: Navigate | Enter: Open in Browser | Backspace: Back | h: Help | q: Quit"
            }
        };
        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .wrap(Wrap { trim: true });
        f.render_widget(footer, chunks[2]);

        // Help overlay
        if self.show_help {
            self.render_help_overlay(f, size);
        }
    }

    fn render_summary(&mut self, f: &mut Frame, area: ratatui::layout::Rect) {
        let platform_tabs: Vec<String> = self
            .platform_order
            .iter()
            .map(|id| {
                let default_icon = "📄".to_string();
                let icon = self.platform_icons.get(id).unwrap_or(&default_icon);
                let name = self.platform_names.get(id).unwrap_or(id);
                format!("{icon} {name}")
            })
            .collect();

        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
            ])
            .split(area);

        // Platform tabs
        let tabs = Tabs::new(platform_tabs)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Available Platforms"),
            )
            .select(self.selected_platform_index)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, content_chunks[0]);

        // Platform summary list
        let platform_items: Vec<ListItem> = self
            .platform_order
            .iter()
            .map(|platform_id| {
                let default_icon = "📄".to_string();
                let icon = self
                    .platform_icons
                    .get(platform_id)
                    .unwrap_or(&default_icon);
                let name = self.platform_names.get(platform_id).unwrap_or(platform_id);

                let summary = if let Some(activities) = self.platform_activities.get(platform_id) {
                    let total_items: usize = activities
                        .items_by_category
                        .values()
                        .map(|items| items.len())
                        .sum();
                    let categories_count = activities.items_by_category.len();
                    format!(
                        "{icon} {name} - {total_items} items across {categories_count} categories"
                    )
                } else {
                    format!("{icon} {name} - No data available")
                };

                ListItem::new(summary)
            })
            .collect();

        let platform_list = List::new(platform_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Platform Summary"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(platform_list, content_chunks[1], &mut self.list_state);
    }

    fn render_platform_view(
        &mut self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        platform_id: &str,
    ) {
        let categories = self.get_available_categories(platform_id);

        let category_items: Vec<ListItem> = categories
            .iter()
            .map(|category| {
                let count = self
                    .platform_activities
                    .get(platform_id)
                    .and_then(|activities| activities.items_by_category.get(category))
                    .map(|items| items.len())
                    .unwrap_or(0);

                let icon = match category {
                    ActivityCategory::ChangesCreated => "📝",
                    ActivityCategory::ChangesMerged => "✅",
                    ActivityCategory::ReviewsGiven => "👀",
                    ActivityCategory::ReviewsReceived => "📥",
                    ActivityCategory::IssuesCreated => "🎫",
                    ActivityCategory::IssuesResolved => "✅",
                    ActivityCategory::IssuesAssigned => "📌",
                    ActivityCategory::IssuesCommented => "💬",
                    _ => "📄",
                };

                ListItem::new(format!("{} {} ({})", icon, category.display_name(), count))
            })
            .collect();

        let category_list = List::new(category_items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                    "Categories in {}",
                    self.platform_names
                        .get(platform_id)
                        .map_or(platform_id, |name| name.as_str())
                )))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(category_list, area, &mut self.list_state);
    }

    fn render_category_view(
        &mut self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        platform_id: &str,
        category: &ActivityCategory,
    ) {
        let items = self.get_category_items(platform_id, category);
        let selected_idx = self.list_state.selected();

        let (list_area, detail_area) = if let Some(idx) = selected_idx {
            if idx < items.len() {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(10),   // List takes most space but at least 10 lines
                        Constraint::Length(8), // Details panel fixed height
                    ])
                    .split(area);
                (chunks[0], Some(chunks[1]))
            } else {
                (area, None)
            }
        } else {
            (area, None)
        };

        // Item list
        let list_items: Vec<ListItem> = items
            .iter()
            .map(|item| {
                let truncated_title = if item.title.len() > 60 {
                    format!("{}...", &item.title[..57])
                } else {
                    item.title.clone()
                };

                let project_display = if item.project.len() > 20 {
                    format!("{}...", &item.project[..17])
                } else {
                    item.project.clone()
                };

                ListItem::new(format!(
                    "[{}] {} - {}",
                    item.id, truncated_title, project_display
                ))
            })
            .collect();

        let item_list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{} Items", category.display_name())),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(item_list, list_area, &mut self.list_state);

        // Details panel
        if let (Some(detail_area), Some(idx)) = (detail_area, selected_idx) {
            if let Some(selected_item) = items.get(idx) {
                let details_text = format!(
                    "ID: {}\nTitle: {}\nProject: {}\nStatus: {}\nCreated: {}\nUpdated: {}",
                    selected_item.id,
                    selected_item.title,
                    selected_item.project,
                    selected_item.status,
                    selected_item.created,
                    selected_item.updated
                );

                let details = Paragraph::new(details_text)
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .wrap(Wrap { trim: true });

                f.render_widget(details, detail_area);
            }
        }
    }

    fn render_help_overlay(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let help_text = "📋 Multi-Platform Review Browser Help

NAVIGATION:
  ↑/↓         Navigate through lists
  Enter       Select item / View details / Open in browser
  Backspace   Go back to previous view
  Tab         Switch between platforms (in summary)
  Shift+Tab   Switch platforms backwards

VIEWS:
  s           Go to Summary view
  h/?         Show/hide this help
  q/Esc       Quit application

FEATURES:
  • Summary: Overview of all configured platforms
  • Platform View: Browse categories within a platform
  • Category View: View specific items (changes, tickets, etc.)
  • Open items directly in your web browser

Press h or Esc to close this help.";

        let help_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .margin(2)
            .split(area)[0];

        f.render_widget(Clear, help_area);
        let help_popup = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .wrap(Wrap { trim: true });
        f.render_widget(help_popup, help_area);
    }
}
