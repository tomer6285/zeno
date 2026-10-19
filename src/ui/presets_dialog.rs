use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::audio::AudioController;
use crate::ui::theme::Theme;

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_preset_browser(
    f: &mut Frame,
    area: Rect,
    audio: &AudioController,
    selected_index: usize,
    theme: &Theme,
) {
    let dialog_area = centered_rect(65, 75, area);
    f.render_widget(Clear, dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            " 🎧 Soundscape Preset Browser ",
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // Presets List
            Constraint::Length(5), // Details / Track Breakdown of Selected
            Constraint::Length(2), // Hotkey instructions
        ])
        .split(inner);

    // Combine factory presets and user presets
    let total_factory = audio.factory_presets.len();

    let mut lines = Vec::new();

    // Section 1: Factory Presets
    lines.push(Line::from(Span::styled(
        "── Factory Presets ──────────────────────────",
        Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
    )));

    for (i, preset) in audio.factory_presets.iter().enumerate() {
        let is_selected = i == selected_index;
        let is_active = audio.active_preset_name.as_deref() == Some(&preset.name);

        let marker = if is_selected { " ▶ " } else { "   " };
        let active_badge = if is_active { " [ACTIVE]" } else { "" };

        let name_span = Span::styled(
            format!("{:<20}{}", preset.name, active_badge),
            if is_selected {
                Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
            } else if is_active {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(theme.text)
            },
        );

        let row_style = if is_selected {
            Style::default().bg(theme.highlight)
        } else {
            Style::default()
        };

        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(theme.accent)),
            name_span,
            Span::raw("  "),
            Span::styled(&preset.description, Style::default().fg(theme.text_muted)),
        ]).style(row_style));
    }

    // Section 2: User Presets
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "── User Saved Presets ──────────────────────",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));

    if audio.user_presets.is_empty() {
        lines.push(Line::from(Span::styled(
            "   (No user presets saved yet. Press 'S' in mixer to save one!)",
            Style::default().fg(theme.text_muted),
        )));
    } else {
        for (i, preset) in audio.user_presets.iter().enumerate() {
            let actual_idx = total_factory + i;
            let is_selected = actual_idx == selected_index;
            let is_active = audio.active_preset_name.as_deref() == Some(&preset.name);

            let marker = if is_selected { " ▶ " } else { "   " };
            let active_badge = if is_active { " [ACTIVE]" } else { "" };

            let row_style = if is_selected {
                Style::default().bg(theme.highlight)
            } else {
                Style::default()
            };

            lines.push(Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.accent)),
                Span::styled(
                    format!("{:<20}{}", preset.name, active_badge),
                    if is_selected {
                        Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
                    } else if is_active {
                        Style::default().fg(theme.success)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                Span::raw("  "),
                Span::styled(&preset.description, Style::default().fg(theme.text_muted)),
            ]).style(row_style));
        }
    }

    f.render_widget(Paragraph::new(lines), chunks[0]);

    // Selected Preset Details
    let selected_preset = if selected_index < total_factory {
        audio.factory_presets.get(selected_index)
    } else {
        audio.user_presets.get(selected_index - total_factory)
    };

    if let Some(preset) = selected_preset {
        let mut track_desc = Vec::new();
        for (track_id, vol) in &preset.tracks {
            let pct = (*vol * 100.0).round() as u32;
            track_desc.push(format!("{} ({}%)", track_id, pct));
        }
        let tracks_str = if track_desc.is_empty() {
            "None".to_string()
        } else {
            track_desc.join(", ")
        };

        let details_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(" Preset Details ", Style::default().fg(theme.secondary)));

        let details_inner = details_block.inner(chunks[1]);
        f.render_widget(details_block, chunks[1]);

        let binaural_info = match preset.binaural_hz {
            Some(hz) => format!("Binaural: {:.1} Hz | ", hz),
            None => String::new(),
        };

        let text = vec![
            Line::from(vec![
                Span::styled("Tracks: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(tracks_str, Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled(binaural_info, Style::default().fg(theme.accent)),
                Span::styled("Visualizer: ", Style::default().fg(theme.primary)),
                Span::styled(preset.visualizer.name(), Style::default().fg(theme.text)),
            ]),
        ];
        f.render_widget(Paragraph::new(text), details_inner);
    }

    // Modal Footer Instructions
    let footer_hints = Line::from(vec![
        Span::styled("[↑/↓] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Select  "),
        Span::styled("[Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Apply  "),
        Span::styled("[d] ", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
        Span::raw("Delete User Preset  "),
        Span::styled("[Esc] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Close"),
    ]);
    f.render_widget(
        Paragraph::new(footer_hints).alignment(Alignment::Center),
        chunks[2],
    );
}

