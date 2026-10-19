use std::time::{Duration, Instant};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::audio::AudioController;
use crate::config::AppConfig;
use crate::timer::{TimerAlert, TimerManager, TimerMode};
use crate::ui::theme::Theme;
use crate::ui::visual::VisualCoordinator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveFocus {
    Timer,
    Mixer,
    Visualizer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalState {
    None,
    PresetBrowser { selected_index: usize },
    SaveMixPrompt { input_text: String },
    Help,
}

pub struct App {
    pub should_quit: bool,
    pub focus: ActiveFocus,
    pub modal: ModalState,
    pub mixer_selected_index: usize,
    pub timer: TimerManager,
    pub audio: AudioController,
    pub visual: VisualCoordinator,
    pub theme: Theme,
    pub config: AppConfig,
    pub last_tick: Instant,
}

impl App {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let theme = Theme::new(config.theme);
        let mut audio = AudioController::new();

        // Load saved user presets
        audio.user_presets = config.presets.clone();
        audio.engine.master_volume = config.master_volume;
        audio.engine.is_muted = config.muted;

        let visual = VisualCoordinator::new(config.visualizer);

        let mut timer = TimerManager::new();
        timer.pomodoro = crate::timer::pomodoro::PomodoroTimer::new(
            config.work_duration_mins,
            config.short_break_mins,
            config.long_break_mins,
        );
        timer.pomodoro.auto_advance = config.auto_advance_pomodoro;

        Self {
            should_quit: false,
            focus: ActiveFocus::Timer,
            modal: ModalState::None,
            mixer_selected_index: 0,
            timer,
            audio,
            visual,
            theme,
            config,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_tick);
        self.last_tick = now;

        // 1. Tick timer logic
        if let Some(alert) = self.timer.tick(delta) {
            self.handle_timer_alert(alert);
        }

        // 2. Tick audio engine (smooth tweens)
        self.audio.tick();

        // 3. Tick visualizer with dynamic spectrum data
        let spectrum = self.audio.get_spectrum_energy();
        self.visual.tick(&spectrum);
    }

    fn handle_timer_alert(&mut self, alert: TimerAlert) {
        // Play gentle singing bowl chime
        self.audio.play_chime();

        // Optional desktop notification
        let (title, message) = match alert {
            TimerAlert::PomodoroFinished(phase) => (
                "Zeno Focus Timer",
                format!("{} finished! Time for the next interval.", phase.label()),
            ),
            TimerAlert::CountdownFinished => (
                "Zeno Countdown",
                "Countdown timer completed!".to_string(),
            ),
        };

        // Fire-and-forget notification (graceful if daemon unavailable)
        let _ = notify_rust::Notification::new()
            .summary(title)
            .body(&message)
            .appname("Zeno")
            .timeout(Duration::from_secs(5))
            .show();
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // 1. Handle modals if open
        match &mut self.modal {
            ModalState::Help => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')) {
                    self.modal = ModalState::None;
                }
                return;
            }
            ModalState::SaveMixPrompt { input_text } => {
                match key.code {
                    KeyCode::Esc => {
                        self.modal = ModalState::None;
                    }
                    KeyCode::Enter => {
                        let name = input_text.trim().to_string();
                        if !name.is_empty() {
                            let _preset = self.audio.save_user_preset(
                                name,
                                "Custom user created mix".to_string(),
                                self.visual.mode,
                            );
                            self.config.presets = self.audio.user_presets.clone();
                            let _ = self.config.save();
                        }
                        self.modal = ModalState::None;
                    }
                    KeyCode::Backspace => {
                        input_text.pop();
                    }
                    KeyCode::Char(c) => {
                        if input_text.len() < 30 {
                            input_text.push(c);
                        }
                    }
                    _ => {}
                }
                return;
            }
            ModalState::PresetBrowser { selected_index } => {
                let total_factory = self.audio.factory_presets.len();
                let total_user = self.audio.user_presets.len();
                let total = total_factory + total_user;

                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('p') => {
                        self.modal = ModalState::None;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if total > 0 && *selected_index + 1 < total {
                            *selected_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        let idx = *selected_index;
                        let preset_to_apply = if idx < total_factory {
                            self.audio.factory_presets.get(idx).cloned()
                        } else {
                            self.audio.user_presets.get(idx - total_factory).cloned()
                        };

                        if let Some(p) = preset_to_apply {
                            self.visual.set_mode(p.visualizer);
                            self.audio.apply_preset(&p);
                        }
                        self.modal = ModalState::None;
                    }
                    KeyCode::Char('d') => {
                        let idx = *selected_index;
                        if idx >= total_factory {
                            let user_idx = idx - total_factory;
                            if let Some(user_preset) = self.audio.user_presets.get(user_idx).cloned() {
                                self.audio.delete_user_preset(&user_preset.name);
                                self.config.presets = self.audio.user_presets.clone();
                                let _ = self.config.save();
                                if *selected_index > 0 {
                                    *selected_index -= 1;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                return;
            }
            ModalState::None => {}
        }

        // 2. Global Hotkeys
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            self.modal = ModalState::SaveMixPrompt {
                input_text: String::new(),
            };
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.save_and_quit();
            }
            KeyCode::Char('?') => {
                self.modal = ModalState::Help;
            }
            KeyCode::Char('p') => {
                self.modal = ModalState::PresetBrowser { selected_index: 0 };
            }
            KeyCode::Char('S') => {
                self.modal = ModalState::SaveMixPrompt {
                    input_text: String::new(),
                };
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    ActiveFocus::Timer => ActiveFocus::Mixer,
                    ActiveFocus::Mixer => ActiveFocus::Visualizer,
                    ActiveFocus::Visualizer => ActiveFocus::Timer,
                };
            }
            KeyCode::Char(' ') => {
                self.timer.toggle_run();
            }
            KeyCode::Char('r') => {
                if let Some(alert) = self.timer.reset_or_skip() {
                    self.handle_timer_alert(alert);
                }
            }
            KeyCode::Char('1') => {
                self.timer.set_mode(TimerMode::Pomodoro);
            }
            KeyCode::Char('2') => {
                self.timer.set_mode(TimerMode::Stopwatch);
            }
            KeyCode::Char('3') => {
                self.timer.set_mode(TimerMode::Countdown);
            }
            KeyCode::Char('v') => {
                self.visual.cycle_mode();
            }
            KeyCode::Char('b') => {
                self.audio.cycle_binaural_mode();
            }
            KeyCode::Char('m') => {
                self.audio.toggle_mute();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.audio.adjust_master_volume(0.05);
            }
            KeyCode::Char('-') => {
                self.audio.adjust_master_volume(-0.05);
            }
            KeyCode::Char('t') => {
                let next_kind = self.theme.kind.next();
                self.theme = Theme::new(next_kind);
                self.config.theme = next_kind;
                let _ = self.config.save();
            }

            // Mixer navigation
            KeyCode::Up | KeyCode::Char('k') => {
                if self.mixer_selected_index > 0 {
                    self.mixer_selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.audio.tracks.is_empty() && self.mixer_selected_index + 1 < self.audio.tracks.len() {
                    self.mixer_selected_index += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.audio.adjust_track_volume(self.mixer_selected_index, -0.05);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.audio.adjust_track_volume(self.mixer_selected_index, 0.05);
            }
            KeyCode::Enter => {
                self.audio.toggle_track(self.mixer_selected_index);
            }

            _ => {}
        }
    }

    pub fn save_and_quit(&mut self) {
        self.config.master_volume = self.audio.engine.master_volume;
        self.config.muted = self.audio.engine.is_muted;
        self.config.visualizer = self.visual.mode;
        self.config.theme = self.theme.kind;
        self.config.presets = self.audio.user_presets.clone();
        let _ = self.config.save();
        self.should_quit = true;
    }
}
