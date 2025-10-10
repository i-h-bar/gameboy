pub mod cpu;
pub mod memory;

pub struct GameBoy {
    pub cpu: cpu::Cpu,
    pub memory: memory::Memory,
}

impl GameBoy {
    pub fn new() -> Self {
        Self {
            cpu: cpu::Cpu::default(),
            memory: memory::Memory::default(),
        }
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
}

impl Default for GameBoy {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let mut game = GameBoy::default();
    game.power_on();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gameboy_creation() {
        let gb = GameBoy::new();
        assert_eq!(gb.cpu.pc, 0x0100);
        assert_eq!(gb.cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_register_pairs() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_bc(0x1234);
        assert_eq!(gb.cpu.registers.b, 0x12);
        assert_eq!(gb.cpu.registers.c, 0x34);
        assert_eq!(gb.cpu.registers.bc(), 0x1234);
    }

    #[test]
    fn test_memory_read_write() {
        let mut gb = GameBoy::new();
        gb.memory.write_byte(0xC000, 0x42);
        assert_eq!(gb.memory.read_byte(0xC000), 0x42);

        gb.memory.write_word(0xC100, 0xBEEF);
        assert_eq!(gb.memory.read_word(0xC100), 0xBEEF);
    }

    #[test]
    fn test_nop() {
        let mut gb = GameBoy::new();
        gb.memory.write_byte(0x0100, 0x00); // NOP
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.pc, 0x0101);
    }

    #[test]
    fn test_ld_a_n() {
        let mut gb = GameBoy::new();
        gb.memory.write_byte(0x0100, 0x3E); // LD A, n
        gb.memory.write_byte(0x0101, 0x42);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.a, 0x42);
        assert_eq!(gb.cpu.pc, 0x0102);
    }

