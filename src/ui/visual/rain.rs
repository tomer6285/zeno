use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
struct Drop {
    x: u16,
    y: f32,
    speed: f32,
    char: char,
}

#[derive(Debug, Clone)]
struct Splash {
    x: u16,
    #[allow(dead_code)]
    y: u16,
    stage: u8,
}

pub struct RainSimulator {
    rng: FastRng,
    drops: Vec<Drop>,
    splashes: Vec<Splash>,
    width: u16,
    height: u16,
}

impl RainSimulator {
    pub fn new() -> Self {
        Self {
            rng: FastRng::new(999),
            drops: Vec::new(),
            splashes: Vec::new(),
            width: 80,
            height: 24,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width.max(1);
            self.height = height.max(1);
            self.drops.clear();
            self.splashes.clear();
        }
    }

    pub fn tick(&mut self) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        // Spawn new raindrops
        let target_drops = (self.width as usize * self.height as usize) / 35;
        while self.drops.len() < target_drops {
            let x = self.rng.gen_range_u16(0, self.width);
            let speed = self.rng.gen_range_f32(0.6, 1.4);
            let char = match self.rng.gen_range_u16(0, 4) {
                0 => '|',
                1 => '/',
                2 => '│',
                _ => '·',
            };
            self.drops.push(Drop {
                x,
                y: 0.0,
                speed,
                char,
            });
        }

        // Update drops
        let mut i = 0;
        let ground_y = self.height.saturating_sub(1);
        while i < self.drops.len() {
            self.drops[i].y += self.drops[i].speed;
            if self.drops[i].y >= ground_y as f32 {
                let x = self.drops[i].x;
                self.drops.swap_remove(i);
                // Trigger ground splash ripple
                if self.splashes.len() < 30 {
                    self.splashes.push(Splash { x, y: ground_y, stage: 0 });
                }
            } else {
                i += 1;
            }
        }

        // Update splashes
        let mut s = 0;
        while s < self.splashes.len() {
            self.splashes[s].stage += 1;
            if self.splashes[s].stage > 3 {
                self.splashes.swap_remove(s);
            } else {
                s += 1;
            }
        }
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        if area.width == 0 || area.height == 0 {
            return Paragraph::new("");
        }

        let mut grid = vec![vec![' '; area.width as usize]; area.height as usize];

        // Draw raindrops
        for d in &self.drops {
            let x = d.x as usize;
            let y = d.y as usize;
            if y < area.height as usize && x < area.width as usize {
                grid[y][x] = d.char;
            }
        }

        // Draw splashes on bottom row
        let ground = area.height.saturating_sub(1) as usize;
        for sp in &self.splashes {
            let x = sp.x as usize;
            if x < area.width as usize && ground < area.height as usize {
                grid[ground][x] = match sp.stage {
                    1 => '.',
                    2 => 'o',
                    _ => '_',
                };
            }
        }

        let lines: Vec<Line<'static>> = grid
            .into_iter()
            .map(|row| {
                let s: String = row.into_iter().collect();
                Line::from(Span::styled(s, Style::default().fg(theme.rain)))
            })
            .collect();

        Paragraph::new(lines)
    }
}
