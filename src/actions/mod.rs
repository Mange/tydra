use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ActionFile {
    #[serde(rename = "settings", default = "default_settings")]
    global_settings: Settings,
    pages: HashMap<String, Page>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    layout: Option<Layout>,
    color: Option<Color>,
}

#[derive(Debug, Deserialize)]
pub struct Page {
    title: String,
    header: Option<String>,
    footer: Option<String>,
    settings: Option<Settings>,
    groups: Vec<Group>,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    title: Option<String>,
    settings: Option<Settings>,
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    title: Option<String>,
    shortcut: char,
    #[serde(default = "default_command")]
    command: String,
    color: Option<Color>,
    #[serde(rename = "return")]
    return_to: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    List,
    Columns,
}

#[derive(Debug, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    extern crate serde_yaml;

    #[test]
    fn it_loads_complex_yaml() {
        let _: ActionFile =
            serde_yaml::from_str(include_str!("../../tests/fixtures/complex.yml")).unwrap();
    }
}
