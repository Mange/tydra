use super::{Entry, Group, Page};

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    layout: Option<Layout>,
    shortcut_color: Option<Color>,
}

#[derive(Debug, Default, Clone)]
pub struct SettingsAccumulator {
    pub layout: Layout,
    pub shortcut_color: Color,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    List,
    Columns,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    Reset,
    Black,
    Blue,
    Cyan,
    Green,
    Magenta,
    Red,
    White,
    Yellow,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            shortcut_color: Some(Color::Red),
            layout: Some(Layout::default()),
        }
    }
}

impl SettingsAccumulator {
    pub fn with_settings(&self, settings: &Settings) -> SettingsAccumulator {
        SettingsAccumulator {
            layout: settings.layout.unwrap_or(self.layout),
            shortcut_color: settings.shortcut_color.unwrap_or(self.shortcut_color),
        }
    }

    pub fn with_page(&self, page: &Page) -> SettingsAccumulator {
        match page.settings() {
            Some(settings) => self.with_settings(settings),
            None => self.clone(),
        }
    }

    pub fn with_group(&self, group: &Group) -> SettingsAccumulator {
        match group.settings() {
            Some(settings) => self.with_settings(settings),
            None => self.clone(),
        }
    }

    pub fn with_entry(&self, entry: &Entry) -> SettingsAccumulator {
        SettingsAccumulator {
            layout: self.layout,
            shortcut_color: entry.shortcut_color().unwrap_or(self.shortcut_color),
        }
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }
}

impl<'a> From<&'a Settings> for SettingsAccumulator {
    fn from(settings: &'a Settings) -> SettingsAccumulator {
        let default_settings = Settings::default();
        SettingsAccumulator {
            layout: settings
                .layout
                .or(default_settings.layout)
                .unwrap_or_default(),
            shortcut_color: settings
                .shortcut_color
                .or(default_settings.shortcut_color)
                .unwrap_or_default(),
        }
    }
}

impl Default for Layout {
    fn default() -> Layout {
        Layout::List
    }
}

impl Default for Color {
    fn default() -> Color {
        Color::Reset
    }
}

impl Color {
    pub fn markup_name(&self) -> &'static str {
        match *self {
            Color::Reset => "reset",
            Color::Black => "black",
            Color::Blue => "blue",
            Color::Cyan => "cyan",
            Color::Green => "green",
            Color::Magenta => "magenta",
            Color::Red => "red",
            Color::White => "white",
            Color::Yellow => "yellow",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_sane_defaults() {
        let default_settings = Settings::default();
        assert_eq!(default_settings.layout, Some(Layout::List));
        assert_eq!(default_settings.shortcut_color, Some(Color::Red));
    }

    #[test]
    fn it_accumulates_settings() {
        let settings1 = Settings {
            layout: Some(Layout::Columns),
            shortcut_color: Some(Color::Green),
        };
        let settings2 = Settings {
            layout: None,
            shortcut_color: Some(Color::Yellow),
        };

        let accumulator = SettingsAccumulator::from(&settings1);
        assert_eq!(accumulator.layout, Layout::Columns);
        assert_eq!(accumulator.shortcut_color, Color::Green);

        let accumulator = accumulator.with_settings(&settings2);
        assert_eq!(accumulator.layout, Layout::Columns);
        assert_eq!(accumulator.shortcut_color, Color::Yellow);
    }

    #[test]
    fn it_accumulates_default_settings_on_none() {
        let default_settings = Settings::default();

        let blank_settings = Settings {
            layout: None,
            shortcut_color: None,
        };

        let accumulator = SettingsAccumulator::from(&blank_settings);
        assert_eq!(accumulator.layout, default_settings.layout.unwrap());
        assert_eq!(
            accumulator.shortcut_color,
            default_settings.shortcut_color.unwrap()
        );
    }
}
