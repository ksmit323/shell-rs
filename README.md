# shell-rs

A lightweight command-line shell written in Rust. Supports built-in commands, file operations, custom executable handling, autocompletion, I/O redirection, and proper quoting. Built as an educational project to demonstrate shell fundamentals.

## Features

- Built-in commands: `cd`, `pwd`, `echo`, `type`
- File operations: `cat`
- Custom executable handling
- Path resolution
- Command autocompletion
- I/O redirection (`>`, `>>`, `<`, `|`)
- Proper quote handling (single and double quotes)
- Simple and readable Rust implementation

## Installation

```bash
# Clone the repository
git clone https://github.com/ksmit323/shell-rs.git
cd shell-rs

# Build and run the shell
./run_shell.sh
```

## Usage

```bash
$ pwd
/home/user
$ echo 'Hello World'
Hello World
$ cat 'file.txt'
Contents of file.txt
$ cd ~/documents
```

## Built-in Commands

- `cd [directory]` - Change current directory
- `pwd` - Print working directory
- `echo [text]` - Display a line of text
- `type [command]` - Display command type
- `cat [file]` - Display file contents
- `exit` - Exit the shell

