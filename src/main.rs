use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::time::UNIX_EPOCH;

use chrono::{DateTime, Utc};

const HISTORY_SIZE: usize = 10;

struct Emulator {
    writer: io::BufWriter<io::Stdout>,
    reader: io::BufReader<io::Stdin>,
    path: std::path::PathBuf,
    history: VecDeque<String>,
}

impl Emulator {
    fn new() -> Emulator {
        Emulator {
            writer: io::BufWriter::new(io::stdout()),
            reader: io::BufReader::new(io::stdin()),
            path: std::env::current_dir().unwrap(),
            history: VecDeque::with_capacity(HISTORY_SIZE),
        }
    }

    fn clone(&self) -> Self {
        Emulator {
            writer: io::BufWriter::new(io::stdout()),
            reader: io::BufReader::new(io::stdin()),
            path: self.path.clone(),
            history: self.history.clone(),
        }
    }

    fn print_prompt(&mut self) {
        self.print_to_stdout("$ ", false);
    }

    fn read_and_process_input(&mut self) {
        let mut input_buffer = String::new();
        if let Err(err) = self.reader.read_line(&mut input_buffer) {
            panic!("Failed to read from stdin: {}", err);
        }

        // TODO: Fix formatting of new line
        if input_buffer.trim().ends_with(" &") {
            let input_buffer = input_buffer.clone();
            let input_buffer_trimmed = input_buffer.trim().trim_end_matches(" &").to_string(); // Remove the trailing `&`
            let mut emulator = self.clone(); // Clone the emulator to be used in the spawned thread
            std::thread::spawn(
                move || match emulator.process_command(&input_buffer_trimmed) {
                    Ok(result) => emulator.print_to_stdout(&result, true),
                    Err(err) => {
                        emulator.print_to_stdout(format!("mini-shell: {}", err).as_str(), true)
                    }
                },
            );
            return;
        }

        match self.process_command(&input_buffer) {
            Ok(result) => self.print_to_stdout(&result, true),
            Err(err) => self.print_to_stdout(format!("mini-shell: {}", err).as_str(), true),
        }
    }

