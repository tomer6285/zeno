use std::path::{Path, PathBuf};
use crate::audio::track::{TrackCategory, TrackId, TrackInfo};

pub fn discover_user_sounds() -> Vec<TrackInfo> {
    let mut tracks = Vec::new();
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let sounds_dir = config_dir.join("zeno").join("sounds");

    if !sounds_dir.exists() {
        return tracks;
    }

    if let Ok(entries) = std::fs::read_dir(&sounds_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if is_audio_file(&path) {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("custom")
                    .to_string();
                let title = stem
                    .split(['-', '_'])
                    .map(|w| {
                        let mut chars = w.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                let mut track = TrackInfo::new(
                    TrackId::Custom(stem.clone()),
                    title,
                    format!("User file: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                    TrackCategory::Custom,
                );
                track.custom_path = Some(path);
                tracks.push(track);
            }
        }
    }

    tracks
}

fn is_audio_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "ogg" | "mp3" | "flac" | "wav" | "aac" | "m4a"
        )
    } else {
        false
    }
}
