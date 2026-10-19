mod app;
mod audio;
mod config;
mod event;
mod timer;
mod ui;

use std::error::Error;
use std::io::{self, stdout};
use std::panic;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use event::{Event, EventHandler};
use timer::TimerMode;

#[derive(Parser, Debug)]
#[command(
    name = "zeno",
    author = "Zeno Contributors",
    version = "0.1.0",
    about = "A calm terminal ambient noise generator and focus companion."
)]
struct Cli {
    /// Start with a specific preset activated
    #[arg(short, long)]
    preset: Option<String>,

    /// Start directly in Pomodoro mode
    #[arg(long)]
    pomodoro: bool,

    /// Start directly in Stopwatch count-up mode
    #[arg(long)]
    stopwatch: bool,

    /// Start in Countdown mode with specified minutes
    #[arg(long)]
    countdown: Option<u64>,

    /// List all available built-in and user presets
    #[arg(long)]
    list_presets: bool,
}

fn setup_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, Show);
        original_hook(panic_info);
    }));
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, Show)?;
    terminal.show_cursor()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // If --list-presets flag is given, display presets and exit
    if cli.list_presets {
        let app = App::new();
        println!("Built-in Factory Presets:");
        for p in &app.audio.factory_presets {
            println!("  • {:<20} - {}", p.name, p.description);
        }
        if !app.audio.user_presets.is_empty() {
            println!("\nUser Saved Presets:");
            for p in &app.audio.user_presets {
                println!("  • {:<20} - {}", p.name, p.description);
            }
        }
        return Ok(());
    }

    setup_panic_hook();

    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App instance
    let mut app = App::new();

    // Apply CLI flags
    if let Some(preset_name) = cli.preset {
        let found = app
            .audio
            .factory_presets
            .iter()
            .chain(app.audio.user_presets.iter())
            .find(|p| p.name.eq_ignore_ascii_case(&preset_name))
            .cloned();

        if let Some(preset) = found {
            app.visual.set_mode(preset.visualizer);
            app.audio.apply_preset(&preset);
        }
    }

    if cli.pomodoro {
        app.timer.set_mode(TimerMode::Pomodoro);
    } else if cli.stopwatch {
        app.timer.set_mode(TimerMode::Stopwatch);
    } else if let Some(mins) = cli.countdown {
        app.timer.countdown.set_duration_mins(mins);
        app.timer.set_mode(TimerMode::Countdown);
    }

    // 33ms tick rate (~30 FPS)
    let events = EventHandler::new(Duration::from_millis(33));

    while !app.should_quit {
        terminal.draw(|f| ui::render(f, &mut app))?;

        match events.next()? {
            Event::Tick => {
                app.tick();
            }
            Event::Key(key) => {
                app.handle_key(key);
            }
            Event::Resize(_, _) => {
                terminal.autoresize()?;
            }
            Event::Mouse(_) => {}
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}
