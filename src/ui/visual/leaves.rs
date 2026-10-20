use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
struct Leaf {
    base_x: f32,
    y: f32,
    fall_speed: f32,
    sway_phase: f32,
    sway_freq: f32,
    sway_amp: f32,
    glyph: char,
    color_type: u8,
}

pub struct LeavesSimulator {
    rng: FastRng,
    leaves: Vec<Leaf>,
    ground_pile: Vec<u8>,
    width: u16,
    height: u16,
    wind: f32,
    target_wind: f32,
}

impl LeavesSimulator {
    pub fn new() -> Self {
        let mut sim = Self {
            rng: FastRng::new(4321),
            leaves: Vec::new(),
            ground_pile: Vec::new(),
            width: 80,
            height: 24,
            wind: 0.1,
            target_wind: 0.1,
        };
        sim.init_leaves(80, 24);
        sim
    }

    fn create_leaf(rng: &mut FastRng, base_x: f32, y: f32) -> Leaf {
        let glyph = match rng.next_u32() % 6 {
            0 => '🍃',
            1 => '🍂',
            2 => '❀',
            3 => '~',
            4 => '*',
            _ => '\'',
        };
        let color_type = (rng.next_u32() % 3) as u8;
        let fall_speed = 0.15 + rng.next_f32() * 0.25;
        let sway_amp = 1.8 + rng.next_f32() * 3.5;
        let sway_freq = 0.05 + rng.next_f32() * 0.06;
        let sway_phase = rng.next_f32() * std::f32::consts::TAU;

        Leaf {
            base_x,
            y,
            fall_speed,
            sway_phase,
            sway_freq,
            sway_amp,
            glyph,
            color_type,
        }
    }

    fn init_leaves(&mut self, width: u16, height: u16) {
        self.leaves.clear();
        self.ground_pile = vec![0; width as usize];

        let count = ((width as usize * height as usize) / 60).clamp(15, 60);
        for _ in 0..count {
            let base_x = (self.rng.next_u32() % (width.max(1) as u32)) as f32;
            let y = (self.rng.next_u32() % (height.max(1) as u32)) as f32;
            let leaf = Self::create_leaf(&mut self.rng, base_x, y);
            self.leaves.push(leaf);
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width.max(1);
            self.height = height.max(1);
            self.init_leaves(self.width, self.height);
        }
    }

    pub fn tick(&mut self, spectrum_energy: &[f32; 16]) {
        let w = self.width as usize;
        let h = self.height as usize;
        if w == 0 || h == 0 {
            return;
        }

        // Adjust wind smoothly
        let wind_energy = (spectrum_energy[3] + spectrum_energy[4]) * 0.5;
        if self.rng.next_f32() < 0.03 {
            self.target_wind = self.rng.next_signed() * (0.3 + wind_energy * 0.5);
        }
        self.wind += (self.target_wind - self.wind) * 0.05;

        // Update ground decay
        if self.rng.next_f32() < 0.05 && !self.ground_pile.is_empty() {
            let idx = (self.rng.next_u32() % (self.ground_pile.len() as u32)) as usize;
            self.ground_pile[idx] = self.ground_pile[idx].saturating_sub(1);
        }

        let wind = self.wind;
        // Update leaves
        for leaf in &mut self.leaves {
            leaf.sway_phase += leaf.sway_freq;
            leaf.base_x += wind;
            leaf.y += leaf.fall_speed;

            // Wrap horizontal
            if leaf.base_x < 0.0 {
                leaf.base_x += w as f32;
            } else if leaf.base_x >= w as f32 {
                leaf.base_x -= w as f32;
            }

            // Check ground collision
            if leaf.y >= (h.saturating_sub(1)) as f32 {
                let gx = leaf.base_x.round() as usize;
                if gx < self.ground_pile.len() {
                    self.ground_pile[gx] = self.ground_pile[gx].saturating_add(1).min(3);
                }
                let new_x = (self.rng.next_u32() % (w as u32)) as f32;
                let new_y = -(self.rng.next_f32() * 3.0);
                *leaf = Self::create_leaf(&mut self.rng, new_x, new_y);
            }
        }
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        let w = area.width as usize;
        let h = area.height as usize;
        if w == 0 || h == 0 {
            return Paragraph::new("");
        }

        let mut char_grid = vec![vec![' '; w]; h];
        let mut style_grid = vec![vec![Style::default(); w]; h];

        // 1. Fallen leaves bed on the ground
        let ground_y = h.saturating_sub(1);
        for x in 0..w.min(self.ground_pile.len()) {
            let count = self.ground_pile[x];
            if count > 0 {
                let ch = match count {
                    1 => '·',
                    2 => '~',
                    _ => '🍂',
                };
                char_grid[ground_y][x] = ch;
                let color = if count > 1 { theme.warning } else { theme.text_muted };
                style_grid[ground_y][x] = Style::default().fg(color).add_modifier(Modifier::DIM);
            }
        }

        // 2. Falling leaves
        for leaf in &self.leaves {
            let actual_x = leaf.base_x + leaf.sway_phase.sin() * leaf.sway_amp;
            let lx = actual_x.round() as usize;
            let ly = leaf.y.floor() as usize;

            if lx < w && ly < h {
                char_grid[ly][lx] = leaf.glyph;
                let color = match leaf.color_type {
                    0 => theme.warning,  // Amber
                    1 => theme.accent,   // Maple / Blossom
                    _ => theme.success,  // Green
                };
                style_grid[ly][lx] = Style::default().fg(color).add_modifier(Modifier::BOLD);
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
