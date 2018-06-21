mod rendering;
mod validator;
pub use self::validator::ValidationError;

use super::Term;
use failure::Error;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionFile {
    #[serde(rename = "global", default = "default_settings")]
    global_settings: Settings,
    pages: BTreeMap<String, Page>, // BTreeMap so order is preserved; helps with validation logic, etc.
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    layout: Option<Layout>,
    color: Option<Color>,
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
    color: Option<Color>,
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
    Red,
    Green,
    Yellow,
    Blue,
    Cyan,
    Purple,
}

fn default_page_title() -> String {
    String::from("Tydra")
}

fn default_command() -> String {
    String::from("/bin/true")
}

fn default_settings() -> Settings {
    Settings {
        color: Some(Color::Red),
        layout: Some(Layout::List),
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
}

impl Page {
    pub fn all_entries(&self) -> impl Iterator<Item = &Entry> {
        self.groups.iter().flat_map(|group| group.entries.iter())
    }

    pub fn layout(&self) -> Option<Layout> {
        match self.settings {
            Some(ref settings) => settings.layout,
            None => None,
        }
    }
}

impl Layout {
    pub fn render(&self, terminal: &mut Term, page: &Page) -> Result<(), Error> {
        match *self {
            Layout::List => self::rendering::render_list(terminal, page),
            Layout::Columns => self::rendering::render_columns(terminal, page),
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
