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

impl ActiveFocus {
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            ActiveFocus::Timer => "Timer",
            ActiveFocus::Mixer => "Mixer",
            ActiveFocus::Visualizer => "Visualizer",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ActiveFocus::Timer => ActiveFocus::Mixer,
            ActiveFocus::Mixer => ActiveFocus::Visualizer,
            ActiveFocus::Visualizer => ActiveFocus::Timer,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            ActiveFocus::Timer => ActiveFocus::Visualizer,
            ActiveFocus::Visualizer => ActiveFocus::Mixer,
            ActiveFocus::Mixer => ActiveFocus::Timer,
        }
    }
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
    pub fullscreen: bool,
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
        let theme = Theme::default();
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
            fullscreen: false,
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
            KeyCode::Char('q') => {
                self.save_and_quit();
                return;
            }
            KeyCode::Esc => {
                if self.fullscreen {
                    self.fullscreen = false;
                } else {
                    self.save_and_quit();
                }
                return;
            }
            KeyCode::Char('?') => {
                self.modal = ModalState::Help;
                return;
            }
            KeyCode::Char('p') => {
                self.modal = ModalState::PresetBrowser { selected_index: 0 };
                return;
            }
            KeyCode::Char('S') => {
                self.modal = ModalState::SaveMixPrompt {
                    input_text: String::new(),
                };
                return;
            }
            KeyCode::Tab => {
                self.focus = self.focus.next();
                return;
            }
            KeyCode::BackTab => {
                self.focus = self.focus.prev();
                return;
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.fullscreen = !self.fullscreen;
                return;
            }
            KeyCode::Char('b') => {
                self.audio.cycle_binaural_mode();
                return;
            }
            KeyCode::Char('m') => {
                self.audio.toggle_mute();
                return;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.audio.adjust_master_volume(0.05);
                return;
            }
            KeyCode::Char('-') => {
                self.audio.adjust_master_volume(-0.05);
                return;
            }
            _ => {}
        }

        // 3. Module Focus-Scoped Controls
        match self.focus {
            ActiveFocus::Timer => {
                match key.code {
                    KeyCode::Char(' ') | KeyCode::Enter => {
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
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.timer.prev_mode();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.timer.cycle_mode();
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.timer.adjust_duration(-5);
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.timer.adjust_duration(5);
                    }
                    _ => {}
                }
            }
            ActiveFocus::Mixer => {
                match key.code {
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
                    KeyCode::Char(' ') => {
                        // Global convenience: Space still toggles timer
                        self.timer.toggle_run();
                    }
                    _ => {}
                }
            }
            ActiveFocus::Visualizer => {
                match key.code {
                    KeyCode::Up | KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('h') => {
                        self.visual.prev_mode();
                    }
                    KeyCode::Down | KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('l') | KeyCode::Char('v') | KeyCode::Enter => {
                        self.visual.cycle_mode();
                    }
                    KeyCode::Char(' ') => {
                        // Global convenience: Space still toggles timer
                        self.timer.toggle_run();
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn save_and_quit(&mut self) {
        self.config.master_volume = self.audio.engine.master_volume;
        self.config.muted = self.audio.engine.is_muted;
        self.config.visualizer = self.visual.mode;
        self.config.presets = self.audio.user_presets.clone();
        let _ = self.config.save();
        self.should_quit = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_focus_cycling() {
        let focus = ActiveFocus::Timer;
        assert_eq!(focus.name(), "Timer");
        assert_eq!(focus.next(), ActiveFocus::Mixer);
        assert_eq!(focus.next().next(), ActiveFocus::Visualizer);
        assert_eq!(focus.next().next().next(), ActiveFocus::Timer);

        assert_eq!(focus.prev(), ActiveFocus::Visualizer);
        assert_eq!(focus.prev().prev(), ActiveFocus::Mixer);
        assert_eq!(focus.prev().prev().prev(), ActiveFocus::Timer);
    }

    #[test]
    fn test_fullscreen_toggle_and_tab_navigation() {
        let mut app = App::new();
        assert_eq!(app.focus, ActiveFocus::Timer);
        assert!(!app.fullscreen);

        // Toggle fullscreen with 'f'
        app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
        assert!(app.fullscreen);

        // Tab in fullscreen switches focused module while remaining in fullscreen
        app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.focus, ActiveFocus::Mixer);
        assert!(app.fullscreen);

        app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.focus, ActiveFocus::Visualizer);
        assert!(app.fullscreen);

        // Esc exits fullscreen instead of quitting
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.fullscreen);
        assert!(!app.should_quit);

        // Esc again quits
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.should_quit);
    }

    #[test]
    fn test_focus_scoped_key_handling() {
        let mut app = App::new();
        assert_eq!(app.focus, ActiveFocus::Timer);

        // When timer is focused, h/l adjusts duration
        let initial_dur = app.timer.pomodoro.work_duration;
        app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));
        assert_eq!(app.timer.pomodoro.work_duration, initial_dur + Duration::from_secs(5 * 60));

        // When timer is focused, j/k switches mode
        app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        assert_eq!(app.timer.mode, TimerMode::Stopwatch);

        // Switch focus to Mixer
        app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.focus, ActiveFocus::Mixer);

        // In mixer, j navigates track index
        assert_eq!(app.mixer_selected_index, 0);
        app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        assert_eq!(app.mixer_selected_index, 1);

        // Switch focus to Visualizer
        app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.focus, ActiveFocus::Visualizer);

        // In visualizer, j cycles visual mode
        let current_visual = app.visual.mode;
        app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        assert_ne!(app.visual.mode, current_visual);
    }
}
