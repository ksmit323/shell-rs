use std::env;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{exit, Command};

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::{Context, Editor, Result as RustylineResult};

use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;

fn main() {
    let paths: Vec<String> = env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .map(String::from)
        .collect();

    let mut rl = Editor::with_config(
        rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build(),
    )
    .expect("Should create readline instance");
    rl.set_helper(Some(BuiltInCompleter::new()));

    loop {
        let input = match rl.readline("$ ") {
            Ok(line) => {
                rl.add_history_entry(&line);
                line
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        };

        let parsed_args = parse_arguments(&input);
        if parsed_args.is_empty() {
            continue;
        }

        let command = &parsed_args[0].clone();
        let args = parsed_args[1..].to_vec();

        let (processed_args, stdout_redir, stderr_redir) = process_redirections(args);

        match command.as_str() {
            "cd" => {
                let output =
                    change_directory(processed_args.first().map(String::as_str).unwrap_or(""));
                handle_redirections(output, &stdout_redir, &stderr_redir);
            }
            "exit" => exit(0),
            "echo" => {
                let output = echo_input(&processed_args);
                handle_redirections(output, &stdout_redir, &stderr_redir);
            }
            "pwd" => {
                let output = print_working_directory();
                handle_redirections(output, &stdout_redir, &stderr_redir);
            }
            "type" => {
                let cmd = processed_args.first().map(String::as_str).unwrap_or("");
                let output = handle_type_command(cmd, &paths);
                handle_redirections(output, &stdout_redir, &stderr_redir);
            }
            _ => execute_command(
                &command,
                &paths,
                &processed_args,
                &stdout_redir,
                &stderr_redir,
            ),
        }
    }
}

#[derive(Debug, PartialEq)]
enum RedirectionType {
    Stdout,
    Stderr,
}

#[derive(Debug)]
enum RedirectionMode {
    Truncate,
    Append,
}

#[derive(Debug)]
struct Redirection {
    ty: RedirectionType,
    mode: RedirectionMode,
    filename: String,
}

struct CommandOutput {
    stdout: String,
    stderr: String,
}

struct BuiltInCompleter {
    paths: Vec<String>,
    last_prefix: String,
    tab_count: usize,
}

impl BuiltInCompleter {
    fn new() -> Self {
        let paths = env::var("PATH")
            .unwrap_or_default()
            .split(':')
            .map(String::from)
            .collect();
        Self {
            paths,
            last_prefix: String::new(),
            tab_count: 0,
        }
    }

    fn find_executables(&self, prefix: &str) -> Vec<String> {
        let mut matches = Vec::new();

        for path_dir in &self.paths {
            if let Ok(entries) = std::fs::read_dir(path_dir) {
                for entry in entries.filter_map(Result::ok) {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        if file_name.starts_with(prefix) {
                            // Check if the file is executable
                            if let Ok(metadata) = entry.metadata() {
                                use std::os::unix::fs::PermissionsExt;
                                if metadata.permissions().mode() & 0o111 != 0 {
                                    matches.push(file_name);
                                }
                            }
                        }
                    }
                }
            }
        }

        matches.sort(); // Sort alphabetically
        matches
    }
}

impl Completer for BuiltInCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> RustylineResult<(usize, Vec<Pair>)> {
        let mut completions = Vec::new();
        let prefix = &line[..pos];

        // Only complete first token
        if let Some(_first_space) = prefix.find(' ') {
            return Ok((0, vec![]));
        }

        // Try builtin commands first
        for cmd in ["echo", "exit"] {
            if cmd.starts_with(prefix) {
                completions.push(Pair {
                    display: cmd.to_string(),
                    replacement: format!("{} ", cmd),
                });
            }
        }

        // Then try executables in PATH
        let mut exe_matches = self.find_executables(prefix);

        // No matches: ring bell and return original
        if exe_matches.is_empty() && completions.is_empty() {
            print!("\x07");
            io::stdout().flush().unwrap();
            return Ok((0, vec![]));
        }

        // Single match: return it with a space
        if exe_matches.len() == 1 {
            completions.push(Pair {
                display: exe_matches[0].clone(),
                replacement: format!("{} ", exe_matches[0]),
            });
        }
        // Single builtin match: already handled above
        else if completions.len() == 1 {
            // Keep the existing completion
        }
        // Multiple matches: show all and ring bell
        else if exe_matches.len() > 1 {
            exe_matches.sort();
            println!("\n{}", exe_matches.join("  "));
            print!("\x07$ {}", prefix);
            io::stdout().flush().unwrap();
            return Ok((0, vec![]));
        }

        Ok((0, completions))
    }
}

impl Helper for BuiltInCompleter {}

impl Hinter for BuiltInCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for BuiltInCompleter {}
impl Validator for BuiltInCompleter {}

