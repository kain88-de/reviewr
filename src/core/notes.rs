use crate::core::config::ConfigService;
use crate::core::models::DataPath;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

pub struct NotesService;

impl NotesService {
    pub fn open_notes(data_path: &DataPath, employee_name: &str) -> io::Result<()> {
        let note_path = data_path.notes_dir.join(format!("{employee_name}.md"));
        if !note_path.exists() {
            let mut file = fs::File::create(&note_path)?;
            let now = chrono::Local::now();
            writeln!(
                file,
                "# Notes for {employee_name}\n\n## {}\n\n",
                now.format("%Y-%m-%d")
            )?;
        }

        let config = ConfigService::load_config(data_path)?;
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
}
