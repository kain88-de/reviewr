use crate::core::gerrit::GerritConfig;
use crate::core::models::DataPath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

/// Unified configuration supporting multiple review platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConfig {
    pub platforms: PlatformConfigs,
    pub ui_preferences: UiPreferences,
    #[serde(default)]
    pub version: u32,
}

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            platforms: PlatformConfigs::default(),
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
    #[serde(default)]
    pub gitlab: Option<GitLabConfig>,
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

/// GitLab platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    pub gitlab_url: String,
    pub access_token: String,
    pub username: String,
    #[serde(default)]
    pub group_filter: Vec<String>,
    #[serde(default)]
    pub project_filter: Vec<String>,
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
            preferred_platform_order: vec![
                "gerrit".to_string(),
                "jira".to_string(),
                "gitlab".to_string(),
            ],
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

        // If no unified config, try to migrate from legacy configs
        Self::migrate_from_legacy(data_path)
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

    /// Migrate from legacy configuration files
    fn migrate_from_legacy(data_path: &DataPath) -> io::Result<UnifiedConfig> {
        let mut config = UnifiedConfig::default();

        // Try to load legacy Gerrit config
        let gerrit_config_path = data_path.root.join("gerrit_config.toml");
        if gerrit_config_path.exists() {
            let content = std::fs::read_to_string(&gerrit_config_path)?;
            let gerrit_config: GerritConfig = toml::from_str(&content).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid Gerrit config format: {e}"),
                )
            })?;
            config.platforms.gerrit = Some(gerrit_config);
            log::info!("Migrated Gerrit configuration from legacy file");
        }

        // Try to load legacy main config (allowed_domains)
        let main_config_path = data_path.config_path();
        if main_config_path.exists() {
            let content = std::fs::read_to_string(&main_config_path)?;
            if let Ok(legacy_config) = toml::from_str::<crate::core::models::Config>(&content) {
                // Preserve allowed_domains in UI preferences or platform-specific config
                log::info!(
                    "Found legacy config with allowed_domains: {:?}",
                    legacy_config.allowed_domains
                );
                // TODO: Decide where to store allowed_domains in unified config
            }
        }

        Ok(config)
    }

    /// Get configuration for a specific platform
    pub fn get_platform_config<'a, T>(config: &'a UnifiedConfig, platform_id: &str) -> Option<&'a T>
    where
        T: 'static,
    {
        match platform_id {
            "gerrit" => {
                config.platforms.gerrit.as_ref().and_then(|c| {
                    // Safe cast since we know the type
                    (c as &dyn std::any::Any).downcast_ref::<T>()
                })
            }
            "jira" => config
                .platforms
                .jira
                .as_ref()
                .and_then(|c| (c as &dyn std::any::Any).downcast_ref::<T>()),
            "gitlab" => config
                .platforms
                .gitlab
                .as_ref()
                .and_then(|c| (c as &dyn std::any::Any).downcast_ref::<T>()),
            _ => None,
        }
    }

    /// Update configuration for a specific platform
    pub fn update_platform_config(
        &self,
        config: &mut UnifiedConfig,
        platform_id: &str,
        platform_config: serde_json::Value,
    ) -> io::Result<()> {
        match platform_id {
            "gerrit" => {
                let gerrit_config: GerritConfig =
                    serde_json::from_value(platform_config).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid Gerrit config: {e}"),
                        )
                    })?;
                config.platforms.gerrit = Some(gerrit_config);
            }
            "jira" => {
                let jira_config: JiraConfig =
                    serde_json::from_value(platform_config).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid JIRA config: {e}"),
                        )
                    })?;
                config.platforms.jira = Some(jira_config);
            }
            "gitlab" => {
                let gitlab_config: GitLabConfig =
                    serde_json::from_value(platform_config).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid GitLab config: {e}"),
                        )
                    })?;
                config.platforms.gitlab = Some(gitlab_config);
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown platform: {platform_id}"),
                ));
            }
        }
        Ok(())
    }

    /// Validate configuration for a platform
    pub fn validate_platform_config(
        platform_id: &str,
        config: &serde_json::Value,
    ) -> io::Result<()> {
        match platform_id {
            "gerrit" => {
                let _: GerritConfig = serde_json::from_value(config.clone()).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid Gerrit config: {e}"),
                    )
                })?;
                // Additional validation could go here
            }
            "jira" => {
                let jira_config: JiraConfig =
                    serde_json::from_value(config.clone()).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid JIRA config: {e}"),
                        )
                    })?;

                // Validate URL format
                if !jira_config.jira_url.starts_with("http") {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "JIRA URL must start with http:// or https://",
                    ));
                }

                // Validate required fields
                if jira_config.username.trim().is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "JIRA username cannot be empty",
                    ));
                }

                if jira_config.api_token.trim().is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "JIRA API token cannot be empty",
                    ));
                }
            }
            "gitlab" => {
                let gitlab_config: GitLabConfig =
                    serde_json::from_value(config.clone()).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid GitLab config: {e}"),
                        )
                    })?;

                // Validate URL format
                if !gitlab_config.gitlab_url.starts_with("http") {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "GitLab URL must start with http:// or https://",
                    ));
                }

                // Validate required fields
                if gitlab_config.access_token.trim().is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "GitLab access token cannot be empty",
                    ));
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown platform: {platform_id}"),
                ));
            }
        }
        Ok(())
    }

    /// Get enabled platforms from configuration
    pub fn get_enabled_platforms(config: &UnifiedConfig) -> Vec<String> {
        let mut platforms = Vec::new();

        if config.platforms.gerrit.is_some() {
            platforms.push("gerrit".to_string());
        }
        if config.platforms.jira.is_some() {
            platforms.push("jira".to_string());
        }
        if config.platforms.gitlab.is_some() {
            platforms.push("gitlab".to_string());
        }

        platforms
    }

}
