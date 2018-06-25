use super::Color;

const DEFAULT_COMMAND: &str = "/bin/true";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    title: String,
    shortcut: char,
    #[serde(default = "Entry::default_command")]
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

impl Entry {
    fn default_command() -> String {
        String::from(DEFAULT_COMMAND)
    }

    pub fn shortcut(&self) -> char {
        self.shortcut
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn shortcut_color(&self) -> Option<Color> {
        self.shortcut_color.clone()
    }

    pub fn return_to(&self) -> Return {
        match self.return_to.as_ref().map(String::as_ref) {
            Some("quit") | None => Return::Quit,
            Some(page_name) => Return::Page(page_name.to_owned()),
        }
    }
}

impl<'a> From<&'a Entry> for Action {
    fn from(entry: &'a Entry) -> Action {
        Action::Run {
            command: entry.command.clone(),
            return_to: entry.return_to(),
        }
    }
}
