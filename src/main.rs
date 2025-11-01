mod shell;

use std::process::ExitCode;

use crate::shell::Shell;

fn main() -> ExitCode {
    let mut shell = Shell::new();
    shell.run();

    ExitCode::SUCCESS
}
