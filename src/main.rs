mod shell;

use std::io;

use crate::shell::Shell;

fn main() -> io::Result<()> {
    let mut shell = Shell::new();

    shell.run();

    Ok(())
}
