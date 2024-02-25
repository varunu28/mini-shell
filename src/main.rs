use std::fmt::Error;
use std::io::{self, BufRead, Write};

struct Emulator {
    writer: io::BufWriter<io::Stdout>,
    reader: io::BufReader<io::Stdin>,
    input_buffer: String,
}

impl Emulator {
    fn new() -> Emulator {
        Emulator {
            writer: io::BufWriter::new(io::stdout()),
            reader: io::BufReader::new(io::stdin()),
            input_buffer: String::new(),
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
        Ok(command.to_string())
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
}
