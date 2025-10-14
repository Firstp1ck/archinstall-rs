use ratatui::style::Color;

#[derive(Clone, Copy)]
pub struct Theme {
    pub text: Color,
    pub subtext: Color,
    pub success: Color,
    pub warning: Color,
    pub accent: Color,
    pub highlight: Color,
}

/// Catppuccin Mocha palette
/// Ref: https://github.com/catppuccin/catppuccin
pub fn catppuccin_mocha() -> Theme {
    Theme {
        // text / subtext
        text: Color::Rgb(0xcd, 0xd6, 0xf4),      // text
        subtext: Color::Rgb(0xba, 0xc2, 0xde),   // subtext1
        // semantic
        success: Color::Rgb(0xa6, 0xe3, 0xa1),   // green
        warning: Color::Rgb(0xfa, 0xb3, 0x87),   // peach
        // accents
        accent: Color::Rgb(0xcb, 0xa6, 0xf7),    // mauve
        highlight: Color::Rgb(0x89, 0xdc, 0xeb), // sky
    }
}


