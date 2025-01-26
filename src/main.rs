use std::env;
use std::fs::File;
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
        let args = parsed_args[1..].to_vec();

        let (processed_args, stdout_redir) = process_redirections(args);

        match command.as_str() {
            // "cat" => {
            //     let output = cat_files(&processed_args);
            //     handle_output(output, &stdout_redir);
            // }
            "cd" => change_directory(processed_args.first().map(String::as_str).unwrap_or("")),
            "exit" => exit(0),
            "echo" => {
                let output = echo_input(&processed_args);
                handle_output(output, &stdout_redir);
            }
            "pwd" => {
                let output = print_working_directory();
                handle_output(output, &stdout_redir);
            }
            "type" => {
                let cmd = processed_args.first().map(String::as_str).unwrap_or("");
                let output = handle_type_command(cmd, &paths);
                handle_output(output, &stdout_redir);
            }
            _ => execute_command(&command, &paths, &processed_args, &stdout_redir),
        }
    }
}

#[derive(Debug)]
enum RedirectionType {
    Stdout,
}

#[derive(Debug)]
struct Redirection {
    ty: RedirectionType,
    filename: String,
}

fn process_redirections(args: Vec<String>) -> (Vec<String>, Option<Redirection>) {
    let mut processed_args = Vec::new();
    let mut stdout_redir = None;

    let mut i = 0;
    while i < args.len() {
        if args[i] == ">" || args[i] == "1>" {
            if i + 1 < args.len() {
                stdout_redir = Some(Redirection {
                    ty: RedirectionType::Stdout,
                    filename: args[i + 1].clone(),
                });
                i += 2;
            } else {
                processed_args.push(args[i].clone());
                i += 1;
            }
        } else {
            processed_args.push(args[i].clone());
            i += 1;
        }
    }

    (processed_args, stdout_redir)
}

fn read_input() -> String {
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn cat_files(files: &[String]) -> String {
    let mut output = String::new();
    for file in files {
        match std::fs::read_to_string(file) {
            Ok(content) => output.push_str(&content),
            Err(e) => output.push_str(&format!("cat: {}: {}\n", file, e)),
        }
    }
    output
}

fn echo_input(args: &[String]) -> String {
    args.join(" ") + "\n"
}

fn execute_command(
    command: &str,
    paths: &[String],
    args: &[String],
    stdout_redir: &Option<Redirection>,
) {
    if let Some(command_path) = find_command(command, paths) {
        let mut cmd = Command::new(&command_path);
        cmd.arg0(command);
        cmd.args(args);

        if let Some(redir) = stdout_redir {
            match File::create(&redir.filename) {
                Ok(file) => {
                    cmd.stdout(file);
                }
                Err(e) => {
                    eprintln!("Error creating file {}: {}", redir.filename, e);
                    return;
                }
            }
        }

        let status = cmd
            .status()
            .unwrap_or_else(|_| panic!("Failed to execute: {}", command_path));
    } else {
        println!("{}: command not found", command);
    }
}

fn handle_type_command(command: &str, paths: &[String]) -> String {
    if is_builtin(command) {
        format!("{} is a shell builtin\n", command)
    } else if let Some(command_path) = find_command(command, &paths) {
        format!("{} is {}\n", command, command_path)
    } else {
        format!("{}: not found\n", command)
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

fn print_working_directory() -> String {
    format!("{}\n", env::current_dir().unwrap().display())
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

fn handle_output(output: String, stdout_redir: &Option<Redirection>) {
    if let Some(redir) = stdout_redir {
        if let Err(e) = std::fs::write(&redir.filename, output) {
            eprintln!("Error writing to {}: {}", redir.filename, e);
        }
    } else {
        print!("{}", output);
        io::stdout().flush().unwrap();
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
