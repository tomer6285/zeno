use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
struct Firefly {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    pulse_phase: f32,
    pulse_speed: f32,
    glow_type: u8,
}

pub struct FirefliesSimulator {
    rng: FastRng,
    fireflies: Vec<Firefly>,
    width: u16,
    height: u16,
}

impl FirefliesSimulator {
    pub fn new() -> Self {
        let mut sim = Self {
            rng: FastRng::new(8888),
            fireflies: Vec::new(),
            width: 80,
            height: 24,
        };
        sim.init_fireflies(80, 24);
        sim
    }

    fn init_fireflies(&mut self, width: u16, height: u16) {
        self.fireflies.clear();
        let count = ((width as usize * height as usize) / 45).clamp(16, 45);
        for _ in 0..count {
            let x = (self.rng.next_u32() % (width.max(1) as u32)) as f32;
            let y = (self.rng.next_u32() % (height.max(1) as u32)) as f32;
            let vx = self.rng.next_signed() * 0.15;
            let vy = self.rng.next_signed() * 0.08;
            let pulse_phase = self.rng.next_f32() * std::f32::consts::TAU;
            let pulse_speed = 0.03 + self.rng.next_f32() * 0.04;
            let glow_type = (self.rng.next_u32() % 2) as u8;

            self.fireflies.push(Firefly {
                x,
                y,
                vx,
                vy,
                pulse_phase,
                pulse_speed,
                glow_type,
            });
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width.max(1);
            self.height = height.max(1);
            self.init_fireflies(self.width, self.height);
        }
    }

    pub fn tick(&mut self) {
        let w = self.width as f32;
        let h = self.height as f32;
        if w <= 1.0 || h <= 1.0 {
            return;
        }

        for fly in &mut self.fireflies {
            // Pulse cycle
            fly.pulse_phase = (fly.pulse_phase + fly.pulse_speed).fract();

            // Organic steering (smooth random walk)
            fly.vx += self.rng.next_signed() * 0.02;
            fly.vy += self.rng.next_signed() * 0.01;

            // Terminal aspect ratio: clamp y speed more than x
            fly.vx = fly.vx.clamp(-0.25, 0.25);
            fly.vy = fly.vy.clamp(-0.12, 0.12);

            fly.x += fly.vx;
            fly.y += fly.vy;

            // Soft wall bounce
            if fly.x < 1.0 {
                fly.x = 1.0;
                fly.vx = fly.vx.abs();
            } else if fly.x >= w - 1.0 {
                fly.x = w - 1.0;
                fly.vx = -fly.vx.abs();
            }

            if fly.y < 1.0 {
                fly.y = 1.0;
                fly.vy = fly.vy.abs();
            } else if fly.y >= h - 1.0 {
                fly.y = h - 1.0;
                fly.vy = -fly.vy.abs();
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

        for fly in &self.fireflies {
            let fx = fly.x.round() as usize;
            let fy = fly.y.round() as usize;
            if fx >= w || fy >= h {
                continue;
            }

            // Glow intensity: raised to power 3 for sharp luminous flash
            let raw_glow = ((fly.pulse_phase * std::f32::consts::TAU).sin() + 1.0) * 0.5;
            let glow = raw_glow * raw_glow * raw_glow;

            if glow > 0.75 {
                // Peak illumination
                let primary_color = if fly.glow_type == 0 {
                    theme.warning // Warm amber/yellow
                } else {
                    theme.success // Bioluminescent emerald
                };

                char_grid[fy][fx] = '✦';
                style_grid[fy][fx] = Style::default().fg(primary_color).add_modifier(Modifier::BOLD);

                // Subtle 4-neighbor halo
                let halo_style = Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM);
                if fx > 0 && char_grid[fy][fx - 1] == ' ' {
                    char_grid[fy][fx - 1] = '·';
                    style_grid[fy][fx - 1] = halo_style;
                }
                if fx + 1 < w && char_grid[fy][fx + 1] == ' ' {
                    char_grid[fy][fx + 1] = '·';
                    style_grid[fy][fx + 1] = halo_style;
                }
                if fy > 0 && char_grid[fy - 1][fx] == ' ' {
                    char_grid[fy - 1][fx] = '·';
                    style_grid[fy - 1][fx] = halo_style;
                }
                if fy + 1 < h && char_grid[fy + 1][fx] == ' ' {
                    char_grid[fy + 1][fx] = '·';
                    style_grid[fy + 1][fx] = halo_style;
                }
            } else if glow > 0.40 {
                let color = if fly.glow_type == 0 { theme.warning } else { theme.success };
                char_grid[fy][fx] = '•';
                style_grid[fy][fx] = Style::default().fg(color);
            } else if glow > 0.18 {
                char_grid[fy][fx] = '·';
                style_grid[fy][fx] = Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM);
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
