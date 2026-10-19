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

    let header_line = Line::from(vec![
        title_span,
        Span::raw("   "),
        Span::styled(format!("Mix: {}", preset_name), Style::default().fg(theme.secondary)),
        Span::raw("  │  "),
        Span::styled(format!("Visual: {}", app.visual.mode.name()), Style::default().fg(theme.accent)),
        Span::raw("  │  "),
        Span::styled(format!("Theme: {}", theme.kind.name()), Style::default().fg(theme.primary)),
        Span::raw("  │  "),
        audio_badge,
    ]);
    f.render_widget(Paragraph::new(header_line), chunks[0]);

    // 2. Central Layout Split: Left (Timer + Visualizer) vs Right (Sound Mixer)
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
    );

    // Render Living Visualizer Panel
    let visual_border = if app.focus == ActiveFocus::Visualizer {
        theme.border_active
    } else {
        theme.border
    };
    let visual_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(visual_border))
        .title(Span::styled(
            format!(" 🌌 Ambient Atmosphere ({}) ", app.visual.mode.name()),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
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
    );

    // 3. Footer Bar with Hotkey Hints
    let footer_hints = Line::from(vec![
        Span::styled("[Space] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Timer  "),
        Span::styled("[Tab] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Pane  "),
        Span::styled("[p] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Presets  "),
        Span::styled("[S] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Save Mix  "),
        Span::styled("[v] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Visual  "),
        Span::styled("[t] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Theme  "),
        Span::styled("[m] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Mute  "),
        Span::styled("[?] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Help  "),
        Span::styled("[q] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Quit"),
    ]);
    f.render_widget(
        Paragraph::new(footer_hints)
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
