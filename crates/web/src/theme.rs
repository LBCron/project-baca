//! Shell theme (Espresso or Paper) and reader preferences, persisted in
//! localStorage. `boot.js` applies the stored shell theme before first paint
//! so the page never flashes the wrong surface.

use serde::{Deserialize, Serialize};

const SHELL_KEY: &str = "rotaria_theme";
const READER_KEY: &str = "rotaria_reader_prefs";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellTheme {
    Espresso,
    Paper,
}

impl ShellTheme {
    pub fn load() -> Self {
        match storage()
            .and_then(|s| s.get_item(SHELL_KEY).ok().flatten())
            .as_deref()
        {
            Some("paper") => ShellTheme::Paper,
            _ => ShellTheme::Espresso,
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            ShellTheme::Espresso => ShellTheme::Paper,
            ShellTheme::Paper => ShellTheme::Espresso,
        }
    }

    /// Persists and stamps `data-theme` on the root element.
    pub fn apply(self) {
        if let Some(s) = storage() {
            let _ = s.set_item(
                SHELL_KEY,
                match self {
                    ShellTheme::Espresso => "espresso",
                    ShellTheme::Paper => "paper",
                },
            );
        }
        if let Some(root) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            match self {
                ShellTheme::Paper => {
                    let _ = root.set_attribute("data-theme", "paper");
                }
                ShellTheme::Espresso => {
                    let _ = root.remove_attribute("data-theme");
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadingTheme {
    Paper,
    Sepia,
    Espresso,
}

impl ReadingTheme {
    pub fn attr(self) -> &'static str {
        match self {
            ReadingTheme::Paper => "paper",
            ReadingTheme::Sepia => "sepia",
            ReadingTheme::Espresso => "espresso",
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            ReadingTheme::Paper => "theme_paper",
            ReadingTheme::Sepia => "theme_sepia",
            ReadingTheme::Espresso => "theme_espresso",
        }
    }

    pub const ALL: [ReadingTheme; 3] = [
        ReadingTheme::Paper,
        ReadingTheme::Sepia,
        ReadingTheme::Espresso,
    ];
}

pub const FONT_SIZE_MIN: u8 = 15;
pub const FONT_SIZE_MAX: u8 = 24;
/// Line height stored in tenths (16 = 1.6).
pub const LINE_HEIGHT_MIN: u8 = 15;
pub const LINE_HEIGHT_MAX: u8 = 20;
/// The three leading presets offered in the reader, as (tenths, label key).
pub const LEADING_PRESETS: [(u8, &str); 3] = [
    (15, "leading_tight"),
    (17, "leading_normal"),
    (19, "leading_loose"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaderPrefs {
    pub theme: ReadingTheme,
    pub font_size: u8,
    pub line_height: u8,
}

impl Default for ReaderPrefs {
    fn default() -> Self {
        Self {
            theme: ReadingTheme::Paper,
            font_size: 19,
            line_height: 17,
        }
    }
}

impl ReaderPrefs {
    pub fn load() -> Self {
        storage()
            .and_then(|s| s.get_item(READER_KEY).ok().flatten())
            .and_then(|json| serde_json::from_str(&json).ok())
            .map(Self::clamped)
            .unwrap_or_default()
    }

    pub fn save(self) {
        if let (Some(s), Ok(json)) = (storage(), serde_json::to_string(&self)) {
            let _ = s.set_item(READER_KEY, &json);
        }
    }

    pub fn clamped(self) -> Self {
        Self {
            theme: self.theme,
            font_size: self.font_size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX),
            line_height: self.line_height.clamp(LINE_HEIGHT_MIN, LINE_HEIGHT_MAX),
        }
    }

    pub fn line_height_css(self) -> String {
        format!("{:.1}", f32::from(self.line_height) / 10.0)
    }
}
