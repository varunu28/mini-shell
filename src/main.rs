mod emulator;
use emulator::Emulator;

fn main() {
    let mut emulator = Emulator::new();

    loop {
        emulator.print_prompt();
        emulator.read_and_process_input();
    }
}
