use crate::core::{
    config::ConfigService,
    employee::EmployeeService,
    gerrit::GerritService,
    models::{DataPath, validate_domain},
    notes::NotesService,
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

    // Fetch metrics from Gerrit
    match GerritService::get_employee_metrics(data_path, email).await {
        Ok(metrics) => {
            // Display the formatted report
            display_activity_report(&employee, &metrics);
            Ok(())
        }
        Err(e) => {
            error!("Failed to fetch Gerrit metrics: {e}");
            println!("❌ Failed to fetch review data: {e}");
            println!("\nPossible issues:");
            println!("• Check your Gerrit configuration in gerrit_config.toml");
            println!("• Verify network connectivity to Gerrit instance");
            println!("• Ensure HTTP credentials are correct");
            Err(e)
        }
    }
}

fn display_activity_report(
    employee: &crate::core::models::Employee,
    metrics: &crate::core::gerrit::ActivityMetrics,
) {
    println!("📊 Review Activity Report");
    println!("========================");
    println!("Employee: {} ({})", employee.name, employee.title);
    if let Some(email) = &employee.committer_email {
        println!("Email: {email}");
    }
    println!("Period: Last 30 days");
    println!();

    println!("📈 Activity Metrics:");
    println!("  • Commits Merged:     {}", metrics.commits_merged);
    println!("  • Changes Created:    {}", metrics.changes_created);
    println!("  • Reviews Given:      {}", metrics.reviews_given);
    println!("  • Reviews Received:   {}", metrics.reviews_received);
    println!();

    // Add some basic insights
    if metrics.commits_merged == 0 && metrics.changes_created > 0 {
        println!("💡 Insights:");
        println!("  • Has created changes but none have merged yet");
    } else if metrics.reviews_given > metrics.reviews_received * 2 {
        println!("💡 Insights:");
        println!("  • Very active in reviewing others' code");
    } else if metrics.changes_created > 0 && metrics.reviews_received == 0 {
        println!("💡 Insights:");
        println!("  • May need more code review on their changes");
    }

    if metrics.commits_merged + metrics.changes_created + metrics.reviews_given == 0 {
        println!("ℹ️  No activity found in the last 30 days");
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
            let config = ConfigService::load_config(data_path)?;
            match key.as_str() {
                "allowed_domains" => {
                    println!("allowed_domains: {:?}", config.allowed_domains);
                    println!("Config file: {}", data_path.config_path().display());
                }
                _ => {
                    println!("Unknown key: {key}");
                }
            }
        }
        Some(ConfigCommands::Set { key, value }) => {
            let mut config = ConfigService::load_config(data_path)?;
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
                            config.allowed_domains = valid_domains;
                            ConfigService::save_config(&config, data_path)?;
                            info!("Updated allowed_domains configuration");
                            println!("allowed_domains set to: {:?}", config.allowed_domains);
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
            let config = ConfigService::load_config(data_path)?;
            println!("Current Configuration:");
            println!("======================");
            println!("allowed_domains: {:?}", config.allowed_domains);
            println!();
            println!("Config file: {}", data_path.config_path().display());
        }
    }
    Ok(())
}
