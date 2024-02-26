use std::io::{self, BufRead, Write};
use std::os::unix::fs::PermissionsExt;
use std::time::UNIX_EPOCH;

use chrono::{DateTime, Utc};

const INVALID_LIST_DIRECTORY_COMMAND: &'static str =
    "Invalid ls command. Only `ls` and `ls -l` are supported";
const INVALID_ECHO_COMMAND: &'static str = "Invalid echo command. Correct usage: `echo <message>`";
const ERROR_FORMATING_SYSTEM_TIME: &'static str =
    "Error while formatting SystemTime to DateTime<Utc>";

struct Emulator {
    writer: io::BufWriter<io::Stdout>,
    reader: io::BufReader<io::Stdin>,
    input_buffer: String,
    path: std::path::PathBuf,
}

impl Emulator {
    fn new() -> Emulator {
        Emulator {
            writer: io::BufWriter::new(io::stdout()),
            reader: io::BufReader::new(io::stdin()),
            input_buffer: String::new(),
            path: std::env::current_dir().unwrap(),
        }
    }

    fn print_prompt(&mut self) {
        self.print_to_stdout("$ ", false);
    }

    fn read_and_process_input(&mut self) {
        self.input_buffer.clear();
        if let Err(err) = self.reader.read_line(&mut self.input_buffer) {
            panic!("Failed to read from stdin: {}", err);
        }
        match self.process_command(&self.input_buffer) {
            Ok(result) => self.print_to_stdout(&result, true),
            Err(err) => println!("Error: {}", err),
        }
    }

    fn process_command(&self, command: &str) -> Result<String, &'static str> {
        match command.trim() {
            "exit" => std::process::exit(0),
            "pwd" => return Ok((self.path.to_str().unwrap()).to_string()),
            cmd if cmd.starts_with("ls") => self.list_directory(command),
            cmd if cmd.starts_with("echo") => self.echo(command),
            _ => Ok(command.trim().to_string()),
        }
    }

    fn echo(&self, command: &str) -> Result<String, &'static str> {
        if command.trim() == "echo" {
            return Ok("".to_string());
        }
        if !command.starts_with("echo ") {
            return Err(INVALID_ECHO_COMMAND);
        }
        Ok(command.trim().strip_prefix("echo ").unwrap().to_string())
    }

    fn list_directory(&self, command: &str) -> Result<String, &'static str> {
        if command.trim() != "ls" && command.trim() != "ls -l" {
            return Err(INVALID_LIST_DIRECTORY_COMMAND);
        }
        let list_format = command.trim() == "ls -l";
        let mut result: String = String::new();
        if list_format {
            result.push_str("Type\tMode\tSize\tModification Time\tName\n");
        }
        for entry in std::fs::read_dir(&self.path).unwrap() {
            let entry = entry.unwrap();
            if list_format {
                let metadata = entry.metadata().unwrap();
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                let file_type = if metadata.is_dir() {
                    "d"
                } else if metadata.is_file() {
                    "f"
                } else {
                    "?"
                };

                let permissions = metadata.permissions();
                let mode = permissions.mode();

                // Format permissions using Unix file mode
                let mode_string = format!("{:04o}", mode & 0o7777);

                let size = metadata.len();

                // Format the DateTime<Utc> to a human-readable string
                let modification_time = metadata.modified().unwrap();
                let datetime: DateTime<Utc> = match modification_time.duration_since(UNIX_EPOCH) {
                    Ok(duration) => (UNIX_EPOCH + duration).into(),
                    Err(_) => {
                        return Err(ERROR_FORMATING_SYSTEM_TIME);
                    }
                };
                let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

                // Append the formatted string to the result
                result.push_str(
                    format!(
                        "{}\t{}\t{}\t{}\t{}",
                        file_type, mode_string, size, formatted_time, file_name_str
                    )
                    .as_str(),
                );
                result.push_str("\n");
            } else {
                result.push_str(&entry.file_name().to_str().unwrap());
                result.push_str("\t");
            }
        }
        Ok(result.trim().to_string())
    }

    fn print_to_stdout(&mut self, output: &str, new_line: bool) {
        let output = if new_line {
            format!("{}\n", output)
        } else {
            output.to_string()
        };
        if let Err(err) = write!(self.writer, "{}", output) {
            panic!("Failed to write to stdout: {}", err);
        }
        if let Err(err) = self.writer.flush() {
            panic!("Failed to flush stdout: {}", err);
        }
    }
}

fn main() {
    let mut emulator = Emulator::new();

    loop {
        emulator.print_prompt();
        emulator.read_and_process_input();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_command() {
        let emulator = Emulator::new();
        let result = emulator.process_command("test_input");
        match result {
            Ok(value) => assert_eq!(value, "test_input"),
            Err(_) => panic!("[test_process_command] expected Ok, got error"),
        }
    }

    #[test]
    fn test_process_command_pwd() {
        let emulator = Emulator::new();
        let result = emulator.process_command("pwd");
        match result {
            Ok(value) => assert_eq!(value, (emulator.path.to_str().unwrap()).to_string()),
            Err(_) => panic!("[test_process_command_pwd] expected Ok, got error"),
        }
    }

    #[test]
    fn test_process_command_ls() {
        use tempfile::tempdir;
        // one line for output + one line for new line
        // `ls -l` adds an extra line for the header.
        let test_cases = [("ls", 1), ("ls -l", 2)];

        for (input, expected) in test_cases.iter() {
            // create a temp directory and add a file
            let temp_dir = tempdir().unwrap();
            // Create a file within the temporary directory
            let file_path = temp_dir.path().join("sample.txt");
            std::fs::File::create(&file_path).unwrap();
            // create an emulator and set the path to the temp directory
            let mut emulator = Emulator::new();
            emulator.path = temp_dir.path().to_path_buf();

            match emulator.process_command(&input) {
                Ok(value) => {
                    let lines: Vec<&str> = value.split('\n').collect();
                    assert_eq!(lines.len(), *expected);
                }
                Err(_) => panic!("[test_process_command_ls] expected Ok, got error"),
            }
            temp_dir.close().unwrap();
        }
    }

    #[test]
    fn test_process_command_echo() {
        let test_cases = [("echo hello", "hello"), ("echo", "")];

        let emulator = Emulator::new();

        for (input, expected) in test_cases.iter() {
            match emulator.process_command(&input) {
                Ok(value) => assert_eq!(value, *expected),
                Err(_) => panic!("[test_process_command_echo] expected Ok, got error"),
            }
        }
    }
}
