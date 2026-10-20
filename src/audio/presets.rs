use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualizerType {
    #[serde(alias = "rain", alias = "rainfall")]
    Rain,
    #[serde(alias = "window", alias = "windowpane")]
    Window,
    #[serde(alias = "ripples", alias = "pond")]
    Ripples,
    #[serde(alias = "shore", alias = "ocean", alias = "shoreline")]
    Shore,
    #[serde(alias = "fire", alias = "campfire")]
    Fire,
    #[serde(alias = "fireflies", alias = "motes")]
    Fireflies,
    #[serde(alias = "leaves")]
    Leaves,
    #[serde(alias = "aurora", alias = "northern_lights")]
    Aurora,
    #[serde(alias = "starfield", alias = "space", alias = "deep_space")]
    Starfield,
    #[serde(alias = "wave", alias = "spectrum", alias = "spectrum_wave")]
    Wave,
    #[serde(alias = "mandala", alias = "breathing")]
    Mandala,
    #[serde(alias = "life", alias = "game_of_life")]
    Life,
    #[serde(alias = "off", alias = "disabled", alias = "none")]
    Off,
}

impl VisualizerType {
    pub const ALL: &'static [VisualizerType] = &[
        VisualizerType::Rain,
        VisualizerType::Window,
        VisualizerType::Ripples,
        VisualizerType::Shore,
        VisualizerType::Fire,
        VisualizerType::Fireflies,
        VisualizerType::Leaves,
        VisualizerType::Aurora,
        VisualizerType::Starfield,
        VisualizerType::Wave,
        VisualizerType::Mandala,
        VisualizerType::Life,
        VisualizerType::Off,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            VisualizerType::Rain => "Rainfall",
            VisualizerType::Window => "Windowpane",
            VisualizerType::Ripples => "Zen Pond Ripples",
            VisualizerType::Shore => "Ocean Shoreline",
            VisualizerType::Fire => "Campfire",
            VisualizerType::Fireflies => "Fireflies",
            VisualizerType::Leaves => "Drifting Leaves",
            VisualizerType::Aurora => "Aurora Borealis",
            VisualizerType::Starfield => "Deep Space Starfield",
            VisualizerType::Wave => "Spectrum Wave",
            VisualizerType::Mandala => "Breathing Mandala",
            VisualizerType::Life => "Game of Life",
            VisualizerType::Off => "Disabled",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            VisualizerType::Rain => "🌧️",
            VisualizerType::Window => "🪟",
            VisualizerType::Ripples => "💧",
            VisualizerType::Shore => "🌊",
            VisualizerType::Fire => "🔥",
            VisualizerType::Fireflies => "✨",
            VisualizerType::Leaves => "🍃",
            VisualizerType::Aurora => "🌌",
            VisualizerType::Starfield => "⭐",
            VisualizerType::Wave => "📊",
            VisualizerType::Mandala => "🌸",
            VisualizerType::Life => "👾",
            VisualizerType::Off => "⏹️",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            VisualizerType::Rain => "Falling rain streaks with puddle splash ripples",
            VisualizerType::Window => "Gentle condensation and trickling rivulets on glass",
            VisualizerType::Ripples => "Tranquil water garden with expanding droplet rings",
            VisualizerType::Shore => "Rhythmic rolling surf and receding tidal wash",
            VisualizerType::Fire => "Cellular heat simulation with crackling embers",
            VisualizerType::Fireflies => "Bioluminescent motes drifting peacefully in the dark",
            VisualizerType::Leaves => "Autumn breeze with swaying, falling leaves",
            VisualizerType::Aurora => "Translucent shimmering polar curtains dancing in the night",
            VisualizerType::Starfield => "3D cosmic starfield drifting through deep space",
            VisualizerType::Wave => "Real-time equalizer reacting to track audio frequencies",
            VisualizerType::Mandala => "Paced breathing focus ring to ease tension and anchor attention",
            VisualizerType::Life => "Conway's cellular automata evolving in slow harmony",
            VisualizerType::Off => "Disabled visualizer to minimize motion and CPU usage",
        }
    }

    pub fn companion_tracks(&self) -> &'static str {
        match self {
            VisualizerType::Rain => "Rain / Thunderstorm, Rainy Cafe",
            VisualizerType::Window => "Rain, Cozy Cabin, Cafe chatter",
            VisualizerType::Ripples => "Zen Garden, Soft Rain, 10Hz Alpha tone",
            VisualizerType::Shore => "Ocean Waves, Pink Noise",
            VisualizerType::Fire => "Campfire, Forest Wind, Distant rain",
            VisualizerType::Fireflies => "Forest, Campfire, Night breeze",
            VisualizerType::Leaves => "Forest, Gentle Wind, Birds",
            VisualizerType::Aurora => "Deep Space, 40Hz Gamma tone, Brown Noise",
            VisualizerType::Starfield => "Brown Noise, White Noise, 40Hz Gamma tone",
            VisualizerType::Wave => "Active multi-track frequencies",
            VisualizerType::Mandala => "Binaural Beats, Zen Garden, Calm drone",
            VisualizerType::Life => "White Noise, Brown Noise, Minimal ambient",
            VisualizerType::Off => "None",
        }
    }

    pub fn next(&self) -> Self {
        let idx = Self::ALL.iter().position(|m| m == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(&self) -> Self {
        let idx = Self::ALL.iter().position(|m| m == self).unwrap_or(0);
        if idx == 0 {
            Self::ALL[Self::ALL.len() - 1]
        } else {
            Self::ALL[idx - 1]
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
                VisualizerType::Window,
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
                VisualizerType::Starfield,
                Some(40.0),
                &[("brown_noise", 0.70), ("white_noise", 0.10), ("binaural", 0.30)],
                true,
            ),
            Preset::new(
                "Zen Garden",
                "Breeze rustling bamboo, soft rain, and 10Hz Alpha tone",
                VisualizerType::Ripples,
                Some(10.0),
                &[("forest", 0.50), ("rain", 0.20), ("binaural", 0.25)],
                true,
            ),
            Preset::new(
                "Ocean Slumber",
                "Slow rhythmic ocean swells and soothing pink noise",
                VisualizerType::Shore,
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
