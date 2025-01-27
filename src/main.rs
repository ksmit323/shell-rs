mod autocompletion;
mod builtins;
mod command;
mod output;
mod parser;
mod redirection;
mod shell;
mod utils;

use crate::shell::Shell;

fn main() {
    let mut shell = Shell::new();
    shell.run();
}