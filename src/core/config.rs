use crate::core::models::{Config, DataPath};
use std::fs;
use std::io;

pub struct ConfigService;

impl ConfigService {
    pub fn load_config(data_path: &DataPath) -> io::Result<Config> {
        let config_path = data_path.config_path();
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

    pub fn save_config(config: &Config, data_path: &DataPath) -> io::Result<()> {
        let config_path = data_path.config_path();
        let toml = toml::to_string(config).unwrap();
        fs::write(config_path, toml)?;
        Ok(())
    }
}
