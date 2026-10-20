use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
struct TwinkleStar {
    x: u16,
    y: u16,
    phase: f32,
    speed: f32,
}

pub struct AuroraSimulator {
    rng: FastRng,
    stars: Vec<TwinkleStar>,
    time: f32,
    width: u16,
    height: u16,
}

impl AuroraSimulator {
    pub fn new() -> Self {
        let mut sim = Self {
            rng: FastRng::new(9090),
            stars: Vec::new(),
            time: 0.0,
            width: 80,
            height: 24,
        };
        sim.init_stars(80, 24);
        sim
    }

    fn init_stars(&mut self, width: u16, height: u16) {
        self.stars.clear();
        let count = ((width as usize * height as usize) / 50).clamp(15, 60);
        for _ in 0..count {
            let x = (self.rng.next_u32() % (width.max(1) as u32)) as u16;
            let y = (self.rng.next_u32() % (height.max(1) as u32)) as u16;
            let phase = self.rng.next_f32() * std::f32::consts::TAU;
            let speed = 0.02 + self.rng.next_f32() * 0.03;
            self.stars.push(TwinkleStar { x, y, phase, speed });
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width.max(1);
            self.height = height.max(1);
            self.init_stars(self.width, self.height);
        }
    }

    pub fn tick(&mut self, spectrum_energy: &[f32; 16]) {
        let energy = (spectrum_energy[0] + spectrum_energy[1] + spectrum_energy[2]) / 3.0;
        self.time += 0.035 + energy * 0.03;

        for s in &mut self.stars {
            s.phase = (s.phase + s.speed).fract();
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

        // 1. Distant twinkling stars behind the aurora
        for s in &self.stars {
            let sx = s.x as usize;
            let sy = s.y as usize;
            if sx < w && sy < h {
                let tw = (s.phase * std::f32::consts::TAU).sin();
                if tw > 0.3 {
                    char_grid[sy][sx] = if tw > 0.85 { '✦' } else { '·' };
                    let style = if tw > 0.85 {
                        Style::default().fg(theme.primary).add_modifier(Modifier::DIM)
                    } else {
                        Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM)
                    };
                    style_grid[sy][sx] = style;
                }
            }
        }

        // 2. Multi-harmonic Aurora Curtains
        let t = self.time;
        for x in 0..w {
            let x_f = x as f32;

            // Curtain 1 (Emerald Primary Ribbon)
            let wave1 = (x_f * 0.08 + t * 0.8).sin() * 3.5
                + (x_f * 0.18 - t * 0.5).cos() * 2.0;
            let curtain1_top = (h as f32 * 0.40) + wave1;
            let curtain1_len = 5.0 + (x_f * 0.12 + t * 0.4).sin().abs() * 4.0;

            // Curtain 2 (Violet / Cyan Secondary Ribbon)
            let wave2 = (x_f * 0.06 - t * 0.6).sin() * 4.0
                + (x_f * 0.14 + t * 0.7).cos() * 2.5;
            let curtain2_top = (h as f32 * 0.25) + wave2;
            let curtain2_len = 4.0 + (x_f * 0.09 - t * 0.5).cos().abs() * 5.0;

            for y in 0..h {
                let y_f = y as f32;

                // Test Curtain 1
                if y_f >= curtain1_top && y_f <= curtain1_top + curtain1_len {
                    let depth = (y_f - curtain1_top) / curtain1_len;
                    let ray_hash = (x * 17 + y * 7) % 5;

                    let (ch, style) = if depth < 0.25 {
                        // Bright crest
                        ('█', Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
                    } else if depth < 0.55 {
                        let c = if ray_hash == 0 { '║' } else if ray_hash == 1 { '▓' } else { '▒' };
                        (c, Style::default().fg(theme.primary))
                    } else if depth < 0.85 {
                        let c = if ray_hash % 2 == 0 { '│' } else { '░' };
                        (c, Style::default().fg(theme.secondary).add_modifier(Modifier::DIM))
                    } else {
                        ('~', Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM))
                    };

                    char_grid[y][x] = ch;
                    style_grid[y][x] = style;
                }
                // Test Curtain 2 (Higher crown)
                else if y_f >= curtain2_top && y_f <= curtain2_top + curtain2_len && char_grid[y][x] == ' ' {
                    let depth = (y_f - curtain2_top) / curtain2_len;
                    let (ch, style) = if depth < 0.3 {
                        ('▓', Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
                    } else if depth < 0.7 {
                        ('▒', Style::default().fg(theme.accent).add_modifier(Modifier::DIM))
                    } else {
                        ('░', Style::default().fg(theme.primary).add_modifier(Modifier::DIM))
                    };

                    char_grid[y][x] = ch;
                    style_grid[y][x] = style;
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
