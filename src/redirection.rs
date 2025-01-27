#[derive(Debug, PartialEq)]
pub enum RedirectionType {
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub enum RedirectionMode {
    Truncate,
    Append,
}

#[derive(Debug)]
pub struct Redirection {
    pub ty: RedirectionType,
    pub mode: RedirectionMode,
    pub filename: String,
}

pub fn process_redirections(
    args: Vec<String>,
) -> (Vec<String>, Option<Redirection>, Option<Redirection>) {
    let mut processed_args = Vec::new();
    let mut stdout_redir = None;
    let mut stderr_redir = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            ">" | "1>" => parse_redirection_operator(
                &mut stdout_redir,
                &args,
                &mut i,
                RedirectionType::Stdout,
                RedirectionMode::Truncate,
            ),
            ">>" | "1>>" => parse_redirection_operator(
                &mut stdout_redir,
                &args,
                &mut i,
                RedirectionType::Stdout,
                RedirectionMode::Append,
            ),
            "2>" => parse_redirection_operator(
                &mut stderr_redir,
                &args,
                &mut i,
                RedirectionType::Stderr,
                RedirectionMode::Truncate,
            ),
            "2>>" => parse_redirection_operator(
                &mut stderr_redir,
                &args,
                &mut i,
                RedirectionType::Stderr,
                RedirectionMode::Append,
            ),
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
        *i += 2;
    } else {
        target.as_mut().map(|r| r.filename.push_str(&args[*i]));
        *i += 1;
    }
}