    fn process_command(&mut self, command: &str) -> Result<String, &'static str> {
        self.record_history(command);
        match command.trim() {
            "exit" => std::process::exit(0),
            "history" => self.history(),
            "pwd" => Ok((self.path.to_str().unwrap()).to_string()),
            cmd if cmd.contains(">") => self.process_command_with_output_redirection(command),
            cmd if cmd.contains("<") => self.process_command_with_input_redirection(command),
            cmd if cmd.starts_with("ls") => self.list_directory(command),
            cmd if cmd.starts_with("echo") => self.echo(command),
            cmd if cmd.starts_with("cd") => self.change_directory(command),
            cmd if cmd.starts_with("sleep") => self.sleep(command),
            cmd if cmd.starts_with("cat") => self.cat(command),
            cmd if cmd.starts_with("rmdir") => self.rm(command, true),
            cmd if cmd.starts_with("rm") => self.rm(command, false),
            _ => Err("mini-shell: command not found"),
        }
    }

    fn process_command_with_input_redirection(
        &mut self,
        command: &str,
    ) -> Result<String, &'static str> {
        let count = command.chars().filter(|&c| c == '<').count();
        if count != 1 {
            return Err("Invalid command. Correct usage `command < file`");
        }
        let (operation, file_name) = match command.trim().split_once('<') {
            Some((op, file)) => (op.trim(), file.trim()),
            None => ("", ""),
        };
        if operation.trim().is_empty() || file_name.trim().is_empty() {
            return Err("Invalid command. Correct usage `command < file`");
        }
        let file = std::fs::OpenOptions::new().read(true).open(file_name);
        match file {
            Ok(mut file) => {
                let mut buffer = String::new();
                if let Err(_) = file.read_to_string(&mut buffer) {
                    return Err("Failed to read from file");
                }
                match operation.trim() {
                    "sort" => self.process_sort_command(buffer.as_str()),
                    _ => Err("mini-shell: command not found"),
                }
            }
            Err(_) => return Err("Failed to open file"),
        }
    }

    fn process_sort_command(&mut self, buffer: &str) -> Result<String, &'static str> {
        let mut lines: Vec<&str> = buffer.split('\n').collect();
        lines.sort();
        Ok(lines.join("\n"))
    }

    fn process_command_with_output_redirection(
        &mut self,
        command: &str,
    ) -> Result<String, &'static str> {
        let count = command.chars().filter(|&c| c == '>').count();
        if count > 2 {
            return Err("Invalid command. Correct usage `command > file` OR `command >> file`");
        }
        let (operation, file_name) = match count {
            1 => {
                let parts = command.trim().split_once('>');
                if let Some((op, file)) = parts {
                    (op.trim(), file.trim())
                } else {
                    // Handle case when '>' is missing
                    ("", "")
                }
            }
            2 => {
                let parts = command.trim().split_once(">>");
                if let Some((op, file)) = parts {
                    (op.trim(), file.trim())
                } else {
                    ("", "")
                }
            }
            _ => ("", ""),
        };
        if operation.is_empty() || file_name.is_empty() {
            return Err("Invalid command. Correct usage `command > file` OR `command >> file`");
        }
        match self.process_command(operation) {
            Ok(result) => {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(count == 2)
                    .write(true)
                    .open(file_name);
                match file {
                    Ok(mut file) => {
                        if let Err(_) = file.write_all((format!("{}\n", result)).as_bytes()) {
                            return Err("Failed to write to file");
                        }
                    }
                    Err(_) => return Err("Failed to open file"),
                }
            }
            Err(err) => return Err(err),
        };
        Ok("".to_string())
    }

    fn list_directory(&mut self, command: &str) -> Result<String, &'static str> {
        if command.trim() != "ls" && command.trim() != "ls -l" {
            return Err("Invalid ls command. Only `ls` and `ls -l` are supported");
        }
        let list_format = command.trim() == "ls -l";
        if list_format {
            return self.list_directory_with_args();
        } else {
            return self.list_directory_simple();
        }
    }

    fn list_directory_simple(&mut self) -> Result<String, &'static str> {
        let mut result: String = String::new();
        for entry in std::fs::read_dir(&self.path).unwrap() {
            let entry = entry.unwrap();
            result.push_str(&entry.file_name().to_str().unwrap());
            result.push_str("\t");
        }
        Ok(result.trim().to_string())
    }

    fn list_directory_with_args(&mut self) -> Result<String, &'static str> {
        let mut result: String = String::new();
        result.push_str("Type\tMode\tSize\tModification Time\tName\n");
        for entry in std::fs::read_dir(&self.path).unwrap() {
            let entry = entry.unwrap();
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
                    return Err("Error while formatting SystemTime to DateTime<Utc>");
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
        }
        Ok(result.trim().to_string())
    }

    fn echo(&mut self, command: &str) -> Result<String, &'static str> {
        if command.trim() == "echo" {
            return Ok("".to_string());
        }
        if !command.starts_with("echo ") {
            return Err("Invalid echo command. Correct usage: `echo <message>`");
        }
        Ok(command.trim().strip_prefix("echo ").unwrap().to_string())
    }

    fn change_directory(&mut self, command: &str) -> Result<String, &'static str> {
        let new_directory_path = command.trim().strip_prefix("cd ").unwrap();
        let new_path = std::path::Path::new(new_directory_path);
        if !new_path.exists() {
            return Err("Path does not exist");
        }
        if !new_path.is_dir() {
            return Err("Path is not a directory");
        }
        self.path = self.path.join(new_path);
        Ok("".to_string())
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

    fn record_history(&mut self, command: &str) {
        if command.trim().is_empty() {
            return;
        }
        if self.history.len() == HISTORY_SIZE {
            self.history.pop_front();
        }
        self.history.push_back(command.trim().to_string());
    }

    fn history(&mut self) -> Result<String, &'static str> {
        Ok(self
            .history
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("\n"))
    }

    fn sleep(&mut self, command: &str) -> Result<String, &'static str> {
        if command.trim() == "sleep" {
            return Err("correct usage: `sleep <duration>`");
        }
        let duration = command.trim().strip_prefix("sleep ").unwrap();
        match duration.parse::<u64>() {
            Ok(value) => std::thread::sleep(std::time::Duration::from_secs(value)),
            Err(_) => return Err("sleep duration should be a positive integer"),
        }
        Ok("".to_string())
    }

    fn cat(&mut self, command: &str) -> Result<String, &'static str> {
        if command.trim() == "cat" {
            return Err("correct usage: `cat <file>`");
        }
        let file_name = command.trim().strip_prefix("cat ").unwrap();
        let file_path = self.path.join(file_name);
        let file = std::fs::OpenOptions::new().read(true).open(file_path);
        match file {
            Ok(mut file) => {
                let mut buffer = String::new();
                if let Err(_) = file.read_to_string(&mut buffer) {
                    return Err("Failed to read from file");
                }
                Ok(buffer)
            }
            Err(_) => return Err("Failed to open file"),
        }
    }

    fn rm(&mut self, command: &str, is_directory: bool) -> Result<String, &'static str> {
        if command.trim() == "rm" || command.trim() == "rmdir" {
            return Err("correct usage: `rm <file> OR rmdir <directory>`");
        }
        let prefix = if is_directory { "rmdir " } else { "rm " };
        let file_name = command.trim().strip_prefix(prefix).unwrap();
        let file_path = self.path.join(file_name);
        let file = File::open(file_path);
        match file {
            Ok(_) => {
                if file.unwrap().metadata().unwrap().is_dir() && !is_directory {
                    return Err("rm: cannot remove directory. Use `rmdir` instead");
                }
                let file_path = self.path.join(file_name);
                if is_directory {
                    std::fs::remove_dir(file_path).unwrap();
                } else {
                    std::fs::remove_file(file_path).unwrap();
                }
            }
            Err(_) => return Err("Failed to open file"),
        }
        Ok("".to_string())
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
        let mut emulator = Emulator::new();
        let result = emulator.process_command("test_input");
        match result {
            Ok(_) => panic!("expected error, got Ok"),
            Err(err) => assert_eq!(err, "mini-shell: command not found"),
        }
    }

    #[test]
    fn test_process_command_pwd() {
        let mut emulator = Emulator::new();
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

        let mut emulator = Emulator::new();

        for (input, expected) in test_cases.iter() {
            match emulator.process_command(&input) {
                Ok(value) => assert_eq!(value, *expected),
                Err(_) => panic!("[test_process_command_echo] expected Ok, got error"),
            }
        }
    }

    #[test]
    fn test_process_command_cd() {
        use tempfile::tempdir;
        let temp_dir = tempdir().unwrap();
        let mut emulator = Emulator::new();
        emulator.path = temp_dir.path().to_path_buf();

        let result = emulator.process_command("cd /tmp");
        match result {
            Ok(_) => assert_eq!(emulator.path, std::path::PathBuf::from("/tmp")),
            Err(_) => panic!("[test_process_command_cd] expected Ok, got error"),
        }
        temp_dir.close().unwrap();
    }

    #[test]
    fn test_process_command_history() {
        let mut emulator = Emulator::new();
        // run few commands
        let _ignored = emulator.process_command("ls");
        let _ignored = emulator.process_command("pwd");
        let _ignored = emulator.process_command("echo");

        let expected_result = "ls\npwd\necho\nhistory";

        // verify history
        match emulator.process_command("history") {
            Ok(value) => assert_eq!(value, expected_result),
            Err(_) => panic!("[test_process_command_history] expected Ok, got error"),
        }
    }

    #[test]
    fn test_process_command_cat() {
        use tempfile::tempdir;
        let temp_dir = tempdir().unwrap();
        let mut emulator = Emulator::new();
        emulator.path = temp_dir.path().to_path_buf();

        let file_path = temp_dir.path().join("sample.txt");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&file_path);
        match file {
            Ok(mut file) => {
                if let Err(_) = file.write_all("hello".as_bytes()) {
                    panic!("Failed to write to file");
                }
            }
            Err(_) => panic!("Failed to open file"),
        }

        match emulator.process_command("cat sample.txt") {
            Ok(value) => assert_eq!(value, "hello"),
            Err(_) => panic!("[test_process_command_cat] expected Ok, got error"),
        }
        temp_dir.close().unwrap();
    }

    #[test]
    fn test_process_command_rm_file() {
        use tempfile::tempdir;
        let temp_dir = tempdir().unwrap();
        let mut emulator = Emulator::new();
        emulator.path = temp_dir.path().to_path_buf();

        let file_name = "sample.txt";
        let file_path = temp_dir.path().join(file_name);
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&file_path)
            .unwrap();

        match emulator.process_command(format!("rm {}", file_name).as_str()) {
            Ok(value) => assert_eq!(value, ""),
            Err(_) => panic!("[test_process_command_rm_file] expected Ok, got error"),
        }

        match emulator.process_command("pwd") {
            Ok(value) => assert!(!value.contains(file_name)),
            Err(_) => panic!("[test_process_command_rm_file] expected Ok, got error"),
        }
    }

    #[test]
    fn test_process_command_rmdir() {
        use tempfile::tempdir;
        let temp_dir = tempdir().unwrap();
        let mut emulator = Emulator::new();
        emulator.path = temp_dir.path().to_path_buf();

        let dir_name = "sample_dir";
        let dir_path = temp_dir.path().join(dir_name);
        std::fs::create_dir(&dir_path).unwrap();

        match emulator.process_command(format!("rmdir {}", dir_name).as_str()) {
            Ok(value) => assert_eq!(value, ""),
            Err(_) => panic!("[test_process_command_rmdir] expected Ok, got error"),
        }

        match emulator.process_command("pwd") {
            Ok(value) => assert!(!value.contains(dir_name)),
            Err(_) => panic!("[test_process_command_rmdir] expected Ok, got error"),
        }
    }
}
