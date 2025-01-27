#[derive(Default)]
struct Parser {
    args: Vec<String>,
    current_arg: String,
    in_single: bool,
    in_double: bool,
    escape_next: bool,
}

impl Parser {
    fn new() -> Self {
        Self::default()
    }

    fn parse(mut self, input: &str) -> Vec<String> {
        for c in input.chars() {
            self.process_char(c);
        }
        self.finish_parsing()
    }

    fn process_char(&mut self, c: char) {
        if self.escape_next {
            self.handle_escaped_char(c);
        } else if c == '\\' {
            self.handle_backslash();
        } else {
            self.handle_regular_char(c);
        }
    }

    fn handle_escaped_char(&mut self, c: char) {
        if self.in_double {
            match c {
                '\\' | '"' | '$' | '`' | '\n' => self.current_arg.push(c),
                _ => {
                    self.current_arg.push('\\');
                    self.current_arg.push(c);
                }
            }
        } else {
            self.current_arg.push(c);
        }
        self.escape_next = false;
    }

    fn handle_backslash(&mut self) {
        if self.in_double || !self.in_single {
            self.escape_next = true;
        } else {
            self.current_arg.push('\\');
        }
    }

    fn handle_regular_char(&mut self, c: char) {
        match c {
            '\'' => self.handle_single_quote(),
            '"' => self.handle_double_quote(),
            ' ' | '\t' | '\n' => self.handle_whitespace(),
            _ => self.current_arg.push(c),
        }
    }

    fn handle_single_quote(&mut self) {
        if !self.in_double {
            self.in_single = !self.in_single;
        } else {
            self.current_arg.push('\'');
        }
    }

    fn handle_double_quote(&mut self) {
        if !self.in_single {
            self.in_double = !self.in_double;
        } else {
            self.current_arg.push('"');
        }
    }

    fn handle_whitespace(&mut self) {
        if !self.in_single && !self.in_double {
            if !self.current_arg.is_empty() {
                self.args.push(std::mem::take(&mut self.current_arg));
            }
        } else {
            // If we're inside quotes, preserve the whitespace
            self.current_arg.push(' ');
        }
    }

    fn finish_parsing(mut self) -> Vec<String> {
        if !self.current_arg.is_empty() {
            self.args.push(self.current_arg);
        }
        self.args
    }
}

pub fn parse_arguments(input: &str) -> Vec<String> {
    let parser = Parser::new();
    parser.parse(input)
}