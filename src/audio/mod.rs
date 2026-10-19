pub mod engine;
pub mod loader;
pub mod presets;
pub mod synth;
pub mod track;

use std::collections::HashMap;
use engine::AudioEngine;
use loader::discover_user_sounds;
pub use presets::{Preset, VisualizerType};
pub use track::{TrackCategory, TrackId, TrackInfo};

pub struct AudioController {
    pub engine: AudioEngine,
    pub tracks: Vec<TrackInfo>,
    pub factory_presets: Vec<Preset>,
    pub user_presets: Vec<Preset>,
    pub active_preset_name: Option<String>,
}

impl AudioController {
    pub fn new() -> Self {
        let mut tracks = vec![
            TrackInfo::new(TrackId::Rain, "Rain", "Rainfall and distant thunder", TrackCategory::Nature),
            TrackInfo::new(TrackId::Fireplace, "Fireplace", "Cozy wood crackles and embers", TrackCategory::Nature),
            TrackInfo::new(TrackId::Forest, "Forest", "Gentle wind and wild bird chirps", TrackCategory::Nature),
            TrackInfo::new(TrackId::Cafe, "Cafe", "Warm background murmurs and clinks", TrackCategory::Urban),
            TrackInfo::new(TrackId::WhiteNoise, "White Noise", "Uniform broadband static for blocking noise", TrackCategory::Noise),
            TrackInfo::new(TrackId::PinkNoise, "Pink Noise", "Balanced 1/f noise for concentration", TrackCategory::Noise),
            TrackInfo::new(TrackId::BrownNoise, "Brown Noise", "Deep, warm low-frequency roar", TrackCategory::Noise),
            TrackInfo::new(TrackId::OceanWaves, "Ocean Waves", "Rhythmic surf and rolling tides", TrackCategory::Nature),
            TrackInfo::new(TrackId::Binaural, "Binaural Beats", "Pure sine wave brainwave entrainment", TrackCategory::Synthesis),
        ];

        // Discover any custom audio files in ~/.config/zeno/sounds
        let custom_tracks = discover_user_sounds();
        tracks.extend(custom_tracks);

        let mut engine = AudioEngine::new();
        engine.init_tracks(&tracks);

        let factory_presets = Preset::factory_presets();

        Self {
            engine,
            tracks,
            factory_presets,
            user_presets: Vec::new(),
            active_preset_name: None,
        }
    }

    pub fn tick(&mut self) {
        self.engine.update_track_volumes(&mut self.tracks);
    }

    pub fn toggle_track(&mut self, index: usize) {
        if let Some(track) = self.tracks.get_mut(index) {
            track.enabled = !track.enabled;
            self.active_preset_name = None;
        }
    }

    pub fn adjust_track_volume(&mut self, index: usize, delta: f32) {
        if let Some(track) = self.tracks.get_mut(index) {
            track.volume = (track.volume + delta).clamp(0.0, 1.0);
            if track.volume > 0.0 {
                track.enabled = true;
            } else {
                track.enabled = false;
            }
            self.active_preset_name = None;
        }
    }

    #[allow(dead_code)]
    pub fn set_track_volume(&mut self, index: usize, volume: f32) {
        if let Some(track) = self.tracks.get_mut(index) {
            track.volume = volume.clamp(0.0, 1.0);
            track.enabled = track.volume > 0.0;
        }
    }

    #[allow(dead_code)]
    pub fn toggle_binaural(&mut self) {
        if let Some(idx) = self.tracks.iter().position(|t| t.id == TrackId::Binaural) {
            self.toggle_track(idx);
        }
    }

    pub fn cycle_binaural_mode(&mut self) {
        if let Some(track) = self.tracks.iter_mut().find(|t| t.id == TrackId::Binaural) {
            track.binaural_hz = if track.binaural_hz <= 10.0 {
                14.0 // Beta
            } else if track.binaural_hz <= 15.0 {
                40.0 // Gamma
            } else {
                10.0 // Alpha
            };
            let hz = track.binaural_hz;
            self.engine.set_binaural_hz(hz);
        }
    }

    pub fn toggle_mute(&mut self) {
        self.engine.toggle_mute();
    }

    pub fn adjust_master_volume(&mut self, delta: f32) {
        self.engine.adjust_master_volume(delta);
    }

    pub fn play_chime(&self) {
        self.engine.play_chime();
    }

    pub fn apply_preset(&mut self, preset: &Preset) {
        self.active_preset_name = Some(preset.name.clone());

        // Update tracks
        for track in self.tracks.iter_mut() {
            let key = track.id.as_str();
            if let Some(&vol) = preset.tracks.get(key) {
                track.volume = vol;
                track.enabled = vol > 0.001;
            } else {
                track.volume = 0.0;
                track.enabled = false;
            }
        }

        if let Some(hz) = preset.binaural_hz {
            if let Some(track) = self.tracks.iter_mut().find(|t| t.id == TrackId::Binaural) {
                track.binaural_hz = hz;
            }
            self.engine.set_binaural_hz(hz);
        }
    }

