mod rendering;
mod validator;
pub use self::validator::ValidationError;

use super::Term;
use failure::Error;
use std::collections::BTreeMap;

const DEFAULT_COMMAND: &str = "/bin/true";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionFile {
    #[serde(rename = "global", default = "Settings::default")]
    global_settings: Settings,
    pages: BTreeMap<String, Page>, // BTreeMap so order is preserved; helps with validation logic, etc.
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    layout: Option<Layout>,
    shortcut_color: Option<Color>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    #[serde(default = "default_page_title")]
    title: String,
    header: Option<String>,
    footer: Option<String>,
    settings: Option<Settings>,
    groups: Vec<Group>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    title: Option<String>,
    settings: Option<Settings>,
    entries: Vec<Entry>,
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

fn default_page_title() -> String {
    String::from("Tydra")
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
        match page.settings {
            Some(ref settings) => self.with_settings(settings),
            None => self.clone(),
        }
    }

    pub fn with_group(&self, group: &Group) -> SettingsAccumulator {
        match group.settings {
            Some(ref settings) => self.with_settings(settings),
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

impl ActionFile {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        self::validator::validate(self)
    }

    pub fn has_page(&self, page_name: &str) -> bool {
        self.pages.contains_key(page_name)
    }

    pub fn get_page(&self, page_name: &str) -> &Page {
        self.pages.get(page_name).unwrap()
    }

    pub fn layout(&self) -> Option<Layout> {
        self.global_settings.layout
    }

    pub fn settings_accumulator(&self) -> SettingsAccumulator {
        let settings = self.global_settings.clone();
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

impl Page {
    pub fn all_entries(&self) -> impl Iterator<Item = &Entry> {
        self.groups.iter().flat_map(|group| group.entries.iter())
    }

    pub fn entry_with_shortcut(&self, shortcut: char) -> Option<&Entry> {
        self.all_entries().find(|entry| entry.shortcut == shortcut)
    }

    pub fn layout(&self) -> Option<Layout> {
        match self.settings {
            Some(ref settings) => settings.layout,
            None => None,
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

#[cfg(test)]
mod tests {
    use super::*;
    extern crate serde_yaml;

    #[test]
    fn it_loads_minimal_yaml() {
        let actions: ActionFile =
            serde_yaml::from_str(include_str!("../../tests/fixtures/minimal.yml")).unwrap();
        actions.validate().unwrap();
    }

    #[test]
    fn it_loads_complex_yaml() {
        let actions: ActionFile =
            serde_yaml::from_str(include_str!("../../tests/fixtures/complex.yml")).unwrap();
        actions.validate().unwrap();
    }
}
