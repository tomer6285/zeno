use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisualizerType {
    Rain,
    Fire,
    Wave,
    Off,
}

impl VisualizerType {
    pub fn name(&self) -> &'static str {
        match self {
            VisualizerType::Rain => "Rainfall",
            VisualizerType::Fire => "Campfire",
            VisualizerType::Wave => "Spectrum Wave",
            VisualizerType::Off => "Disabled",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            VisualizerType::Rain => VisualizerType::Fire,
            VisualizerType::Fire => VisualizerType::Wave,
            VisualizerType::Wave => VisualizerType::Off,
            VisualizerType::Off => VisualizerType::Rain,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: String,
    #[serde(default = "default_visualizer")]
    pub visualizer: VisualizerType,
    #[serde(default)]
    pub binaural_hz: Option<f32>,
    pub tracks: HashMap<String, f32>,
    #[serde(skip)]
    #[allow(dead_code)]
    pub is_builtin: bool,
}

fn default_visualizer() -> VisualizerType {
    VisualizerType::Rain
}

impl Preset {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        visualizer: VisualizerType,
        binaural_hz: Option<f32>,
        tracks: &[(&str, f32)],
        is_builtin: bool,
    ) -> Self {
        let mut track_map = HashMap::new();
        for (k, v) in tracks {
            track_map.insert(k.to_string(), *v);
        }
        Self {
            name: name.into(),
            description: description.into(),
            visualizer,
            binaural_hz,
            tracks: track_map,
            is_builtin,
        }
    }

    pub fn factory_presets() -> Vec<Preset> {
        vec![
            Preset::new(
                "Cozy Cabin",
                "Fireplace, gentle rain, and soft forest wind",
                VisualizerType::Fire,
                None,
                &[("fireplace", 0.60), ("rain", 0.35), ("forest", 0.15)],
                true,
            ),
            Preset::new(
                "Midnight Express",
                "Deep brown noise rumble with night rain on the window",
                VisualizerType::Wave,
                None,
                &[("brown_noise", 0.50), ("rain", 0.30), ("white_noise", 0.10)],
                true,
            ),
            Preset::new(
                "Rainy Cafe",
                "Warm cafe chatter and gentle rain drops",
                VisualizerType::Rain,
                None,
                &[("cafe", 0.45), ("rain", 0.40)],
                true,
            ),
            Preset::new(
                "Deep Space",
                "Cosmic brown noise drone and 40Hz Gamma focus tone",
                VisualizerType::Wave,
                Some(40.0),
                &[("brown_noise", 0.70), ("white_noise", 0.10), ("binaural", 0.30)],
                true,
            ),
            Preset::new(
                "Zen Garden",
                "Breeze rustling bamboo, soft rain, and 10Hz Alpha tone",
                VisualizerType::Rain,
                Some(10.0),
                &[("forest", 0.50), ("rain", 0.20), ("binaural", 0.25)],
                true,
            ),
            Preset::new(
                "Ocean Slumber",
                "Slow rhythmic ocean swells and soothing pink noise",
                VisualizerType::Wave,
                None,
                &[("ocean_waves", 0.65), ("pink_noise", 0.25)],
                true,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_serialization() {
        let preset = Preset::new(
            "Late Night Hack",
            "Rain, fire, and 40Hz",
            VisualizerType::Rain,
            Some(40.0),
            &[("rain", 0.60), ("fireplace", 0.25), ("brown_noise", 0.15)],
            false,
        );

        let toml_str = toml::to_string(&preset).unwrap();
        assert!(toml_str.contains("Late Night Hack"));
        assert!(toml_str.contains("visualizer = \"rain\""));
        assert!(toml_str.contains("binaural_hz = 40.0"));

        let deserialized: Preset = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.name, "Late Night Hack");
        assert_eq!(deserialized.visualizer, VisualizerType::Rain);
        assert_eq!(deserialized.binaural_hz, Some(40.0));
        assert_eq!(deserialized.tracks.get("rain"), Some(&0.60));
    }

    #[test]
    fn test_factory_presets() {
        let factory = Preset::factory_presets();
        assert!(factory.len() >= 5);
        assert!(factory.iter().any(|p| p.name == "Cozy Cabin"));
        assert!(factory.iter().any(|p| p.name == "Midnight Express"));
        assert!(factory.iter().any(|p| p.name == "Deep Space"));
    }
}