    #[test]
    fn test_ld_bc_nn() {
        let mut gb = GameBoy::new();
        gb.memory.write_byte(0x0100, 0x01); // LD BC, nn
        gb.memory.write_word(0x0101, 0x1234);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12);
        assert_eq!(gb.cpu.registers.bc(), 0x1234);
        assert_eq!(gb.cpu.pc, 0x0103);
    }

    #[test]
    fn test_ld_hl_operations() {
        let mut gb = GameBoy::new();

        // Set HL to 0xC000
        gb.cpu.registers.set_hl(0xC000);

        // LD A, 0x42
        gb.cpu.registers.a = 0x42;

        // LD (HL), A - Store A to memory at HL
        gb.memory.write_byte(0x0100, 0x77);
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.memory.read_byte(0xC000), 0x42);

        // LD B, (HL) - Load from memory at HL to B
        gb.cpu.pc = 0x0100;
        gb.memory.write_byte(0x0100, 0x46);
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.b, 0x42);
    }

    #[test]
    fn test_xor_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0xFF;
        gb.memory.write_byte(0x0100, 0xAF); // XOR A
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0x00);
        assert_eq!(gb.cpu.registers.f.z, true); // Zero flag set
        assert_eq!(gb.cpu.registers.f.n, false);
        assert_eq!(gb.cpu.registers.f.h, false);
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_xor_b() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b11001100;
        gb.cpu.registers.b = 0b10101010;
        gb.memory.write_byte(0x0100, 0xA8); // XOR B
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0b01100110);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, false);
        assert_eq!(gb.cpu.registers.f.h, false);
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_inc_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x0F;
        gb.memory.write_byte(0x0100, 0x3C); // INC A
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0x10);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, false);
        assert_eq!(gb.cpu.registers.f.h, true); // Half carry set
    }

    #[test]
    fn test_inc_zero() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0xFF;
        gb.memory.write_byte(0x0100, 0x3C); // INC A
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x00);
        assert_eq!(gb.cpu.registers.f.z, true); // Zero flag set
        assert_eq!(gb.cpu.registers.f.h, true); // Half carry set
    }

    #[test]
    fn test_dec_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x10;
        gb.memory.write_byte(0x0100, 0x3D); // DEC A
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0x0F);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, true); // N flag set
        assert_eq!(gb.cpu.registers.f.h, true); // Half carry set
    }

    #[test]
    fn test_inc_bc() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_bc(0x1234);
        gb.memory.write_byte(0x0100, 0x03); // INC BC
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.bc(), 0x1235);
    }

    #[test]
    fn test_dec_hl_16() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0x1000);
        gb.memory.write_byte(0x0100, 0x2B); // DEC HL
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.hl(), 0x0FFF);
    }

    #[test]
    fn test_jp_nn() {
        let mut gb = GameBoy::new();
        gb.memory.write_byte(0x0100, 0xC3); // JP nn
        gb.memory.write_word(0x0101, 0x8000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16);
        assert_eq!(gb.cpu.pc, 0x8000);
    }

    #[test]
    fn test_jr_n_positive() {
        let mut gb = GameBoy::new();
        gb.memory.write_byte(0x0100, 0x18); // JR n
        gb.memory.write_byte(0x0101, 0x10); // Jump forward 16 bytes
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12);
        assert_eq!(gb.cpu.pc, 0x0112); // 0x0100 + 2 (instruction) + 16 (offset)
    }

    #[test]
    fn test_jr_n_negative() {
        let mut gb = GameBoy::new();
        gb.cpu.pc = 0x0200;
        gb.memory.write_byte(0x0200, 0x18); // JR n
        gb.memory.write_byte(0x0201, 0xFE as u8); // -2 as signed byte
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.pc, 0x0200); // 0x0200 + 2 - 2 = 0x0200 (infinite loop)
    }

    #[test]
    fn test_jr_z_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = true; // Set zero flag
        gb.memory.write_byte(0x0100, 0x28); // JR Z, n
        gb.memory.write_byte(0x0101, 0x10); // Jump forward 16 bytes
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12); // Taken
        assert_eq!(gb.cpu.pc, 0x0112);
    }

    #[test]
    fn test_jr_z_not_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = false; // Clear zero flag
        gb.memory.write_byte(0x0100, 0x28); // JR Z, n
        gb.memory.write_byte(0x0101, 0x10);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8); // Not taken
        assert_eq!(gb.cpu.pc, 0x0102); // Just skips the offset byte
    }

    #[test]
    fn test_jr_nz_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = false; // Clear zero flag
        gb.memory.write_byte(0x0100, 0x20); // JR NZ, n
        gb.memory.write_byte(0x0101, 0x05);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12); // Taken
        assert_eq!(gb.cpu.pc, 0x0107);
    }

    #[test]
    fn test_jr_nz_not_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = true; // Set zero flag
        gb.memory.write_byte(0x0100, 0x20); // JR NZ, n
        gb.memory.write_byte(0x0101, 0x05);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8); // Not taken
        assert_eq!(gb.cpu.pc, 0x0102);
    }

    #[test]
    fn test_jr_c_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.c = true; // Set carry flag
        gb.memory.write_byte(0x0100, 0x38); // JR C, n
        gb.memory.write_byte(0x0101, 0xFE as u8); // -2
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12); // Taken
        assert_eq!(gb.cpu.pc, 0x0100);
    }

    #[test]
    fn test_jr_nc_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.c = false; // Clear carry flag
        gb.memory.write_byte(0x0100, 0x30); // JR NC, n
        gb.memory.write_byte(0x0101, 0x20);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12); // Taken
        assert_eq!(gb.cpu.pc, 0x0122);
    }

    #[test]
    fn test_jp_z_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = true; // Set zero flag
        gb.memory.write_byte(0x0100, 0xCA); // JP Z, nn
        gb.memory.write_word(0x0101, 0x8000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16); // Taken
        assert_eq!(gb.cpu.pc, 0x8000);
    }

    #[test]
    fn test_jp_z_not_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = false; // Clear zero flag
        gb.memory.write_byte(0x0100, 0xCA); // JP Z, nn
        gb.memory.write_word(0x0101, 0x8000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12); // Not taken
        assert_eq!(gb.cpu.pc, 0x0103); // Skips the address bytes
    }

    #[test]
    fn test_jp_nz_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = false; // Clear zero flag
        gb.memory.write_byte(0x0100, 0xC2); // JP NZ, nn
        gb.memory.write_word(0x0101, 0x4000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16); // Taken
        assert_eq!(gb.cpu.pc, 0x4000);
    }

    #[test]
    fn test_jp_c_not_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.c = false; // Clear carry flag
        gb.memory.write_byte(0x0100, 0xDA); // JP C, nn
        gb.memory.write_word(0x0101, 0x5000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12); // Not taken
        assert_eq!(gb.cpu.pc, 0x0103);
    }
}
