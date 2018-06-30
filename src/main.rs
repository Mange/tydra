extern crate serde;
extern crate serde_yaml;
extern crate termion;
extern crate tui;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate failure_derive;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate structopt;

mod actions;
mod runner;

use actions::{render, Action, ActionFile, Page, Return, SettingsAccumulator};
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

    /// When a command fails, ignore it and do not exit tydra.
    #[structopt(long = "ignore-exit-status", short = "e")]
    ignore_exit_status: bool,
}

fn main() {
    let options = AppOptions::from_args();
    let actions: ActionFile = load_actions(&options.filename).expect("Failed to parse file");
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

    if let Err(error) = run_menu(actions, &options) {
        show_cursor();
        eprintln!("Error: {}", error);
    }
}

fn load_actions(path: &String) -> Result<ActionFile, Error> {
    std::fs::read_to_string(path)
        .map_err(|e| Error::from(e))
        .and_then(|data| serde_yaml::from_str(&data).map_err(|e| Error::from(e)))
}

fn open_alternate_screen() -> Result<Term, Error> {
    let backend = AlternateScreenBackend::new()?;
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    Ok(terminal)
}

fn stop_alternate_screen(terminal: Term) -> Result<(), Error> {
    drop(terminal);
    show_cursor();
    Ok(())
}

/// Show cursor without depending on a Terminal; useful since then it can be called just after
/// dropping a Terminal to get out of AlternateScreenBackend.
fn show_cursor() {
    println!("{}", termion::cursor::Show);
}

fn run_menu(actions: ActionFile, options: &AppOptions) -> Result<(), Error> {
    let ignore_exit_status = options.ignore_exit_status;
    let settings = actions.settings_accumulator();
    let mut current_page = actions.get_page("root");

    let mut terminal = open_alternate_screen()?;

    loop {
        render_menu(&mut terminal, current_page, &settings)?;
        let action = process_input(current_page)?;
        match action {
            Action::Exit => break,
            Action::Redraw => {
                stop_alternate_screen(terminal)?;
                terminal = open_alternate_screen()?;
            }
            Action::Run {
                command,
                return_to,
                wait,
            } => {
                stop_alternate_screen(terminal)?;
                match runner::run_normal(&command) {
                    Ok(exit_status) => {
                        if exit_status.success() || ignore_exit_status {
                            // Intentionally left blank
                        } else {
                            // Error, AND it should not be ignored
                            return Err(format_err!(
                                "Command exited with exit status {}: {}",
                                exit_status.code().unwrap_or(1),
                                command
                            ));
                        }
                    }
                    Err(err) => return Err(err),
                }
                if wait {
                    wait_for_confirmation()?;
                }
                terminal = open_alternate_screen()?;
                match return_to {
                    Return::Quit => break,
                    Return::SamePage => continue,
                    Return::OtherPage(page_name) => current_page = actions.get_page(&page_name),
                }
            }
            Action::RunExec { command } => {
                stop_alternate_screen(terminal)?;
                // If this returns, then it failed to exec the process
                return Err(runner::run_exec(&command));
            }
            Action::RunBackground { .. } => {
                unimplemented!("Running in background is still not implemented")
            }
        }
    }

    terminal.show_cursor()?;
    Ok(())
}

fn render_menu<'a>(
    terminal: &mut Term,
    page: &'a Page,
    settings: &SettingsAccumulator,
) -> Result<(), Error> {
    let settings = settings.with_page(&page);

    render(terminal, page, &settings)
}

fn process_input<'a>(page: &'a Page) -> Result<Action, Error> {
    use termion::input::TermRead;
    let stdin = std::io::stdin();

    for evnt in stdin.keys().flat_map(Result::ok) {
        match evnt {
            event::Key::Esc => return Ok(Action::Exit),
            event::Key::Ctrl('l') => return Ok(Action::Redraw),
            event::Key::Char(c) => match page.entry_with_shortcut(c) {
                Some(entry) => return Ok(entry.into()),
                None => {}
            },
            _ => {}
        }
    }

    // stdin closed, or other state that makes stdin not produce any output anymore
    Err(format_err!("stdin eof"))
}

fn wait_for_confirmation() -> Result<(), Error> {
    use termion::input::TermRead;
    let stdin = std::io::stdin();

    println!("Press enter to continue... ");

    for evnt in stdin.keys().flat_map(Result::ok) {
        match evnt {
            event::Key::Char('\n') | event::Key::Esc => return Ok(()),
            _ => {}
        }
    }

    // stdin closed, or other state that makes stdin not produce any output anymore
    Err(format_err!("stdin eof"))
}
