use dealve_core::models::{Platform, Region};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Persistent configuration saved to disk
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_platform: String,
    pub enabled_platforms: Vec<String>,
    #[serde(default = "default_region")]
    pub region: String,
}

fn default_region() -> String {
    Region::default().code().to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_platform: "All".to_string(),
            enabled_platforms: Platform::ALL.iter().map(|p| p.name().to_string()).collect(),
            region: default_region(),
        }
    }
}

impl Config {
    /// Get the config file path (~/.config/dealve/config.json)
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("dealve").join("config.json"))
    }

    /// Load config from disk, or return default if not found
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save config to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let Some(path) = Self::config_path() else {
            return Ok(());
        };

        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Convert default_platform string to Platform enum
    pub fn get_default_platform(&self) -> Platform {
        Platform::ALL
            .iter()
            .find(|p| p.name() == self.default_platform)
            .copied()
            .unwrap_or(Platform::All)
    }

    /// Convert enabled_platforms strings to Platform HashSet
    pub fn get_enabled_platforms(&self) -> HashSet<Platform> {
        self.enabled_platforms
            .iter()
            .filter_map(|name| Platform::ALL.iter().find(|p| p.name() == name).copied())
            .collect()
    }

    /// Get the region from config
    pub fn get_region(&self) -> Region {
        Region::from_code(&self.region).unwrap_or_default()
    }

    /// Update from OptionsState
    pub fn update_from_options(&mut self, default_platform: Platform, enabled_platforms: &HashSet<Platform>, region: Region) {
        self.default_platform = default_platform.name().to_string();
        self.enabled_platforms = enabled_platforms
            .iter()
            .map(|p| p.name().to_string())
            .collect();
        self.region = region.code().to_string();
    }
}
