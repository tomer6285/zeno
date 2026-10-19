pub mod mixer;
pub mod presets_dialog;
pub mod theme;
pub mod timer;
pub mod visual;
pub mod widgets;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{ActiveFocus, App, ModalState};
use crate::ui::mixer::render_mixer;
use crate::ui::presets_dialog::{render_help_modal, render_preset_browser, render_save_preset_modal};
use crate::ui::timer::render_timer;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;

    // Outer background block
    let root_block = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(root_block, area);

    // Main layout: Header (1 line), Body (Min), Footer (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top Header Bar
            Constraint::Min(10),   // Central Working Area
            Constraint::Length(1), // Bottom Status / Keybinding Footer
        ])
        .split(area);

    // 1. Header Bar
    let title_span = Span::styled(
        " 🌿 ZENO ",
        Style::default().fg(theme.primary).bg(theme.surface).add_modifier(Modifier::BOLD),
    );

    let preset_name = app
        .audio
        .active_preset_name
        .as_deref()
        .unwrap_or("Custom Mix");

    let audio_badge = if !app.audio.engine.audio_available {
        Span::styled("🔇 QUIET MODE", Style::default().fg(theme.warning))
    } else if app.audio.engine.is_muted {
        Span::styled("🔇 MUTED", Style::default().fg(theme.error).add_modifier(Modifier::BOLD))
    } else {
        let pct = (app.audio.engine.master_volume * 100.0).round() as u32;
        Span::styled(format!("🔊 {}%", pct), Style::default().fg(theme.success))
    };

    let focus_color = match app.focus {
        ActiveFocus::Timer => theme.primary,
        ActiveFocus::Mixer => theme.secondary,
        ActiveFocus::Visualizer => theme.accent,
    };
    let focus_badge = Span::styled(
        format!(" [Focus: {}] ", app.focus.name()),
        Style::default().fg(focus_color).bg(theme.surface).add_modifier(Modifier::BOLD),
    );

    let mut header_spans = vec![
        title_span,
        Span::raw("   "),
        Span::styled(format!("Mix: {}", preset_name), Style::default().fg(theme.secondary)),
        Span::raw("  │  "),
        Span::styled(format!("Visual: {}", app.visual.mode.name()), Style::default().fg(theme.accent)),
        Span::raw("  │  "),
        Span::styled(format!("Theme: {}", theme.kind.name()), Style::default().fg(theme.primary)),
        Span::raw("  │  "),
        audio_badge,
        Span::raw("   "),
        focus_badge,
    ];

    if app.fullscreen {
        header_spans.push(Span::raw(" "));
        header_spans.push(Span::styled(
            " ⛶ FULLSCREEN (f: exit) ",
            Style::default().fg(theme.background).bg(theme.accent).add_modifier(Modifier::BOLD),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(header_spans)), chunks[0]);

    // 2. Central Layout Split:
    // If fullscreen is toggled ON, render only the active focused module!
    if app.fullscreen {
        match app.focus {
            ActiveFocus::Timer => {
                render_timer(f, chunks[1], &app.timer, theme, true, true);
            }
            ActiveFocus::Mixer => {
                render_mixer(f, chunks[1], &app.audio, app.mixer_selected_index, theme, true, true);
            }
            ActiveFocus::Visualizer => {
                let visual_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_active))
                    .title(Span::styled(
                        format!(" 🌌 Ambient Atmosphere ({}) [FULLSCREEN - 'f' to exit] ", app.visual.mode.name()),
                        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                    ));
                let visual_inner = visual_block.inner(chunks[1]);
                f.render_widget(visual_block, chunks[1]);
                app.visual.resize(visual_inner.width, visual_inner.height);
                let visual_widget = app.visual.render(visual_inner, theme);
                f.render_widget(visual_widget, visual_inner);
            }
        }
    } else {
        // Standard multi-pane dashboard layout
        let body_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(52), // Left Pane: Timer & Visualizer
                Constraint::Percentage(48), // Right Pane: Soundscape Mixer
            ])
            .split(chunks[1]);

        // Left Pane vertical split: Timer (top) & Visualizer (bottom)
        let left_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(14), // Big digit timer + gauge
                Constraint::Min(6),     // ASCII Living Visualizer
            ])
            .split(body_split[0]);

        // Render Timer
        render_timer(
            f,
            left_split[0],
            &app.timer,
            theme,
            app.focus == ActiveFocus::Timer,
            false,
        );

        // Render Living Visualizer Panel
        let visual_border = if app.focus == ActiveFocus::Visualizer {
            theme.border_active
        } else {
            theme.border
        };
        let visual_title_badge = if app.focus == ActiveFocus::Visualizer {
            " [FOCUSED]"
        } else {
            ""
        };
        let visual_title_style = if app.focus == ActiveFocus::Visualizer {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_muted)
        };

        let visual_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(visual_border))
            .title(Span::styled(
                format!(" 🌌 Ambient Atmosphere ({}){} ", app.visual.mode.name(), visual_title_badge),
                visual_title_style,
            ));
        let visual_inner = visual_block.inner(left_split[1]);
        f.render_widget(visual_block, left_split[1]);

        // Resize visualizer if panel dimensions changed
        app.visual.resize(visual_inner.width, visual_inner.height);
        let visual_widget = app.visual.render(visual_inner, theme);
        f.render_widget(visual_widget, visual_inner);

        // Render Mixer (Right Pane)
        render_mixer(
            f,
            body_split[1],
            &app.audio,
            app.mixer_selected_index,
            theme,
            app.focus == ActiveFocus::Mixer,
            false,
        );
    }

    // 3. Footer Bar with Dynamic Context-Aware Hotkey Hints
    let mut footer_spans = Vec::new();

    // Active module focus-specific controls
    match app.focus {
        ActiveFocus::Timer => {
            footer_spans.push(Span::styled("[Space] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            footer_spans.push(Span::raw(if app.timer.is_running() { "Pause  " } else { "Play  " }));
            footer_spans.push(Span::styled("[h/l] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            footer_spans.push(Span::raw("±5m  "));
            footer_spans.push(Span::styled("[j/k] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            footer_spans.push(Span::raw("Mode  "));
            footer_spans.push(Span::styled("[r] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            footer_spans.push(Span::raw("Reset  "));
        }
        ActiveFocus::Mixer => {
            footer_spans.push(Span::styled("[j/k] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            footer_spans.push(Span::raw("Select  "));
            footer_spans.push(Span::styled("[h/l] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            footer_spans.push(Span::raw("Volume  "));
            footer_spans.push(Span::styled("[Enter] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            footer_spans.push(Span::raw("Toggle  "));
        }
        ActiveFocus::Visualizer => {
            footer_spans.push(Span::styled("[j/k] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            footer_spans.push(Span::raw("Effect  "));
            footer_spans.push(Span::styled("[v/Enter] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            footer_spans.push(Span::raw("Cycle  "));
        }
    }

    // Primary navigation & global controls
    footer_spans.push(Span::styled("[Tab] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)));
    footer_spans.push(Span::raw("Focus  "));

    footer_spans.push(Span::styled("[f] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)));
    footer_spans.push(Span::raw(if app.fullscreen { "Normal  " } else { "Fullscreen  " }));

    footer_spans.push(Span::styled("[p] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)));
    footer_spans.push(Span::raw("Presets  "));

    footer_spans.push(Span::styled("[S] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)));
    footer_spans.push(Span::raw("Save  "));

    footer_spans.push(Span::styled("[t] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)));
    footer_spans.push(Span::raw("Theme  "));

    footer_spans.push(Span::styled("[m] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)));
    footer_spans.push(Span::raw("Mute  "));

    footer_spans.push(Span::styled("[?] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)));
    footer_spans.push(Span::raw("Help  "));

    footer_spans.push(Span::styled("[q] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)));
    footer_spans.push(Span::raw("Quit"));

    f.render_widget(
        Paragraph::new(Line::from(footer_spans))
            .style(Style::default().bg(theme.surface))
            .alignment(Alignment::Center),
        chunks[2],
    );

    // 4. Modal Overlays
    match &app.modal {
        ModalState::PresetBrowser { selected_index } => {
            render_preset_browser(f, area, &app.audio, *selected_index, theme);
        }
        ModalState::SaveMixPrompt { input_text } => {
            render_save_preset_modal(f, area, input_text, theme);
        }
        ModalState::Help => {
            render_help_modal(f, area, theme);
        }
        ModalState::None => {}
    }
}
