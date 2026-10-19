use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::audio::track::TrackId;
use crate::audio::AudioController;
use crate::ui::theme::Theme;

pub fn render_mixer(
    f: &mut Frame,
    area: Rect,
    audio: &AudioController,
    selected_index: usize,
    theme: &Theme,
    is_focused: bool,
    is_fullscreen: bool,
) {
    let border_color = if is_focused {
        theme.border_active
    } else {
        theme.border
    };

    let title_badge = if is_fullscreen {
        " [FULLSCREEN - 'f' to exit]"
    } else if is_focused {
        " [FOCUSED]"
    } else {
        ""
    };

    let title_style = if is_focused {
        Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(" 🎛 Soundscape Mixer{} ", title_badge),
            title_style,
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 5 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Master volume & Mute status
            Constraint::Min(1),    // Track list
            Constraint::Length(1), // Selected track description
            Constraint::Length(1), // Footer hint
        ])
        .split(inner);

    // 1. Master Volume & Mute Bar
    let mute_badge = if audio.engine.is_muted {
        Span::styled(
            " [MUTED] ",
            Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " [UNMUTED] ",
            Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
        )
    };

    let master_pct = (audio.engine.master_volume * 100.0).round() as usize;
    let master_bars = master_pct / 5;
    let master_slider = format!(
        "[{}{}] {:>3}%",
        "█".repeat(master_bars),
        "░".repeat(20 - master_bars),
        master_pct
    );

    let master_line = Line::from(vec![
        Span::styled(" Master Vol: ", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        Span::styled(master_slider, Style::default().fg(theme.primary)),
        Span::raw("  "),
        mute_badge,
        Span::styled(" [m: Mute, +/-: Vol]", Style::default().fg(theme.text_muted)),
    ]);
    f.render_widget(Paragraph::new(master_line), chunks[0]);

    // 2. Track List
    let max_visible = chunks[1].height as usize;
    let scroll_offset = if selected_index >= max_visible {
        selected_index + 1 - max_visible
    } else {
        0
    };

    let mut track_lines = Vec::new();

    for (i, track) in audio.tracks.iter().enumerate().skip(scroll_offset).take(max_visible) {
        let is_selected = i == selected_index;
        let prefix = if is_selected { " ▶ " } else { "   " };

        let status_span = if track.enabled && !audio.engine.is_muted {
            Span::styled(
                " [ON ] ",
                Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(" [OFF] ", Style::default().fg(theme.text_muted))
        };

        let pct = (track.volume * 100.0).round() as usize;
        let filled_bars = pct / 10;
        let slider = format!(
            "[{}{}] {:>3}%",
            "█".repeat(filled_bars),
            "░".repeat(10 - filled_bars),
            pct
        );

        let name_style = if is_selected {
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };

        let mut spans = vec![
            Span::styled(
                prefix,
                if is_selected {
                    Style::default().fg(theme.accent)
                } else {
                    Style::default()
                },
            ),
            status_span,
            Span::styled(format!("{:<14}", track.name), name_style),
            Span::raw(" "),
            Span::styled(
                slider,
                if track.enabled {
                    Style::default().fg(theme.secondary)
                } else {
                    Style::default().fg(theme.text_muted)
                },
            ),
        ];

        if track.id == TrackId::Binaural {
            let label = if track.binaural_hz <= 10.0 {
                "Alpha (10Hz)"
            } else if track.binaural_hz <= 15.0 {
                "Beta (14Hz)"
            } else {
                "Gamma (40Hz)"
            };
            spans.push(Span::styled(
                format!("  [{}]", label),
                Style::default().fg(theme.accent),
            ));
        }

        let row_style = if is_selected {
            Style::default().bg(theme.highlight)
        } else {
            Style::default()
        };

        track_lines.push(Line::from(spans).style(row_style));
    }

    f.render_widget(Paragraph::new(track_lines), chunks[1]);

    // 3. Selected Track Description
    if let Some(track) = audio.tracks.get(selected_index) {
        let desc_line = Line::from(vec![
            Span::styled(" ℹ ", Style::default().fg(theme.accent)),
            Span::styled(&track.description, Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(desc_line), chunks[2]);
    }

    // 4. Footer Navigation Hint
    let hint_spans = if is_focused {
        Line::from(vec![
            Span::styled("[j/k] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw("Select  "),
            Span::styled("[h/l] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw("Volume  "),
            Span::styled("[Enter] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle ON/OFF  "),
            Span::styled("[b] ", Style::default().fg(theme.primary)),
            Span::raw("Binaural Hz  "),
            Span::styled("[f] ", Style::default().fg(theme.primary)),
            Span::raw(if is_fullscreen { "Normal" } else { "Fullscreen" }),
        ])
    } else {
        Line::from(vec![
            Span::styled("[Tab] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw("Focus Mixer  "),
            Span::styled("[m] ", Style::default().fg(theme.text_muted)),
            Span::raw("Mute  "),
            Span::styled("[+/-] ", Style::default().fg(theme.text_muted)),
            Span::raw("Master Vol"),
        ])
    };
    f.render_widget(Paragraph::new(hint_spans), chunks[3]);
}
