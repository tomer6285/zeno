use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};

use crate::audio::synth::{
    BellSource, BinauralSource, CafeSource, FireplaceSource, ForestSource, NoiseKind,
    NoiseSource, OceanSource, RainSource,
};
use crate::audio::track::{TrackId, TrackInfo};

pub struct AudioEngine {
    sink_handle: Option<MixerDeviceSink>,
    players: HashMap<String, Player>,
    pub master_volume: f32,
    pub is_muted: bool,
    #[allow(dead_code)]
    pub audio_available: bool,
}

impl AudioEngine {
    pub fn new() -> Self {
        let (sink_handle, available) = match DeviceSinkBuilder::open_default_sink() {
            Ok(mut handle) => {
                handle.log_on_drop(false);
                (Some(handle), true)
            }
            Err(e) => {
                eprintln!("Audio output device notice: {e:?}. Running in quiet mode.");
                (None, false)
            }
        };

        Self {
            sink_handle,
            players: HashMap::new(),
            master_volume: 0.8,
            is_muted: false,
            audio_available: available,
        }
    }

    pub fn init_tracks(&mut self, tracks: &[TrackInfo]) {
        let Some(handle) = &self.sink_handle else { return };

        for track in tracks {
            let track_key = track.id.as_str().to_string();
            if self.players.contains_key(&track_key) {
                continue;
            }

            let player = Player::connect_new(&handle.mixer());
            player.set_volume(0.0);
            player.pause();

            // Spawn appropriate audio source
            match &track.id {
                TrackId::Rain => {
                    let src = RainSource::new(101).repeat_infinite();
                    player.append(src);
                }
                TrackId::Fireplace => {
                    let src = FireplaceSource::new(202).repeat_infinite();
                    player.append(src);
                }
                TrackId::Forest => {
                    let src = ForestSource::new(303).repeat_infinite();
                    player.append(src);
                }
                TrackId::Cafe => {
                    let src = CafeSource::new(404).repeat_infinite();
                    player.append(src);
                }
                TrackId::WhiteNoise => {
                    let src = NoiseSource::new(NoiseKind::White, 505).repeat_infinite();
                    player.append(src);
                }
                TrackId::PinkNoise => {
                    let src = NoiseSource::new(NoiseKind::Pink, 606).repeat_infinite();
                    player.append(src);
                }
                TrackId::BrownNoise => {
                    let src = NoiseSource::new(NoiseKind::Brown, 707).repeat_infinite();
                    player.append(src);
                }
                TrackId::OceanWaves => {
                    let src = OceanSource::new(808).repeat_infinite();
                    player.append(src);
                }
                TrackId::Binaural => {
                    let src = BinauralSource::new(180.0, track.binaural_hz).repeat_infinite();
                    player.append(src);
                }
                TrackId::Custom(_) => {
                    if let Some(path) = &track.custom_path {
                        if let Ok(file) = File::open(path) {
                            if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                                player.append(decoder.repeat_infinite());
                            }
                        }
                    }
                }
            }

            self.players.insert(track_key, player);
        }
    }

    pub fn set_binaural_hz(&mut self, hz: f32) {
        let Some(handle) = &self.sink_handle else { return };
        let key = TrackId::Binaural.as_str().to_string();

        let current_vol = self.players.get(&key).map(|p| p.volume()).unwrap_or(0.0);
        let player = Player::connect_new(&handle.mixer());
        player.set_volume(current_vol);
        let src = BinauralSource::new(180.0, hz).repeat_infinite();
        player.append(src);
        if current_vol > 0.001 {
            player.play();
        } else {
            player.pause();
        }
        self.players.insert(key, player);
    }

    pub fn play_chime(&self) {
        let Some(handle) = &self.sink_handle else { return };
        if self.is_muted {
            return;
        }

        let bell = BellSource::new(528.0, 3.5);
        let vol = self.master_volume.clamp(0.1, 1.0);
        let player = Player::connect_new(&handle.mixer());
        player.set_volume(vol);
        player.append(bell);
        player.detach();
    }

    pub fn update_track_volumes(&mut self, tracks: &mut [TrackInfo]) {
        for track in tracks.iter_mut() {
            let target_vol = if self.is_muted || !track.enabled {
                0.0
            } else {
                track.volume * self.master_volume
            };

            // Smooth interpolation (crossfade / tween) to eliminate clicks
            let diff = target_vol - track.current_volume;
            if diff.abs() > 0.005 {
                track.current_volume += diff * 0.15;
            } else {
                track.current_volume = target_vol;
            }

            if let Some(player) = self.players.get(track.id.as_str()) {
                player.set_volume(track.current_volume);
                if track.current_volume > 0.001 {
                    player.play();
                } else {
                    player.pause();
                }
            }
        }
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }

    pub fn adjust_master_volume(&mut self, delta: f32) {
        self.master_volume = (self.master_volume + delta).clamp(0.0, 1.0);
    }
}
