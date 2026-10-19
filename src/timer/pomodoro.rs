use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PomodoroPhase {
    Work,
    ShortBreak,
    LongBreak,
}

impl PomodoroPhase {
    pub fn label(&self) -> &'static str {
        match self {
            PomodoroPhase::Work => "FOCUS SESSION",
            PomodoroPhase::ShortBreak => "SHORT BREAK",
            PomodoroPhase::LongBreak => "LONG BREAK",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PomodoroTransition {
    PhaseCompleted(PomodoroPhase),
}

#[derive(Debug, Clone)]
pub struct PomodoroTimer {
    pub phase: PomodoroPhase,
    pub time_remaining: Duration,
    pub total_duration: Duration,
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub completed_sessions: u32,
    pub cycles_before_long_break: u32,
    pub is_running: bool,
    pub auto_advance: bool,
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        let work = Duration::from_secs(25 * 60);
        Self {
            phase: PomodoroPhase::Work,
            time_remaining: work,
            total_duration: work,
            work_duration: work,
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            completed_sessions: 0,
            cycles_before_long_break: 4,
            is_running: false,
            auto_advance: false,
        }
    }
}

impl PomodoroTimer {
    pub fn new(work_mins: u64, short_break_mins: u64, long_break_mins: u64) -> Self {
        let work = Duration::from_secs(work_mins * 60);
        Self {
            phase: PomodoroPhase::Work,
            time_remaining: work,
            total_duration: work,
            work_duration: work,
            short_break_duration: Duration::from_secs(short_break_mins * 60),
            long_break_duration: Duration::from_secs(long_break_mins * 60),
            completed_sessions: 0,
            cycles_before_long_break: 4,
            is_running: false,
            auto_advance: false,
        }
    }

    pub fn toggle_run(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn reset_current_phase(&mut self) {
        self.is_running = false;
        self.time_remaining = self.phase_duration(self.phase);
        self.total_duration = self.time_remaining;
    }

    pub fn phase_duration(&self, phase: PomodoroPhase) -> Duration {
        match phase {
            PomodoroPhase::Work => self.work_duration,
            PomodoroPhase::ShortBreak => self.short_break_duration,
            PomodoroPhase::LongBreak => self.long_break_duration,
        }
    }

    pub fn next_phase(&mut self) -> PomodoroTransition {
        let completed = self.phase;
        match completed {
            PomodoroPhase::Work => {
                self.completed_sessions += 1;
                if self.completed_sessions % self.cycles_before_long_break == 0 {
                    self.phase = PomodoroPhase::LongBreak;
                } else {
                    self.phase = PomodoroPhase::ShortBreak;
                }
            }
            PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak => {
                self.phase = PomodoroPhase::Work;
            }
        }
        self.time_remaining = self.phase_duration(self.phase);
        self.total_duration = self.time_remaining;
        if !self.auto_advance {
            self.is_running = false;
        }
        PomodoroTransition::PhaseCompleted(completed)
    }

    pub fn tick(&mut self, elapsed: Duration) -> Option<PomodoroTransition> {
        if !self.is_running {
            return None;
        }

        if self.time_remaining <= elapsed {
            self.time_remaining = Duration::ZERO;
            Some(self.next_phase())
        } else {
            self.time_remaining -= elapsed;
            None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pomodoro_cycle_transitions() {
        let mut timer = PomodoroTimer::new(25, 5, 15);
        assert_eq!(timer.phase, PomodoroPhase::Work);
        assert_eq!(timer.completed_sessions, 0);

        // 1st Work finishes -> ShortBreak
        let trans1 = timer.next_phase();
        assert_eq!(trans1, PomodoroTransition::PhaseCompleted(PomodoroPhase::Work));
        assert_eq!(timer.phase, PomodoroPhase::ShortBreak);
        assert_eq!(timer.completed_sessions, 1);

        // 1st ShortBreak finishes -> Work
        let trans2 = timer.next_phase();
        assert_eq!(trans2, PomodoroTransition::PhaseCompleted(PomodoroPhase::ShortBreak));
        assert_eq!(timer.phase, PomodoroPhase::Work);

        // 2nd Work -> ShortBreak
        timer.next_phase();
        // 2nd Break -> Work
        timer.next_phase();
        // 3rd Work -> ShortBreak
        timer.next_phase();
        // 3rd Break -> Work
        timer.next_phase();

        // 4th Work finishes -> LongBreak (since 4 cycles reached!)
        let trans_4th_work = timer.next_phase();
        assert_eq!(trans_4th_work, PomodoroTransition::PhaseCompleted(PomodoroPhase::Work));
        assert_eq!(timer.phase, PomodoroPhase::LongBreak);
        assert_eq!(timer.completed_sessions, 4);
    }

    #[test]
    fn test_pomodoro_tick_and_progress() {
        let mut timer = PomodoroTimer::new(10, 2, 5);
        timer.is_running = true;
        assert_eq!(timer.progress_ratio(), 0.0);

        // Halfway tick (5 minutes)
        timer.tick(Duration::from_secs(5 * 60));
        assert!((timer.progress_ratio() - 0.5).abs() < 0.01);

        // Remaining tick triggers transition
        let alert = timer.tick(Duration::from_secs(5 * 60));
        assert!(alert.is_some());
        assert_eq!(timer.phase, PomodoroPhase::ShortBreak);
    }
}
