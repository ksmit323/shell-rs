use std::env;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result as RustylineResult};

pub struct BuiltInCompleter {
    paths: Vec<String>,
    last_prefix: String,
    tab_count: usize,
}

impl BuiltInCompleter {
    pub fn new() -> Self {
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
        let mut matches = self.paths.iter()
            .filter_map(|path_dir| self.get_dir_entries(path_dir))
            .flat_map(|entries| entries.filter_map(Result::ok))  // Handle Result<DirEntry>
            .filter_map(|entry| self.process_entry(entry, prefix))
            .collect::<Vec<_>>();
            
        matches.sort();
        matches
    }

    fn get_dir_entries(&self, path_dir: &str) -> Option<std::fs::ReadDir> {
        std::fs::read_dir(path_dir).ok()
    }

    fn process_entry(&self, entry: std::fs::DirEntry, prefix: &str) -> Option<String> {
        let file_name = entry.file_name().into_string().ok()?;
        if !file_name.starts_with(prefix) {
            return None;
        }

        let metadata = entry.metadata().ok()?;
        let is_executable = metadata.permissions().mode() & 0o111 != 0;

        if is_executable {
            Some(file_name)
        } else {
            None
        }
    }

    fn get_builtin_completions(&self, prefix: &str) -> Vec<Pair> {
        let mut completions = Vec::new();
        for cmd in ["echo", "exit"] {
            if cmd.starts_with(prefix) {
                completions.push(self.create_completion(cmd));
            }
        }
        completions
    }

    fn create_completion(&self, cmd: &str) -> Pair {
        Pair {
            display: cmd.to_string(),
            replacement: format!("{} ", cmd),
        }
    }

    fn handle_no_matches(&self) -> RustylineResult<(usize, Vec<Pair>)> {
        print!("\x07");
        io::stdout().flush().unwrap();
        Ok((0, vec![]))
    }

    fn handle_single_executable(&self, exe: String) -> Vec<Pair> {
        vec![self.create_completion(&exe)]
    }

    fn handle_multiple_executables(&self, exe_matches: Vec<String>, prefix: &str) -> RustylineResult<(usize, Vec<Pair>)> {
        println!("\n{}", exe_matches.join("  "));
        print!("\x07$ {}", prefix);
        io::stdout().flush().unwrap();
        Ok((0, vec![]))
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
        let prefix = &line[..pos];

        // Skip if not first token
        if prefix.contains(' ') {
            return Ok((0, vec![]));
        }

        // Get completions from both sources
        let mut completions = self.get_builtin_completions(prefix);
        let mut exe_matches = self.find_executables(prefix);

        // Handle no matches
        if exe_matches.is_empty() && completions.is_empty() {
            return self.handle_no_matches();
        }

        // Handle single executable match
        if exe_matches.len() == 1 {
            completions.extend(self.handle_single_executable(exe_matches.remove(0)));
        } 
        // Handle multiple executable matches
        else if exe_matches.len() > 1 && completions.is_empty() {
            return self.handle_multiple_executables(exe_matches, prefix);
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