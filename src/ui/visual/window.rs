use std::collections::VecDeque;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
struct StaticDrop {
    x: u16,
    y: u16,
    char_glyph: char,
    age: u8,
    max_age: u8,
}

#[derive(Debug, Clone)]
struct Rivulet {
    x: f32,
    y: f32,
    speed: f32,
    trail: VecDeque<(u16, u16, char)>,
    max_trail: usize,
    active: bool,
}

pub struct WindowSimulator {
    rng: FastRng,
    static_drops: Vec<StaticDrop>,
    rivulets: Vec<Rivulet>,
    width: u16,
    height: u16,
    tick_count: u64,
    lightning_flash: u8,
}

impl WindowSimulator {
    pub fn new() -> Self {
        let mut sim = Self {
            rng: FastRng::new(2026),
            static_drops: Vec::new(),
            rivulets: Vec::new(),
            width: 80,
            height: 24,
            tick_count: 0,
            lightning_flash: 0,
        };
        sim.init_drops(80, 24);
        sim
    }

    fn init_drops(&mut self, width: u16, height: u16) {
        self.static_drops.clear();
        let count = ((width as usize * height as usize) / 35).clamp(15, 120);
        for _ in 0..count {
            let x = (self.rng.next_u32() % (width.max(1) as u32)) as u16;
            let y = (self.rng.next_u32() % (height.max(1) as u32)) as u16;
            let glyph = match self.rng.next_u32() % 4 {
                0 => '·',
                1 => '•',
                2 => '°',
                _ => '·',
            };
            let max_age = 60 + (self.rng.next_u32() % 120) as u8;
            self.static_drops.push(StaticDrop {
                x,
                y,
                char_glyph: glyph,
                age: (self.rng.next_u32() % max_age as u32) as u8,
                max_age,
            });
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width.max(1);
            self.height = height.max(1);
            self.init_drops(self.width, self.height);
            self.rivulets.clear();
        }
    }

    pub fn tick(&mut self, spectrum_energy: &[f32; 16]) {
        self.tick_count = self.tick_count.wrapping_add(1);

        if self.lightning_flash > 0 {
            self.lightning_flash -= 1;
        }

        let bass = (spectrum_energy[0] + spectrum_energy[1]) * 0.5;
        let rain_energy = (spectrum_energy[4] + spectrum_energy[5] + spectrum_energy[6]) / 3.0;

        // Occasional lightning shimmer on heavy bass
        if bass > 0.85 && self.rng.next_f32() < 0.04 && self.lightning_flash == 0 {
            self.lightning_flash = 2;
        }

        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // 1. Age static condensation drops
        for drop in &mut self.static_drops {
            drop.age = drop.age.saturating_add(1);
            if drop.age >= drop.max_age {
                drop.x = (self.rng.next_u32() % (w as u32)) as u16;
                drop.y = (self.rng.next_u32() % (h as u32)) as u16;
                drop.age = 0;
                drop.max_age = 60 + (self.rng.next_u32() % 100) as u8;
                drop.char_glyph = match self.rng.next_u32() % 5 {
                    0 => '·',
                    1 => '•',
                    2 => '°',
                    3 => '◦',
                    _ => '·',
                };
            }
        }

        // 2. Spawn sliding rivulets when condensation builds up
        let spawn_chance = 0.05 + rain_energy * 0.12;
        if self.rivulets.len() < 8 && self.rng.next_f32() < spawn_chance {
            let start_x = (self.rng.next_u32() % (w as u32)) as f32;
            let start_y = (self.rng.next_f32() * (h as f32 * 0.3)).floor();
            let speed = 0.25 + self.rng.next_f32() * 0.45;
            let max_trail = 4 + (self.rng.next_u32() % 6) as usize;

            self.rivulets.push(Rivulet {
                x: start_x,
                y: start_y,
                speed,
                trail: VecDeque::new(),
                max_trail,
                active: true,
            });
        }

        // 3. Update rivulets
        for riv in &mut self.rivulets {
            if !riv.active {
                continue;
            }

            let prev_x = riv.x.round() as u16;
            let prev_y = riv.y.floor() as u16;

            riv.y += riv.speed;

            // Meander left or right occasionally
            let meander = self.rng.next_signed();
            let mut trail_glyph = '│';
            if meander < -0.45 {
                riv.x = (riv.x - 0.3).max(0.0);
                trail_glyph = '╱';
            } else if meander > 0.45 {
                riv.x = (riv.x + 0.3).min((w.saturating_sub(1)) as f32);
                trail_glyph = '╲';
            }

            let _new_x = riv.x.round() as u16;
            let new_y = riv.y.floor() as u16;

            if new_y != prev_y && prev_y < h && prev_x < w {
                riv.trail.push_back((prev_x, prev_y, trail_glyph));
                if riv.trail.len() > riv.max_trail {
                    riv.trail.pop_front();
                }
            }

            if riv.y >= h as f32 {
                if !riv.trail.is_empty() {
                    riv.trail.pop_front();
                }
                if riv.trail.is_empty() {
                    riv.active = false;
                }
            }
        }

        self.rivulets.retain(|r| r.active);
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        let w = area.width as usize;
        let h = area.height as usize;
        if w == 0 || h == 0 {
            return Paragraph::new("");
        }

        let is_lightning = self.lightning_flash > 0;
        let mut char_grid = vec![vec![' '; w]; h];
        let mut style_grid = vec![vec![Style::default(); w]; h];

        // 1. Static condensation drops
        let drop_style = if is_lightning {
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM)
        };

        for drop in &self.static_drops {
            let x = drop.x as usize;
            let y = drop.y as usize;
            if x < w && y < h {
                char_grid[y][x] = drop.char_glyph;
                style_grid[y][x] = drop_style;
            }
        }

        // 2. Rivulet trails and moving heads
        for riv in &self.rivulets {
            // Trails
            for (i, &(tx, ty, ch)) in riv.trail.iter().enumerate() {
                let x = tx as usize;
                let y = ty as usize;
                if x < w && y < h {
                    char_grid[y][x] = ch;
                    let trail_ratio = (i + 1) as f32 / riv.trail.len() as f32;
                    let color = if trail_ratio > 0.6 {
                        theme.primary
                    } else {
                        theme.secondary
                    };
                    style_grid[y][x] = Style::default().fg(color).add_modifier(Modifier::DIM);
                }
            }

            // Head droplet
            let hx = riv.x.round() as usize;
            let hy = riv.y.floor() as usize;
            if hx < w && hy < h {
                char_grid[hy][hx] = '●';
                style_grid[hy][hx] = Style::default().fg(theme.accent).add_modifier(Modifier::BOLD);
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
