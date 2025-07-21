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
        /// The name of the employee
        employee: String,
    },
    /// Open notes for an employee
    Notes {
        /// The name of the employee (optional - if not provided, opens TUI selector)
        employee: Option<String>,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
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

pub fn handle_add_command(data_path: &DataPath, employee: &str) -> io::Result<()> {
    EmployeeService::add_employee(data_path, employee)
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

pub fn handle_config_command(data_path: &DataPath, command: &ConfigCommands) -> io::Result<()> {
    match command {
        ConfigCommands::Get { key } => {
            let config = ConfigService::load_config(data_path)?;
            match key.as_str() {
                "allowed_domains" => {
                    println!("{:?}", config.allowed_domains);
                }
                _ => {
                    println!("Unknown key: {key}");
                }
            }
        }
        ConfigCommands::Set { key, value } => {
            let mut config = ConfigService::load_config(data_path)?;
            match key.as_str() {
                "allowed_domains" => {
                    config.allowed_domains =
                        value.split(',').map(|s| s.trim().to_string()).collect();
                    ConfigService::save_config(&config, data_path)?;
                    println!("allowed_domains set to: {:?}", config.allowed_domains);
                }
                _ => {
                    println!("Unknown key: {key}");
                }
            }
        }
    }
    Ok(())
}
