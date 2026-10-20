use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

pub struct LifeSimulator {
    rng: FastRng,
    grid: Vec<bool>,
    next_grid: Vec<bool>,
    decay: Vec<u8>,
    width: u16,
    height: u16,
    tick_counter: u8,
    stagnation_ticks: u32,
    prev_pop: usize,
}

impl LifeSimulator {
    pub fn new() -> Self {
        let mut sim = Self {
            rng: FastRng::new(4242),
            grid: Vec::new(),
            next_grid: Vec::new(),
            decay: Vec::new(),
            width: 80,
            height: 24,
            tick_counter: 0,
            stagnation_ticks: 0,
            prev_pop: 0,
        };
        sim.init_grid(80, 24);
        sim
    }

    fn init_grid(&mut self, width: u16, height: u16) {
        let size = (width as usize) * (height as usize);
        self.grid = vec![false; size];
        self.next_grid = vec![false; size];
        self.decay = vec![0; size];

        // Random initial seeding (sparse 18% density)
        for i in 0..size {
            if self.rng.next_f32() < 0.18 {
                self.grid[i] = true;
            }
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width.max(1);
            self.height = height.max(1);
            self.init_grid(self.width, self.height);
        }
    }

    fn spawn_glider(&mut self, top_left_x: usize, top_left_y: usize) {
        let w = self.width as usize;
        let h = self.height as usize;
        let pattern = [
            (0, 1), (1, 2), (2, 0), (2, 1), (2, 2)
        ];
        for (dx, dy) in pattern {
            let x = (top_left_x + dx) % w;
            let y = (top_left_y + dy) % h;
            self.grid[y * w + x] = true;
        }
    }

    fn spawn_random_cluster(&mut self) {
        let w = self.width as usize;
        let h = self.height as usize;
        if w < 5 || h < 5 {
            return;
        }
        let cx = (self.rng.next_u32() % (w as u32)) as usize;
        let cy = (self.rng.next_u32() % (h as u32)) as usize;

        if self.rng.next_f32() < 0.5 {
            self.spawn_glider(cx, cy);
        } else {
            // 3x3 seed
            for dy in 0..3 {
                for dx in 0..3 {
                    if self.rng.next_f32() < 0.6 {
                        let x = (cx + dx) % w;
                        let y = (cy + dy) % h;
                        self.grid[y * w + x] = true;
                    }
                }
            }
        }
    }

    pub fn tick(&mut self) {
        self.tick_counter = (self.tick_counter + 1) % 3; // Update every 3 ticks (~10 gen/s)
        if self.tick_counter != 0 {
            return;
        }

        let w = self.width as usize;
        let h = self.height as usize;
        let size = w * h;
        if size == 0 || self.grid.len() != size {
            return;
        }

        let mut current_pop = 0;

        for y in 0..h {
            let y_up = if y == 0 { h - 1 } else { y - 1 };
            let y_down = if y + 1 == h { 0 } else { y + 1 };

            for x in 0..w {
                let x_left = if x == 0 { w - 1 } else { x - 1 };
                let x_right = if x + 1 == w { 0 } else { x + 1 };

                let idx = y * w + x;
                let alive = self.grid[idx];

                let neighbors =
                    self.grid[y_up * w + x_left] as u8
                    + self.grid[y_up * w + x] as u8
                    + self.grid[y_up * w + x_right] as u8
                    + self.grid[y * w + x_left] as u8
                    + self.grid[y * w + x_right] as u8
                    + self.grid[y_down * w + x_left] as u8
                    + self.grid[y_down * w + x] as u8
                    + self.grid[y_down * w + x_right] as u8;

                let next_alive = if alive {
                    neighbors == 2 || neighbors == 3
                } else {
                    neighbors == 3
                };

                self.next_grid[idx] = next_alive;

                if next_alive {
                    current_pop += 1;
                    self.decay[idx] = 4; // Max life
                } else if self.decay[idx] > 0 {
                    self.decay[idx] -= 1;
                }
            }
        }

        std::mem::swap(&mut self.grid, &mut self.next_grid);

        // Stagnation / extinction recovery
        if current_pop == self.prev_pop || current_pop < 12 {
            self.stagnation_ticks += 1;
        } else {
            self.stagnation_ticks = 0;
        }
        self.prev_pop = current_pop;

        if self.stagnation_ticks > 25 || current_pop < 8 {
            self.spawn_random_cluster();
            self.stagnation_ticks = 0;
        }
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        let w = area.width as usize;
        let h = area.height as usize;
        let size = w * h;
        if size == 0 || self.grid.len() != size {
            return Paragraph::new("");
        }

        let mut char_grid = vec![vec![' '; w]; h];
        let mut style_grid = vec![vec![Style::default(); w]; h];

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                if self.grid[idx] {
                    char_grid[y][x] = '█';
                    style_grid[y][x] = Style::default().fg(theme.primary).add_modifier(Modifier::BOLD);
                } else {
                    let d = self.decay[idx];
                    if d > 0 {
                        let (ch, style) = match d {
                            3 => ('▓', Style::default().fg(theme.secondary)),
                            2 => ('▒', Style::default().fg(theme.accent).add_modifier(Modifier::DIM)),
                            1 => ('░', Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM)),
                            _ => ('·', Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM)),
                        };
                        char_grid[y][x] = ch;
                        style_grid[y][x] = style;
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
