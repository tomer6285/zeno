use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

pub struct FireSimulator {
    rng: FastRng,
    buffer: Vec<u8>,
    width: u16,
    height: u16,
}

impl FireSimulator {
    pub fn new() -> Self {
        Self {
            rng: FastRng::new(1234),
            buffer: Vec::new(),
            width: 80,
            height: 24,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height || self.buffer.is_empty() {
            self.width = width.max(1);
            self.height = height.max(1);
            self.buffer = vec![0; (self.width as usize) * (self.height as usize)];
        }
    }

    pub fn tick(&mut self) {
        let w = self.width as usize;
        let h = self.height as usize;
        if w == 0 || h == 0 || self.buffer.len() != w * h {
            return;
        }

        // Generate fiery embers at the bottom line
        let bottom_start = (h - 1) * w;
        for x in 0..w {
            let heat = if self.rng.gen_bool(0.78) {
                self.rng.gen_range_u8(200, 255)
            } else {
                self.rng.gen_range_u8(0, 80)
            };
            self.buffer[bottom_start + x] = heat;
        }

        // Propagate heat upwards (Doom fire simulation)
        for y in (1..h).rev() {
            for x in 0..w {
                let from_idx = y * w + x;
                let heat = self.buffer[from_idx];

                if heat == 0 {
                    if from_idx >= w {
                        self.buffer[from_idx - w] = 0;
                    }
                } else {
                    let decay = self.rng.gen_range_u8(12, 35);
                    let drift = self.rng.gen_range_u8(0, 3);
                    let to_x = if drift == 0 && x > 0 {
                        x - 1
                    } else if drift == 2 && x + 1 < w {
                        x + 1
                    } else {
                        x
                    };

                    let to_idx = (y - 1) * w + to_x;
                    self.buffer[to_idx] = heat.saturating_sub(decay);
                }
            }
        }
    }

    pub fn render(&self, _area: Rect, theme: &Theme) -> Paragraph<'static> {
        let w = self.width as usize;
        let h = self.height as usize;
        if w == 0 || h == 0 || self.buffer.len() != w * h {
            return Paragraph::new("");
        }

        let chars = [' ', '.', ':', '-', '=', '+', '*', '%', '#', '@'];
        let palette = &theme.fire_palette;

        let mut lines = Vec::with_capacity(h);
        for y in 0..h {
            let mut spans = Vec::with_capacity(w);
            for x in 0..w {
                let heat = self.buffer[y * w + x];
                let char_idx = ((heat as usize * (chars.len() - 1)) / 255).min(chars.len() - 1);
                let color_idx = ((heat as usize * (palette.len() - 1)) / 255).min(palette.len() - 1);

                let c = chars[char_idx];
                let color = palette[color_idx];

                spans.push(Span::styled(c.to_string(), Style::default().fg(color)));
            }
            lines.push(Line::from(spans));
        }

        Paragraph::new(lines)
    }
}
