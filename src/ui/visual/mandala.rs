use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::ui::theme::Theme;

pub struct MandalaSimulator {
    tick_count: u64,
    width: u16,
    height: u16,
}

impl MandalaSimulator {
    pub fn new() -> Self {
        Self {
            tick_count: 0,
            width: 80,
            height: 24,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width.max(1);
        self.height = height.max(1);
    }

    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        let w = area.width as usize;
        let h = area.height as usize;
        if w == 0 || h == 0 {
            return Paragraph::new("");
        }

        // 30 FPS tick rate: 480 ticks = 16 seconds full box-breathing cycle
        let cycle_ticks = 480;
        let current_tick = (self.tick_count % cycle_ticks) as f32;
        let phase_ticks = 120.0; // 4 seconds per phase

        let (phase_name, progress, _is_expanding) = if current_tick < phase_ticks {
            ("Inhale", current_tick / phase_ticks, true)
        } else if current_tick < phase_ticks * 2.0 {
            ("Hold", (current_tick - phase_ticks) / phase_ticks, false)
        } else if current_tick < phase_ticks * 3.0 {
            ("Exhale", (current_tick - phase_ticks * 2.0) / phase_ticks, false)
        } else {
            ("Rest", (current_tick - phase_ticks * 3.0) / phase_ticks, false)
        };

        // Determine target radius
        let max_r = (h as f32 * 0.38).clamp(3.0, 10.0);
        let min_r = 2.0;

        let r = match phase_name {
            "Inhale" => min_r + (max_r - min_r) * (progress * std::f32::consts::PI * 0.5).sin(),
            "Hold" => max_r,
            "Exhale" => max_r - (max_r - min_r) * (progress * std::f32::consts::PI * 0.5).sin(),
            _ => min_r,
        };

        let cx = w as f32 * 0.5;
        let cy = (h as f32 * 0.45).max(1.0);

        let mut char_grid = vec![vec![' '; w]; h];
        let mut style_grid = vec![vec![Style::default(); w]; h];

        let phase_color = match phase_name {
            "Inhale" => theme.primary,
            "Hold" => theme.accent,
            "Exhale" => theme.success,
            _ => theme.secondary,
        };

        // 1. Center Sacred Blossom
        let center_x = cx.round() as usize;
        let center_y = cy.round() as usize;
        if center_x < w && center_y < h {
            char_grid[center_y][center_x] = '✿';
            style_grid[center_y][center_x] = Style::default().fg(theme.accent).add_modifier(Modifier::BOLD);
        }

        // 2. Concentric geometric petals (8-fold and 16-fold symmetry)
        let petal_count = 12;
        let angle_step = std::f32::consts::TAU / petal_count as f32;

        // Outer ring
        for i in 0..petal_count {
            let angle = i as f32 * angle_step + (self.tick_count as f32 * 0.005);
            let px = cx + angle.cos() * r * 2.0; // Aspect ratio ~2x width
            let py = cy + angle.sin() * r;

            let x = px.round() as usize;
            let y = py.round() as usize;

            if x < w && y < h {
                char_grid[y][x] = if i % 2 == 0 { '◈' } else { '◇' };
                style_grid[y][x] = Style::default().fg(phase_color).add_modifier(Modifier::BOLD);
            }
        }

        // Inner petal ring
        if r > 3.0 {
            let inner_r = r * 0.55;
            for i in 0..petal_count {
                let angle = i as f32 * angle_step - (self.tick_count as f32 * 0.008) + (angle_step * 0.5);
                let px = cx + angle.cos() * inner_r * 2.0;
                let py = cy + angle.sin() * inner_r;

                let x = px.round() as usize;
                let y = py.round() as usize;

                if x < w && y < h {
                    char_grid[y][x] = if i % 2 == 0 { '○' } else { '•' };
                    style_grid[y][x] = Style::default().fg(theme.primary);
                }
            }
        }

        // Subtle radiating spokes / aura
        for y in 0..h {
            for x in 0..w {
                let dx = (x as f32 - cx) / 2.0;
                let dy = y as f32 - cy;
                let dist = (dx * dx + dy * dy).sqrt();

                if (dist - r).abs() < 0.6 && char_grid[y][x] == ' ' {
                    let angle = dy.atan2(dx);
                    let ch = if angle.sin().abs() > 0.85 {
                        '─'
                    } else if angle.cos().abs() > 0.85 {
                        '│'
                    } else {
                        '·'
                    };
                    char_grid[y][x] = ch;
                    style_grid[y][x] = Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM);
                }
            }
        }

        // 3. Status text and breathing guide indicator
        let guide_y = (h.saturating_sub(2)).min(h.saturating_sub(1));
        let dots = match (progress * 4.0).floor() as usize {
            0 => "● ○ ○ ○",
            1 => "● ● ○ ○",
            2 => "● ● ● ○",
            _ => "● ● ● ●",
        };
        let status_text = format!("• {} ({}) •", phase_name, dots);
        let start_x = (w.saturating_sub(status_text.chars().count())) / 2;

        for (i, ch) in status_text.chars().enumerate() {
            let x = start_x + i;
            if x < w && guide_y < h {
                char_grid[guide_y][x] = ch;
                style_grid[guide_y][x] = Style::default().fg(phase_color).add_modifier(Modifier::BOLD);
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
