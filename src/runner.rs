extern crate nix;

use failure::Error;
use std::process::{Command, ExitStatus, Stdio};

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

#[cfg(unix)]
pub fn run_background(command: &str) -> Result<(), Error> {
    use std::os::unix::process::CommandExt;
    Command::new("/bin/sh")
        .arg("-c")
        .arg(&command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .before_exec(|| {
            // Make forked process into a new session leader; child will therefore not quit if
            // parent quits.
            nix::unistd::setsid().ok();
            Ok(())
        })
        .spawn()
        .map_err(|e| e.into())
        .map(|_| ())
}

#[cfg(not(unix))]
pub fn run_background(command: &str) -> Result<(), Error> {
    return Err(format_err!(
        "Running in background is currently only supported on unix platforms."
    ));
}
