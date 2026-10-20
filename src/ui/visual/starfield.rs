use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
struct Star3D {
    x: f32,
    y: f32,
    z: f32,
}

pub struct StarfieldSimulator {
    rng: FastRng,
    stars: Vec<Star3D>,
    width: u16,
    height: u16,
    speed: f32,
}

impl StarfieldSimulator {
    pub fn new() -> Self {
        let mut sim = Self {
            rng: FastRng::new(10101),
            stars: Vec::new(),
            width: 80,
            height: 24,
            speed: 0.8,
        };
        sim.init_stars();
        sim
    }

    fn init_stars(&mut self) {
        self.stars.clear();
        for _ in 0..120 {
            let x = self.rng.next_signed() * 100.0;
            let y = self.rng.next_signed() * 100.0;
            let z = 1.0 + self.rng.next_f32() * 99.0;
            self.stars.push(Star3D { x, y, z });
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width.max(1);
            self.height = height.max(1);
        }
    }

    pub fn tick(&mut self, spectrum_energy: &[f32; 16]) {
        let energy = (spectrum_energy[0] + spectrum_energy[1]) * 0.5;
        let base_speed = 0.7 + energy * 1.5;
        self.speed = base_speed;

        let w = self.width as f32;
        let h = self.height as f32;
        let cx = w * 0.5;
        let cy = h * 0.5;
        let fov_x = w * 0.45;
        let fov_y = h * 0.45;

        for star in &mut self.stars {
            star.z -= self.speed;

            let mut reset = star.z <= 1.0;
            if !reset {
                let sx = cx + (star.x / star.z) * fov_x;
                let sy = cy + (star.y / star.z) * fov_y;
                if sx < 0.0 || sx >= w || sy < 0.0 || sy >= h {
                    reset = true;
                }
            }

            if reset {
                star.x = self.rng.next_signed() * 100.0;
                star.y = self.rng.next_signed() * 100.0;
                star.z = 100.0;
            }
        }
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        let w = area.width as usize;
        let h = area.height as usize;
        if w == 0 || h == 0 {
            return Paragraph::new("");
        }

        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.5;
        let fov_x = w as f32 * 0.45;
        let fov_y = h as f32 * 0.45;

        let mut char_grid = vec![vec![' '; w]; h];
        let mut style_grid = vec![vec![Style::default(); w]; h];

        for star in &self.stars {
            if star.z <= 1.0 {
                continue;
            }

            let sx_f = cx + (star.x / star.z) * fov_x;
            let sy_f = cy + (star.y / star.z) * fov_y;

            let sx = sx_f.round() as usize;
            let sy = sy_f.round() as usize;

            if sx < w && sy < h {
                let z = star.z;
                let (ch, style) = if z > 70.0 {
                    ('·', Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM))
                } else if z > 40.0 {
                    ('•', Style::default().fg(theme.secondary))
                } else if z > 16.0 {
                    ('*', Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                } else {
                    ('✦', Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
                };

                char_grid[sy][sx] = ch;
                style_grid[sy][sx] = style;
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
