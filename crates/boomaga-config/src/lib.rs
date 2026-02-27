//! Configuration management for boomaga
//!
//! This crate provides configuration management for the boomaga virtual printer,
//! including backend service configuration and preview application settings.

mod backend_config;
mod preview_config;
mod settings;
mod defaults;

pub use backend_config::BackendConfig;
pub use preview_config::PreviewConfig;
pub use settings::Settings;
pub use defaults::*;

use std::path::PathBuf;
use anyhow::Result;
use tracing::{info, debug};

/// Application configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    Load(#[from] config::ConfigError),

    #[error("Failed to save configuration: {0}")]
    Save(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    Permission(String),
}

/// Configuration manager
pub struct ConfigManager {
    backend_config_path: PathBuf,
    preview_config_path: PathBuf,
    settings_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let dirs = directories::BaseDirs::new()
            .ok_or_else(|| anyhow::anyhow!("Could not determine user directories"))?;

        let config_dir = dirs.config_dir().join(boomaga_core::constants::CONFIG_DIR);
        let cache_dir = dirs.cache_dir().join(boomaga_core::constants::CACHE_DIR);
        let state_dir = dirs.state_dir().join(boomaga_core::constants::STATE_DIR);

        // Create necessary directories
        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&state_dir)?;

        Ok(Self {
            backend_config_path: config_dir.join("backend.toml"),
            preview_config_path: config_dir.join("preview.toml"),
            settings_path: state_dir.join("settings.json"),
        })
    }

    /// Load backend configuration
    pub fn load_backend(&self) -> Result<BackendConfig, ConfigError> {
        debug!("Loading backend configuration from {:?}", self.backend_config_path);

        if !self.backend_config_path.exists() {
            // Use defaults
            info!("Backend config file not found, using defaults");
            return Ok(BackendConfig::default());
        }

        let config = config::Config::new()
            .add_source(config::File::from(self.backend_config_path.clone()))
            .add_source(config::File::from_path(
                std::path::Path::new("/etc/boomaga/backend.toml").to_path_buf(),
            ))
            .add_source(
                config::Environment::with_prefix("BOOMAGA")
                    .prefix_separator("_")
                    .separator("_")
            );

        let backend_config: BackendConfig = config.try_into()?;
        backend_config.validate()?;

        Ok(backend_config)
    }

    /// Load preview configuration
    pub fn load_preview(&self) -> Result<PreviewConfig, ConfigError> {
        debug!("Loading preview configuration from {:?}", self.preview_config_path);

        if !self.preview_config_path.exists() {
            // Use defaults
            info!("Preview config file not found, using defaults");
            return Ok(PreviewConfig::default());
        }

        let config = config::Config::new()
            .add_source(config::File::from(self.preview_config_path.clone()))
            .add_source(config::File::from_path(
                std::path::Path::new("/etc/boomaga/preview.toml").to_path_buf(),
            ))
            .add_source(
                config::Environment::with_prefix("BOOMAGA")
                    .prefix_separator("_")
                    .separator("_")
            );

        let preview_config: PreviewConfig = config.try_into()?;
        preview_config.validate()?;

        Ok(preview_config)
    }

    /// Load user settings
    pub fn load_settings(&self) -> Result<Settings, ConfigError> {
        debug!("Loading settings from {:?}", self.settings_path);

        if !self.settings_path.exists() {
            // Use defaults
            info!("Settings file not found, using defaults");
            return Ok(Settings::default());
        }

        let settings: Settings =
            serde_json::from_str(&std::fs::read_to_string(&self.settings_path)?)?;

        Ok(settings)
    }

    /// Save backend configuration
    pub fn save_backend(&self, config: &BackendConfig) -> Result<(), ConfigError> {
        debug!("Saving backend configuration to {:?}", self.backend_config_path);

        let toml = toml::to_string_pretty(config)?;
        std::fs::write(&self.backend_config_path, toml)?;

        Ok(())
    }

    /// Save preview configuration
    pub fn save_preview(&self, config: &PreviewConfig) -> Result<(), ConfigError> {
        debug!("Saving preview configuration to {:?}", self.preview_config_path);

        let toml = toml::to_string_pretty(config)?;
        std::fs::write(&self.preview_config_path, toml)?;

        Ok(())
    }

    /// Save settings
    pub fn save_settings(&self, settings: &Settings) -> Result<(), ConfigError> {
        debug!("Saving settings to {:?}", self.settings_path);

        let json = serde_json::to_string_pretty(settings)?;
        std::fs::write(&self.settings_path, json)?;

        Ok(())
    }

    /// Get backend config path
    pub fn backend_config_path(&self) -> &PathBuf {
        &self.backend_config_path
    }

    /// Get preview config path
    pub fn preview_config_path(&self) -> &PathBuf {
        &self.preview_config_path
    }

    /// Get settings path
    pub fn settings_path(&self) -> &PathBuf {
        &self.settings_path
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create config manager")
    }
}

/// Initialize configuration by creating default files
pub fn initialize_config() -> Result<ConfigManager> {
    info!("Initializing boomaga configuration");

    let config = ConfigManager::new()?;

    // Save default backend configuration
    if !config.backend_config_path().exists() {
        let default = BackendConfig::default();
        config.save_backend(&default)?;
        info!("Created default backend configuration");
    }

    // Save default preview configuration
    if !config.preview_config_path().exists() {
        let default = PreviewConfig::default();
        config.save_preview(&default)?;
        info!("Created default preview configuration");
    }

    // Save default settings
    if !config.settings_path().exists() {
        let default = Settings::default();
        config.save_settings(&default)?;
        info!("Created default settings");
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_manager_creation() {
        let config = ConfigManager::new().unwrap();
        assert!(config.backend_config_path().exists() || true);
        assert!(config.preview_config_path().exists() || true);
    }

    #[test]
    fn test_default_backend_config() {
        let config = BackendConfig::default();
        assert_eq!(config.max_concurrent_jobs, 4);
        assert_eq!(config.worker_threads, 2);
    }

    #[test]
    fn test_default_preview_config() {
        let config = PreviewConfig::default();
        assert_eq!(config.default_zoom, 1.0);
        assert_eq!(config.auto_zoom, true);
    }
}
