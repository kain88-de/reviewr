pub mod cli;
pub mod core;
pub mod terminal;
pub mod tui;

use clap::Parser;
use cli::{
    Cli, Commands, handle_add_command, handle_config_command, handle_edit_command,
    handle_list_command, handle_notes_command, handle_review_command,
};
use core::models::DataPath;
use std::fs;
use std::io;
use tui::EmployeeSelector;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize logging
    env_logger::init();

    let cli = Cli::parse();
    let data_path = DataPath::new(cli.data_path)?;

    fs::create_dir_all(&data_path.employees_dir)?;
    fs::create_dir_all(&data_path.notes_dir)?;

    match &cli.command {
        Commands::Add { employee } => {
            handle_add_command(&data_path, employee)?;
        }
        Commands::Notes { employee } => {
            if let Some(employee_name) = employee {
                handle_notes_command(&data_path, employee_name)?;
            } else {
                let mut selector = EmployeeSelector::new(&data_path)?;
                if let Some(selected_employee) = selector.run()? {
                    handle_notes_command(&data_path, &selected_employee)?;
                }
            }
        }
        Commands::Edit { employee } => {
            handle_edit_command(&data_path, employee)?;
        }
        Commands::List => {
            handle_list_command(&data_path)?;
        }
        Commands::Review { employee } => {
            handle_review_command(&data_path, employee).await?;
        }
        Commands::Config { command } => {
            handle_config_command(&data_path, command)?;
        }
    }

    Ok(())
}
