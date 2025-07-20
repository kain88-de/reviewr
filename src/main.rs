use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "reviewr")]
#[command(about = "A CLI tool for employee reviews.", long_about = None)]
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

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    allowed_domains: Vec<String>,
}

fn get_config_path(data_path: &Option<PathBuf>) -> PathBuf {
    match data_path {
        Some(path) => path.join("config.toml"),
        None => dirs::home_dir()
            .unwrap()
            .join(".reviewr")
            .join("config.toml"),
    }
}

fn load_config(data_path: &Option<PathBuf>) -> io::Result<Config> {
    let config_path = get_config_path(data_path);
    if !config_path.exists() {
        let config = Config {
            allowed_domains: vec![],
        };
        let toml = toml::to_string(&config).unwrap();
        let config_dir = config_path.parent().unwrap();
        fs::create_dir_all(config_dir)?;
        fs::write(&config_path, toml)?;
        return Ok(config);
    }

    let content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content).unwrap();
    Ok(config)
}

fn save_config(config: &Config, data_path: &Option<PathBuf>) -> io::Result<()> {
    let config_path = get_config_path(data_path);
    let toml = toml::to_string(config).unwrap();
    fs::write(config_path, toml)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let eval_dir = match cli.data_path {
        Some(ref path) => path.clone(),
        None => dirs::home_dir().unwrap().join(".reviewr"),
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
            let employee_file = employees_dir.join(format!("{employee}.toml"));
            if !employee_file.exists() {
                println!("Employee '{employee}' not found.");
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

            open_notes(&notes_dir, employee, &cli.data_path)?;
        }
        Commands::Config { command } => match command {
            ConfigCommands::Get { key } => {
                let config = load_config(&cli.data_path)?;
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
                let mut config = load_config(&cli.data_path)?;
                match key.as_str() {
                    "allowed_domains" => {
                        config.allowed_domains =
                            value.split(',').map(|s| s.trim().to_string()).collect();
                        save_config(&config, &cli.data_path)?;
                        println!("allowed_domains set to: {:?}", config.allowed_domains);
                    }
                    _ => {
                        println!("Unknown key: {key}");
                    }
                }
            }
        },
    }

    Ok(())
}

fn add_employee(employees_dir: &Path, employee_name: &str) -> io::Result<()> {
    println!("Adding new employee: {employee_name}");
    print!("Title: ");
    io::stdout().flush()?;
    let mut title = String::new();
    io::stdin().read_line(&mut title)?;

    let employee = Employee {
        name: employee_name.to_string(),
        title: title.trim().to_string(),
    };

    let toml = toml::to_string(&employee).unwrap();
    let path = employees_dir.join(format!("{employee_name}.toml"));
    fs::write(path, toml)?;
    println!("Employee '{employee_name}' added.");
    Ok(())
}

fn open_notes(
    notes_dir: &Path,
    employee_name: &str,
    data_path: &Option<PathBuf>,
) -> io::Result<()> {
    let note_path = notes_dir.join(format!("{employee_name}.md"));
    if !note_path.exists() {
        let mut file = fs::File::create(&note_path)?;
        let now = chrono::Local::now();
        writeln!(
            file,
            "# Notes for {employee_name}\n\n## {}\n\n",
            now.format("%Y-%m-%d")
        )?;
    }

    let config = load_config(data_path)?;
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Ok(text) = clipboard.get_text() {
            if let Ok(url) = url::Url::parse(&text) {
                if let Some(domain) = url.domain() {
                    if config.allowed_domains.is_empty()
                        || config.allowed_domains.iter().any(|d| d == domain)
                    {
                        let mut file = fs::OpenOptions::new().append(true).open(&note_path)?;
                        writeln!(file, "- Evidence: {url}")?;
                    }
                }
            }
        }
    }

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    Command::new(editor).arg(&note_path).status()?;

    Ok(())
}
