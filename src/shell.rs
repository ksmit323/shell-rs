use std::env;
use std::process::exit;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::history::DefaultHistory;

use crate::autocompletion::BuiltInCompleter;
use crate::builtins;  // Add this import
use crate::command::execute_command;
use crate::output;
use crate::parser::parse_arguments;
use crate::redirection::process_redirections;

pub struct Shell {
    paths: Vec<String>,
    editor: Editor<BuiltInCompleter, DefaultHistory>, 
}

impl Shell {
    pub fn new() -> Self {
        let paths: Vec<String> = env::var("PATH")
            .unwrap_or_default()
            .split(':')
            .map(String::from)
            .collect();

        let mut editor = Editor::with_config(
            rustyline::Config::builder()
                .completion_type(rustyline::CompletionType::List)
                .build(),
        )
        .expect("Should create readline instance");
        
        editor.set_helper(Some(BuiltInCompleter::new()));

        Shell { 
            paths,
            editor,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.read_input() {
                Ok(input) => self.process_input(&input),
                Err(true) => break, // Control-C or EOF
                Err(false) => continue,
            }
        }
    }

    fn read_input(&mut self) -> Result<String, bool> {
        match self.editor.readline("$ ") {
            Ok(line) => {
                self.editor.add_history_entry(&line);
                Ok(line)
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => Err(true),
            Err(err) => {
                eprintln!("Error: {:?}", err);
                Err(true)
            }
        }
    }

    fn process_input(&self, input: &str) {
        let parsed_args = parse_arguments(input);
        if parsed_args.is_empty() {
            return;
        }

        let command = &parsed_args[0];
        let args = parsed_args[1..].to_vec();
        
        let (processed_args, stdout_redir, stderr_redir) = process_redirections(args);
        
        match command.as_str() {
            "cd" => {
                let output = builtins::change_directory(
                    processed_args.first().map(String::as_str).unwrap_or(""),
                );
                output::apply_output_redirections(output, &stdout_redir, &stderr_redir);
            }
            "exit" => exit(0),
            "echo" => {
                let output = builtins::echo_input(&processed_args);
                output::apply_output_redirections(output, &stdout_redir, &stderr_redir);
            }
            "pwd" => {
                let output = builtins::print_working_directory();
                output::apply_output_redirections(output, &stdout_redir, &stderr_redir);
            }
            "type" => {
                let cmd = processed_args.first().map(String::as_str).unwrap_or("");
                let output = builtins::handle_type_command(cmd, &self.paths);
                output::apply_output_redirections(output, &stdout_redir, &stderr_redir);
            }
            _ => execute_command(
                command,
                &self.paths,
                &processed_args,
                &stdout_redir,
                &stderr_redir,
            ),
        }
    }
}