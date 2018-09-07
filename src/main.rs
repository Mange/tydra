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
        flush_terminal();
        eprintln!("Error: {}", error);
    }
}

fn load_actions_from_path(path: &str) -> Result<ActionFile, Error> {
    std::fs::read_to_string(path)
        .map_err(Error::from)
        .and_then(|data| serde_yaml::from_str(&data).map_err(Error::from))
}

fn flush_terminal() {
    // Flush the output from Terminal being dropped; this is not done by termion itself.
    // https://gitlab.redox-os.org/redox-os/termion/issues/158
    //
    // Printing to stderr before stdout is flushed, or letting other processes write to it,
    // means that the text ends up on the alternate screen that will be removed as soon as *our*
    // stdout buffer is flushed.
    use std::io::Write;
    ::std::io::stdout().flush().ok();
}

/// Wrapper around an AlternateScreen terminal, that handles restoration on drop.
struct TermHandle(Term);

impl TermHandle {
    /// Opens the terminal's "Alternate screen" and hide the cursor.
    ///
    /// This is like a separate screen that you can ouput to freely, and when this screen is closed
    /// the previous screen is restored. Most terminal UIs use this in order to not clobber output
    /// from earlier commands. For example, run vim and exit it again and you can see that your
    /// terminal is restored to look like it did before you started vim.
    ///
    /// Will restore cursor when dropped.
    fn new() -> Result<TermHandle, Error> {
        let backend = AlternateScreenBackend::new()?;
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        terminal.clear()?;
        Ok(TermHandle(terminal))
    }

    fn restart(self) -> Result<TermHandle, Error> {
        TermHandle::new()
    }
}

impl Drop for TermHandle {
    fn drop(&mut self) {
        self.0.show_cursor().ok();
    }
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

    let mut terminal = TermHandle::new()?;

    // Loop
    loop {
        render(&mut terminal.0, current_page, &page_settings)?;

        // Wait for an event from user input.
        let action = process_input(current_page)?;
        let return_to = match action {
            // Quit / Exit.
            Action::Exit => Return::Quit,

            // Redraw menu.
            Action::Redraw => {
                terminal = terminal.restart()?;
                Return::SamePage
            }

            // Run a command in normal mode, e.g. pause tydra and run the command. Return to tydra
            // after the command exits.
            Action::Run {
                command,
                return_to,
                wait,
            } => {
                terminal = run_normal(terminal, error_on_failure, command, wait)?;
                return_to
            }

            // Replace tydra with the command's process.
            // If it returns, it has to be an error.
            Action::RunExec { command } => return Err(run_exec(terminal, command)),

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

    Ok(())
}

fn run_normal(
    terminal: TermHandle,
    error_on_failure: bool,
    command: actions::Command,
    wait: bool,
) -> Result<TermHandle, Error> {
    // Run commands on the normal screen. This preserves the command's output even
    // after tydra exits.
    drop(terminal);
    flush_terminal();

    match runner::run_normal(&command) {
        Ok(exit_status) => {
            if error_on_failure && !exit_status.success() {
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

    TermHandle::new()
}

// Can use `!` when it is stable; it never returns a non-error
fn run_exec(terminal: TermHandle, command: actions::Command) -> Error {
    // Restore screen for the new command.
    drop(terminal);
    flush_terminal();

    // If this returns, then it failed to exec the process so wrap that value in a
    // error.
    runner::run_exec(&command)
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