pub fn render_save_preset_modal(
    f: &mut Frame,
    area: Rect,
    input_text: &str,
    theme: &Theme,
) {
    let dialog_area = centered_rect(50, 30, area);
    f.render_widget(Clear, dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            " 💾 Save Current Mix as Preset ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Prompt
            Constraint::Length(3), // Input box
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Controls
        ])
        .split(inner);

    let prompt = Paragraph::new(Span::styled(
        "Enter a name for this custom sound mix:",
        Style::default().fg(theme.text),
    ));
    f.render_widget(prompt, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));
    let input_para = Paragraph::new(Line::from(vec![
        Span::styled(input_text, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("█", Style::default().fg(theme.accent)), // cursor
    ]))
    .block(input_block);
    f.render_widget(input_para, chunks[1]);

    let instructions = Line::from(vec![
        Span::styled("[Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Save Preset   "),
        Span::styled("[Esc] ", Style::default().fg(theme.text_muted).add_modifier(Modifier::BOLD)),
        Span::raw("Cancel"),
    ]);
    f.render_widget(
        Paragraph::new(instructions).alignment(Alignment::Center),
        chunks[3],
    );
}

pub fn render_help_modal(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let dialog_area = centered_rect(70, 80, area);
    f.render_widget(Clear, dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            " 📖 Zeno Keybindings & Help ",
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let shortcuts = [
        ("Space", "Start / Pause active timer"),
        ("Tab", "Switch focus between Timer and Sound Mixer"),
        ("r", "Reset active timer or skip to next Pomodoro interval"),
        ("1 / 2 / 3", "Switch Timer mode (1: Pomodoro, 2: Stopwatch, 3: Countdown)"),
        ("↑ / ↓ / j / k", "Navigate sound tracks in mixer"),
        ("← / → / h / l", "Decrease / Increase volume of selected track"),
        ("Enter", "Toggle selected track ON / OFF"),
        ("p", "Open Preset Browser modal (select, preview, apply, or delete)"),
        ("S / Ctrl+S", "Save current mix as a new User Preset"),
        ("v", "Cycle ambient visualizer (Rain / Campfire / Wave / Off)"),
        ("b", "Cycle binaural beats frequency (Alpha 10Hz / Beta 14Hz / Gamma 40Hz)"),
        ("m", "Mute / Unmute all audio output"),
        ("+ / -", "Master volume up / down"),
        ("t", "Cycle color theme (Catppuccin, Nord, Gruvbox, Tokyo Night, Monokai)"),
        ("?", "Toggle this Help overlay"),
        ("q / Esc", "Quit application (or close active modal)"),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Zeno is a calm terminal focus companion and ambient noise generator.",
        Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::raw(""));

    for (key, desc) in shortcuts {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<16}", key), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(desc, Style::default().fg(theme.text)),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "Press [Esc], [q], or [?] to close this help window.",
        Style::default().fg(theme.text_muted).add_modifier(Modifier::ITALIC),
    )));

    f.render_widget(Paragraph::new(lines), inner);
}
