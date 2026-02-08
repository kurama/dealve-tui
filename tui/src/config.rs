use crate::model::{SortCriteria, SortDirection, SortState};
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
    /// Number of deals to load per page (pagination batch size)
    #[serde(default = "default_page_size")]
    pub deals_page_size: usize,
    /// Debounce delay (ms) before loading game info after selection change
    #[serde(default = "default_game_info_delay")]
    pub game_info_delay_ms: u64,
    /// IsThereAnyDeal API key (optional, can also be set via ITAD_API_KEY env var)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Default sort criteria (Price, Cut, Hottest, Release, Expiring, Popular)
    #[serde(default = "default_sort_criteria")]
    pub default_sort_criteria: String,
    /// Default sort direction (Ascending or Descending)
    #[serde(default = "default_sort_direction")]
    pub default_sort_direction: String,
}

fn default_region() -> String {
    Region::default().code().to_string()
}

fn default_page_size() -> usize {
    50
}

fn default_game_info_delay() -> u64 {
    200
}

fn default_sort_criteria() -> String {
    "Price".to_string()
}

fn default_sort_direction() -> String {
    "Ascending".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_platform: "All".to_string(),
            enabled_platforms: Platform::ALL.iter().map(|p| p.name().to_string()).collect(),
            region: default_region(),
            deals_page_size: default_page_size(),
            game_info_delay_ms: default_game_info_delay(),
            api_key: None,
            default_sort_criteria: default_sort_criteria(),
            default_sort_direction: default_sort_direction(),
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
    pub fn update_from_options(
        &mut self,
        default_platform: Platform,
        enabled_platforms: &HashSet<Platform>,
        region: Region,
        default_sort: SortState,
    ) {
        self.default_platform = default_platform.name().to_string();
        self.enabled_platforms = enabled_platforms
            .iter()
            .map(|p| p.name().to_string())
            .collect();
        self.region = region.code().to_string();
        self.default_sort_criteria = default_sort.criteria.name().to_string();
        self.default_sort_direction = match default_sort.direction {
            SortDirection::Ascending => "Ascending".to_string(),
            SortDirection::Descending => "Descending".to_string(),
        };
    }

    /// Get the default sort state from config
    pub fn get_default_sort(&self) -> SortState {
        let criteria = match self.default_sort_criteria.as_str() {
            "Price" => SortCriteria::Price,
            "Cut" => SortCriteria::Cut,
            "Hottest" => SortCriteria::Hottest,
            "Release" => SortCriteria::ReleaseDate,
            "Expiring" => SortCriteria::Expiring,
            "Popular" => SortCriteria::Popular,
            _ => SortCriteria::Price,
        };
        let direction = match self.default_sort_direction.as_str() {
            "Descending" => SortDirection::Descending,
            _ => SortDirection::Ascending,
        };
        SortState {
            criteria,
            direction,
        }
    }

    /// Set the API key and save to config
    pub fn set_api_key(&mut self, key: String) -> Result<(), std::io::Error> {
        self.api_key = Some(key);
        self.save()
    }

    /// Load API key from environment variable or config file
    /// Priority: 1. ITAD_API_KEY env var, 2. config file
    pub fn load_api_key() -> Option<String> {
        // Priority 1: Environment variable
        if let Ok(key) = std::env::var("ITAD_API_KEY") {
            if !key.is_empty() {
                return Some(key);
            }
        }

        // Priority 2: Config file
        let config = Self::load();
        config.api_key.filter(|k| !k.is_empty())
    }
}
