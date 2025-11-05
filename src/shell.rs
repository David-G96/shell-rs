use std::{
    io::{Stdin, Write},
    path::PathBuf,
    process::{Child, Command, Output, Stdio},
};

#[derive(Debug)]
pub struct Shell {
    curr_dir: PathBuf,
    #[allow(dead_code)]
    args: Vec<String>,
    prompt: String,
    home_dir: PathBuf,
}

impl Shell {
    pub fn new() -> Self {
        let args = std::env::args();
        let home_dir = dirs::home_dir().expect("Unable to find home!");
        Self {
            curr_dir: home_dir.clone(),
            args: args.collect(),
            prompt: "shell-rs> ".to_string(),
            home_dir,
        }
    }

    /// change the current directory, only directory path is accepted
    pub fn cd(&mut self, path: PathBuf) {
        debug_assert!(path.is_dir(), "cd could only move to a directory!");
        self.curr_dir = path;
    }

    /// read a line from stdin
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

    /// Internal command: ls
    /// # NOTE
    ///  current implementation uses a system call of ls, which is NOT portable
    pub fn ls(&mut self, rest_args: &[String]) -> Result<Output, String> {
        // default path: pwd
        let child = if rest_args
            .last()
            .map(|s| s.trim().starts_with("-"))
            .unwrap_or(false)
        {
            Command::new("ls")
                .args(rest_args)
                .arg(self.curr_dir.as_os_str())
                .stdout(Stdio::piped())
                .spawn()
        } else {
            Command::new("ls")
                .args(rest_args)
                .stdout(Stdio::piped())
                .spawn()
        };

        let child = match child {
            Ok(child) => child,
            Err(e) => return Err(format!("failed to call ls: {}", e)),
        };
        let output = child.wait_with_output();
        match output {
            Ok(output) => Ok(output),
            Err(e) => Err(format!("failed to get the output of ls: {}", e)),
        }
    }

    /// execute an internal command, panics if the cmd is not a internal command
    /// if the internal command has a output, Ok(Some(output)) will be returned
    pub fn execute_internal(
        &mut self,
        cmd: &str,
        rest_args: &[String],
    ) -> Result<Option<String>, String> {
        match cmd {
            "cd" => {
                if let Some(target_dir_str) = rest_args.first() {
                    let target_dir = if target_dir_str == "~" {
                        // Handle home directory shortcut
                        self.home_dir.clone()
                    } else {
                        let path = PathBuf::from(target_dir_str);
                        if path.is_absolute() {
                            // Absolute path
                            path
                        } else {
                            // Relative path - join with current directory
                            self.curr_dir.join(path)
                        }
                    };
                    // Canonicalize the path to resolve .. and . components
                    let canonical_path = target_dir
                        .canonicalize()
                        .map_err(|e| format!("cd: {}: No such file or directory", e))?;

                    if !canonical_path.is_dir() {
                        return Err(format!("cd: '{}' is not a directory", target_dir_str));
                    }
                    self.cd(canonical_path);
                } else {
                    self.cd(self.home_dir.clone());
                }
                return Ok(None);
            }
            "pwd" => {
                return Ok(Some(
                    self.curr_dir
                        .to_str()
                        .ok_or("pwd: current directory is not valid UTF-8")?
                        .to_string(),
                ));
            }
            s => {
                unreachable!(
                    "internal logic error: reaching a unconsidered internal command: '{}'",
                    s
                )
            }
        }
    }

    pub fn execute_external(&mut self, cmd: &str, rest_args: &[String]) -> Result<Output, String> {
        match cmd {
            "ls" => {
                return self.ls(rest_args);
            }
            s => {
                unreachable!(
                    "internal error: reaching a unreachable external command: '{}'",
                    s
                )
            }
        }
    }

    /// execute the command.
    pub fn execute(&mut self, args: &Vec<String>) -> Result<(), ()> {
        if let Some((cmd, rest_args)) = args.split_first() {
            match cmd.as_str() {
                "cd" | "pwd" => {
                    let internal_result = self.execute_internal(cmd, rest_args);
                    match internal_result {
                        Ok(Some(output)) => println!("{}", &output),
                        Err(err) => eprintln!("{}", &err),
                        _ => {}
                    }
                }
                "ls" => {
                    let external_result = self.execute_external(cmd, rest_args);
                    match external_result {
                        Ok(output) => {
                            if !output.stdout.is_empty() {
                                match String::from_utf8(output.stdout) {
                                    Ok(stdout) => println!("{}", stdout),
                                    Err(e) => eprintln!("Failed to parse stdout: {}", e),
                                }
                            }
                            if !output.stderr.is_empty() {
                                match String::from_utf8(output.stderr) {
                                    Ok(stderr) => eprintln!("{}", stderr),
                                    Err(e) => eprintln!("Failed to parse stderr: {}", e),
                                }
                            }
                        }
                        Err(err) => eprintln!("{}", &err),
                    }
                }
                _ => {
                    println!("unknown command: '{}'", cmd)
                }
            }
        }

        Ok(())
    }

    pub fn run(&mut self) {
        loop {
            print!("{}", self.prompt);
            std::io::stdout().flush().unwrap();
            let args = Self::read_line().unwrap();
            // exit command, exit loop immediately
            if let Some("exit") = args.first().map(|s| s.as_str()) {
                println!("exiting...");
                break;
            }
            // regular command, execute it
            let _ = self.execute(&args);
        }
    }
}
