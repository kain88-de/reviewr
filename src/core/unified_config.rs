use crate::core::gerrit::GerritConfig;
use crate::core::models::DataPath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

/// Global settings that apply across all platforms
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalSettings {
    #[serde(default)]
    pub allowed_domains: Vec<String>,
}

/// Unified configuration supporting multiple review platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConfig {
    pub platforms: PlatformConfigs,
    #[serde(default)]
    pub global_settings: GlobalSettings,
    pub ui_preferences: UiPreferences,
    #[serde(default)]
    pub version: u32,
}

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            platforms: PlatformConfigs::default(),
            global_settings: GlobalSettings::default(),
            ui_preferences: UiPreferences::default(),
            version: 1,
        }
    }
}

/// Configuration for all supported platforms
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformConfigs {
    pub gerrit: Option<GerritConfig>,
    #[serde(default)]
    pub jira: Option<JiraConfig>,
}

/// JIRA platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    pub jira_url: String,
    pub username: String,
    pub api_token: String,
    #[serde(default)]
    pub project_filter: Vec<String>,
    #[serde(default)]
    pub custom_fields: HashMap<String, String>,
}

/// UI preferences and customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    #[serde(default = "default_time_period")]
    pub default_time_period_days: u32,
    #[serde(default = "default_true")]
    pub show_platform_icons: bool,
    #[serde(default)]
    pub preferred_platform_order: Vec<String>,
    #[serde(default)]
    pub theme: UiTheme,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            default_time_period_days: 30,
            show_platform_icons: true,
            preferred_platform_order: vec!["gerrit".to_string(), "jira".to_string()],
            theme: UiTheme::Default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum UiTheme {
    #[default]
    Default,
    Dark,
    Light,
    HighContrast,
}

fn default_time_period() -> u32 {
    30
}
fn default_true() -> bool {
    true
}

/// Service for managing unified configuration
pub struct UnifiedConfigService;

impl UnifiedConfigService {
    /// Load unified configuration, migrating from legacy if needed
    pub fn load_config(data_path: &DataPath) -> io::Result<UnifiedConfig> {
        let unified_config_path = data_path.root.join("config.toml");

        // Try to load unified config first
        if unified_config_path.exists() {
            let content = std::fs::read_to_string(&unified_config_path)?;
            let config: UnifiedConfig = toml::from_str(&content).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid unified config format: {e}"),
                )
            })?;
            return Ok(config);
        }

        // If no unified config, create default
        Self::create_default_config(data_path)
    }

    /// Save unified configuration
    pub fn save_config(config: &UnifiedConfig, data_path: &DataPath) -> io::Result<()> {
        let config_path = data_path.root.join("config.toml");
        let toml_content = toml::to_string_pretty(config).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize config: {e}"),
            )
        })?;

        std::fs::write(&config_path, toml_content)?;
        log::info!("Saved unified configuration to {}", config_path.display());
        Ok(())
    }

    /// Create default configuration if none exists
    fn create_default_config(_data_path: &DataPath) -> io::Result<UnifiedConfig> {
        Ok(UnifiedConfig::default())
    }

    /// Load Gerrit configuration from unified config
    pub fn load_gerrit_config(data_path: &DataPath) -> io::Result<Option<GerritConfig>> {
        let config = Self::load_config(data_path)?;
        if let Some(gerrit_config) = config.platforms.gerrit {
            log::info!("Loaded Gerrit config from unified config");
            Ok(Some(gerrit_config))
        } else {
            log::info!("Gerrit config not found in unified config");
            Ok(None)
        }
    }

    /// Load JIRA configuration from unified config
    pub fn load_jira_config(data_path: &DataPath) -> io::Result<Option<JiraConfig>> {
        let config = Self::load_config(data_path)?;
        if let Some(jira_config) = config.platforms.jira {
            log::info!("Loaded JIRA config from unified config");
            Ok(Some(jira_config))
        } else {
            log::info!("JIRA config not found in unified config");
            Ok(None)
        }
    }
}
