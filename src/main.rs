extern crate failure;
extern crate serde;
extern crate serde_yaml;
extern crate termion;
extern crate tui;

#[macro_use]
extern crate failure_derive;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate structopt;

mod actions;

use actions::{Action, ActionFile, Layout, Page, Return};
use failure::Error;
use structopt::StructOpt;
use termion::event;
use tui::backend::AlternateScreenBackend;
use tui::Terminal;

type Term = Terminal<AlternateScreenBackend>;

#[derive(Debug, StructOpt)]
struct AppOptions {
    #[structopt(value_name = "ACTION_FILE")]
    filename: String,

    /// Instead of showing the menu, validate the action file.
    #[structopt(long = "validate")]
    validate: bool,
}

fn main() {
    let options = AppOptions::from_args();
    let actions: ActionFile = load_actions(options.filename).expect("Failed to parse file");
    match actions.validate() {
        Ok(_) => {}
        Err(errors) => {
            eprintln!("Actions are invalid: {:#?}", errors);
            std::process::exit(1);
        }
    }

    if options.validate {
        eprintln!("File is valid.");
        std::process::exit(0);
    }

    run_menu(actions).unwrap();
}

fn load_actions(path: String) -> Result<ActionFile, Error> {
    std::fs::read_to_string(path)
        .map_err(|e| Error::from(e))
        .and_then(|data| serde_yaml::from_str(&data).map_err(|e| Error::from(e)))
}

fn run_menu(actions: ActionFile) -> Result<(), Error> {
    let default_layout = actions.layout().unwrap_or_default();
    let mut current_page = actions.get_page("root");

    // Render menu, wait for keypress and determine new action. Between all actions the terminal
    // needs to be restored so the nested command output becomes visible.
    loop {
        let action = {
            let backend = AlternateScreenBackend::new()?;
            let mut terminal = Terminal::new(backend)?;
            terminal.hide_cursor()?;

            let action = match render_menu(&mut terminal, current_page, default_layout) {
                Ok(action) => action,
                Err(error) => {
                    terminal.show_cursor().ok();
                    return Err(error);
                }
            };

            terminal.show_cursor().ok();
            drop(terminal);
            action
        };

        match action {
            Action::Exit => return Ok(()),
            Action::Run { command, return_to } => {
                println!("Running command: {}", command);
                run_command(&command)?;
                match return_to {
                    Return::Quit => return Ok(()),
                    Return::Page(next_page) => current_page = actions.get_page(&next_page),
                }
            }
        }
    }
}

fn render_menu<'a>(
    terminal: &mut Term,
    page: &'a Page,
    default_layout: Layout,
) -> Result<Action, Error> {
    use termion::input::TermRead;
    let current_layout = page.layout().unwrap_or(default_layout);
    let stdin = std::io::stdin();

    terminal.clear()?;
    current_layout.render(terminal, page)?;
    for evnt in stdin.keys().flat_map(Result::ok) {
        match evnt {
            event::Key::Esc => return Ok(Action::exit()),
            _ => {}
        }
    }

    Ok(Action::default())
}

fn run_command(command: &str) -> Result<(), Error> {
    use std::process::Command;

    let _ = Command::new("/bin/sh").arg("-c").arg(&command).status()?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(())
}
