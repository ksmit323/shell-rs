use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};
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
        let args: Vec<&str> = input.split_whitespace().collect();
        if args.is_empty() {
            continue;
        }
        
        match args[0] {
            "cat" => cat_files(&input[4..]),
            "cd" => change_directory(args[1]),
            "exit" => exit(0),
            "echo" => echo_input(&input[5..]),
            "pwd" => print_working_directory(),
            "type" => handle_type_command(args[1], &paths),
            command if command.starts_with("custom_exe") => {
                execute_command(command, &args[1..], &paths)
            }
            command => println!("{}: command not found", command),
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

fn cat_files(input: &str) {
    let files = parse_arguments(input);
    for file in files {
        if let Ok(content) = std::fs::read_to_string(&file) {
            print!("{}", content);
        }
    }
}

fn echo_input(input: &str) {
    let args = parse_arguments(input);
    println!("{}", args.join(" "));
}

fn execute_command(command: &str, args: &[&str], paths: &[String]) {
    if let Some(_command_path) = find_command(command, paths) {
        Command::new(command)
            .args(args)
            .status()
            .expect("Failed to execute command");
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
    for path in paths {
        let full_path = format!("{}/{}", path, command);
        if Path::new(&full_path).exists() {
            return Some(full_path);
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
                current_arg.push('\\');
                current_arg.push(c);
            }
            escape_next = false;
        } else if c == '\\' && in_double {
            escape_next = true;
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