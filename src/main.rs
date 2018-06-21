extern crate failure;
extern crate serde;
extern crate serde_yaml;

#[macro_use]
extern crate failure_derive;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate structopt;

mod actions;

use actions::ActionFile;
use failure::Error;
use structopt::StructOpt;

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

    println!("{:#?}", actions);
}

fn load_actions(path: String) -> Result<ActionFile, Error> {
    std::fs::read_to_string(path)
        .map_err(|e| Error::from(e))
        .and_then(|data| serde_yaml::from_str(&data).map_err(|e| Error::from(e)))
}
