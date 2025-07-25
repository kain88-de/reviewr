use crate::core::config::ConfigService;
use crate::core::models::DataPath;
use log::{info, warn};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

pub struct NotesService;

impl NotesService {
    pub fn open_notes(data_path: &DataPath, employee_name: &str) -> io::Result<()> {
        let note_path = data_path.notes_dir.join(format!("{employee_name}.md"));
        if !note_path.exists() {
            info!("Creating new notes file for employee: {employee_name}");
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
                        if Self::is_domain_allowed(domain, &config.allowed_domains) {
                            info!("Adding evidence URL from clipboard: {url} (domain: {domain})");
                            let mut file = fs::OpenOptions::new().append(true).open(&note_path)?;
                            writeln!(file, "- Evidence: {url}")?;
                        } else {
                            warn!(
                                "Clipboard URL domain '{}' not in allowed domains: {:?}",
                                domain, config.allowed_domains
                            );
                        }
                    }
                } else {
                    info!("Clipboard content is not a valid URL, skipping evidence insertion");
                }
            }
        }

        let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        info!(
            "Opening notes file {} with editor: {}",
            note_path.display(),
            editor
        );
        Command::new(editor).arg(&note_path).status()?;

        Ok(())
    }

    fn is_domain_allowed(domain: &str, allowed_domains: &[String]) -> bool {
        allowed_domains.is_empty()
            || allowed_domains
                .iter()
                .any(|d| domain == d || domain.ends_with(&format!(".{d}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_matching() {
        let allowed_domains = vec!["company.example.com".to_string()];

        // Exact match should work
        assert!(NotesService::is_domain_allowed(
            "company.example.com",
            &allowed_domains
        ));

        // Subdomain should work
        assert!(NotesService::is_domain_allowed(
            "review.company.example.com",
            &allowed_domains
        ));

        // Different domain should not work
        assert!(!NotesService::is_domain_allowed(
            "other.com",
            &allowed_domains
        ));

        // Partial match should not work
        assert!(!NotesService::is_domain_allowed(
            "example.com",
            &allowed_domains
        ));

        // Empty allowed domains should allow everything
        assert!(NotesService::is_domain_allowed("any.domain.com", &[]));
    }
}
