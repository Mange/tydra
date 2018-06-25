mod action_file;
mod group;
mod page;
mod rendering;
mod settings;
mod validator;

pub use self::action_file::ActionFile;
pub use self::group::Group;
pub use self::page::Page;
pub use self::rendering::render;
pub use self::settings::{Color, Layout, Settings, SettingsAccumulator};
pub use self::validator::ValidationError;

const DEFAULT_COMMAND: &str = "/bin/true";

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

fn default_command() -> String {
    String::from(DEFAULT_COMMAND)
}

impl Entry {
    pub fn shortcut(&self) -> char {
        self.shortcut
    }

    pub fn title(&self) -> &str {
        &self.title
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
