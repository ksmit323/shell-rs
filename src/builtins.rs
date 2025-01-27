use std::env;
use std::path::Path;

use crate::output::CommandOutput;
use crate::utils;

pub fn echo_input(args: &[String]) -> CommandOutput {
    CommandOutput {
        stdout: args.join(" ") + "\n",
        stderr: String::new(),
    }
}

pub fn print_working_directory() -> CommandOutput {
    CommandOutput {
        stdout: format!("{}\n", env::current_dir().unwrap().display()),
        stderr: String::new(),
    }
}

pub fn change_directory(new_working_directory: &str) -> CommandOutput {
    let path = if new_working_directory == "~" {
        #[allow(deprecated)]
        env::home_dir().unwrap_or_else(|| Path::new("").to_path_buf())
    } else {
        Path::new(new_working_directory).to_path_buf()
    };

    match env::set_current_dir(&path) {
        Ok(()) => CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
        },
        Err(_) => CommandOutput {
            stdout: String::new(),
            stderr: format!("cd: {}: No such file or directory\n", new_working_directory),
        },
    }
}

pub fn handle_type_command(command: &str, paths: &[String]) -> CommandOutput {
    let output = if is_builtin(command) {
        format!("{} is a shell builtin\n", command)
    } else if let Some(command_path) = utils::find_command(command, paths) {  // Change this
        format!("{} is {}\n", command, command_path)
    } else {
        format!("{}: not found\n", command)
    };
    CommandOutput {
        stdout: output,
        stderr: String::new(),
    }
}

pub fn is_builtin(command: &str) -> bool {
    let builtins = ["cd", "echo", "exit", "pwd", "type"];
    builtins.contains(&command)
}