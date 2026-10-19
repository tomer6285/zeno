use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrackId {
    Rain,
    Fireplace,
    Forest,
    Cafe,
    WhiteNoise,
    PinkNoise,
    BrownNoise,
    OceanWaves,
    Binaural,
    Custom(String),
}

impl TrackId {
    pub fn as_str(&self) -> &str {
        match self {
            TrackId::Rain => "rain",
            TrackId::Fireplace => "fireplace",
            TrackId::Forest => "forest",
            TrackId::Cafe => "cafe",
            TrackId::WhiteNoise => "white_noise",
            TrackId::PinkNoise => "pink_noise",
            TrackId::BrownNoise => "brown_noise",
            TrackId::OceanWaves => "ocean_waves",
            TrackId::Binaural => "binaural",
            TrackId::Custom(s) => s.as_str(),
        }
    }

    #[allow(dead_code)]
    pub fn from_str_id(s: &str) -> Self {
        match s {
            "rain" => TrackId::Rain,
            "fireplace" => TrackId::Fireplace,
            "forest" => TrackId::Forest,
            "cafe" => TrackId::Cafe,
            "white_noise" => TrackId::WhiteNoise,
            "pink_noise" => TrackId::PinkNoise,
            "brown_noise" => TrackId::BrownNoise,
            "ocean_waves" => TrackId::OceanWaves,
            "binaural" => TrackId::Binaural,
            other => TrackId::Custom(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackCategory {
    Nature,
    Noise,
    Urban,
    Synthesis,
    Custom,
}

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub id: TrackId,
    pub name: String,
    pub description: String,
    #[allow(dead_code)]
    pub category: TrackCategory,
    pub volume: f32,          // Target volume [0.0, 1.0]
    pub current_volume: f32,  // Actual interpolated volume for smoothing
    pub enabled: bool,
    pub binaural_hz: f32,     // If binaural: beat frequency (e.g. 10.0 or 40.0 Hz)
    pub custom_path: Option<PathBuf>,
}

impl TrackInfo {
    pub fn new(
        id: TrackId,
        name: impl Into<String>,
        description: impl Into<String>,
        category: TrackCategory,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            category,
            volume: 0.5,
            current_volume: 0.0,
            enabled: false,
            binaural_hz: 10.0,
            custom_path: None,
        }
    }
}
