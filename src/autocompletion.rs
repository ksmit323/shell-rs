use std::env;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::sync::Mutex;

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result as RustylineResult};

pub struct BuiltInCompleter {
    paths: Vec<String>,
    completion_state: Mutex<CompletionState>,
}

struct CompletionState {
    last_prefix: String,
    last_matches: Vec<String>,
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
            completion_state: Mutex::new(CompletionState {
                last_prefix: String::new(),
                last_matches: Vec::new(),
            }),
        }
    }

    fn get_builtin_completions(&self, prefix: &str) -> Vec<Pair> {
        let mut completions = Vec::new();
        for cmd in ["cd", "echo", "exit", "pwd", "type"] {
            if cmd.starts_with(prefix) {
                completions.push(Pair {
                    display: cmd.to_string(),
                    replacement: format!("{} ", cmd),  // Built-in commands always get a space
                });
            }
        }
        completions
    }

    fn find_executables(&self, prefix: &str) -> Vec<String> {
        let normalized_prefix = prefix.replace('*', "");
        let mut matches = self.paths.iter()
            .filter_map(|path_dir| self.get_dir_entries(path_dir))
            .flat_map(|entries| entries.filter_map(Result::ok))
            .filter_map(|entry| self.process_entry(entry, &normalized_prefix))
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

    fn find_longest_common_prefix(&self, matches: &[String]) -> String {
        if matches.is_empty() {
            return String::new();
        }
        
        let first = &matches[0];
        let mut common_prefix_len = first.len();
        
        for str in matches.iter().skip(1) {
            common_prefix_len = first
                .chars()
                .zip(str.chars())
                .take(common_prefix_len)
                .take_while(|(a, b)| a == b)
                .count();
                
            if common_prefix_len == 0 {
                break;
            }
        }
        
        first[..common_prefix_len].to_string()
    }

    fn handle_completion(&self, line: &str, pos: usize) -> RustylineResult<(usize, Vec<Pair>)> {
        let prefix = &line[..pos];
        
        // First check built-in commands
        let builtin_completions = self.get_builtin_completions(prefix);
        if !builtin_completions.is_empty() {
            return Ok((0, builtin_completions));
        }
        
        // Get matches for the current prefix
        let matches = self.find_executables(prefix);
        
        // No matches
        if matches.is_empty() {
            print!("\x07");  // Bell sound
            io::stdout().flush().unwrap();
            return Ok((0, vec![]));
        }
        
        // Find the longest common prefix
        let common_prefix = self.find_longest_common_prefix(&matches);
        
        // Single exact match
        if matches.len() == 1 {
            return Ok((0, vec![Pair {
                display: matches[0].clone(),
                replacement: format!("{} ", matches[0]),  // Add space for exact match
            }]));
        }
        
        // Multiple matches with common prefix
        if common_prefix.len() > prefix.len() {
            // Update completion state
            if let Ok(mut state) = self.completion_state.lock() {
                state.last_matches = matches.clone();
                state.last_prefix = common_prefix.clone();
            }
            
            // For partial matches, don't add a space
            return Ok((0, vec![Pair {
                display: common_prefix.clone(),
                replacement: common_prefix,  // No space for partial completion
            }]));
        }
        
        // Show all matches if no further completion possible
        println!("\n{}", matches.join("  "));
        print!("$ {}", prefix);
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
        // Skip if not first token
        if line[..pos].contains(' ') {
            return Ok((0, vec![]));
        }

        self.handle_completion(line, pos)
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