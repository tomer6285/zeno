use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
struct Ripple {
    cx: f32,
    cy: f32,
    radius: f32,
    max_radius: f32,
    speed: f32,
    intensity: f32,
}

#[derive(Debug, Clone)]
struct LilyPad {
    x: u16,
    y: u16,
    glyph: &'static str,
}

pub struct RipplesSimulator {
    rng: FastRng,
    ripples: Vec<Ripple>,
    lily_pads: Vec<LilyPad>,
    width: u16,
    height: u16,
    water_phase: f32,
}

impl RipplesSimulator {
    pub fn new() -> Self {
        let mut sim = Self {
            rng: FastRng::new(5555),
            ripples: Vec::new(),
            lily_pads: Vec::new(),
            width: 80,
            height: 24,
            water_phase: 0.0,
        };
        sim.init_pads(80, 24);
        sim
    }

    fn init_pads(&mut self, width: u16, height: u16) {
        self.lily_pads.clear();
        let count = ((width as usize * height as usize) / 250).clamp(2, 6);
        for _ in 0..count {
            let x = (self.rng.next_u32() % (width.max(1) as u32)) as u16;
            let y = (self.rng.next_u32() % (height.max(1) as u32)) as u16;
            let glyph = match self.rng.next_u32() % 3 {
                0 => "❀",
                1 => "☘",
                _ => "•",
            };
            self.lily_pads.push(LilyPad { x, y, glyph });
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width.max(1);
            self.height = height.max(1);
            self.init_pads(self.width, self.height);
            self.ripples.clear();
        }
    }

    pub fn tick(&mut self, spectrum_energy: &[f32; 16]) {
        self.water_phase = (self.water_phase + 0.03).fract();

        let energy = (spectrum_energy[1] + spectrum_energy[2] + spectrum_energy[3]) / 3.0;
        let spawn_chance = 0.08 + energy * 0.15;

        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // Spawn new ripple
        if self.ripples.len() < 7 && self.rng.next_f32() < spawn_chance {
            let cx = (self.rng.next_u32() % (w as u32)) as f32;
            let cy = (self.rng.next_u32() % (h as u32)) as f32;
            let max_radius = 6.0 + self.rng.next_f32() * 8.0;
            let speed = 0.28 + self.rng.next_f32() * 0.22;

            self.ripples.push(Ripple {
                cx,
                cy,
                radius: 0.0,
                max_radius,
                speed,
                intensity: 1.0,
            });
        }

        // Update active ripples
        for r in &mut self.ripples {
            r.radius += r.speed;
            r.intensity = (1.0 - (r.radius / r.max_radius)).clamp(0.0, 1.0);
        }

        self.ripples.retain(|r| r.radius < r.max_radius && r.intensity > 0.05);
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        let w = area.width as usize;
        let h = area.height as usize;
        if w == 0 || h == 0 {
            return Paragraph::new("");
        }

        let mut char_grid = vec![vec![' '; w]; h];
        let mut style_grid = vec![vec![Style::default(); w]; h];

        // 1. Water baseline shimmer
        for y in 0..h {
            for x in 0..w {
                let wave = ((x as f32 * 0.25 + y as f32 * 0.35 + self.water_phase * std::f32::consts::TAU).sin() + 1.0) * 0.5;
                if wave > 0.94 {
                    char_grid[y][x] = '~';
                    style_grid[y][x] = Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM);
                } else if wave > 0.88 && (x + y) % 3 == 0 {
                    char_grid[y][x] = '·';
                    style_grid[y][x] = Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM);
                }
            }
        }

        // 2. Lily pads & blossoms
        for pad in &self.lily_pads {
            let px = pad.x as usize;
            let py = pad.y as usize;
            if px < w && py < h {
                let ch = pad.glyph.chars().next().unwrap_or('•');
                char_grid[py][px] = ch;
                let color = if pad.glyph == "❀" {
                    theme.accent
                } else {
                    theme.success
                };
                style_grid[py][px] = Style::default().fg(color).add_modifier(Modifier::BOLD);
            }
        }

        // 3. Render expanding ripples
        for r in &self.ripples {
            let r_curr = r.radius;
            let int = r.intensity;

            // Bounding box for this ripple
            let min_x = (r.cx - r_curr - 2.0).max(0.0) as usize;
            let max_x = ((r.cx + r_curr + 2.0).ceil() as usize).min(w);
            let min_y = (r.cy - (r_curr * 0.5) - 1.0).max(0.0) as usize;
            let max_y = ((r.cy + (r_curr * 0.5) + 1.0).ceil() as usize).min(h);

            for y in min_y..max_y {
                for x in min_x..max_x {
                    let dx = x as f32 - r.cx;
                    let dy = y as f32 - r.cy;
                    // Terminal character aspect ratio correction: height is ~2x width
                    let dist = (dx * dx + (dy * 2.0) * (dy * 2.0)).sqrt();
                    let diff = (dist - r_curr).abs();

                    if diff < 0.75 {
                        let angle = (dy * 2.0).atan2(dx);
                        let abs_sin = angle.sin().abs();
                        let abs_cos = angle.cos().abs();

                        let ch = if r_curr < 1.4 {
                            'o'
                        } else if abs_sin > 0.85 {
                            '─'
                        } else if abs_cos > 0.85 {
                            if dx > 0.0 { ')' } else { '(' }
                        } else if (dx > 0.0 && dy > 0.0) || (dx < 0.0 && dy < 0.0) {
                            '╮'
                        } else {
                            '╭'
                        };

                        let style = if int > 0.7 {
                            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
                        } else if int > 0.35 {
                            Style::default().fg(theme.secondary)
                        } else {
                            Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM)
                        };

                        char_grid[y][x] = ch;
                        style_grid[y][x] = style;
                    }
                }
            }

            // Splash center droplet
            let scx = r.cx.round() as usize;
            let scy = r.cy.round() as usize;
            if r_curr < 2.0 && scx < w && scy < h {
                char_grid[scy][scx] = '•';
                style_grid[scy][scx] = Style::default().fg(theme.primary).add_modifier(Modifier::BOLD);
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
