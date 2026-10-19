use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::audio::presets::{Preset, VisualizerType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_volume")]
    pub master_volume: f32,
    #[serde(default)]
    pub muted: bool,
    #[serde(default = "default_work_mins")]
    pub work_duration_mins: u64,
    #[serde(default = "default_short_break_mins")]
    pub short_break_mins: u64,
    #[serde(default = "default_long_break_mins")]
    pub long_break_mins: u64,
    #[serde(default)]
    pub auto_advance_pomodoro: bool,
    #[serde(default = "default_visualizer")]
    pub visualizer: VisualizerType,
    #[serde(default)]
    pub presets: Vec<Preset>,
}

fn default_volume() -> f32 {
    0.8
}
fn default_work_mins() -> u64 {
    25
}
fn default_short_break_mins() -> u64 {
    5
}
fn default_long_break_mins() -> u64 {
    15
}
fn default_visualizer() -> VisualizerType {
    VisualizerType::Rain
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            master_volume: default_volume(),
            muted: false,
            work_duration_mins: default_work_mins(),
            short_break_mins: default_short_break_mins(),
            long_break_mins: default_long_break_mins(),
            auto_advance_pomodoro: false,
            visualizer: default_visualizer(),
            presets: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".config"));
        base.join("zeno").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
        AppConfig::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, content)?;
        Ok(())
    }
}
