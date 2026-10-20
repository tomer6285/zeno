use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

pub struct ShoreSimulator {
    _rng: FastRng,
    width: u16,
    height: u16,
    tide_phase: f32,
    wave_phase: f32,
    wet_sand: Vec<Vec<f32>>,
}

impl ShoreSimulator {
    pub fn new() -> Self {
        Self {
            _rng: FastRng::new(777),
            width: 80,
            height: 24,
            tide_phase: 0.0,
            wave_phase: 0.0,
            wet_sand: vec![vec![0.0; 80]; 24],
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width.max(1);
            self.height = height.max(1);
            self.wet_sand = vec![vec![0.0; self.width as usize]; self.height as usize];
        }
    }

    pub fn tick(&mut self, spectrum_energy: &[f32; 16]) {
        // Slow natural breath-paced ocean swell (about 6 seconds per full tide cycle)
        let ocean_energy = (spectrum_energy[0] + spectrum_energy[1] + spectrum_energy[2]) / 3.0;
        let speed = 0.025 + ocean_energy * 0.015;

        self.tide_phase = (self.tide_phase + speed).fract();
        self.wave_phase = (self.wave_phase + 0.06).fract();

        let w = self.width as usize;
        let h = self.height as usize;
        if w == 0 || h == 0 || self.wet_sand.len() != h || self.wet_sand[0].len() != w {
            return;
        }

        // Tide line height (0.0 = low tide at bottom, 1.0 = high tide halfway up)
        // Using smooth cosine curve for natural surge & recession
        let tide_height = (-(self.tide_phase * std::f32::consts::TAU).cos() + 1.0) * 0.5;

        // Base water line in rows from top (lower row number = higher water mark)
        let min_water_y = (h as f32 * 0.28).max(2.0);
        let max_water_y = (h as f32 * 0.85).min(h.saturating_sub(1) as f32);
        let current_water_base = max_water_y - tide_height * (max_water_y - min_water_y);

        // Update wet sand: where water reaches, wetness is set to 1.0. Dry over time elsewhere.
        for y in 0..h {
            for x in 0..w {
                let wave_offset = ((x as f32 * 0.12 + self.wave_phase * std::f32::consts::TAU).sin() * 1.5)
                    + ((x as f32 * 0.05 - self.wave_phase * 1.5).cos() * 1.0);
                let water_y = current_water_base + wave_offset;

                if (y as f32) >= water_y {
                    // Underwater
                    self.wet_sand[y][x] = 1.0;
                } else {
                    // Drying out in the sun/air
                    self.wet_sand[y][x] = (self.wet_sand[y][x] * 0.985).max(0.0);
                }
            }
        }
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        let w = area.width as usize;
        let h = area.height as usize;
        if w == 0 || h == 0 || self.wet_sand.len() != h || self.wet_sand[0].len() != w {
            return Paragraph::new("");
        }

        let tide_height = (-(self.tide_phase * std::f32::consts::TAU).cos() + 1.0) * 0.5;
        let min_water_y = (h as f32 * 0.28).max(2.0);
        let max_water_y = (h as f32 * 0.85).min(h.saturating_sub(1) as f32);
        let current_water_base = max_water_y - tide_height * (max_water_y - min_water_y);

        let mut char_grid = vec![vec![' '; w]; h];
        let mut style_grid = vec![vec![Style::default(); w]; h];

        for y in 0..h {
            for x in 0..w {
                let wave_offset = ((x as f32 * 0.12 + self.wave_phase * std::f32::consts::TAU).sin() * 1.5)
                    + ((x as f32 * 0.05 - self.wave_phase * 1.5).cos() * 1.0);
                let water_y = current_water_base + wave_offset;
                let dy = y as f32 - water_y;

                if dy >= 0.0 {
                    // Underwater / Surf
                    if dy < 1.2 {
                        // Leading foam line / surf edge
                        let foam_ch = if (x + y) % 3 == 0 {
                            '░'
                        } else if (x + y) % 2 == 0 {
                            '~'
                        } else {
                            '≈'
                        };
                        char_grid[y][x] = foam_ch;
                        style_grid[y][x] = Style::default().fg(theme.primary).add_modifier(Modifier::BOLD);
                    } else if dy < 3.0 {
                        // Shallow white water
                        let ch = match (x + y * 2) % 4 {
                            0 => '~',
                            1 => '≈',
                            2 => '░',
                            _ => ' ',
                        };
                        char_grid[y][x] = ch;
                        style_grid[y][x] = Style::default().fg(theme.primary);
                    } else {
                        // Deep water swells
                        let deep_wave = ((x as f32 * 0.18 + y as f32 * 0.3 + self.wave_phase * 2.0).sin() + 1.0) * 0.5;
                        let ch = if deep_wave > 0.85 {
                            '≋'
                        } else if deep_wave > 0.65 {
                            '~'
                        } else if deep_wave > 0.45 {
                            '━'
                        } else {
                            ' '
                        };
                        char_grid[y][x] = ch;
                        style_grid[y][x] = Style::default().fg(theme.secondary);
                    }
                } else {
                    // Sand shore
                    let wetness = self.wet_sand[y][x];
                    if wetness > 0.55 {
                        // Glistening wet sand
                        let ch = if (x * 7 + y * 13) % 5 == 0 {
                            '·'
                        } else if (x * 3 + y * 11) % 4 == 0 {
                            '•'
                        } else {
                            '░'
                        };
                        char_grid[y][x] = ch;
                        style_grid[y][x] = Style::default().fg(theme.warning).add_modifier(Modifier::DIM);
                    } else if wetness > 0.15 {
                        // Damp sand with light sparkles
                        if (x * 7 + y * 13) % 6 == 0 {
                            char_grid[y][x] = '·';
                            style_grid[y][x] = Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM);
                        }
                    } else {
                        // Dry beach
                        if (x * 11 + y * 17) % 19 == 0 {
                            char_grid[y][x] = '·';
                            style_grid[y][x] = Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM);
                        }
                    }
                }
            }
        }

        // Assemble Ratatui lines
        let mut lines = Vec::with_capacity(h);
        for y in 0..h {
            let mut spans = Vec::new();
            let mut curr_str = String::new();
            let mut curr_style = style_grid[y][0];

            for x in 0..w {
                let cell_char = char_grid[y][x];
                let cell_style = style_grid[y][x];

                if cell_style == curr_style {
                    curr_str.push(cell_char);
                } else {
                    if !curr_str.is_empty() {
                        spans.push(Span::styled(curr_str, curr_style));
                        curr_str = String::new();
                    }
                    curr_style = cell_style;
                    curr_str.push(cell_char);
                }
            }
            if !curr_str.is_empty() {
                spans.push(Span::styled(curr_str, curr_style));
            }
            lines.push(Line::from(spans));
        }

        Paragraph::new(lines)
    }
}
