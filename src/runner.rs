extern crate nix;

use actions::Command;
use failure::Error;
use std::process;
use std::process::{ExitStatus, Stdio};

impl Command {
    fn to_process_command(&self) -> process::Command {
        match *self {
            Command::ShellScript(ref script) => {
                let mut command = process::Command::new("/bin/sh");
                command.arg("-c").arg(script);
                command
            }
            Command::Executable { ref name, ref args } => {
                let mut command = process::Command::new(name);
                command.args(args);
                command
            }
        }
    }
}

pub fn run_normal(command: &Command) -> Result<ExitStatus, Error> {
    command.to_process_command().status().map_err(|e| e.into())
}

#[cfg(unix)]
pub fn run_exec(command: &Command) -> Error {
    use std::os::unix::process::CommandExt;
    command.to_process_command().exec().into()
}

#[cfg(not(unix))]
pub fn run_exec(command: &Command) -> Error {
    match run_normal(command) {
        Ok(exit_status) => std::process::exit(exit_status.code().unwrap_or(0)),
        Err(error) => error,
    }
}

#[cfg(unix)]
pub fn run_background(command: &Command) -> Result<(), Error> {
    use std::os::unix::process::CommandExt;
    command
        .to_process_command()
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
pub fn run_background(_command: &Command) -> Result<(), Error> {
    return Err(format_err!(
        "Running in background is currently only supported on unix platforms."
    ));
}
