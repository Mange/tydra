extern crate serde;

use super::Color;
use serde::de::{self, Deserialize, Deserializer, Visitor};
use std::fmt;

/// Represents a single entry in the action file. This entry is something a user can select when
/// they are on the page that contains this entry.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// The title of the entry. Will be rendered in the menu.
    title: String,

    /// The character used to activate this shortcut; e.g. 'c' to activate when user presses the C
    /// key on their keyboard, or 'C' to activate when user presses Shift+C keys.
    shortcut: char,

    /// The Command to run when activating this entry. By default this will be a small no-op
    /// Command that should always succeed.
    #[serde(default)]
    command: Command,

    /// Optional color to use when rendering the shortcut key in the menu. Will be inherited from
    /// the Page's settings if unset here.
    shortcut_color: Option<Color>,

    /// The runner mode, e.g. if the command should run in the background, replace the process, or
    /// some other runner mode.
    #[serde(default, rename = "mode")]
    runner_mode: RunMode,

    /// Specification on where to return to after executing the command.
    #[serde(rename = "return", default)]
    return_to: Return,
}

/// Represents something to execute when an Entry is selected.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields, untagged)]
pub enum Command {
    /// A full shell script; will be run inside /bin/sh.
    ShellScript(String),

    /// A raw executable and a list of arguments. Will not do any shell processing or extra
    /// wrapping of the executable.
    Executable {
        /// Command name (from $PATH) or full path.
        name: String,

        /// List of arguments to pass to the command.
        #[serde(default)]
        args: Vec<String>,
    }
}

/// An action, aka something to do in the menu event loop.
#[derive(Debug)]
pub enum Action {
    /// Run a command in normal mode.
    Run {
        command: Command,
        return_to: Return,

        /// For RunMode::Wait commands
        wait: bool,
    },
    /// Replace tydra with a Command.
    RunExec {
        command: Command,
    },

    /// Run a Command in the background and return to tydra.
    RunBackground {
        command: Command,
        return_to: Return,
    },

    /// Exit tydra.
    Exit,

    /// Redraw (re-render) the menu again. Good if your terminal window has been resized or on any
    /// other display problems.
    Redraw,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunMode {
    /// Runs the command and then returns to tydra as soon as it has finished.
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

    pub fn runner_mode(&self) -> RunMode {
        self.runner_mode
    }
}

impl<'a> From<&'a Entry> for Action {
    /// Convert a Entry into an Action for consumption by the main event loop.
    fn from(entry: &'a Entry) -> Action {
        let command = entry.command.clone();
        match entry.runner_mode {
            RunMode::Normal | RunMode::Wait => Action::Run {
                command,
                return_to: entry.return_to.clone(),
                wait: entry.runner_mode.is_wait(),
            },
            RunMode::Exec => Action::RunExec { command },
            RunMode::Background => Action::RunBackground {
                command,
                return_to: entry.return_to.clone(),
            },
        }
    }
}

impl Default for Command {
    /// The default command should run a simple no-op command.
    fn default() -> Command {
        Command::Executable { name: String::from("/bin/true"), args: vec![] }
    }
}

impl Default for RunMode {
    fn default() -> RunMode {
        RunMode::Normal
    }
}

impl RunMode {
    fn is_wait(self) -> bool {
        match self {
            RunMode::Wait => true,
            _ => false,
        }
    }
}

impl Default for Return {
    fn default() -> Return {
        Return::Quit
    }
}

/// Parse a string as a page name, or true as "SamePage" and false as "Quit".
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
    /// Parse a string as a page name, or true as "SamePage" and false as "Quit".
    fn deserialize<D>(deserializer: D) -> Result<Return, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ReturnVisitor)
    }
}

impl fmt::Display for Command {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Command::ShellScript(ref script) => script.fmt(formatter),
            Command::Executable { ref name, ref args } => {
                if args.is_empty() {
                    write!(formatter, "{}", name)
                } else {
                    write!(formatter, "{} {:?}", name, args)
                }
            }
        }
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
    fn it_displays_commands() {
        let script = Command::ShellScript(String::from("echo foo bar baz"));
        let executable = Command::Executable {
            name: String::from("ls"),
            args: vec![String::from("-l"), String::from("/")],
        };
        let no_args = Command::Executable {
            name: String::from("/bin/true"),
            args: vec![],
        };

        assert_eq!(&format!("{}", script), "echo foo bar baz");
        assert_eq!(&format!("{}", executable), "ls [\"-l\", \"/\"]");
        assert_eq!(&format!("{}", no_args), "/bin/true");
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
