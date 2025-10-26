pub mod cartridge;
pub mod cpu;
mod gameboy;
pub mod memory;
mod timer;
mod args;

use crate::gameboy::GameBoy;
use clap::Parser;
use crate::args::{GameboyArgs, RunCommand, RunType, TestCommand};

fn main() {
    let args = GameboyArgs::parse();
    let mut game = GameBoy::new();

    match args.run_type {
        RunType::Run(RunCommand { rom }) => {
            if let Err(e) = game.load_rom(&rom) {
                eprintln!("Error loading ROM: {e}");
                std::process::exit(1);
            }
        },
        RunType::Test(TestCommand { rom, log }) => {
            if let Err(e) = game.load_rom(&rom) {
                eprintln!("Error loading ROM: {e}");
                std::process::exit(1);
            }

            if let Err(e) = game.enable_logging(&log) {
                eprintln!("Error creating log file: {e}");
                std::process::exit(1);
            }
            println!("Logging enabled to: {log}");
        }
    }

    game.power_on();

    // Run for a large number of instructions (or until HALT)
    // For testing with gameboy-doctor, you typically want to run until
    // a specific point or until HALT
    println!("Running emulator...");
    game.run(1_000_000); // Run for 1 million instructions or until HALT

    println!("Emulator stopped. CPU halted: {}", game.cpu.halted);
}

