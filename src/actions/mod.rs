mod action_file;
mod group;
mod page;
mod rendering;
mod validator;
pub use self::action_file::ActionFile;
pub use self::group::Group;
pub use self::page::Page;
pub use self::validator::ValidationError;

use super::Term;
use failure::Error;

const DEFAULT_COMMAND: &str = "/bin/true";

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    layout: Option<Layout>,
    shortcut_color: Option<Color>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    title: String,
    shortcut: char,
    #[serde(default = "default_command")]
    command: String,
    shortcut_color: Option<Color>,
    #[serde(rename = "return")]
    return_to: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    List,
    Columns,
}

#[derive(Debug, Deserialize, Clone, Copy)]
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

#[derive(Debug)]
pub enum Action {
    Run { command: String, return_to: Return },
    Exit,
    Redraw,
}

#[derive(Debug)]
pub enum Return {
    Quit,
    Page(String),
}

#[derive(Debug, Default, Clone)]
pub struct SettingsAccumulator {
    pub layout: Layout,
    pub shortcut_color: Color,
}

impl<'a> From<&'a Settings> for SettingsAccumulator {
    fn from(settings: &Settings) -> SettingsAccumulator {
        SettingsAccumulator {
            layout: settings.layout.unwrap_or_default(),
            shortcut_color: settings.shortcut_color.unwrap_or_default(),
        }
    }
}

fn default_command() -> String {
    String::from(DEFAULT_COMMAND)
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            shortcut_color: Some(Color::Red),
            layout: Some(Layout::default()),
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
            shortcut_color: entry.shortcut_color.unwrap_or(self.shortcut_color),
        }
    }
}

impl Entry {
    pub fn shortcut(&self) -> char {
        self.shortcut
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

impl Color {
    fn markup_name(&self) -> &'static str {
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

impl<'a> From<&'a Entry> for Action {
    fn from(entry: &'a Entry) -> Action {
        Action::run_command(entry.command.clone(), Return::from(entry))
    }
}

impl<'a> From<&'a Entry> for Return {
    fn from(entry: &'a Entry) -> Return {
        match entry.return_to {
            None => Return::Quit,
            Some(ref page_name) if page_name == "quit" => Return::Quit,
            Some(ref page_name) => Return::Page(page_name.clone()),
        }
    }
}

impl Action {
    pub fn run_command<S>(command: S, return_to: Return) -> Action
    where
        S: Into<String>,
    {
        Action::Run {
            command: command.into(),
            return_to,
        }
    }
}

impl Default for Action {
    fn default() -> Action {
        Action::run_command(DEFAULT_COMMAND, Return::Quit)
    }
}

impl Layout {
    pub fn render(
        &self,
        terminal: &mut Term,
        page: &Page,
        settings: &SettingsAccumulator,
    ) -> Result<(), Error> {
        match *self {
            Layout::List => self::rendering::render_list_layout(terminal, page, settings),
            Layout::Columns => self::rendering::render_columns_layout(terminal, page, settings),
        }
    }
}
