pub mod aurora;
pub mod fire;
pub mod fireflies;
pub mod leaves;
pub mod life;
pub mod mandala;
pub mod rain;
pub mod ripples;
pub mod shore;
pub mod starfield;
pub mod wave;
pub mod window;

use ratatui::{
    layout::Rect,
    widgets::Paragraph,
};
use crate::audio::presets::VisualizerType;
use crate::ui::theme::Theme;
use aurora::AuroraSimulator;
use fire::FireSimulator;
use fireflies::FirefliesSimulator;
use leaves::LeavesSimulator;
use life::LifeSimulator;
use mandala::MandalaSimulator;
use rain::RainSimulator;
use ripples::RipplesSimulator;
use shore::ShoreSimulator;
use starfield::StarfieldSimulator;
use wave::WaveSimulator;
use window::WindowSimulator;

pub struct VisualCoordinator {
    pub mode: VisualizerType,
    pub rain: RainSimulator,
    pub window: WindowSimulator,
    pub ripples: RipplesSimulator,
    pub shore: ShoreSimulator,
    pub fire: FireSimulator,
    pub fireflies: FirefliesSimulator,
    pub leaves: LeavesSimulator,
    pub aurora: AuroraSimulator,
    pub starfield: StarfieldSimulator,
    pub wave: WaveSimulator,
    pub mandala: MandalaSimulator,
    pub life: LifeSimulator,
}

impl VisualCoordinator {
    pub fn new(mode: VisualizerType) -> Self {
        Self {
            mode,
            rain: RainSimulator::new(),
            window: WindowSimulator::new(),
            ripples: RipplesSimulator::new(),
            shore: ShoreSimulator::new(),
            fire: FireSimulator::new(),
            fireflies: FirefliesSimulator::new(),
            leaves: LeavesSimulator::new(),
            aurora: AuroraSimulator::new(),
            starfield: StarfieldSimulator::new(),
            wave: WaveSimulator::new(),
            mandala: MandalaSimulator::new(),
            life: LifeSimulator::new(),
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
        self.window.resize(width, height);
        self.ripples.resize(width, height);
        self.shore.resize(width, height);
        self.fire.resize(width, height);
        self.fireflies.resize(width, height);
        self.leaves.resize(width, height);
        self.aurora.resize(width, height);
        self.starfield.resize(width, height);
        self.mandala.resize(width, height);
        self.life.resize(width, height);
    }

    pub fn tick(&mut self, spectrum_energy: &[f32; 16]) {
        match self.mode {
            VisualizerType::Rain => self.rain.tick(spectrum_energy),
            VisualizerType::Window => self.window.tick(spectrum_energy),
            VisualizerType::Ripples => self.ripples.tick(spectrum_energy),
            VisualizerType::Shore => self.shore.tick(spectrum_energy),
            VisualizerType::Fire => self.fire.tick(),
            VisualizerType::Fireflies => self.fireflies.tick(),
            VisualizerType::Leaves => self.leaves.tick(spectrum_energy),
            VisualizerType::Aurora => self.aurora.tick(spectrum_energy),
            VisualizerType::Starfield => self.starfield.tick(spectrum_energy),
            VisualizerType::Wave => self.wave.tick(spectrum_energy),
            VisualizerType::Mandala => self.mandala.tick(),
            VisualizerType::Life => self.life.tick(),
            VisualizerType::Off => {}
        }
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        match self.mode {
            VisualizerType::Rain => self.rain.render(area, theme),
            VisualizerType::Window => self.window.render(area, theme),
            VisualizerType::Ripples => self.ripples.render(area, theme),
            VisualizerType::Shore => self.shore.render(area, theme),
            VisualizerType::Fire => self.fire.render(area, theme),
            VisualizerType::Fireflies => self.fireflies.render(area, theme),
            VisualizerType::Leaves => self.leaves.render(area, theme),
            VisualizerType::Aurora => self.aurora.render(area, theme),
            VisualizerType::Starfield => self.starfield.render(area, theme),
            VisualizerType::Wave => self.wave.render(area, theme),
            VisualizerType::Mandala => self.mandala.render(area, theme),
            VisualizerType::Life => self.life.render(area, theme),
            VisualizerType::Off => Paragraph::new("Visualizer disabled (press 'v' to choose atmosphere)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_all_visualizers_tick_and_render() {
        let theme = Theme::default();
        let spectrum = [0.5; 16];
        let test_sizes = [(80, 24), (40, 10), (10, 5), (1, 1)];

        for &mode in VisualizerType::ALL {
            let mut coord = VisualCoordinator::new(mode);
            for &(w, h) in &test_sizes {
                coord.resize(w, h);
                coord.tick(&spectrum);

                let backend = TestBackend::new(w, h);
                let mut terminal = Terminal::new(backend).unwrap();
                let area = Rect::new(0, 0, w, h);
                terminal
                    .draw(|f| {
                        f.render_widget(coord.render(area, &theme), area);
                    })
                    .unwrap();
            }
        }
    }

    #[test]
    fn test_mode_cycling() {
        let mut coord = VisualCoordinator::new(VisualizerType::ALL[0]);
        for _ in 0..VisualizerType::ALL.len() {
            coord.cycle_mode();
        }
        assert_eq!(coord.mode, VisualizerType::ALL[0]);

        for _ in 0..VisualizerType::ALL.len() {
            coord.prev_mode();
        }
        assert_eq!(coord.mode, VisualizerType::ALL[0]);
    }
}
