use std::fmt::Error;
use std::io::{self, BufRead, Write};

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
        self.print_to_stdout("$ ");
    }

    fn read_and_process_input(&mut self) {
        self.input_buffer.clear();
        if let Err(err) = self.reader.read_line(&mut self.input_buffer) {
            panic!("Failed to read from stdin: {}", err);
        }
        match self.process_command(&self.input_buffer) {
            Ok(result) => self.print_to_stdout(&result),
            Err(err) => println!("Error: {}", err),
        }
    }

    fn process_command(&self, command: &str) -> Result<String, Error> {
        match command.trim() {
            "exit" => std::process::exit(0),
            "pwd" => return Ok((self.path.to_str().unwrap()).to_string() + "\n"),
            cmd if cmd.starts_with("ls") => self.list_directory(),
            cmd if cmd.starts_with("echo") => self.echo(command),
            _ => Ok(command.to_string()),
        }
    }

    fn echo(&self, command: &str) -> Result<String, Error> {
        if command.trim() == "echo" {
            return Ok("\n".to_string());
        }
        Ok(command.to_string())
    }

    fn list_directory(&self) -> Result<String, Error> {
        let mut result = String::new();
        for entry in std::fs::read_dir(&self.path).unwrap() {
            let entry = entry.unwrap();
            result.push_str(&entry.file_name().to_str().unwrap());
            result.push_str("\t");
        }
        result.push_str("\n");
        Ok(result)
    }

    fn print_to_stdout(&mut self, output: &str) {
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
            Ok(value) => assert_eq!(value, (emulator.path.to_str().unwrap()).to_string() + "\n"),
            Err(_) => panic!("[test_process_command_pwd] expected Ok, got error"),
        }
    }

    #[test]
    fn test_process_command_ls() {
        // create a temp directory and add a file
        use tempfile::tempdir;
        let temp_dir = tempdir().unwrap();
        // Create a file within the temporary directory
        let file_path = temp_dir.path().join("sample.txt");
        std::fs::File::create(&file_path).unwrap();

        // create an emulator and set the path to the temp directory
        let mut emulator = Emulator::new();
        emulator.path = temp_dir.path().to_path_buf();

        match emulator.process_command("ls") {
            Ok(value) => assert_eq!(value, "sample.txt\t\n"),
            Err(_) => panic!("[test_process_command_ls] expected Ok, got error"),
        }
    }

    #[test]
    fn test_process_command_echo() {
        let emulator = Emulator::new();
        let result = emulator.process_command("echo");
        match result {
            Ok(value) => assert_eq!(value, "\n"),
            Err(_) => panic!("[test_process_command_echo] expected Ok, got error"),
        }
    }
}
