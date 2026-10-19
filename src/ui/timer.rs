use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::timer::pomodoro::PomodoroPhase;
use crate::timer::{TimerManager, TimerMode};
use crate::ui::theme::Theme;

const DIGIT_HEIGHT: usize = 5;

fn digit_glyph(c: char) -> [&'static str; DIGIT_HEIGHT] {
    match c {
        '0' => [
            "████",
            "█  █",
            "█  █",
            "█  █",
            "████",
        ],
        '1' => [
            "  ██",
            " ███",
            "  ██",
            "  ██",
            "████",
        ],
        '2' => [
            "████",
            "   █",
            "████",
            "█   ",
            "████",
        ],
        '3' => [
            "████",
            "   █",
            "████",
            "   █",
            "████",
        ],
        '4' => [
            "█  █",
            "█  █",
            "████",
            "   █",
            "   █",
        ],
        '5' => [
            "████",
            "█   ",
            "████",
            "   █",
            "████",
        ],
        '6' => [
            "████",
            "█   ",
            "████",
            "█  █",
            "████",
        ],
        '7' => [
            "████",
            "   █",
            "  ██",
            "  █ ",
            "  █ ",
        ],
        '8' => [
            "████",
            "█  █",
            "████",
            "█  █",
            "████",
        ],
        '9' => [
            "████",
            "█  █",
            "████",
            "   █",
            "████",
        ],
        ':' => [
            "    ",
            " ██ ",
            "    ",
            " ██ ",
            "    ",
        ],
        _ => [
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
        ],
    }
}

