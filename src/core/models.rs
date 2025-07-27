use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Employee {
    pub name: String,
    pub title: String,
    pub committer_email: Option<String>,
}


#[derive(Debug, Clone)]
pub struct DataPath {
    pub root: PathBuf,
    pub employees_dir: PathBuf,
    pub notes_dir: PathBuf,
}

impl DataPath {
    pub fn new(data_path: Option<PathBuf>) -> io::Result<Self> {
        let root = match data_path {
            Some(path) => path,
            None => dirs::home_dir()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "Home directory not found. Please specify --data-path.",
                    )
                })?
                .join(".reviewr"),
        };
        let employees_dir = root.join("employees");
        let notes_dir = root.join("notes");

        Ok(Self {
            root,
            employees_dir,
            notes_dir,
        })
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }
}

pub fn validate_employee_name(name: &str) -> io::Result<()> {
    if name.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Employee name cannot be empty",
        ));
    }

    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    if name.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Employee name contains invalid characters (/, \\, :, *, ?, \", <, >, |)",
        ));
    }

    if name.len() > 255 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Employee name too long (max 255 characters)",
        ));
    }

    Ok(())
}

pub fn validate_domain(domain: &str) -> io::Result<()> {
    if domain.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Domain cannot be empty",
        ));
    }

    // Basic domain validation - check for valid characters and structure
    if !domain
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Domain contains invalid characters",
        ));
    }

    if domain.starts_with('.')
        || domain.ends_with('.')
        || domain.starts_with('-')
        || domain.ends_with('-')
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Domain format is invalid",
        ));
    }

    if domain.contains("..") || domain.contains("--") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Domain format is invalid",
        ));
    }

    Ok(())
}
