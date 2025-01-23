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
    let files: Vec<&str> = input
        .split('\'')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1) // Select parts inside quotes (odd indices)
        .map(|(_, part)| part.trim())
        .filter(|&x| !x.is_empty()) 
        .collect();
    
    for file in files {
        if let Ok(content) = std::fs::read_to_string(file) {
            print!("{}", content);
        }
    }
}

fn echo_input(input: &str) {
    if input.contains('\'') {
        let parts: Vec<&str> = input.split('\'')
            .filter(|&x| !x.is_empty())
            .collect();
        println!("{}", parts.join(""));
    } else {
        let words: Vec<&str> = input.split_whitespace().collect();
        println!("{}", words.join(" "));
    }
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
