use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::audio::synth::FastRng;
use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropLayer {
    Background,
    Midground,
    Foreground,
}

#[derive(Debug, Clone)]
struct Drop {
    x: f32,
    y: f32,
    speed: f32,
    layer: DropLayer,
}

#[derive(Debug, Clone)]
struct Splash {
    x: u16,
    stage: u8,
    layer: DropLayer,
}

#[derive(Debug, Clone)]
struct Mist {
    x: f32,
    y: u16,
    speed: f32,
}

pub struct RainSimulator {
    rng: FastRng,
    drops: Vec<Drop>,
    splashes: Vec<Splash>,
    mist: Vec<Mist>,
    width: u16,
    height: u16,
    tick_count: u64,
    wind: f32,
    target_wind: f32,
    lightning_timer: u32,
    lightning_flash: u8,
    water_phase: f32,
}

impl RainSimulator {
    pub fn new() -> Self {
        let mut sim = Self {
            rng: FastRng::new(999),
            drops: Vec::new(),
            splashes: Vec::new(),
            mist: Vec::new(),
            width: 80,
            height: 24,
            tick_count: 0,
            wind: 0.0,
            target_wind: 0.1,
            lightning_timer: 0,
            lightning_flash: 0,
            water_phase: 0.0,
        };
        sim.init_mist();
        sim.seed_drops(true);
        sim
    }

    fn init_mist(&mut self) {
        self.mist.clear();
        let mist_count = (self.width / 12).clamp(3, 8) as usize;
        for _ in 0..mist_count {
            self.mist.push(Mist {
                x: self.rng.gen_range_f32(0.0, self.width as f32),
                y: self.rng.gen_range_u16(0, 2.min(self.height)),
                speed: self.rng.gen_range_f32(0.04, 0.12),
            });
        }
    }

    fn seed_drops(&mut self, distribute_y: bool) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        let area_cells = (self.width as usize) * (self.height as usize);
        let target_drops = (area_cells / 18).clamp(8, 250);

