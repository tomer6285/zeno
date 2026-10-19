pub mod fire;
pub mod rain;
pub mod wave;

use ratatui::{
    layout::Rect,
    widgets::Paragraph,
};
use crate::audio::presets::VisualizerType;
use crate::ui::theme::Theme;
use fire::FireSimulator;
use rain::RainSimulator;
use wave::WaveSimulator;

pub struct VisualCoordinator {
    pub mode: VisualizerType,
    pub rain: RainSimulator,
    pub fire: FireSimulator,
    pub wave: WaveSimulator,
}

impl VisualCoordinator {
    pub fn new(mode: VisualizerType) -> Self {
        Self {
            mode,
            rain: RainSimulator::new(),
            fire: FireSimulator::new(),
            wave: WaveSimulator::new(),
        }
    }

    pub fn set_mode(&mut self, mode: VisualizerType) {
        self.mode = mode;
    }

    pub fn cycle_mode(&mut self) {
        self.mode = self.mode.next();
    }

    pub fn prev_mode(&mut self) {
        self.mode = self.mode.prev();
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.rain.resize(width, height);
        self.fire.resize(width, height);
    }

    pub fn tick(&mut self, spectrum_energy: &[f32; 16]) {
        match self.mode {
            VisualizerType::Rain => self.rain.tick(spectrum_energy),
            VisualizerType::Fire => self.fire.tick(),
            VisualizerType::Wave => self.wave.tick(spectrum_energy),
            VisualizerType::Off => {}
        }
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        match self.mode {
            VisualizerType::Rain => self.rain.render(area, theme),
            VisualizerType::Fire => self.fire.render(area, theme),
            VisualizerType::Wave => self.wave.render(area, theme),
            VisualizerType::Off => Paragraph::new("Visualizer disabled (press 'v' to enable)"),
        }
    }
}