pub fn render_timer(
    f: &mut Frame,
    area: Rect,
    timer: &TimerManager,
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
            format!(" ⏳ Focus Timer ({}){} ", timer.mode.name(), title_badge),
            title_style,
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 6 {
        return;
    }

    // When fullscreen with ample vertical height, add padding to nicely center the digits
    let top_pad = if is_fullscreen && inner.height > 16 {
        (inner.height.saturating_sub(14)) / 3
    } else {
        0
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad), // Top pad for fullscreen centering
            Constraint::Length(1),       // Phase / status banner
            Constraint::Length(1),       // Spacer
            Constraint::Length(5),       // Big Digits clock
            Constraint::Length(1),       // Spacer
            Constraint::Length(1),       // Gauge progress bar
            Constraint::Min(1),          // Session info & hints
        ])
        .split(inner);

    let banner_chunk = chunks[1];
    let clock_chunk = chunks[3];
    let gauge_chunk = chunks[5];
    let info_chunk = chunks[6];

    // 1. Phase / Mode Banner
    let (phase_str, phase_color) = match timer.mode {
        TimerMode::Pomodoro => match timer.pomodoro.phase {
            PomodoroPhase::Work => ("● FOCUS SESSION", theme.success),
            PomodoroPhase::ShortBreak => ("☕ SHORT BREAK", theme.warning),
            PomodoroPhase::LongBreak => ("🌴 LONG REST BREAK", theme.secondary),
        },
        TimerMode::Stopwatch => ("⏱ STOPWATCH (COUNT-UP)", theme.accent),
        TimerMode::Countdown => ("⏳ COUNTDOWN TIMER", theme.primary),
    };

    let running_indicator = if timer.is_running() {
        Span::styled(" [RUNNING]", Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" [PAUSED]", Style::default().fg(theme.warning))
    };

    let banner_line = Line::from(vec![
        Span::styled(phase_str, Style::default().fg(phase_color).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        running_indicator,
    ]);
    f.render_widget(
        Paragraph::new(banner_line).alignment(Alignment::Center),
        banner_chunk,
    );

    // 2. Format Time String (MM:SS or HH:MM:SS)
    let time_str = match timer.mode {
        TimerMode::Pomodoro => {
            let total_secs = timer.pomodoro.time_remaining.as_secs();
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            format!("{:02}:{:02}", mins, secs)
        }
        TimerMode::Stopwatch => {
            let total_secs = timer.stopwatch.elapsed.as_secs();
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            let secs = total_secs % 60;
            if hours > 0 {
                format!("{:02}:{:02}:{:02}", hours, mins, secs)
            } else {
                format!("{:02}:{:02}", mins, secs)
            }
        }
        TimerMode::Countdown => {
            let total_secs = timer.countdown.time_remaining.as_secs();
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            format!("{:02}:{:02}", mins, secs)
        }
    };

    // Render big ASCII digits
    let mut digit_lines = vec![String::new(); DIGIT_HEIGHT];
    for ch in time_str.chars() {
        let glyph = digit_glyph(ch);
        for row in 0..DIGIT_HEIGHT {
            digit_lines[row].push_str(glyph[row]);
            digit_lines[row].push_str("  "); // digit spacing
        }
    }

    let digit_spans: Vec<Line> = digit_lines
        .into_iter()
        .map(|line| {
            Line::from(Span::styled(
                line,
                Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    f.render_widget(
        Paragraph::new(digit_spans).alignment(Alignment::Center),
        clock_chunk,
    );

    // 3. Progress Bar Gauge
    let progress_ratio = match timer.mode {
        TimerMode::Pomodoro => timer.pomodoro.progress_ratio(),
        TimerMode::Countdown => timer.countdown.progress_ratio(),
        TimerMode::Stopwatch => 1.0,
    };

    let gauge_title = match timer.mode {
        TimerMode::Pomodoro => format!("{:.0}% Elapsed", progress_ratio * 100.0),
        TimerMode::Countdown => format!("{:.0}% Elapsed", progress_ratio * 100.0),
        TimerMode::Stopwatch => format!("Elapsed: {}s", timer.stopwatch.elapsed.as_secs()),
    };

    let gauge = Gauge::default()
        .ratio(progress_ratio)
        .label(Span::styled(
            gauge_title,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ))
        .gauge_style(Style::default().fg(phase_color).bg(theme.surface));

    f.render_widget(gauge, gauge_chunk);

    // 4. Session info & Controls hint
    let mut info_spans = Vec::new();
    match timer.mode {
        TimerMode::Pomodoro => {
            let total_cycles = timer.pomodoro.cycles_before_long_break;
            let current = (timer.pomodoro.completed_sessions % total_cycles) + 1;
            let mut dots = String::new();
            for i in 1..=total_cycles {
                if i < current {
                    dots.push('●');
                } else if i == current {
                    dots.push('◉');
                } else {
                    dots.push('○');
                }
                dots.push(' ');
            }
            let duration_mins = timer.pomodoro.work_duration.as_secs() / 60;
            info_spans.push(Span::styled(
                format!("Cycle [{}] {} of {}  |  Duration: {}m  |  Total: {}", dots.trim(), current, total_cycles, duration_mins, timer.pomodoro.completed_sessions),
                Style::default().fg(theme.text_muted),
            ));
        }
        TimerMode::Stopwatch => {
            info_spans.push(Span::styled(
                format!("Recorded Laps: {}", timer.stopwatch.laps.len()),
                Style::default().fg(theme.text_muted),
            ));
        }
        TimerMode::Countdown => {
            info_spans.push(Span::styled(
                format!("Set Duration: {} mins", timer.countdown.total_duration.as_secs() / 60),
                Style::default().fg(theme.text_muted),
            ));
        }
    }

    let control_hints = if is_focused {
        Line::from(vec![
            Span::styled("[h/l] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw("±5m  "),
            Span::styled("[j/k] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw("Mode  "),
            Span::styled("[Space] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(if timer.is_running() { "Pause  " } else { "Start  " }),
            Span::styled("[r] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw("Reset/Skip  "),
            Span::styled("[f] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(if is_fullscreen { "Normal" } else { "Fullscreen" }),
        ])
    } else {
        Line::from(vec![
            Span::styled("[Tab] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw("Focus Timer  "),
            Span::styled("[Space] ", Style::default().fg(theme.text_muted)),
            Span::raw(if timer.is_running() { "Pause  " } else { "Start  " }),
            Span::styled("[r] ", Style::default().fg(theme.text_muted)),
            Span::raw("Reset"),
        ])
    };

    let footer_text = vec![
        Line::from(info_spans),
        Line::raw(""),
        control_hints,
    ];

    f.render_widget(
        Paragraph::new(footer_text).alignment(Alignment::Center),
        info_chunk,
    );
}
