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

use actions::{render, Action, ActionFile, Page, Return};
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
    let actions: ActionFile =
        load_actions_from_path(&options.filename).expect("Failed to parse file");

    // Validate the action file so it is semantically correct before continuing.
    if let Err(errors) = actions.validate() {
        eprintln!("Actions are invalid: {:#?}", errors);
        std::process::exit(1);
    }

    // If running in validation mode, exit with a message after passing validations.
    if options.validate {
        eprintln!("File is valid.");
        std::process::exit(0);
    }

    // Run the menu. If it fails, then print the error message.
    if let Err(error) = run_menu(&actions, &options) {
        // Make sure to restore the cursor before outputing the error. There are some exit
        // conditions of run_menu that does not restore after themselves.
        // TODO: Make run_menu always clean up after itself.
        show_cursor();
        eprintln!("Error: {}", error);
    }
}

fn load_actions_from_path(path: &str) -> Result<ActionFile, Error> {
    std::fs::read_to_string(path)
        .map_err(Error::from)
        .and_then(|data| serde_yaml::from_str(&data).map_err(Error::from))
}

/// Opens the terminal's "Alternate screen" and hide the cursor.
///
/// This is like a separate screen that you can ouput to freely, and when this screen is closed the
/// previous screen is restored. Most terminal UIs use this in order to not clobber output from
/// earlier commands. For example, run vim and exit it again and you can see that your terminal is
/// restored to look like it did before you started vim.
fn open_alternate_screen() -> Result<Term, Error> {
    let backend = AlternateScreenBackend::new()?;
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    Ok(terminal)
}

/// Stop the alternate screen and show the cursor again.
fn stop_alternate_screen(mut terminal: Term) -> Result<(), Error> {
    terminal.show_cursor()?;
    drop(terminal);
    Ok(())
}

/// Show cursor without depending on a Terminal.
fn show_cursor() {
    println!("{}", termion::cursor::Show);
}

/// Starts the main event loop.
///
/// Start:
///     Begin on page "root".
///     Open alternate screen.
/// Loop:
///     Render menu.
///     Wait for a valid input.
///     Process input's event, possibly running a command.
///     Wait for user to press enter, if waiting is enabled.
///     Update which page to be on.
///     Exit if event tells us to.
///     Repeat loop.
/// End:
///     Restore screen.
///
fn run_menu(actions: &ActionFile, options: &AppOptions) -> Result<(), Error> {
    // Code in this function is annotated according to the function documentation comment to help
    // navigate it. It is quite big, sadly.

    // Start
    let error_on_failure = !options.ignore_exit_status;
    let settings = actions.settings_accumulator();
    let mut current_page = actions.get_page("root");
    let mut page_settings = settings.with_page(&current_page);

    let mut terminal = open_alternate_screen()?;

    // Loop
    loop {
        render(&mut terminal, current_page, &page_settings)?;

        // Wait for an event from user input.
        let action = process_input(current_page)?;
        let return_to = match action {
            // Quit / Exit.
            Action::Exit => Return::Quit,

            // Redraw menu.
            Action::Redraw => {
                // Force total reflow by starting over with a new alternate screen.
                stop_alternate_screen(terminal)?;
                terminal = open_alternate_screen()?;

                Return::SamePage
            }

            // Run a command in normal mode, e.g. pause tydra and run the command. Return to tydra
            // after the command exits.
            Action::Run {
                command,
                return_to,
                wait,
            } => {
                // Run commands on the normal screen. This preserves the command's output even
                // after tydra exits.
                stop_alternate_screen(terminal)?;

                // Run the command in normal mode.
                match runner::run_normal(&command) {
                    // Command ran; check if it failed or succeeded.
                    Ok(exit_status) => {
                        if error_on_failure && !exit_status.success() {
                            return Err(format_err!(
                                "Command exited with exit status {}: {}",
                                exit_status.code().unwrap_or(1),
                                command
                            ));
                        }
                    }
                    // Could not run command.
                    Err(err) => return Err(err),
                }

                // Wait for user confirmation ("Press enter to continue")
                if wait {
                    wait_for_confirmation()?;
                }

                // Go back to the alternate screen again.
                terminal = open_alternate_screen()?;

                return_to
            }

            // Replace tydra with the command's process.
            Action::RunExec { command } => {
                // Restore screen first of all.
                stop_alternate_screen(terminal)?;
                // If this returns, then it failed to exec the process so wrap that value in a
                // error.
                return Err(runner::run_exec(&command));
            }

            // Run command in background and immediately return to the menu again.
            Action::RunBackground { command, return_to } => {
                runner::run_background(&command)?;
                return_to
            }
        };

        // Decide on which page to render now.
        match return_to {
            Return::Quit => break,
            Return::SamePage => continue,
            Return::OtherPage(page_name) => {
                current_page = actions.get_page(&page_name);
                page_settings = settings.with_page(&current_page);
            }
        }
    }

    // End
    terminal.show_cursor()?;
    Ok(())
}

/// Reads input events until a valid event is found and returns it as an Action. Reads actions from
/// provided page to determine what events are valid.
fn process_input(page: &Page) -> Result<Action, Error> {
    use termion::input::TermRead;
    let stdin = std::io::stdin();

    // Iterate all valid events
    for event in stdin.keys().flat_map(Result::ok) {
        match event {
            event::Key::Esc => return Ok(Action::Exit),
            event::Key::Ctrl('l') => return Ok(Action::Redraw),
            event::Key::Char(chr) => {
                if let Some(entry) = page.entry_with_shortcut(chr) {
                    return Ok(entry.into());
                }
            }
            _ => {}
        }
    }

    // stdin closed, or other state that makes stdin not produce any output anymore
    // TODO: Have a nicer error message here.
    Err(format_err!("stdin eof"))
}

/// Waits for the user to press Enter (or Escape, just to be nice) before returning.
fn wait_for_confirmation() -> Result<(), Error> {
    use termion::input::TermRead;
    let stdin = std::io::stdin();

    println!("Press enter to continue... ");

    for event in stdin.keys().flat_map(Result::ok) {
        match event {
            event::Key::Char('\n') | event::Key::Esc => return Ok(()),
            _ => {}
        }
    }

    // stdin closed, or other state that makes stdin not produce any output anymore
    // TODO: Have a nicer error message here.
    Err(format_err!("stdin eof"))
}
