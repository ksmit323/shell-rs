use std::fs::OpenOptions;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::redirection::{Redirection, RedirectionMode};
use crate::utils;

struct CommandExecutor<'a> {
    command: &'a str,
    paths: &'a [String],
    args: &'a [String],
    stdout_redir: &'a Option<Redirection>,
    stderr_redir: &'a Option<Redirection>,
}

impl<'a> CommandExecutor<'a> {
    fn new(
        command: &'a str,
        paths: &'a [String],
        args: &'a [String],
        stdout_redir: &'a Option<Redirection>,
        stderr_redir: &'a Option<Redirection>,
    ) -> Self {
        Self {
            command,
            paths,
            args,
            stdout_redir,
            stderr_redir,
        }
    }

    fn execute(&self) {
        match self.find_command() {
            Some(command_path) => self.run_command(&command_path),
            None => eprintln!("{}: command not found", self.command),
        }
    }

    fn run_command(&self, command_path: &str) {
        let mut cmd = self.create_base_command(command_path);
        
        if let Err(e) = self.setup_redirections(&mut cmd) {
            eprintln!("Redirection error: {}", e);
            return;
        }

        let _ = cmd.status();
    }

    fn create_base_command(&self, command_path: &str) -> Command {
        let mut cmd = Command::new(command_path);
        cmd.arg0(self.command);
        cmd.args(self.args);
        cmd
    }

    fn setup_redirections(&self, cmd: &mut Command) -> io::Result<()> {
        if let Some(redir) = self.stdout_redir {
            let file = self.open_redirection_file(redir)?;
            cmd.stdout(file);
        }

        if let Some(redir) = self.stderr_redir {
            let file = self.open_redirection_file(redir)?;
            cmd.stderr(file);
        }

        Ok(())
    }

    fn open_redirection_file(&self, redir: &Redirection) -> io::Result<std::fs::File> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(matches!(redir.mode, RedirectionMode::Truncate))
            .append(matches!(redir.mode, RedirectionMode::Append))
            .open(&redir.filename)
    }

    fn find_command(&self) -> Option<String> {
        utils::find_command(self.command, self.paths)
    }
}

pub fn execute_command(
    command: &str,
    paths: &[String],
    args: &[String],
    stdout_redir: &Option<Redirection>,
    stderr_redir: &Option<Redirection>,
) {
    CommandExecutor::new(command, paths, args, stdout_redir, stderr_redir).execute();
}