fn process_redirections(
    args: Vec<String>,
) -> (Vec<String>, Option<Redirection>, Option<Redirection>) {
    let mut processed_args = Vec::new();
    let mut stdout_redir = None;
    let mut stderr_redir = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            // Stdout truncate
            ">" | "1>" => parse_redirection_operator(
                &mut stdout_redir,
                &args,
                &mut i,
                RedirectionType::Stdout,
                RedirectionMode::Truncate,
            ),
            // Stdout append
            ">>" | "1>>" => parse_redirection_operator(
                &mut stdout_redir,
                &args,
                &mut i,
                RedirectionType::Stdout,
                RedirectionMode::Append,
            ),
            // Stderr truncate
            "2>" => parse_redirection_operator(
                &mut stderr_redir,
                &args,
                &mut i,
                RedirectionType::Stderr,
                RedirectionMode::Truncate,
            ),
            // Stderr append
            "2>>" => parse_redirection_operator(
                &mut stderr_redir,
                &args,
                &mut i,
                RedirectionType::Stderr,
                RedirectionMode::Append,
            ),
            // Regular argument
            _ => {
                processed_args.push(args[i].clone());
                i += 1;
            }
        }
    }
    (processed_args, stdout_redir, stderr_redir)
}

fn parse_redirection_operator(
    target: &mut Option<Redirection>,
    args: &[String],
    i: &mut usize,
    ty: RedirectionType,
    mode: RedirectionMode,
) {
    if *i + 1 < args.len() {
        *target = Some(Redirection {
            ty,
            mode,
            filename: args[*i + 1].clone(),
        });
        *i += 2; // Skip operator and filename
    } else {
        // Malformed redirection, treat as regular argument
        target.as_mut().map(|r| r.filename.push_str(&args[*i]));
        *i += 1;
    }
}

fn echo_input(args: &[String]) -> CommandOutput {
    CommandOutput {
        stdout: args.join(" ") + "\n",
        stderr: String::new(),
    }
}

fn execute_command(
    command: &str,
    paths: &[String],
    args: &[String],
    stdout_redir: &Option<Redirection>,
    stderr_redir: &Option<Redirection>,
) {
    use std::fs::OpenOptions;

    if let Some(command_path) = find_command(command, paths) {
        let mut cmd = Command::new(&command_path);
        cmd.arg0(command);
        cmd.args(args);

        // Handle stdout redirection
        if let Some(redir) = stdout_redir {
            match OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(matches!(redir.mode, RedirectionMode::Truncate))
                .append(matches!(redir.mode, RedirectionMode::Append))
                .open(&redir.filename)
            {
                Ok(file) => {
                    cmd.stdout(file);
                }
                Err(e) => {
                    eprintln!("Failed to open {}: {}", redir.filename, e);
                    return;
                }
            }
        }

        // Handle stderr redirection
        if let Some(redir) = stderr_redir {
            match OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(matches!(redir.mode, RedirectionMode::Truncate))
                .append(matches!(redir.mode, RedirectionMode::Append))
                .open(&redir.filename)
            {
                Ok(file) => {
                    cmd.stderr(file);
                }
                Err(e) => {
                    eprintln!("Failed to open {}: {}", redir.filename, e);
                    return;
                }
            }
        }

        // Execute the command and ignore errors (matching shell behavior)
        let _ = cmd.status();
    } else {
        eprintln!("{}: command not found", command);
    }
}

fn handle_type_command(command: &str, paths: &[String]) -> CommandOutput {
    let output = if is_builtin(command) {
        format!("{} is a shell builtin\n", command)
    } else if let Some(command_path) = find_command(command, paths) {
        format!("{} is {}\n", command, command_path)
    } else {
        format!("{}: not found\n", command)
    };
    CommandOutput {
        stdout: output,
        stderr: String::new(),
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

fn print_working_directory() -> CommandOutput {
    CommandOutput {
        stdout: format!("{}\n", env::current_dir().unwrap().display()),
        stderr: String::new(),
    }
}

fn change_directory(new_working_directory: &str) -> CommandOutput {
    let path = if new_working_directory == "~" {
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

fn handle_redirections(
    output: CommandOutput,
    stdout_redir: &Option<Redirection>,
    stderr_redir: &Option<Redirection>,
) {
    use std::fs::OpenOptions;
    use std::io::Write;

    // Handle stdout redirection (including append)
    if let Some(redir) = stdout_redir {
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(matches!(redir.mode, RedirectionMode::Truncate))
            .append(matches!(redir.mode, RedirectionMode::Append))
            .open(&redir.filename)
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(output.stdout.as_bytes()) {
                    eprintln!("Error writing stdout to {}: {}", redir.filename, e);
                }
            }
            Err(e) => eprintln!("Error opening {}: {}", redir.filename, e),
        }
    } else {
        print!("{}", output.stdout);
        let _ = io::stdout().flush();
    }

    // Handle stderr redirection (including append)
    if let Some(redir) = stderr_redir {
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(matches!(redir.mode, RedirectionMode::Truncate))
            .append(matches!(redir.mode, RedirectionMode::Append))
            .open(&redir.filename)
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(output.stderr.as_bytes()) {
                    eprintln!("Error writing stderr to {}: {}", redir.filename, e);
                }
            }
            Err(e) => eprintln!("Error opening {}: {}", redir.filename, e),
        }
    } else {
        eprint!("{}", output.stderr);
        let _ = io::stderr().flush();
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
