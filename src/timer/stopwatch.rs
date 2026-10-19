use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct StopwatchTimer {
    pub elapsed: Duration,
    pub is_running: bool,
    pub laps: Vec<Duration>,
}

impl StopwatchTimer {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_run(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.is_running = false;
        self.laps.clear();
    }

    #[allow(dead_code)]
    pub fn record_lap(&mut self) {
        if self.is_running || self.elapsed > Duration::ZERO {
            self.laps.push(self.elapsed);
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        if self.is_running {
            self.elapsed += delta;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stopwatch() {
        let mut sw = StopwatchTimer::new();
        assert_eq!(sw.elapsed, Duration::ZERO);
        assert!(!sw.is_running);

        sw.toggle_run();
        assert!(sw.is_running);

        sw.tick(Duration::from_secs(3));
        assert_eq!(sw.elapsed, Duration::from_secs(3));

        sw.record_lap();
        assert_eq!(sw.laps.len(), 1);
        assert_eq!(sw.laps[0], Duration::from_secs(3));

        sw.reset();
        assert_eq!(sw.elapsed, Duration::ZERO);
        assert!(!sw.is_running);
        assert!(sw.laps.is_empty());
    }
}
