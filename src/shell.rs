use std::{
    io::{Write, stdin, stdout},
    path::PathBuf,
    process::{Child, Command, ExitStatus, Output, Stdio},
};

#[derive(Debug)]
pub struct Shell {
    curr_dir: PathBuf,
    args: Vec<String>,
}

impl Shell {
    pub fn new() -> Self {
        let args = std::env::args();
        let dir = dirs::home_dir().expect("Unable to find home!");
        Self {
            curr_dir: dir,
            args: args.collect(),
        }
    }

    /// change the current directory, only directory path is accepted
    pub fn cd(&mut self, path: PathBuf) {
        debug_assert!(path.is_dir(), "cd could only move to a directory!");
        self.curr_dir = path;
    }

    pub fn pwd(&self) {
        println!("{}", self.curr_dir.to_str().unwrap());
    }

    pub fn read_line() -> Result<Vec<String>, String> {
        let mut buffer = String::new();
        let stdin = std::io::stdin();
        match stdin.read_line(&mut buffer) {
            Ok(0) => return Err("EOF reached".to_string()),
            Ok(_) => {}
            Err(_) => return Err("Cannot read from stdin".to_string()),
        };
        Ok(buffer
            .trim()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect())
    }

    pub fn ls(&mut self) -> Output {
        let child = Command::new("ls")
            .arg(self.curr_dir.as_os_str())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to call ls");

        let output = child
            .wait_with_output()
            .expect("failed to get the output of ls");
        output
    }

    /// execute the command.
    pub fn execute(&mut self, args: &Vec<String>) -> Result<(), ()> {
        if let Some(cmd) = args.first() {
            match cmd.as_str() {
                "cd" => {
                    if let Some(target_dir_str) = args.get(1) {
                        let target_dir = PathBuf::from(target_dir_str);
                        if !target_dir.is_dir() {
                            eprintln!("cd: '{}' is not a directory", target_dir_str);
                            return Err(());
                        }
                        self.cd(target_dir);
                    }
                }
                "pwd" => {
                    self.pwd();
                }
                "ls" => {
                    let output = self.ls();
                    std::io::stdout()
                        .write_all(&output.stdout)
                        .expect("failed to write to stdout while showing the output of ls");
                }
                _ => {
                    unimplemented!("only supports cd for now")
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self) {
        loop {
            print!("simple_shell> ");
            std::io::stdout().flush().unwrap();
            let args = Self::read_line().unwrap();
            if let Some("exit") = args.first().map(|s| s.as_str()) {
                println!("exiting...");
                break;
            }
            let _ = self.execute(&args);
        }
    }
}
