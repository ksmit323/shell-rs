use std::fs::OpenOptions;
use std::io::{self, Write};
use crate::redirection::{Redirection, RedirectionMode};

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
}

struct OutputHandler<'a> {
    output: CommandOutput,
    stdout_redir: &'a Option<Redirection>,
    stderr_redir: &'a Option<Redirection>,
}

impl<'a> OutputHandler<'a> {
    fn new(
        output: CommandOutput,
        stdout_redir: &'a Option<Redirection>,
        stderr_redir: &'a Option<Redirection>,
    ) -> Self {
        Self {
            output,
            stdout_redir,
            stderr_redir,
        }
    }

    fn handle_output(self) {
        self.handle_stdout();
        self.handle_stderr();
    }

    fn handle_stdout(&self) {
        match self.stdout_redir {
            Some(redir) => self.redirect_to_file(&self.output.stdout, redir),
            None => self.write_to_stdout(&self.output.stdout),
        }
    }

    fn handle_stderr(&self) {
        match self.stderr_redir {
            Some(redir) => self.redirect_to_file(&self.output.stderr, redir),
            None => self.write_to_stderr(&self.output.stderr),
        }
    }

    fn redirect_to_file(&self, content: &str, redir: &Redirection) {
        match self.open_file(redir) {
            Ok(mut file) => self.write_to_file(&mut file, content, &redir.filename),
            Err(e) => eprintln!("Error opening {}: {}", redir.filename, e),
        }
    }

    fn open_file(&self, redir: &Redirection) -> io::Result<std::fs::File> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(matches!(redir.mode, RedirectionMode::Truncate))
            .append(matches!(redir.mode, RedirectionMode::Append))
            .open(&redir.filename)
    }

    fn write_to_file(&self, file: &mut std::fs::File, content: &str, filename: &str) {
        if let Err(e) = file.write_all(content.as_bytes()) {
            eprintln!("Error writing to {}: {}", filename, e);
        }
    }

    fn write_to_stdout(&self, content: &str) {
        print!("{}", content);
        let _ = io::stdout().flush();
    }

    fn write_to_stderr(&self, content: &str) {
        eprint!("{}", content);
        let _ = io::stderr().flush();
    }
}

pub fn apply_output_redirections(
    output: CommandOutput,
    stdout_redir: &Option<Redirection>,
    stderr_redir: &Option<Redirection>,
) {
    OutputHandler::new(output, stdout_redir, stderr_redir).handle_output();
}