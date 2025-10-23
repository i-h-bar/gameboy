use crate::{cartridge, cpu, memory};
use std::fs::File;
use std::io::Write;

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

    pub fn step(&mut self) {
        // Log CPU state before execution (gameboy-doctor format)
        #[cfg(test)]
        self.log();

        // Execute instruction
        let cycles = self.cpu.execute(&mut self.memory);
        let timer_interrupt = self.memory.timer.tick(cycles);
        if timer_interrupt {
            let if_register = self.memory.read_byte(0xFF0F);
            self.memory.write_byte(0xFF0F, if_register | 0x04);
            // TODO: Implement interrupt system
        }
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

    #[allow(clippy::many_single_char_names)]
    #[cfg(test)]
    fn log(&mut self) {
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
                "A:{a:02X} F:{f:02X} B:{b:02X} C:{c:02X} D:{d:02X} E:{e:02X} H:{h:02X} L:{l:02X} SP:{sp:04X} PC:{pc:04X} PCMEM:{pcmem0:02X},{pcmem1:02X},{pcmem2:02X},{pcmem3:02X}\n",
            );

            if let Some(ref mut log) = self.log_file {
                let _ = log.write_all(line.as_bytes());
            }
        }
    }
}

impl Default for GameBoy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_interrupt_sets_if_flag() {
        let mut gb = GameBoy::new();

        // Setup timer to overflow quickly
        // Frequency 01 = 16 cycles, TIMA = 0xFF, TMA = 0x00
        gb.memory.write_byte(0xFF07, 0x05); // Enable timer, frequency 16
        gb.memory.write_byte(0xFF05, 0xFF); // TIMA = 0xFF (will overflow after 16 cycles)
        gb.memory.write_byte(0xFF06, 0x00); // TMA = 0x00

        // Clear IF register
        gb.memory.write_byte(0xFF0F, 0x00);

        // Write a NOP instruction at PC (0x00 opcode, 4 cycles)
        gb.memory.data[0] = 0x00; // NOP

        // Execute 4 NOPs = 16 cycles total, causing timer overflow
        for _ in 0..4 {
            gb.step();
        }

