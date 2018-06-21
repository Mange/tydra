extern crate failure;
extern crate serde;
extern crate serde_yaml;
extern crate tui;

#[macro_use]
extern crate failure_derive;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate structopt;

mod actions;

use actions::{ActionFile, Layout, Page};
use failure::Error;
use structopt::StructOpt;
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

    let backend = AlternateScreenBackend::new()?;
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    loop {
        match render_menu(&mut terminal, current_page, default_layout) {
            Ok(Some(next_page)) => current_page = next_page,
            Ok(None) => break,
            Err(error) => {
                terminal.show_cursor().ok();
                return Err(error);
            }
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
    terminal.show_cursor().ok();
    Ok(())
}

fn render_menu<'a>(
    terminal: &mut Term,
    page: &'a Page,
    default_layout: Layout,
) -> Result<Option<&'a Page>, Error> {
    let current_layout = page.layout().unwrap_or(default_layout);

    current_layout.render(terminal, page)?;

    Ok(None)
}
