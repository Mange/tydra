use failure::Error;
use std::process::{Command, ExitStatus};

pub fn run_normal(command: &str) -> Result<ExitStatus, Error> {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(&command)
        .status()
        .map_err(|e| e.into())
}

#[cfg(unix)]
pub fn run_exec(command: &str) -> Error {
    use std::os::unix::process::CommandExt;
    Command::new("/bin/sh")
        .arg("-c")
        .arg(&command)
        .exec()
        .into()
}

#[cfg(not(unix))]
pub fn run_exec(command: &str) -> Error {
    match run_normal(command) {
        Ok(exit_status) => std::process::exit(exit_status.code().unwrap_or(0)),
        Err(error) => error,
    }
}