        // Check that bit 2 of IF register is set
        let if_register = gb.memory.read_byte(0xFF0F);
        assert_eq!(
            if_register & 0x04,
            0x04,
            "Bit 2 of IF register (0xFF0F) should be set after timer overflow"
        );
    }

    #[test]
    fn timer_interrupt_preserves_other_if_bits() {
        let mut gb = GameBoy::new();

        // Setup timer to overflow
        gb.memory.write_byte(0xFF07, 0x05);
        gb.memory.write_byte(0xFF05, 0xFF);
        gb.memory.write_byte(0xFF06, 0x00);

        // Set other interrupt flags (bits 0, 1, 3, 4)
        gb.memory.write_byte(0xFF0F, 0x1B); // 0b00011011 (all except timer)

        // Execute instructions to trigger timer overflow
        gb.memory.data[0] = 0x00; // NOP
        for _ in 0..4 {
            gb.step();
        }

        // Check that bit 2 is now set AND other bits are preserved
        let if_register = gb.memory.read_byte(0xFF0F);
        assert_eq!(
            if_register,
            0x1F, // 0b00011111 (all 5 interrupt bits set)
            "Timer interrupt should set bit 2 while preserving other IF bits"
        );
    }

    #[test]
    fn no_timer_interrupt_without_overflow() {
        let mut gb = GameBoy::new();

        // Setup timer but don't overflow
        gb.memory.write_byte(0xFF07, 0x05); // Enable timer, frequency 16
        gb.memory.write_byte(0xFF05, 0x00); // TIMA = 0x00 (won't overflow)
        gb.memory.write_byte(0xFF06, 0x00);

        // Clear IF register
        gb.memory.write_byte(0xFF0F, 0x00);

        // Execute instructions (not enough cycles to overflow)
        gb.memory.data[0] = 0x00; // NOP
        for _ in 0..2 {
            // Only 8 cycles, need 16 for overflow
            gb.step();
        }

        // Check that IF register is still 0
        let if_register = gb.memory.read_byte(0xFF0F);
        assert_eq!(
            if_register, 0x00,
            "IF register should remain 0 when timer doesn't overflow"
        );
    }

    #[test]
    fn timer_interrupt_only_sets_bit_2() {
        let mut gb = GameBoy::new();

        // Setup timer to overflow
        gb.memory.write_byte(0xFF07, 0x05);
        gb.memory.write_byte(0xFF05, 0xFF);
        gb.memory.write_byte(0xFF06, 0x00);

        // Clear IF register
        gb.memory.write_byte(0xFF0F, 0x00);

        // Execute instructions to trigger timer overflow
        gb.memory.data[0] = 0x00; // NOP
        for _ in 0..4 {
            gb.step();
        }

        // Check that ONLY bit 2 is set
        let if_register = gb.memory.read_byte(0xFF0F);
        assert_eq!(
            if_register, 0x04,
            "Only bit 2 (timer interrupt) should be set in IF register"
        );
    }

    #[test]
    fn multiple_timer_overflows_still_set_same_bit() {
        let mut gb = GameBoy::new();

        // Setup timer with TMA = 0xFF so it overflows every 16 cycles
        gb.memory.write_byte(0xFF07, 0x05);
        gb.memory.write_byte(0xFF05, 0xFF);
        gb.memory.write_byte(0xFF06, 0xFF); // TMA = 0xFF, so it immediately overflows again

        // Clear IF register
        gb.memory.write_byte(0xFF0F, 0x00);

        // Execute enough instructions to trigger multiple overflows
        gb.memory.data[0] = 0x00; // NOP
        for _ in 0..8 {
            // 32 cycles = 2 overflows
            gb.step();
        }

        // Check that bit 2 is set (but not "double set" or anything weird)
        let if_register = gb.memory.read_byte(0xFF0F);
        assert_eq!(
            if_register & 0x04,
            0x04,
            "Multiple overflows should still just set bit 2 once"
        );

        // Verify it's exactly 0x04, not some weird value
        assert_eq!(if_register, 0x04);
    }

    #[test]
    fn timer_disabled_no_interrupt() {
        let mut gb = GameBoy::new();

        // Setup timer but keep it disabled
        gb.memory.write_byte(0xFF07, 0x01); // Timer DISABLED (bit 2 = 0)
        gb.memory.write_byte(0xFF05, 0xFF);
        gb.memory.write_byte(0xFF06, 0x00);

        // Clear IF register
        gb.memory.write_byte(0xFF0F, 0x00);

        // Execute many instructions
        gb.memory.data[0] = 0x00; // NOP
        for _ in 0..100 {
            gb.step();
        }

        // Check that IF register is still 0 (no interrupt)
        let if_register = gb.memory.read_byte(0xFF0F);
        assert_eq!(
            if_register, 0x00,
            "Disabled timer should not trigger interrupt"
        );
    }

    #[test]
    fn timer_interrupt_with_slow_frequency() {
        let mut gb = GameBoy::new();

        // Use slowest frequency (1024 cycles)
        gb.memory.write_byte(0xFF07, 0x04); // Enable, frequency 1024
        gb.memory.write_byte(0xFF05, 0xFF);
        gb.memory.write_byte(0xFF06, 0x00);

        // Clear IF register
        gb.memory.write_byte(0xFF0F, 0x00);

        // Execute 256 NOPs = 1024 cycles
        gb.memory.data[0] = 0x00; // NOP
        for _ in 0..256 {
            gb.step();
        }

        // Check that bit 2 is set
        let if_register = gb.memory.read_byte(0xFF0F);
        assert_eq!(
            if_register & 0x04,
            0x04,
            "Timer should trigger interrupt at frequency 1024"
        );
    }
}
