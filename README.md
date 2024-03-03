# mini-shell 
A bare minimum implementation of a shell written in Rust

## Requirements

### Basic Shell Functionality
- [X] As a user, I want to run the shell application.
- [X] As a user, I want to see a command prompt (e.g., `$` or `>`) indicating that the shell is ready to accept commands.
- [X] As a user, I want to be able to type commands into the shell.

### Command Execution
- [X] As a user, I want to execute simple commands (e.g., `ls`, `pwd`, `echo`) and see their output.
- [X] As a user, I want to execute commands with arguments (e.g., `ls -l`, `echo "Hello, world!"`).

### Built-in Commands
- [X] As a user, I want to implement built-in commands such as `cd`, `exit`, and `help`.
- [X] As a user, I want to change the current working directory using the `cd` command.
- [X] As a user, I want to sleep using the `sleep 10` command.

### Input/Output Redirection
- [X] As a user, I want to redirect the output of a command to a file using the `>` operator (e.g., `ls > output.txt`).
- [X] As a user, I want to append the output of a command to a file using the `>>` operator (e.g., `echo "text" >> file.txt`).
- [X] As a user, I want to redirect input from a file to a command using the `<` operator (e.g., `sort < input.txt`).
- [X] As a user, I want to concatenate file & display its contents using `cat` operator (e.g., `cat file.txt`).
- [X] As a user, I want to create file using `touch` operator (e.g., `touch file.txt`).
- [X] As a user, I want to create directory using `mkdir` operator (e.g., `mkdir dir`).
- [X] As a user, I want to remove file using `rm` operator (e.g., `rm file.txt`).
- [X] As a user, I want to remove directory using `rmdir` operator (e.g., `rmdir dir`).

### Pipeline Commands
- [ ] As a user, I want to support command pipelines using the `|` operator (e.g., `ls | grep .txt`).

### Background Processes
- [X] As a user, I want to run commands in the background by appending `&` to the command (e.g., `sleep 10 &`).

### Signal Handling
- [ ] As a user, I want the shell to handle signals like `Ctrl+C` (SIGINT) and `Ctrl+Z` (SIGTSTP) appropriately.

### Command History
- [X] As a user, I want to access a history of previously executed commands using the arrow keys or a `history` command.

### Tab Completion
- [ ] As a user, I want to have tab completion functionality for commands, file paths, and arguments.

### Environment Variables
- [ ] As a user, I want to be able to set and use environment variables within the shell.

### Error Handling
- [X] As a user, I want informative error messages displayed for invalid commands, syntax errors, etc.

### Scripting Support
- [ ] As a user, I want to execute shell scripts (sequences of commands stored in a file) using the shell.

