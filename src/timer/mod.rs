pub mod pomodoro;
pub mod stopwatch;

use std::time::Duration;
use pomodoro::{PomodoroPhase, PomodoroTimer, PomodoroTransition};
use stopwatch::StopwatchTimer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerMode {
    Pomodoro,
    Stopwatch,
    Countdown,
}

impl TimerMode {
    pub fn name(&self) -> &'static str {
        match self {
            TimerMode::Pomodoro => "Pomodoro",
            TimerMode::Stopwatch => "Stopwatch",
            TimerMode::Countdown => "Countdown",
        }
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        match self {
            TimerMode::Pomodoro => TimerMode::Stopwatch,
            TimerMode::Stopwatch => TimerMode::Countdown,
            TimerMode::Countdown => TimerMode::Pomodoro,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CountdownTimer {
    pub time_remaining: Duration,
    pub total_duration: Duration,
    pub is_running: bool,
}

impl Default for CountdownTimer {
    fn default() -> Self {
        let dur = Duration::from_secs(15 * 60);
        Self {
            time_remaining: dur,
            total_duration: dur,
            is_running: false,
        }
    }
}

impl CountdownTimer {
    #[allow(dead_code)]
    pub fn new(minutes: u64) -> Self {
        let dur = Duration::from_secs(minutes * 60);
        Self {
            time_remaining: dur,
            total_duration: dur,
            is_running: false,
        }
    }

    pub fn toggle_run(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn reset(&mut self) {
        self.time_remaining = self.total_duration;
        self.is_running = false;
    }

    pub fn set_duration_mins(&mut self, minutes: u64) {
        let dur = Duration::from_secs(minutes * 60);
        self.total_duration = dur;
        self.time_remaining = dur;
        self.is_running = false;
    }

    pub fn tick(&mut self, delta: Duration) -> bool {
        if !self.is_running {
            return false;
        }

        if self.time_remaining <= delta {
            self.time_remaining = Duration::ZERO;
            self.is_running = false;
            true
        } else {
            self.time_remaining -= delta;
            false
        }
    }

    pub fn progress_ratio(&self) -> f64 {
        if self.total_duration.as_secs_f64() == 0.0 {
            return 1.0;
        }
        let elapsed = self.total_duration.saturating_sub(self.time_remaining);
        (elapsed.as_secs_f64() / self.total_duration.as_secs_f64()).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub enum TimerAlert {
    PomodoroFinished(PomodoroPhase),
    CountdownFinished,
}

#[derive(Debug, Clone)]
pub struct TimerManager {
    pub mode: TimerMode,
    pub pomodoro: PomodoroTimer,
    pub stopwatch: StopwatchTimer,
    pub countdown: CountdownTimer,
}

impl Default for TimerManager {
    fn default() -> Self {
        Self {
            mode: TimerMode::Pomodoro,
            pomodoro: PomodoroTimer::default(),
            stopwatch: StopwatchTimer::default(),
            countdown: CountdownTimer::default(),
        }
    }
}

impl TimerManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_mode(&mut self, mode: TimerMode) {
        self.mode = mode;
    }

    pub fn toggle_run(&mut self) {
        match self.mode {
            TimerMode::Pomodoro => self.pomodoro.toggle_run(),
            TimerMode::Stopwatch => self.stopwatch.toggle_run(),
            TimerMode::Countdown => self.countdown.toggle_run(),
        }
    }

    pub fn is_running(&self) -> bool {
        match self.mode {
            TimerMode::Pomodoro => self.pomodoro.is_running,
            TimerMode::Stopwatch => self.stopwatch.is_running,
            TimerMode::Countdown => self.countdown.is_running,
        }
    }

    pub fn reset_or_skip(&mut self) -> Option<TimerAlert> {
        match self.mode {
            TimerMode::Pomodoro => {
                if self.pomodoro.is_running {
                    let trans = self.pomodoro.next_phase();
                    let PomodoroTransition::PhaseCompleted(p) = trans;
                    Some(TimerAlert::PomodoroFinished(p))
                } else {
                    self.pomodoro.reset_current_phase();
                    None
                }
            }
            TimerMode::Stopwatch => {
                self.stopwatch.reset();
                None
            }
            TimerMode::Countdown => {
                self.countdown.reset();
                None
            }
        }
    }

    pub fn tick(&mut self, delta: Duration) -> Option<TimerAlert> {
        match self.mode {
            TimerMode::Pomodoro => {
                if let Some(PomodoroTransition::PhaseCompleted(phase)) = self.pomodoro.tick(delta) {
                    Some(TimerAlert::PomodoroFinished(phase))
                } else {
                    None
                }
            }
            TimerMode::Stopwatch => {
                self.stopwatch.tick(delta);
                None
            }
            TimerMode::Countdown => {
                if self.countdown.tick(delta) {
                    Some(TimerAlert::CountdownFinished)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_manager_modes() {
        let mut tm = TimerManager::new();
        assert_eq!(tm.mode, TimerMode::Pomodoro);

        tm.set_mode(TimerMode::Countdown);
        assert_eq!(tm.mode, TimerMode::Countdown);

        tm.countdown.set_duration_mins(1);
        tm.toggle_run();
        assert!(tm.is_running());

        let finished = tm.tick(Duration::from_secs(61));
        assert!(matches!(finished, Some(TimerAlert::CountdownFinished)));
        assert!(!tm.is_running());
    }
}
