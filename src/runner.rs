extern crate nix;

use crate::actions::Command;
use failure::Error;
use std::process;
use std::process::{ExitStatus, Stdio};

impl Command {
    fn to_process_command(&self) -> Option<process::Command> {
        match *self {
            Command::None => None,
            Command::ShellScript(ref script) => {
                let mut command = process::Command::new("/bin/sh");
                command.arg("-c").arg(script);
                Some(command)
            }
            Command::Executable { ref name, ref args } => {
                let mut command = process::Command::new(name);
                command.args(args);
                Some(command)
            }
        }
    }
}

pub fn run_normal(command: &Command) -> Option<Result<ExitStatus, Error>> {
    command
        .to_process_command()
        .map(|mut command| command.status().map_err(|e| e.into()))
}

#[cfg(unix)]
pub fn run_exec(command: &Command) -> Error {
    use std::os::unix::process::CommandExt;
    command
        .to_process_command()
        .expect("Validations did not catch an exec with no command. Please report this as a bug!")
        .exec()
        .into()
}

#[cfg(not(unix))]
pub fn run_exec(command: &Command) -> Error {
    match run_normal(command) {
        Ok(exit_status) => std::process::exit(exit_status.code().unwrap_or(0)),
        Err(error) => error,
    }
}

#[cfg(unix)]
pub unsafe fn run_background(command: &Command) -> Result<(), Error> {
    use std::os::unix::process::CommandExt;
    match command.to_process_command() {
        Some(mut command) => command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .pre_exec(|| {
                // Make forked process into a new session leader; child will therefore not quit if
                // parent quits.
                nix::unistd::setsid().ok();
                Ok(())
            }).spawn()
            .map_err(|e| e.into())
            .map(|_| ()),
        None => Ok(()),
    }
}

#[cfg(not(unix))]
pub fn run_background(_command: &Command) -> Result<(), Error> {
    return Err(format_err!(
        "Running in background is currently only supported on unix platforms."
    ));
}
