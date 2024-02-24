# mini-shell
A bare minimum implementation of a shell written in Rust

## Requirements

### Basic Shell Functionality
- [ ] As a user, I want to run the shell application.
- [ ] As a user, I want to see a command prompt (e.g., `$` or `>`) indicating that the shell is ready to accept commands.
- [ ] As a user, I want to be able to type commands into the shell.

### Command Execution
- [ ] As a user, I want to execute simple commands (e.g., `ls`, `pwd`, `echo`) and see their output.
- [ ] As a user, I want to execute commands with arguments (e.g., `ls -l`, `echo "Hello, world!"`).

### Built-in Commands
- [ ] As a user, I want to implement built-in commands such as `cd`, `exit`, and `help`.
- [ ] As a user, I want to change the current working directory using the `cd` command.

### Input/Output Redirection
- [ ] As a user, I want to redirect the output of a command to a file using the `>` operator (e.g., `ls > output.txt`).
- [ ] As a user, I want to append the output of a command to a file using the `>>` operator (e.g., `echo "text" >> file.txt`).
- [ ] As a user, I want to redirect input from a file to a command using the `<` operator (e.g., `sort < input.txt`).

### Pipeline Commands
- [ ] As a user, I want to support command pipelines using the `|` operator (e.g., `ls | grep .txt`).

### Background Processes
- [ ] As a user, I want to run commands in the background by appending `&` to the command (e.g., `sleep 10 &`).

### Signal Handling
- [ ] As a user, I want the shell to handle signals like `Ctrl+C` (SIGINT) and `Ctrl+Z` (SIGTSTP) appropriately.

### Command History
- [ ] As a user, I want to access a history of previously executed commands using the arrow keys or a `history` command.

### Tab Completion
- [ ] As a user, I want to have tab completion functionality for commands, file paths, and arguments.

### Environment Variables
- [ ] As a user, I want to be able to set and use environment variables within the shell.

### Error Handling
- [ ] As a user, I want informative error messages displayed for invalid commands, syntax errors, etc.

### Scripting Support
- [ ] As a user, I want to execute shell scripts (sequences of commands stored in a file) using the shell.

