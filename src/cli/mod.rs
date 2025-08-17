use crate::core::{
    employee::EmployeeService,
    gerrit::GerritPlatform,
    gitlab::GitLabPlatform,
    jira::JiraPlatform,
    models::{DataPath, validate_domain},
    notes::NotesService,
    platform::{ErrorLogReader, PlatformRegistry},
    unified_config::UnifiedConfigService,
};
use clap::{Parser, Subcommand};
use log::{error, info};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "reviewr")]
#[command(about = "A CLI tool for employee reviews.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Sets a custom data path
    #[arg(long, value_name = "FILE")]
    pub data_path: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new employee
    Add {
        /// The name of the employee (optional - if not provided, opens TUI form)
        employee: Option<String>,
    },
    /// Open notes for an employee
    Notes {
        /// The name of the employee (optional - if not provided, opens TUI selector)
        employee: Option<String>,
    },
    /// Edit an employee's information
    Edit {
        /// The name of the employee (optional - if not provided, opens TUI selector)
        employee: Option<String>,
    },
    /// List all employees
    List,
    /// Generate review report for an employee
    Review {
        /// The name of the employee (optional - if not provided, opens TUI selector)
        employee: Option<String>,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
    /// View error reports and diagnostics
    Errors {
        #[command(subcommand)]
        command: Option<ErrorCommands>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Get a configuration value
    Get {
        /// The key to get
        key: String,
    },
    /// Set a configuration value
    Set {
        /// The key to set
        key: String,
        /// The value to set
        value: String,
    },
}

#[derive(Subcommand)]
pub enum ErrorCommands {
    /// Show recent errors
    List {
        /// Platform to filter by
        #[arg(short, long)]
        platform: Option<String>,
        /// Number of errors to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Show error statistics by platform
    Stats,
    /// Export errors to JSON
    Export {
        /// Platform to filter by
        #[arg(short, long)]
        platform: Option<String>,
        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Clear all recorded errors
    Clear,
}

/// Initialize the platform registry with all available platforms
fn create_platform_registry(data_path: &DataPath) -> PlatformRegistry {
    let mut registry = PlatformRegistry::new();

    // Register Gerrit platform
    let gerrit_platform = GerritPlatform::new(data_path.clone());
    registry.register_platform(Box::new(gerrit_platform));

    // Register JIRA platform
    let jira_platform = JiraPlatform::new(data_path.clone());
    registry.register_platform(Box::new(jira_platform));

    // Register GitLab platforms (multiple instances)
    if let Ok(config) = UnifiedConfigService::load_config(data_path) {
        for (instance_id, gitlab_config) in config.platforms.gitlab {
            if gitlab_config.is_configured() {
                let gitlab_platform = GitLabPlatform::new(gitlab_config, instance_id, data_path);
                registry.register_platform(Box::new(gitlab_platform));
            }
        }
    }

    registry
}

pub fn handle_list_command(data_path: &DataPath) -> io::Result<()> {
    let employees = EmployeeService::list_employees(data_path)?;

    if employees.is_empty() {
        println!("No employees found.");
        return Ok(());
    }

    println!("Employees ({}):", employees.len());
    println!("{}", "=".repeat(20));

    for employee_name in &employees {
        match EmployeeService::get_employee(data_path, employee_name) {
            Ok(employee) => {
                println!("• {} - {}", employee.name, employee.title);
            }
            Err(e) => {
                println!("• {employee_name} - (Error loading: {e})");
            }
        }
    }

    Ok(())
}

pub async fn handle_review_command(
    data_path: &DataPath,
    employee: &Option<String>,
) -> io::Result<()> {
    let employee_name = match employee {
        Some(name) => name.clone(),
        None => {
            // Use TUI selector to choose employee
            use crate::tui::EmployeeSelector;
            let mut selector = EmployeeSelector::new(data_path)?;
            match selector.run()? {
                Some(selected) => selected,
                None => {
                    println!("No employee selected.");
                    return Ok(());
                }
            }
        }
    };

    // Get employee details
    let employee = EmployeeService::get_employee(data_path, &employee_name)?;

    // Check if employee has committer email
    let email = match &employee.committer_email {
        Some(email) if !email.trim().is_empty() => email,
        _ => {
            println!("Employee '{employee_name}' does not have a committer email configured.");
            println!("Use 'reviewr edit {employee_name}' to add their committer email.");
            return Ok(());
        }
    };

    println!(
        "Generating review report for {} ({})...",
        employee.name, email
    );
    println!("This may take a moment...\n");

    // Create platform registry and get configured platforms
    let registry = create_platform_registry(data_path);
    let configured_platforms = registry.get_configured_platforms();

    if configured_platforms.is_empty() {
        println!("❌ No review platforms are configured.");
        println!("\nTo get started:");
        println!("• Configure platforms in the [platforms] section of config.toml");
        println!("• Run 'reviewr config' to check current configuration");
        return Ok(());
    }

    // Launch multi-platform review browser
    use crate::tui::MultiPlatformBrowser;
    let mut browser = MultiPlatformBrowser::new(employee.name.clone(), email.clone(), &registry);

    // Load data from all configured platforms with background processing
    println!(
        "🔄 Starting data fetch from {} platform(s)...",
        configured_platforms.len()
    );
    println!("   Loading will continue in background - press ESC to cancel");

    match browser.load_data_async(&registry).await {
        Ok(_) => {
            println!("✅ Data loading completed, launching TUI...");
            browser.run()?;
            Ok(())
        }
        Err(e) => {
            error!("Failed to load review data: {e}");
            println!("❌ Failed to load review data: {e}");
            println!("\nPossible issues:");
            println!("• Check platform configurations");
            println!("• Verify network connectivity to platform instances");
            println!("• Ensure credentials are correct");
            Err(e)
        }
    }
}

pub fn handle_add_command(data_path: &DataPath, employee: &Option<String>) -> io::Result<()> {
    match employee {
        Some(name) => EmployeeService::add_employee(data_path, name),
        None => {
            // TUI mode
            use crate::tui::EmployeeForm;
            let mut form = EmployeeForm::new();
            match form.run(data_path)? {
                Some(_employee_data) => {
                    // Employee was successfully added
                    Ok(())
                }
                None => {
                    println!("Employee creation cancelled.");
                    Ok(())
                }
            }
        }
    }
}

pub fn handle_notes_command(data_path: &DataPath, employee: &str) -> io::Result<()> {
    if !EmployeeService::employee_exists(data_path, employee) {
        println!("Employee '{employee}' not found.");
        print!("Would you like to add them? (y/n) ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("y") {
            EmployeeService::add_employee(data_path, employee)?;
        } else {
            return Ok(());
        }
    }

    NotesService::open_notes(data_path, employee)
}

pub fn handle_edit_command(data_path: &DataPath, employee: &Option<String>) -> io::Result<()> {
    match employee {
        Some(name) => {
            if !EmployeeService::employee_exists(data_path, name) {
                println!("Employee '{name}' not found.");
                return Ok(());
            }

            // Get existing employee data
            let existing_employee = EmployeeService::get_employee(data_path, name)?;

            // Create form with existing data
            use crate::tui::EmployeeForm;
            let mut form = EmployeeForm::new_with_data(
                existing_employee.name.clone(),
                existing_employee.title.clone(),
                existing_employee.committer_email.clone(),
            );

            match form.run(data_path)? {
                Some(_employee_data) => {
                    println!("Employee '{name}' updated.");
                    Ok(())
                }
                None => {
                    println!("Edit cancelled.");
                    Ok(())
                }
            }
        }
        None => {
            // TUI mode - select employee first, then edit
            use crate::tui::EmployeeSelector;
            let mut selector = EmployeeSelector::new(data_path)?;
            if let Some(selected_employee) = selector.run()? {
                handle_edit_command(data_path, &Some(selected_employee))
            } else {
                println!("No employee selected.");
                Ok(())
            }
        }
    }
}

pub fn handle_config_command(
    data_path: &DataPath,
    command: &Option<ConfigCommands>,
) -> io::Result<()> {
    match command {
        Some(ConfigCommands::Get { key }) => {
            let config = UnifiedConfigService::load_config(data_path)?;
            match key.as_str() {
                "allowed_domains" => {
                    println!(
                        "allowed_domains: {:?}",
                        config.global_settings.allowed_domains
                    );
                    println!("Config file: {}", data_path.config_path().display());
                }
                _ => {
                    println!("Unknown key: {key}");
                }
            }
        }
        Some(ConfigCommands::Set { key, value }) => {
            let mut config = UnifiedConfigService::load_config(data_path)?;
            match key.as_str() {
                "allowed_domains" => {
                    let domains: Result<Vec<String>, io::Error> = value
                        .split(',')
                        .map(|s| {
                            let domain = s.trim().to_string();
                            validate_domain(&domain)?;
                            Ok(domain)
                        })
                        .collect();

                    match domains {
                        Ok(valid_domains) => {
                            config.global_settings.allowed_domains = valid_domains;
                            UnifiedConfigService::save_config(&config, data_path)?;
                            info!("Updated allowed_domains configuration");
                            println!(
                                "allowed_domains set to: {:?}",
                                config.global_settings.allowed_domains
                            );
                            println!("Config file: {}", data_path.config_path().display());
                        }
                        Err(e) => {
                            error!("Invalid domain in configuration: {e}");
                            return Err(e);
                        }
                    }
                }
                _ => {
                    println!("Unknown key: {key}");
                }
            }
        }
        None => {
            // Show all current configuration
            let config = UnifiedConfigService::load_config(data_path)?;
            println!("Current Configuration:");
            println!("======================");
            println!(
                "allowed_domains: {:?}",
                config.global_settings.allowed_domains
            );
            println!();
            println!("Config file: {}", data_path.config_path().display());
        }
    }
    Ok(())
}

pub fn handle_errors_command(command: &Option<ErrorCommands>) -> io::Result<()> {
    match command {
        Some(ErrorCommands::List { platform, limit }) => {
            match ErrorLogReader::read_recent_errors(*limit, platform.as_deref()) {
                Ok(errors) => {
                    if errors.is_empty() {
                        println!("No errors found.");
                        return Ok(());
                    }

                    println!("Recent errors (showing {} most recent):", errors.len());
                    println!();

                    for error in errors {
                        println!(
                            "🔴 {} | {} | {}",
                            error.timestamp, error.platform_id, error.operation
                        );
                        println!(
                            "   Type: {} | Message: {}",
                            error.error_type, error.error_message
                        );
                        if let Some(user) = &error.user {
                            println!("   User: {user}");
                        }
                        if let Some(url) = &error.request_url {
                            println!("   URL: {url}");
                        }
                        if let Some(status) = error.status_code {
                            println!("   Status: {status}");
                        }
                        if !error.metadata.is_empty() {
                            println!("   Context: {:?}", error.metadata);
                        }
                        println!();
                    }
                }
                Err(e) => {
                    error!("Failed to read error log: {e}");
                    println!("❌ Failed to read error log: {e}");
                }
            }
        }
        Some(ErrorCommands::Stats) => match ErrorLogReader::get_error_stats() {
            Ok(stats) => {
                if stats.is_empty() {
                    println!("No error statistics available.");
                    return Ok(());
                }

                println!("Error statistics by platform:");
                println!();

                for (platform, platform_stats) in stats {
                    println!("📊 Platform: {platform}");
                    println!("   Total errors: {}", platform_stats.total_errors);
                    if let Some(last_error) = &platform_stats.last_error_time {
                        println!("   Last error: {last_error}");
                    }

                    if !platform_stats.error_types.is_empty() {
                        println!("   Error types:");
                        for (error_type, count) in platform_stats.error_types {
                            println!("     {error_type}: {count}");
                        }
                    }
                    println!();
                }
            }
            Err(e) => {
                error!("Failed to read error statistics: {e}");
                println!("❌ Failed to read error statistics: {e}");
            }
        },
        Some(ErrorCommands::Export { platform, output }) => {
            match ErrorLogReader::read_recent_errors(1000, platform.as_deref()) {
                Ok(errors) => {
                    let json_output = serde_json::to_string_pretty(&errors).map_err(|e| {
                        io::Error::other(format!("Failed to serialize errors: {e}"))
                    })?;

                    match output {
                        Some(output_path) => {
                            std::fs::write(output_path, json_output)?;
                            println!(
                                "✅ Exported {} errors to {}",
                                errors.len(),
                                output_path.display()
                            );
                        }
                        None => {
                            println!("{json_output}");
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to export errors: {e}");
                    println!("❌ Failed to export errors: {e}");
                }
            }
        }
        Some(ErrorCommands::Clear) => {
            let data_dir = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".reviewr");

            let error_log_path = data_dir.join("error.log");

            if error_log_path.exists() {
                std::fs::remove_file(error_log_path)?;
                println!("✅ Error log cleared.");
            } else {
                println!("No error log file found.");
            }
        }
        None => {
            println!("Available error commands:");
            println!("  list    - Show recent errors");
            println!("  stats   - Show error statistics by platform");
            println!("  export  - Export errors to JSON");
            println!("  clear   - Clear all recorded errors");
            println!();
            println!("Use 'reviewr errors <command> --help' for more information.");
        }
    }

    Ok(())
}
