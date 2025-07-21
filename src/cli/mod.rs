use crate::core::{
    config::ConfigService, employee::EmployeeService, models::DataPath, notes::NotesService,
};
use clap::{Parser, Subcommand};
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
                    config.allowed_domains =
                        value.split(',').map(|s| s.trim().to_string()).collect();
                    ConfigService::save_config(&config, data_path)?;
                    println!("allowed_domains set to: {:?}", config.allowed_domains);
                    println!("Config file: {}", data_path.config_path().display());
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
