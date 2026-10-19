use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub border_active: Color,
    pub text: Color,
    pub text_muted: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub highlight: Color,
    pub rain: Color,
    pub fire_palette: [Color; 6],
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            background: Color::Reset,      // Terminal native background (transparent/default)
            surface: Color::Reset,         // Terminal native background
            border: Color::DarkGray,
            border_active: Color::Cyan,
            text: Color::Reset,            // Terminal native text foreground
            text_muted: Color::DarkGray,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            highlight: Color::DarkGray,
            rain: Color::Cyan,
            fire_palette: [
                Color::Reset,
                Color::Red,
                Color::LightRed,
                Color::Yellow,
                Color::LightYellow,
                Color::White,
            ],
        }
    }
}

impl Theme {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}