        while self.drops.len() < target_drops {
            let x = self.rng.gen_range_f32(0.0, self.width as f32);
            let y = if distribute_y {
                self.rng.gen_range_f32(0.0, self.height as f32)
            } else {
                self.rng.gen_range_f32(-3.0, 0.0)
            };

            let roll = self.rng.gen_range_u16(0, 100);
            let (layer, speed) = if roll < 45 {
                (DropLayer::Background, self.rng.gen_range_f32(0.45, 0.85))
            } else if roll < 85 {
                (DropLayer::Midground, self.rng.gen_range_f32(1.1, 1.65))
            } else {
                (DropLayer::Foreground, self.rng.gen_range_f32(1.8, 2.7))
            };

            self.drops.push(Drop { x, y, speed, layer });
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width.max(1);
            self.height = height.max(1);
            self.drops.clear();
            self.splashes.clear();
            self.init_mist();
            self.seed_drops(true);
        }
    }

    pub fn tick(&mut self, spectrum_energy: &[f32; 16]) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        self.tick_count = self.tick_count.wrapping_add(1);
        self.water_phase = (self.water_phase + 0.07).fract();

        // 1. Wind breeze dynamics: smoothly wandering natural breeze
        if self.tick_count % 90 == 0 {
            self.target_wind = self.rng.gen_range_f32(-0.35, 0.35);
        }
        self.wind += (self.target_wind - self.wind) * 0.035;

        // 2. Lightning / Thunder ambiance:
        // Triggered occasionally, or reacting to thunder bass in audio
        self.lightning_timer = self.lightning_timer.wrapping_add(1);
        let thunder_energy = spectrum_energy[1]; // track thunder/bass
        if self.lightning_flash == 0 {
            let thunder_trigger = thunder_energy > 0.45 && self.lightning_timer > 120 && self.rng.gen_bool(0.20);
            let ambient_trigger = self.lightning_timer > 500 && self.rng.gen_bool(0.008);
            if thunder_trigger || ambient_trigger {
                self.lightning_flash = 2; // Peak flash
                self.lightning_timer = 0;
            }
        } else {
            self.lightning_flash -= 1;
        }

        // 3. Audio-reactive rain density:
        let rain_energy = (spectrum_energy[4..13].iter().sum::<f32>() / 9.0).clamp(0.0, 1.0);
        let area_cells = (self.width as usize) * (self.height as usize);
        let base_density = if rain_energy > 0.05 {
            (area_cells as f32 / (22.0 - rain_energy * 8.0)).round() as usize
        } else {
            area_cells / 18
        };
        let target_drops = base_density.clamp(8, 280);

        // Spawn new raindrops at the top
        while self.drops.len() < target_drops {
            let x = self.rng.gen_range_f32(0.0, self.width as f32);
            let y = self.rng.gen_range_f32(-3.0, 0.0);
            let roll = self.rng.gen_range_u16(0, 100);
            let (layer, speed) = if roll < 45 {
                (DropLayer::Background, self.rng.gen_range_f32(0.45, 0.85))
            } else if roll < 85 {
                (DropLayer::Midground, self.rng.gen_range_f32(1.1, 1.65))
            } else {
                (DropLayer::Foreground, self.rng.gen_range_f32(1.8, 2.7))
            };
            self.drops.push(Drop { x, y, speed, layer });
        }

        // 4. Update falling raindrops
        let ground_y = self.height.saturating_sub(1);
        let mut i = 0;
        let w_f32 = self.width as f32;
        while i < self.drops.len() {
            self.drops[i].y += self.drops[i].speed;
            self.drops[i].x += self.wind * (self.drops[i].speed * 0.4);

            // Wrap horizontal coordinates
            if self.drops[i].x < 0.0 {
                self.drops[i].x += w_f32;
            } else if self.drops[i].x >= w_f32 {
                self.drops[i].x -= w_f32;
            }

            if self.drops[i].y >= ground_y as f32 {
                let splash_x = self.drops[i].x.round().clamp(0.0, (self.width.saturating_sub(1)) as f32) as u16;
                let layer = self.drops[i].layer;
                self.drops.swap_remove(i);

                // Spawn splash on ground
                if self.splashes.len() < 40 {
                    self.splashes.push(Splash {
                        x: splash_x,
                        stage: 0,
                        layer,
                    });
                }
            } else {
                i += 1;
            }
        }

        // 5. Update ground puddle splashes
        let mut s = 0;
        while s < self.splashes.len() {
            self.splashes[s].stage += 1;
            let max_stage = match self.splashes[s].layer {
                DropLayer::Foreground => 6,
                DropLayer::Midground => 4,
                DropLayer::Background => 2,
            };
            if self.splashes[s].stage > max_stage {
                self.splashes.swap_remove(s);
            } else {
                s += 1;
            }
        }

        // 6. Update drifting mist particles
        for m in &mut self.mist {
            m.x += self.wind * 0.25 + m.speed;
            if m.x < 0.0 {
                m.x += w_f32;
            } else if m.x >= w_f32 {
                m.x -= w_f32;
            }
        }
    }

    #[allow(dead_code)]
    pub fn tick_default(&mut self) {
        self.tick(&[0.0; 16]);
    }

    pub fn render(&self, area: Rect, theme: &Theme) -> Paragraph<'static> {
        let w = area.width as usize;
        let h = area.height as usize;
        if w == 0 || h == 0 {
            return Paragraph::new("");
        }

        let is_lightning = self.lightning_flash > 0;
        let ground_y = h.saturating_sub(1);

        // Initialize 2D grid of characters and styles
        let default_style = Style::default();
        let mut char_grid = vec![vec![' '; w]; h];
        let mut style_grid = vec![vec![default_style; w]; h];

        // 1. Sky / Mist layer on topmost rows
        if h >= 6 {
            let mist_style = if is_lightning {
                Style::default().fg(theme.primary).add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM)
            };
            for m in &self.mist {
                let mx = m.x.round() as usize;
                let my = (m.y as usize).min(h.saturating_sub(2));
                if mx < w && my < h && char_grid[my][mx] == ' ' {
                    char_grid[my][mx] = '░';
                    style_grid[my][mx] = mist_style;
                }
            }
        }

        // 2. Resting puddle water baseline on ground_y with subtle liquid shimmer
        let puddle_style = if is_lightning {
            Style::default().fg(theme.secondary)
        } else {
            Style::default().fg(theme.highlight)
        };
        for x in 0..w {
            let wave_val = ((x as f32 * 0.45 + self.water_phase * std::f32::consts::TAU).sin() + 1.0) * 0.5;
            let ch = if wave_val > 0.82 {
                '~'
            } else if wave_val > 0.65 {
                '─'
            } else if wave_val > 0.48 {
                ' '
            } else {
                ' '
            };
            char_grid[ground_y][x] = ch;
            style_grid[ground_y][x] = puddle_style;
        }

        // 3. Ground splashes and expanding ripples
        for sp in &self.splashes {
            let sx = sp.x as usize;
            if sx >= w {
                continue;
            }

            match sp.layer {
                DropLayer::Foreground => {
                    match sp.stage {
                        0 => {
                            // Impact spark
                            char_grid[ground_y][sx] = '*';
                            style_grid[ground_y][sx] = Style::default()
                                .fg(if is_lightning { theme.text } else { theme.accent })
                                .add_modifier(Modifier::BOLD);
                        }
                        1 => {
                            // Expanding ripple and upward rebound splash droplet
                            if sx > 0 {
                                char_grid[ground_y][sx - 1] = '‹';
                                style_grid[ground_y][sx - 1] = Style::default().fg(theme.rain);
                            }
                            char_grid[ground_y][sx] = '·';
                            style_grid[ground_y][sx] = Style::default().fg(theme.rain);
                            if sx + 1 < w {
                                char_grid[ground_y][sx + 1] = '›';
                                style_grid[ground_y][sx + 1] = Style::default().fg(theme.rain);
                            }
                            // Upward rebound droplet bouncing up to row above ground
                            if ground_y > 0 {
                                char_grid[ground_y - 1][sx] = '\'';
                                style_grid[ground_y - 1][sx] = Style::default()
                                    .fg(if is_lightning { theme.text } else { theme.accent })
                                    .add_modifier(Modifier::BOLD);
                            }
                        }
                        2 => {
                            // Expanding ripple ring and fading rebound drop
                            if sx >= 2 {
                                char_grid[ground_y][sx - 2] = '(';
                                style_grid[ground_y][sx - 2] = Style::default().fg(theme.secondary);
                            }
                            if sx > 0 {
                                char_grid[ground_y][sx - 1] = '~';
                                style_grid[ground_y][sx - 1] = Style::default().fg(theme.secondary);
                            }
                            char_grid[ground_y][sx] = ' ';
                            if sx + 1 < w {
                                char_grid[ground_y][sx + 1] = '~';
                                style_grid[ground_y][sx + 1] = Style::default().fg(theme.secondary);
                            }
                            if sx + 2 < w {
                                char_grid[ground_y][sx + 2] = ')';
                                style_grid[ground_y][sx + 2] = Style::default().fg(theme.secondary);
                            }
                            if ground_y > 0 {
                                char_grid[ground_y - 1][sx] = '·';
                                style_grid[ground_y - 1][sx] = Style::default().fg(theme.text_muted);
                            }
                        }
                        3 => {
                            // Wide ripple ring
                            if sx >= 3 {
                                char_grid[ground_y][sx - 3] = '(';
                                style_grid[ground_y][sx - 3] = Style::default().fg(theme.highlight);
                            }
                            if sx + 3 < w {
                                char_grid[ground_y][sx + 3] = ')';
                                style_grid[ground_y][sx + 3] = Style::default().fg(theme.highlight);
                            }
                        }
                        4 | 5 => {
                            // Fading outer ripple edges
                            if sx >= 4 {
                                char_grid[ground_y][sx - 4] = '·';
                                style_grid[ground_y][sx - 4] = Style::default().fg(theme.border);
                            }
                            if sx + 4 < w {
                                char_grid[ground_y][sx + 4] = '·';
                                style_grid[ground_y][sx + 4] = Style::default().fg(theme.border);
                            }
                        }
                        _ => {}
                    }
                }
                DropLayer::Midground => {
                    match sp.stage {
                        0 => {
                            char_grid[ground_y][sx] = '•';
                            style_grid[ground_y][sx] = Style::default().fg(theme.rain);
                        }
                        1 => {
                            if sx > 0 {
                                char_grid[ground_y][sx - 1] = '‹';
                                style_grid[ground_y][sx - 1] = Style::default().fg(theme.secondary);
                            }
                            char_grid[ground_y][sx] = '·';
                            style_grid[ground_y][sx] = Style::default().fg(theme.secondary);
                            if sx + 1 < w {
                                char_grid[ground_y][sx + 1] = '›';
                                style_grid[ground_y][sx + 1] = Style::default().fg(theme.secondary);
                            }
                        }
                        2 => {
                            if sx >= 2 {
                                char_grid[ground_y][sx - 2] = '(';
                                style_grid[ground_y][sx - 2] = Style::default().fg(theme.highlight);
                            }
                            if sx + 2 < w {
                                char_grid[ground_y][sx + 2] = ')';
                                style_grid[ground_y][sx + 2] = Style::default().fg(theme.highlight);
                            }
                        }
                        3 => {
                            if sx >= 3 {
                                char_grid[ground_y][sx - 3] = '·';
                                style_grid[ground_y][sx - 3] = Style::default().fg(theme.border);
                            }
                            if sx + 3 < w {
                                char_grid[ground_y][sx + 3] = '·';
                                style_grid[ground_y][sx + 3] = Style::default().fg(theme.border);
                            }
                        }
                        _ => {}
                    }
                }
                DropLayer::Background => {
                    match sp.stage {
                        0 => {
                            char_grid[ground_y][sx] = '.';
                            style_grid[ground_y][sx] = Style::default().fg(theme.text_muted);
                        }
                        1 => {
                            char_grid[ground_y][sx] = '~';
                            style_grid[ground_y][sx] = Style::default().fg(theme.highlight);
                        }
                        _ => {}
                    }
                }
            }
        }

        // 4. Falling raindrops and multi-cell streaks
        let (fg_head_char, fg_trail_char, mid_head_char) = if self.wind > 0.18 {
            ('/', '/', '/')
        } else if self.wind < -0.18 {
            ('\\', '\\', '\\')
        } else {
            ('│', '│', '│')
        };

        // Draw drops by depth: Background -> Midground -> Foreground
        for d in &self.drops {
            let hy = d.y.floor() as isize;
            let hx = d.x.round() as isize;

            match d.layer {
                DropLayer::Background => {
                    if hy >= 0 && (hy as usize) < ground_y && hx >= 0 && (hx as usize) < w {
                        let y = hy as usize;
                        let x = hx as usize;
                        if char_grid[y][x] == ' ' || char_grid[y][x] == '░' {
                            char_grid[y][x] = '·';
                            style_grid[y][x] = Style::default().fg(
                                if is_lightning { theme.secondary } else { theme.text_muted }
                            );
                        }
                    }
                }
                DropLayer::Midground => {
                    let wind_offset = (self.wind * 0.8).round() as isize;

                    // Head
                    if hy >= 0 && (hy as usize) < ground_y && hx >= 0 && (hx as usize) < w {
                        let y = hy as usize;
                        let x = hx as usize;
                        char_grid[y][x] = mid_head_char;
                        style_grid[y][x] = Style::default().fg(
                            if is_lightning { theme.text } else { theme.rain }
                        );
                    }
                    // Trail
                    let ty = hy - 1;
                    let tx = hx - wind_offset;
                    if ty >= 0 && (ty as usize) < ground_y && tx >= 0 && (tx as usize) < w {
                        let y = ty as usize;
                        let x = tx as usize;
                        if char_grid[y][x] == ' ' || char_grid[y][x] == '░' || char_grid[y][x] == '·' {
                            char_grid[y][x] = '·';
                            style_grid[y][x] = Style::default().fg(
                                if is_lightning { theme.primary } else { theme.text_muted }
                            );
                        }
                    }
                }
                DropLayer::Foreground => {
                    let wind_offset = (self.wind * 0.8).round() as isize;

                    // Head: bright bold
                    if hy >= 0 && (hy as usize) < ground_y && hx >= 0 && (hx as usize) < w {
                        let y = hy as usize;
                        let x = hx as usize;
                        char_grid[y][x] = fg_head_char;
                        style_grid[y][x] = Style::default()
                            .fg(if is_lightning { theme.text } else { theme.accent })
                            .add_modifier(Modifier::BOLD);
                    }
                    // Mid streak (y - 1)
                    let my = hy - 1;
                    let mx = hx - wind_offset;
                    if my >= 0 && (my as usize) < ground_y && mx >= 0 && (mx as usize) < w {
                        let y = my as usize;
                        let x = mx as usize;
                        char_grid[y][x] = fg_trail_char;
                        style_grid[y][x] = Style::default().fg(
                            if is_lightning { theme.primary } else { theme.rain }
                        );
                    }
                    // Tail (y - 2)
                    let ty = hy - 2;
                    let tx = hx - (wind_offset * 2);
                    if ty >= 0 && (ty as usize) < ground_y && tx >= 0 && (tx as usize) < w {
                        let y = ty as usize;
                        let x = tx as usize;
                        char_grid[y][x] = '·';
                        style_grid[y][x] = Style::default().fg(
                            if is_lightning { theme.secondary } else { theme.secondary }
                        );
                    }
                }
            }
        }

        // 5. Build Ratatui lines with coalesced spans for maximum terminal performance
        let mut lines = Vec::with_capacity(h);
        for y in 0..h {
            let mut spans = Vec::new();
            let mut current_str = String::new();
            let mut current_style = style_grid[y][0];

            for x in 0..w {
                let ch = char_grid[y][x];
                let st = style_grid[y][x];

                if st == current_style {
                    current_str.push(ch);
                } else {
                    if !current_str.is_empty() {
                        spans.push(Span::styled(current_str, current_style));
                        current_str = String::new();
                    }
                    current_str.push(ch);
                    current_style = st;
                }
            }
            if !current_str.is_empty() {
                spans.push(Span::styled(current_str, current_style));
            }
            lines.push(Line::from(spans));
        }

        Paragraph::new(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::{Theme, ThemeKind};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_rain_simulator_lifecycle() {
        let mut rain = RainSimulator::new();
        assert!(!rain.drops.is_empty());

        let spectrum = [0.0; 16];
        for _ in 0..50 {
            rain.tick(&spectrum);
        }

        rain.tick_default();

        let theme = Theme::new(ThemeKind::Nord);
        let area = Rect::new(0, 0, 60, 15);

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| {
            f.render_widget(rain.render(area, &theme), area);
        }).unwrap();

        let buffer = terminal.backend().buffer();
        println!("\n--- RAINFALL ATMOSPHERE SAMPLE FRAME ---");
        for y in 0..15 {
            let row_str: String = (0..60).map(|x| buffer[(x, y)].symbol()).collect();
            println!("{}", row_str);
        }
        println!("----------------------------------------\n");
    }

    #[test]
    fn test_rain_simulator_audio_responsiveness() {
        let mut rain = RainSimulator::new();
        rain.resize(80, 24);

        // Tick with heavy rain and thunder spectrum
        let mut spectrum = [0.0; 16];
        for i in 4..13 {
            spectrum[i] = 0.9;
        }
        spectrum[1] = 0.8; // thunder

        for _ in 0..30 {
            rain.tick(&spectrum);
        }

        // Should have active drops and splashes
        assert!(rain.drops.len() >= 20);
    }
}
