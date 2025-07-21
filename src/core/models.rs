use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Employee {
    pub name: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DataPath {
    pub root: PathBuf,
    pub employees_dir: PathBuf,
    pub notes_dir: PathBuf,
}

impl DataPath {
    pub fn new(data_path: Option<PathBuf>) -> Self {
        let root = match data_path {
            Some(path) => path,
            None => dirs::home_dir().unwrap().join(".reviewr"),
        };
        let employees_dir = root.join("employees");
        let notes_dir = root.join("notes");

        Self {
            root,
            employees_dir,
            notes_dir,
        }
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }
}
