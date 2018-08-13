extern crate serde;

use super::Color;
use serde::de::{self, Deserialize, Deserializer, Visitor};
use std::fmt;

const DEFAULT_COMMAND: &str = "/bin/true";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    title: String,
    shortcut: char,
    #[serde(default = "Entry::default_command")]
    command: String,
    shortcut_color: Option<Color>,
    #[serde(default)]
    mode: Mode,
    #[serde(rename = "return", default)]
    return_to: Return,
}

#[derive(Debug)]
pub enum Action {
    Run {
        command: String,
        return_to: Return,

        /// For Mode::Wait commands
        wait: bool,
    },
    RunExec {
        command: String,
    },
    RunBackground {
        command: String,
        return_to: Return,
    },
    Exit,
    Redraw,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Normal operation; e.g. none of the other alternatives.
    Normal,

    /// Display a "Press enter to continue" prompt after the command has finished before
    /// progressing. This lets the user read all the output before the next action takes place.
    Wait,

    /// Replace this process with the given command instead of just running it as a child process.
    Exec,

    /// Fork and exec the command with no terminal devices still attached. This is useful for
    /// starting GUI programs.
    Background,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Return {
    Quit,
    SamePage,
    OtherPage(String),
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
        self.shortcut_color
    }

    pub fn return_to(&self) -> &Return {
        &self.return_to
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
}

impl<'a> From<&'a Entry> for Action {
    fn from(entry: &'a Entry) -> Action {
        let command = entry.command.clone();
        match entry.mode {
            Mode::Normal | Mode::Wait => Action::Run {
                command,
                return_to: entry.return_to.clone(),
                wait: entry.mode.is_wait(),
            },
            Mode::Exec => Action::RunExec { command },
            Mode::Background => Action::RunBackground {
                command,
                return_to: entry.return_to.clone(),
            },
        }
    }
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::Normal
    }
}

impl Mode {
    fn is_wait(self) -> bool {
        match self {
            Mode::Wait => true,
            _ => false,
        }
    }
}

impl Default for Return {
    fn default() -> Return {
        Return::Quit
    }
}

struct ReturnVisitor;

impl<'de> Visitor<'de> for ReturnVisitor {
    type Value = Return;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a boolean or string")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Return, E>
    where
        E: de::Error,
    {
        if value {
            Ok(Return::SamePage)
        } else {
            Ok(Return::Quit)
        }
    }

    fn visit_string<E>(self, value: String) -> Result<Return, E>
    where
        E: de::Error,
    {
        Ok(Return::OtherPage(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Return, E>
    where
        E: de::Error,
    {
        Ok(Return::OtherPage(value.to_owned()))
    }

    fn visit_unit<E>(self) -> Result<Return, E>
    where
        E: de::Error,
    {
        Ok(Return::default())
    }
}

impl<'de> Deserialize<'de> for Return {
    fn deserialize<D>(deserializer: D) -> Result<Return, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ReturnVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate serde_yaml;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    pub struct OnlyReturn {
        #[serde(rename = "return")]
        return_to: Return,
    }

    #[test]
    fn it_deserializes_returns() {
        assert_eq!(
            serde_yaml::from_str::<OnlyReturn>(r#"return: false"#).unwrap(),
            OnlyReturn {
                return_to: Return::Quit
            },
        );

        assert_eq!(
            serde_yaml::from_str::<OnlyReturn>(r#"return: true"#).unwrap(),
            OnlyReturn {
                return_to: Return::SamePage
            },
        );

        assert_eq!(
            serde_yaml::from_str::<OnlyReturn>(r#"return: foobar"#).unwrap(),
            OnlyReturn {
                return_to: Return::OtherPage("foobar".into()),
            },
        );

        assert_eq!(
            serde_yaml::from_str::<OnlyReturn>(r#"return: "#).expect("Failed to parse empty value"),
            OnlyReturn {
                return_to: Return::Quit
            },
        );
    }
}
