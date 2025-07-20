use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "eval")]
#[command(about = "A CLI tool for employee evaluations.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Sets a custom data path
    #[arg(long, value_name = "FILE")]
    data_path: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new employee
    Add {
        /// The name of the employee
        employee: String,
    },
    /// Open notes for an employee
    Notes {
        /// The name of the employee
        employee: String,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
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

#[derive(Serialize, Deserialize)]
struct Employee {
    name: String,
    title: String,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let eval_dir = match cli.data_path {
        Some(path) => path,
        None => dirs::home_dir().unwrap().join(".eval"),
    };
    let employees_dir = eval_dir.join("employees");
    let notes_dir = eval_dir.join("notes");

    fs::create_dir_all(&employees_dir)?;
    fs::create_dir_all(&notes_dir)?;

    match &cli.command {
        Commands::Add { employee } => {
            add_employee(&employees_dir, employee)?;
        }
        Commands::Notes { employee } => {
            let employee_file = employees_dir.join(format!("{}.toml", employee));
            if !employee_file.exists() {
                println!("Employee '{}' not found.", employee);
                print!("Would you like to add them? (y/n) ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if input.trim().eq_ignore_ascii_case("y") {
                    add_employee(&employees_dir, employee)?;
                } else {
                    return Ok(());
                }
            }

            open_notes(&notes_dir, employee)?;
        }
    }

    Ok(())
}

fn add_employee(employees_dir: &Path, employee_name: &str) -> io::Result<()> {
    println!("Adding new employee: {}", employee_name);
    print!("Title: ");
    io::stdout().flush()?;
    let mut title = String::new();
    io::stdin().read_line(&mut title)?;

    let employee = Employee {
        name: employee_name.to_string(),
        title: title.trim().to_string(),
    };

    let toml = toml::to_string(&employee).unwrap();
    let path = employees_dir.join(format!("{}.toml", employee_name));
    fs::write(path, toml)?;
    println!("Employee '{}' added.", employee_name);
    Ok(())
}

fn open_notes(notes_dir: &Path, employee_name: &str) -> io::Result<()> {
    let note_path = notes_dir.join(format!("{}.md", employee_name));
    if !note_path.exists() {
        let mut file = fs::File::create(&note_path)?;
        let now = chrono::Local::now();
        writeln!(file, "# Notes for {}

## {}

", employee_name, now.format("%Y-%m-%d"))?;
    }

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    Command::new(editor)
        .arg(&note_path)
        .status()?;

    Ok(())
}