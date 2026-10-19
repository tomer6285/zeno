use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeKind {
    CatppuccinMocha,
    CatppuccinMacchiato,
    Nord,
    GruvboxDark,
    TokyoNight,
    Monokai,
}

impl ThemeKind {
    pub fn all() -> &'static [ThemeKind] {
        &[
            ThemeKind::CatppuccinMocha,
            ThemeKind::CatppuccinMacchiato,
            ThemeKind::Nord,
            ThemeKind::GruvboxDark,
            ThemeKind::TokyoNight,
            ThemeKind::Monokai,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ThemeKind::CatppuccinMocha => "Catppuccin Mocha",
            ThemeKind::CatppuccinMacchiato => "Catppuccin Macchiato",
            ThemeKind::Nord => "Nord",
            ThemeKind::GruvboxDark => "Gruvbox Dark",
            ThemeKind::TokyoNight => "Tokyo Night",
            ThemeKind::Monokai => "Monokai",
        }
    }

    pub fn next(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|t| t == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub kind: ThemeKind,
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

impl Theme {
    pub fn new(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::CatppuccinMocha => Self {
                kind,
                primary: Color::Rgb(203, 166, 247),       // Mauve
                secondary: Color::Rgb(137, 180, 250),     // Blue
                accent: Color::Rgb(148, 226, 213),        // Teal
                background: Color::Rgb(30, 30, 46),       // Base
                surface: Color::Rgb(24, 24, 37),          // Mantle
                border: Color::Rgb(88, 91, 112),          // Surface2
                border_active: Color::Rgb(203, 166, 247), // Mauve
                text: Color::Rgb(205, 214, 244),          // Text
                text_muted: Color::Rgb(166, 173, 200),    // Subtext0
                success: Color::Rgb(166, 227, 161),       // Green
                warning: Color::Rgb(249, 226, 175),       // Yellow
                error: Color::Rgb(243, 139, 168),         // Red
                highlight: Color::Rgb(69, 71, 90),        // Surface1
                rain: Color::Rgb(137, 220, 235),          // Sky
                fire_palette: [
                    Color::Rgb(30, 30, 46),
                    Color::Rgb(180, 50, 50),
                    Color::Rgb(235, 110, 50),
                    Color::Rgb(250, 180, 50),
                    Color::Rgb(255, 220, 100),
                    Color::Rgb(255, 255, 200),
                ],
            },
            ThemeKind::CatppuccinMacchiato => Self {
                kind,
                primary: Color::Rgb(198, 160, 246),
                secondary: Color::Rgb(138, 173, 244),
                accent: Color::Rgb(139, 213, 202),
                background: Color::Rgb(36, 39, 58),
                surface: Color::Rgb(30, 32, 48),
                border: Color::Rgb(91, 96, 120),
                border_active: Color::Rgb(198, 160, 246),
                text: Color::Rgb(202, 211, 245),
                text_muted: Color::Rgb(165, 173, 203),
                success: Color::Rgb(166, 218, 149),
                warning: Color::Rgb(238, 212, 159),
                error: Color::Rgb(237, 135, 150),
                highlight: Color::Rgb(73, 77, 100),
                rain: Color::Rgb(145, 215, 227),
                fire_palette: [
                    Color::Rgb(36, 39, 58),
                    Color::Rgb(190, 55, 55),
                    Color::Rgb(240, 115, 55),
                    Color::Rgb(250, 185, 60),
                    Color::Rgb(255, 225, 110),
                    Color::Rgb(255, 255, 210),
                ],
            },
            ThemeKind::Nord => Self {
                kind,
                primary: Color::Rgb(136, 192, 208),       // Nord8 Frost
                secondary: Color::Rgb(129, 161, 193),     // Nord9 Frost
                accent: Color::Rgb(143, 188, 187),        // Nord7
                background: Color::Rgb(46, 52, 64),       // Nord0 Polar Night
                surface: Color::Rgb(59, 66, 82),          // Nord1
                border: Color::Rgb(76, 86, 106),          // Nord3
                border_active: Color::Rgb(136, 192, 208), // Nord8
                text: Color::Rgb(236, 239, 244),          // Nord6 Snow Storm
                text_muted: Color::Rgb(216, 222, 233),    // Nord4
                success: Color::Rgb(163, 190, 140),       // Nord14 Aurora
                warning: Color::Rgb(235, 203, 139),       // Nord13 Aurora
                error: Color::Rgb(191, 97, 106),          // Nord11 Aurora
                highlight: Color::Rgb(67, 76, 94),        // Nord2
                rain: Color::Rgb(136, 192, 208),
                fire_palette: [
                    Color::Rgb(46, 52, 64),
                    Color::Rgb(191, 97, 106),
                    Color::Rgb(208, 135, 112),
                    Color::Rgb(235, 203, 139),
                    Color::Rgb(245, 225, 170),
                    Color::Rgb(255, 250, 220),
                ],
            },
            ThemeKind::GruvboxDark => Self {
                kind,
                primary: Color::Rgb(250, 189, 47),        // Yellow
                secondary: Color::Rgb(184, 187, 38),      // Green
                accent: Color::Rgb(131, 165, 152),        // Blue
                background: Color::Rgb(40, 40, 40),       // bg0
                surface: Color::Rgb(29, 32, 33),          // bg0_h
                border: Color::Rgb(102, 92, 84),          // gray
                border_active: Color::Rgb(250, 189, 47),  // Yellow
                text: Color::Rgb(235, 219, 178),          // fg
                text_muted: Color::Rgb(168, 153, 132),    // fg4
                success: Color::Rgb(184, 187, 38),        // Green
                warning: Color::Rgb(254, 128, 25),        // Orange
                error: Color::Rgb(251, 73, 52),           // Red
                highlight: Color::Rgb(60, 56, 54),        // bg1
                rain: Color::Rgb(131, 165, 152),
                fire_palette: [
                    Color::Rgb(40, 40, 40),
                    Color::Rgb(204, 36, 29),
                    Color::Rgb(254, 128, 25),
                    Color::Rgb(250, 189, 47),
                    Color::Rgb(254, 220, 120),
                    Color::Rgb(255, 250, 200),
                ],
            },
            ThemeKind::TokyoNight => Self {
                kind,
                primary: Color::Rgb(122, 162, 247),       // Blue
                secondary: Color::Rgb(187, 154, 247),     // Purple
                accent: Color::Rgb(125, 207, 255),        // Cyan
                background: Color::Rgb(26, 27, 38),       // bg
                surface: Color::Rgb(22, 22, 30),          // bg_dark
                border: Color::Rgb(65, 72, 104),          // comment
                border_active: Color::Rgb(122, 162, 247), // Blue
                text: Color::Rgb(192, 202, 245),          // fg
                text_muted: Color::Rgb(115, 122, 162),    // fg_gutter
                success: Color::Rgb(158, 206, 106),       // Green
                warning: Color::Rgb(224, 175, 104),       // Yellow
                error: Color::Rgb(247, 118, 142),         // Red
                highlight: Color::Rgb(41, 46, 66),        // bg_highlight
                rain: Color::Rgb(125, 207, 255),
                fire_palette: [
                    Color::Rgb(26, 27, 38),
                    Color::Rgb(247, 118, 142),
                    Color::Rgb(255, 158, 100),
                    Color::Rgb(224, 175, 104),
                    Color::Rgb(255, 220, 150),
                    Color::Rgb(255, 250, 220),
                ],
            },
            ThemeKind::Monokai => Self {
                kind,
                primary: Color::Rgb(249, 38, 114),        // Pink/Red
                secondary: Color::Rgb(166, 226, 46),      // Green
                accent: Color::Rgb(102, 217, 239),        // Cyan
                background: Color::Rgb(39, 40, 34),       // Background
                surface: Color::Rgb(30, 31, 26),          // Darker
                border: Color::Rgb(117, 113, 94),         // Comment
                border_active: Color::Rgb(249, 38, 114),  // Pink
                text: Color::Rgb(248, 248, 242),          // Foreground
                text_muted: Color::Rgb(150, 145, 130),    // Muted
                success: Color::Rgb(166, 226, 46),        // Green
                warning: Color::Rgb(253, 151, 31),        // Orange
                error: Color::Rgb(249, 38, 114),          // Red
                highlight: Color::Rgb(62, 61, 50),        // Selection
                rain: Color::Rgb(102, 217, 239),
                fire_palette: [
                    Color::Rgb(39, 40, 34),
                    Color::Rgb(249, 38, 114),
                    Color::Rgb(253, 151, 31),
                    Color::Rgb(230, 219, 116),
                    Color::Rgb(255, 245, 160),
                    Color::Rgb(255, 255, 230),
                ],
            },
        }
    }
}
