use failure::Error;
use std::process::{Command, ExitStatus};

pub fn run_normal(command: &str) -> Result<ExitStatus, Error> {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(&command)
        .status()
        .map_err(|e| e.into())
}
