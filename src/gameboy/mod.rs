use std::fs::File;
use std::io::Write;
use crate::{cartridge, cpu, memory};

pub struct GameBoy {
    pub cpu: cpu::Cpu,
    pub memory: memory::Memory,
    log_file: Option<File>,
}

impl GameBoy {
    pub fn new() -> Self {
        Self {
            cpu: cpu::Cpu::default(),
            memory: memory::Memory::default(),
            log_file: None,
        }
    }

    /// Load a ROM file
    pub fn load_rom(&mut self, path: &str) -> std::io::Result<()> {
        let cartridge = cartridge::Cartridge::load(path)?;
        self.memory.load_cartridge(cartridge);
        Ok(())
    }

    /// Enable CPU state logging to a file (gameboy-doctor format)
    pub fn enable_logging(&mut self, path: &str) -> std::io::Result<()> {
        self.log_file = Some(File::create(path)?);
        Ok(())
    }

    /// Initialize to post-boot state (skipping boot ROM for now)
    pub fn power_on(&mut self) {
        // CPU initialized with correct values in Cpu::new()
        // Set initial flag values (from docs: AF = 01B0h for DMG)
        self.cpu.registers.f.z = true;
        self.cpu.registers.f.n = false;
        self.cpu.registers.f.h = true;
        self.cpu.registers.f.c = true;
    }

    /// Execute one instruction and log if enabled
    pub fn step(&mut self) -> u8 {
        // Log CPU state before execution (gameboy-doctor format)
        if self.log_file.is_some() {
            let a = self.cpu.registers.a;
            let f = self.cpu.registers.f.to_u8();
            let b = self.cpu.registers.b;
            let c = self.cpu.registers.c;
            let d = self.cpu.registers.d;
            let e = self.cpu.registers.e;
            let h = self.cpu.registers.h;
            let l = self.cpu.registers.l;
            let sp = self.cpu.sp;
            let pc = self.cpu.pc;

            // Read next 4 bytes at PC for PCMEM
            let pcmem0 = self.memory.read_byte(pc);
            let pcmem1 = self.memory.read_byte(pc.wrapping_add(1));
            let pcmem2 = self.memory.read_byte(pc.wrapping_add(2));
            let pcmem3 = self.memory.read_byte(pc.wrapping_add(3));

            let line = format!(
                "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n",
                a, f, b, c, d, e, h, l, sp, pc, pcmem0, pcmem1, pcmem2, pcmem3
            );

            if let Some(ref mut log) = self.log_file {
                let _ = log.write_all(line.as_bytes());
            }
        }

        // Execute instruction
        self.cpu.execute(&mut self.memory)
    }

    /// Run the emulator for a number of instructions
    pub fn run(&mut self, num_instructions: usize) {
        for _ in 0..num_instructions {
            if self.cpu.halted {
                break;
            }
            self.step();
        }
    }
}

impl Default for GameBoy {
    fn default() -> Self {
        Self::new()
    }
}