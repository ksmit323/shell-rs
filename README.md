# shell-rs

A lightweight POSIX-style command-line shell implementation in Rust with modern features and safety guarantees.

## Features

### Core Functionality
- Command execution with PATH resolution
- Built-in commands:
  - `cd`: Directory navigation with tilde expansion
  - `pwd`: Working directory display
  - `echo`: Argument expansion with quote handling
  - `type`: Command type inspection (builtins vs external)
  - `exit`: Shell termination

### Advanced Functionality
- I/O Redirection:
  - Input redirection (`<`)
  - Output truncation (`>`, `1>`)
  - Output appending (`>>`, `1>>`)
  - Error stream redirection (`2>`, `2>>`)
- Pipeline support (`|`)
- Quoting mechanisms:
  - Single quotes (literal strings)
  - Double quotes (with escape sequence support)
  - Backslash escaping
  
  ### Autocompletion System
  - **Multi-stage prefix completion**:
    - Handles nested executable names with common prefixes
    - Dynamically extends completion based on longest common prefix (LCP)
    - Supports iterative refinement through subsequent tab presses
  - **Advanced matching logic**:
    - Prioritizes exact matches when available
    - Automatically completes to maximally unambiguous prefixes
    - Falls back to list display for disjoint matches
  - **Path-aware resolution**:
    - Respects PATH directory order precedence
    - Handles executables across multiple PATH components
    - Maintains POSIX-style executable discovery rules

### Safety & Reliability
- Memory-safe implementation leveraging Rust's ownership model
- Graceful error handling for filesystem operations
- Proper signal handling for Ctrl-C/Ctrl-D
- Configurable readline interface

## Installation

### Prerequisites
- Rust 1.70+ toolchain
- Cargo package manager
- Linux/Unix-like environment

```bash
# Clone the repository
git clone https://github.com/ksmit323/shell-rs.git
cd shell-rs

# Build and run the shell
./run_shell.sh
```

## Usage

### Basic Operations
```bash
$ echo "Hello World" > output.txt
$ cat < input.txt | wc -l
$ ls -l | grep Cargo 2> errors.log
```

### Redirection Examples
```bash
# Combined output/error redirection
$ command 1> output.log 2>&1

# Append mode
$ date >> timestamps.log

# Error stream redirection
$ compile 2> build_errors.txt
```

### Quoting Rules
```bash
$ echo 'Single quotes $preserve literals'
$ echo "Double quotes allow $VARIABLE expansions"
$ echo Escaping\ special\ characters
```

## Architecture

### Key Components
- Parser: Handles tokenization with quoted string awareness
- Command Executor: Manages process forking/execution
- Redirection Engine: Implements file descriptor management
- Autocompletion: Integrated with system PATH resolution 

### Built-in Commands

- `cd [directory]` - Change current directory
- `pwd` - Print working directory
- `echo [text]` - Display a line of text
- `type [command]` - Display command type
- `cat [file]` - Display file contents
- `exit` - Exit the shell

### Dependencies
- rustyline for line editing features
- Standard library POSIX API bindings