    pub fn save_user_preset(
        &mut self,
        name: String,
        description: String,
        visualizer: VisualizerType,
    ) -> Preset {
        let mut track_map = HashMap::new();
        let mut binaural_hz = None;

        for track in &self.tracks {
            if track.enabled && track.volume > 0.001 {
                track_map.insert(track.id.as_str().to_string(), track.volume);
                if track.id == TrackId::Binaural {
                    binaural_hz = Some(track.binaural_hz);
                }
            }
        }

        let preset = Preset {
            name: name.clone(),
            description,
            visualizer,
            binaural_hz,
            tracks: track_map,
            is_builtin: false,
        };

        // If preset with same name exists, overwrite
        if let Some(pos) = self.user_presets.iter().position(|p| p.name == name) {
            self.user_presets[pos] = preset.clone();
        } else {
            self.user_presets.push(preset.clone());
        }

        self.active_preset_name = Some(name);
        preset
    }

    pub fn delete_user_preset(&mut self, name: &str) -> bool {
        if let Some(pos) = self.user_presets.iter().position(|p| p.name == name) {
            self.user_presets.remove(pos);
            if self.active_preset_name.as_deref() == Some(name) {
                self.active_preset_name = None;
            }
            true
        } else {
            false
        }
    }

    /// Computes live pseudo-FFT spectrum bands for active audio tracks to drive the ASCII wave visualizer
    pub fn get_spectrum_energy(&self) -> [f32; 16] {
        let mut bands = [0.0f32; 16];
        if self.engine.is_muted {
            return bands;
        }

        for track in &self.tracks {
            if !track.enabled || track.current_volume <= 0.001 {
                continue;
            }
            let vol = track.current_volume;
            match &track.id {
                TrackId::BrownNoise => {
                    // Heavily in sub-bass / low end (bands 0..4)
                    bands[0] += vol * 0.9;
                    bands[1] += vol * 0.8;
                    bands[2] += vol * 0.6;
                    bands[3] += vol * 0.4;
                    bands[4] += vol * 0.2;
                }
                TrackId::OceanWaves => {
                    // Periodic low-mid surge (bands 1..6)
                    bands[1] += vol * 0.7;
                    bands[2] += vol * 0.8;
                    bands[3] += vol * 0.7;
                    bands[4] += vol * 0.5;
                    bands[5] += vol * 0.3;
                }
                TrackId::Fireplace => {
                    // Low rumble + mid bursts (bands 1..3 and 7..11)
                    bands[1] += vol * 0.5;
                    bands[2] += vol * 0.4;
                    bands[7] += vol * 0.5;
                    bands[8] += vol * 0.6;
                    bands[9] += vol * 0.4;
                }
                TrackId::Cafe => {
                    // Mid voice range (bands 4..9)
                    bands[4] += vol * 0.4;
                    bands[5] += vol * 0.6;
                    bands[6] += vol * 0.7;
                    bands[7] += vol * 0.6;
                    bands[8] += vol * 0.4;
                }
                TrackId::PinkNoise => {
                    // Broad gentle slope across all bands
                    for (i, b) in bands.iter_mut().enumerate() {
                        *b += vol * (1.0 - (i as f32 * 0.04));
                    }
                }
                TrackId::Forest => {
                    // Low wind + high chirps (bands 2..4 and 11..14)
                    bands[2] += vol * 0.4;
                    bands[3] += vol * 0.5;
                    bands[11] += vol * 0.5;
                    bands[12] += vol * 0.7;
                    bands[13] += vol * 0.6;
                }
                TrackId::Rain => {
                    // Mid-high hiss and rumble (bands 3..14)
                    for i in 4..13 {
                        bands[i] += vol * 0.5;
                    }
                    bands[1] += vol * 0.3; // thunder bass
                }
                TrackId::WhiteNoise => {
                    // Uniform across all bands
                    for b in bands.iter_mut() {
                        *b += vol * 0.45;
                    }
                }
                TrackId::Binaural => {
                    // Sharp resonant peak at specific band
                    let idx = if track.binaural_hz <= 12.0 { 2 } else if track.binaural_hz <= 20.0 { 5 } else { 8 };
                    bands[idx] += vol * 0.95;
                    if idx > 0 { bands[idx - 1] += vol * 0.3; }
                    if idx < 15 { bands[idx + 1] += vol * 0.3; }
                }
                TrackId::Custom(_) => {
                    for i in 2..10 {
                        bands[i] += vol * 0.4;
                    }
                }
            }
        }

        // Clamp each band to 1.0
        for b in &mut bands {
            *b = b.clamp(0.0, 1.0);
        }

        bands
    }
}
