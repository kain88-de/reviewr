use crate::core::models::{Config, DataPath, validate_domain};
use log::{info, warn};
use std::fs;
use std::io;

pub struct ConfigService;

impl ConfigService {
    pub fn load_config(data_path: &DataPath) -> io::Result<Config> {
        let config_path = data_path.config_path();
        if !config_path.exists() {
            info!(
                "Config file not found, creating default config at {}",
                config_path.display()
            );
            let config = Config {
                allowed_domains: vec![],
            };
            let toml = toml::to_string(&config).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to serialize config: {e}"),
                )
            })?;
            let config_dir = config_path
                .parent()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid config path"))?;
            fs::create_dir_all(config_dir)?;
            fs::write(&config_path, toml)?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content).map_err(|e| {
            warn!(
                "Failed to parse config file {}: {}",
                config_path.display(),
                e
            );
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid config file format: {e}"),
            )
        })?;

        // Validate domains in the loaded config
        for domain in &config.allowed_domains {
            if let Err(e) = validate_domain(domain) {
                warn!("Invalid domain in config '{domain}': {e}");
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid domain '{domain}': {e}"),
                ));
            }
        }

        Ok(config)
    }

    pub fn save_config(config: &Config, data_path: &DataPath) -> io::Result<()> {
        // Validate all domains before saving
        for domain in &config.allowed_domains {
            validate_domain(domain)?;
        }

        let config_path = data_path.config_path();
        let toml = toml::to_string(config).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize config: {e}"),
            )
        })?;

        info!("Saving config to {}", config_path.display());
        fs::write(config_path, toml)?;
        Ok(())
    }
}
