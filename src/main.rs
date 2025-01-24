use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{exit, Command};

fn main() {
    let paths: Vec<String> = env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .map(String::from)
        .collect();

    loop {
        let input = read_input();
        let parsed_args = parse_arguments(&input);
        if parsed_args.is_empty() {
            continue;
        }
        
        let command = &parsed_args[0];
        let args = &parsed_args[1..];
        
        match command.as_str() {
            "cat" => cat_files(args),
            "cd" => change_directory(args.first().map(String::as_str).unwrap_or("")),
            "exit" => exit(0),
            "echo" => echo_input(args),
            "pwd" => print_working_directory(),
            "type" => handle_type_command(args.first().map(String::as_str).unwrap_or(""), &paths),
            _ => execute_command(command, &paths, args),
        }
    }
}

fn read_input() -> String {
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn cat_files(files: &[String]) {
    for file in files {
        if let Ok(content) = std::fs::read_to_string(file) {
            print!("{}", content);
        }
    }
}

fn echo_input(args: &[String]) {
    println!("{}", args.join(" "));
}

fn execute_command(command: &str, paths: &Vec<String>, args: &[String]) {
    if let Some(command_path) = find_command(command, &paths) {    
        Command::new(&command_path)
            .arg0(command)
            .args(args)
            .status()
            .unwrap_or_else(|_| panic!("Failed to execute: {}", command_path));
    } else {
        println!("{}: command not found", command);
    }
}

fn handle_type_command(command: &str, paths: &[String]) {
    if is_builtin(command) {
        println!("{} is a shell builtin", command);
    } else if let Some(command_path) = find_command(command, &paths) {
        println!("{command} is {command_path}");
    } else {
        println!("{}: not found", command);
    }
}

fn is_builtin(command: &str) -> bool {
    let builtins = ["cd", "echo", "exit", "pwd", "type"];
    builtins.contains(&command)
}

fn find_command(command: &str, paths: &[String]) -> Option<String> {
    if command.contains('/') {
        let path = Path::new(command);
        return if path.exists() {
            Some(command.to_string())
        } else {
            None
        };
    }
    
    for path_dir in paths {
        let candidate = Path::new(path_dir).join(command);
        if candidate.exists() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

fn print_working_directory() {
    println!("{}", env::current_dir().unwrap().display());
}

fn change_directory(new_working_directory: &str) {
    let path = if new_working_directory == "~" {
        env::home_dir().unwrap_or_default()
    } else {
        Path::new(new_working_directory).to_path_buf()
    };
    
    if let Err(_) = env::set_current_dir(path) {
        println!("cd: {}: No such file or directory", new_working_directory);
    }
}

fn parse_arguments(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape_next = false;

    for c in input.chars() {
        if escape_next {
            if in_double {
                match c {
                    '\\' | '"' | '$' | '`' | '\n' => current_arg.push(c),
                    _ => {
                        current_arg.push('\\');
                        current_arg.push(c);
                    }
                }
            } else {
                // Non-quoted escape: Add character without backslash
                current_arg.push(c);
            }
            escape_next = false;
        } else if c == '\\' {
            if in_double {
                escape_next = true;
            } else if !in_single {
                // Handle backslash in non-quoted context
                escape_next = true;
            } else {
                // In single quotes, backslash is literal
                current_arg.push(c);
            }
        } else {
            match c {
                '\'' => {
                    if !in_double {
                        in_single = !in_single;
                    } else {
                        current_arg.push(c);
                    }
                }
                '"' => {
                    if !in_single {
                        in_double = !in_double;
                    } else {
                        current_arg.push(c);
                    }
                }
                ' ' | '\t' | '\n' if !in_single && !in_double => {
                    if !current_arg.is_empty() {
                        args.push(current_arg);
                        current_arg = String::new();
                    }
                }
                _ => current_arg.push(c),
            }
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}