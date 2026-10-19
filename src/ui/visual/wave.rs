use std::f32::consts::PI;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::ui::theme::Theme;

pub struct WaveSimulator {
    phase: f32,
    smoothed_bands: [f32; 16],
}

impl WaveSimulator {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            smoothed_bands: [0.0; 16],
        }
    }

    pub fn tick(&mut self, spectrum_energy: &[f32; 16]) {
        self.phase = (self.phase + 0.08).fract();

        // Smoothly interpolate towards target spectrum energy
        for i in 0..16 {
            let target = spectrum_energy[i];
            let diff = target - self.smoothed_bands[i];
            self.smoothed_bands[i] += diff * 0.25;
            self.smoothed_bands[i] = self.smoothed_bands[i].clamp(0.0, 1.0);
        }
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        let w = area.width as usize;
        let h = area.height as usize;
        if w == 0 || h == 0 {
            return Paragraph::new("");
        }

        let mut lines = Vec::with_capacity(h);
        let blocks = [' ', ' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        // Split panel: top half spectrum bars, bottom half oscillating wave
        let bar_height = (h * 2) / 3;
        let wave_height = h.saturating_sub(bar_height);

        // Render equalizer spectrum columns
        let num_bars = 16.min(w / 3).max(4);
        let col_width = (w / num_bars).max(1);

        for row in 0..bar_height {
            let level_from_bottom = bar_height.saturating_sub(1).saturating_sub(row);
            let row_ratio = level_from_bottom as f32 / bar_height as f32;

            let mut spans = Vec::new();
            for bar_idx in 0..num_bars {
                let energy = self.smoothed_bands[bar_idx % 16];
                let is_filled = energy >= row_ratio;

                let char = if is_filled {
                    let fractional = ((energy - row_ratio) * bar_height as f32 * 8.0) as usize;
                    blocks[fractional.min(8)]
                } else {
                    ' '
                };

                let color = if energy > 0.8 {
                    theme.error
                } else if energy > 0.5 {
                    theme.warning
                } else {
                    theme.accent
                };

                let s = char.to_string().repeat(col_width.saturating_sub(1).max(1));
                spans.push(Span::styled(s, Style::default().fg(color)));
                spans.push(Span::raw(" "));
            }
            lines.push(Line::from(spans));
        }

        // Render animated sine waveform on remaining rows
        for row in 0..wave_height {
            let mut spans = Vec::with_capacity(w);
            let row_center = (wave_height as f32 / 2.0).max(1.0);

            for col in 0..w {
                let x = col as f32 / w as f32;
                // Modulate with active audio energy
                let overall_energy: f32 = self.smoothed_bands.iter().sum::<f32>() / 16.0;
                let amp = 0.8 + overall_energy * 2.2;

                let y_val = ((x * 4.0 * PI + self.phase * 2.0 * PI).sin()
                    + (x * 2.0 * PI - self.phase * 3.0 * PI).cos() * 0.5)
                    * amp;

                let target_row = (row_center + y_val).round() as isize;

                let (ch, color) = if target_row == row as isize {
                    ('~', theme.primary)
                } else if (target_row - row as isize).abs() == 1 {
                    ('·', theme.secondary)
                } else {
                    (' ', theme.text_muted)
                };

                spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }
            lines.push(Line::from(spans));
        }

        Paragraph::new(lines)
    }
}
