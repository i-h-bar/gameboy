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

    #[test]
    fn test_add_a_b() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x3A;
        gb.cpu.registers.b = 0x15;
        gb.memory.write_byte(0x0100, 0x80); // ADD A, B
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0x4F);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, false);
        assert_eq!(gb.cpu.registers.f.h, false);
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_add_a_half_carry() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x0F;
        gb.cpu.registers.b = 0x01;
        gb.memory.write_byte(0x0100, 0x80); // ADD A, B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x10);
        assert_eq!(gb.cpu.registers.f.h, true); // Half carry
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_add_a_full_carry() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0xFF;
        gb.cpu.registers.b = 0x02;
        gb.memory.write_byte(0x0100, 0x80); // ADD A, B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x01);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.c, true); // Carry
        assert_eq!(gb.cpu.registers.f.h, true); // Half carry
    }

    #[test]
    fn test_add_a_zero() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x00;
        gb.memory.write_byte(0x0100, 0x87); // ADD A, A
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x00);
        assert_eq!(gb.cpu.registers.f.z, true); // Zero flag
    }

    #[test]
    fn test_sub_a_b() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x3E;
        gb.cpu.registers.b = 0x0F;
        gb.memory.write_byte(0x0100, 0x90); // SUB A, B
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0x2F);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, true); // N always set for SUB
        assert_eq!(gb.cpu.registers.f.h, true); // Half borrow
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_sub_a_zero() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x42;
        gb.memory.write_byte(0x0100, 0x97); // SUB A, A
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x00);
        assert_eq!(gb.cpu.registers.f.z, true); // Zero flag
        assert_eq!(gb.cpu.registers.f.n, true);
    }

    #[test]
    fn test_sub_a_borrow() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x0F;
        gb.cpu.registers.b = 0x1F;
        gb.memory.write_byte(0x0100, 0x90); // SUB A, B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0xF0); // Wraps around
        assert_eq!(gb.cpu.registers.f.c, true); // Borrow
    }

    #[test]
    fn test_and_a_b() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b11110000;
        gb.cpu.registers.b = 0b10101010;
        gb.memory.write_byte(0x0100, 0xA0); // AND A, B
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0b10100000);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, false);
        assert_eq!(gb.cpu.registers.f.h, true); // H always set for AND
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_and_a_zero() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b11110000;
        gb.cpu.registers.b = 0b00001111;
        gb.memory.write_byte(0x0100, 0xA0); // AND A, B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x00);
        assert_eq!(gb.cpu.registers.f.z, true); // Zero flag
        assert_eq!(gb.cpu.registers.f.h, true);
    }

    #[test]
    fn test_or_a_b() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b11110000;
        gb.cpu.registers.b = 0b00001111;
        gb.memory.write_byte(0x0100, 0xB0); // OR A, B
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0b11111111);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, false);
        assert_eq!(gb.cpu.registers.f.h, false);
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_or_a_zero() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x00;
        gb.memory.write_byte(0x0100, 0xB7); // OR A, A
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x00);
        assert_eq!(gb.cpu.registers.f.z, true); // Zero flag
    }

    #[test]
    fn test_cp_a_equal() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x42;
        gb.cpu.registers.b = 0x42;
        gb.memory.write_byte(0x0100, 0xB8); // CP A, B
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0x42); // A unchanged
        assert_eq!(gb.cpu.registers.f.z, true); // Equal
        assert_eq!(gb.cpu.registers.f.n, true);
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_cp_a_less_than() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x10;
        gb.cpu.registers.b = 0x20;
        gb.memory.write_byte(0x0100, 0xB8); // CP A, B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x10); // A unchanged
        assert_eq!(gb.cpu.registers.f.z, false); // Not equal
        assert_eq!(gb.cpu.registers.f.c, true); // A < B
    }

    #[test]
    fn test_cp_a_greater_than() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x30;
        gb.cpu.registers.b = 0x20;
        gb.memory.write_byte(0x0100, 0xB8); // CP A, B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x30); // A unchanged
        assert_eq!(gb.cpu.registers.f.z, false); // Not equal
        assert_eq!(gb.cpu.registers.f.c, false); // A >= B
    }

    // Stack operation tests
    #[test]
    fn test_push_pop_bc() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_bc(0x1234);
        let initial_sp = gb.cpu.sp;

        // PUSH BC
        gb.memory.write_byte(0x0100, 0xC5);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16);
        assert_eq!(gb.cpu.sp, initial_sp - 2); // SP decremented by 2
        // Check stack contents
        assert_eq!(gb.memory.read_byte(gb.cpu.sp), 0x34); // Low byte
        assert_eq!(gb.memory.read_byte(gb.cpu.sp + 1), 0x12); // High byte

        // Change BC to verify POP restores it
        gb.cpu.registers.set_bc(0x0000);

        // POP BC
        gb.memory.write_byte(0x0101, 0xC1);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12);
        assert_eq!(gb.cpu.sp, initial_sp); // SP restored
        assert_eq!(gb.cpu.registers.bc(), 0x1234); // BC restored
    }

    #[test]
    fn test_push_pop_af() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x42;
        gb.cpu.registers.f.z = true;
        gb.cpu.registers.f.n = false;
        gb.cpu.registers.f.h = true;
        gb.cpu.registers.f.c = false;
        let initial_sp = gb.cpu.sp;

        // PUSH AF
        gb.memory.write_byte(0x0100, 0xF5);
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.sp, initial_sp - 2);

        // Modify AF
        gb.cpu.registers.a = 0x00;
        gb.cpu.registers.f.z = false;

        // POP AF
        gb.memory.write_byte(0x0101, 0xF1);
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.sp, initial_sp);
        assert_eq!(gb.cpu.registers.a, 0x42);
        assert_eq!(gb.cpu.registers.f.z, true);
        assert_eq!(gb.cpu.registers.f.h, true);
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_call_ret() {
        let mut gb = GameBoy::new();
        let initial_sp = gb.cpu.sp;

        // CALL 0x8000
        gb.memory.write_byte(0x0100, 0xCD); // CALL nn
        gb.memory.write_word(0x0101, 0x8000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 24);
        assert_eq!(gb.cpu.pc, 0x8000); // Jumped to subroutine
        assert_eq!(gb.cpu.sp, initial_sp - 2); // Return address pushed
        // Check return address on stack (should be 0x0103)
        assert_eq!(gb.memory.read_word(gb.cpu.sp), 0x0103);

        // RET
        gb.memory.write_byte(0x8000, 0xC9); // RET
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16);
        assert_eq!(gb.cpu.pc, 0x0103); // Returned to after CALL
        assert_eq!(gb.cpu.sp, initial_sp); // SP restored
    }

    #[test]
    fn test_call_z_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = true; // Set zero flag
        let initial_sp = gb.cpu.sp;

        // CALL Z, 0x5000
        gb.memory.write_byte(0x0100, 0xCC); // CALL Z, nn
        gb.memory.write_word(0x0101, 0x5000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 24); // Taken
        assert_eq!(gb.cpu.pc, 0x5000);
        assert_eq!(gb.cpu.sp, initial_sp - 2);
    }

    #[test]
    fn test_call_z_not_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = false; // Clear zero flag
        let initial_sp = gb.cpu.sp;

        // CALL Z, 0x5000
        gb.memory.write_byte(0x0100, 0xCC); // CALL Z, nn
        gb.memory.write_word(0x0101, 0x5000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12); // Not taken
        assert_eq!(gb.cpu.pc, 0x0103); // Skips to next instruction
        assert_eq!(gb.cpu.sp, initial_sp); // SP unchanged
    }

    #[test]
    fn test_call_nz_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = false; // Clear zero flag

        // CALL NZ, 0x4000
        gb.memory.write_byte(0x0100, 0xC4); // CALL NZ, nn
        gb.memory.write_word(0x0101, 0x4000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 24); // Taken
        assert_eq!(gb.cpu.pc, 0x4000);
    }

    #[test]
    fn test_call_c_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.c = true; // Set carry flag

        // CALL C, 0x3000
        gb.memory.write_byte(0x0100, 0xDC); // CALL C, nn
        gb.memory.write_word(0x0101, 0x3000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 24); // Taken
        assert_eq!(gb.cpu.pc, 0x3000);
    }

    #[test]
    fn test_call_nc_not_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.c = true; // Set carry flag

        // CALL NC, 0x2000
        gb.memory.write_byte(0x0100, 0xD4); // CALL NC, nn
        gb.memory.write_word(0x0101, 0x2000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12); // Not taken
        assert_eq!(gb.cpu.pc, 0x0103);
    }

    #[test]
    fn test_ret_z_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = true; // Set zero flag

        // Setup: push a return address onto stack
        gb.cpu.sp = gb.cpu.sp.wrapping_sub(2);
        gb.memory.write_word(gb.cpu.sp, 0x0500);

        // RET Z
        gb.memory.write_byte(0x0100, 0xC8); // RET Z
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 20); // Taken
        assert_eq!(gb.cpu.pc, 0x0500);
    }

    #[test]
    fn test_ret_z_not_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = false; // Clear zero flag
        let initial_sp = gb.cpu.sp;

        // RET Z
        gb.memory.write_byte(0x0100, 0xC8); // RET Z
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8); // Not taken
        assert_eq!(gb.cpu.pc, 0x0101); // Just moves to next instruction
        assert_eq!(gb.cpu.sp, initial_sp); // SP unchanged
    }

    #[test]
    fn test_ret_nz_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.z = false; // Clear zero flag

        // Setup: push a return address onto stack
        gb.cpu.sp = gb.cpu.sp.wrapping_sub(2);
        gb.memory.write_word(gb.cpu.sp, 0x0600);

        // RET NZ
        gb.memory.write_byte(0x0100, 0xC0); // RET NZ
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 20); // Taken
        assert_eq!(gb.cpu.pc, 0x0600);
    }

    #[test]
    fn test_ret_c_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.c = true; // Set carry flag

        // Setup: push a return address onto stack
        gb.cpu.sp = gb.cpu.sp.wrapping_sub(2);
        gb.memory.write_word(gb.cpu.sp, 0x0700);

        // RET C
        gb.memory.write_byte(0x0100, 0xD8); // RET C
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 20); // Taken
        assert_eq!(gb.cpu.pc, 0x0700);
    }

    #[test]
    fn test_ret_nc_not_taken() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.c = true; // Set carry flag
        let initial_sp = gb.cpu.sp;

        // RET NC
        gb.memory.write_byte(0x0100, 0xD0); // RET NC
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8); // Not taken
        assert_eq!(gb.cpu.pc, 0x0101);
        assert_eq!(gb.cpu.sp, initial_sp);
    }

    #[test]
    fn test_reti() {
        let mut gb = GameBoy::new();

        // Setup: push a return address onto stack
        gb.cpu.sp = gb.cpu.sp.wrapping_sub(2);
        gb.memory.write_word(gb.cpu.sp, 0x0800);
        let stack_sp = gb.cpu.sp;

        // RETI
        gb.memory.write_byte(0x0100, 0xD9); // RETI
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16);
        assert_eq!(gb.cpu.pc, 0x0800);
        assert_eq!(gb.cpu.sp, stack_sp + 2); // SP restored
    }

    #[test]
    fn test_nested_calls() {
        let mut gb = GameBoy::new();
        let initial_sp = gb.cpu.sp;

        // First CALL to 0x2000
        gb.memory.write_byte(0x0100, 0xCD); // CALL nn
        gb.memory.write_word(0x0101, 0x2000);
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.pc, 0x2000);
        assert_eq!(gb.cpu.sp, initial_sp - 2);

        // Second CALL to 0x3000 (nested)
        gb.memory.write_byte(0x2000, 0xCD); // CALL nn
        gb.memory.write_word(0x2001, 0x3000);
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.pc, 0x3000);
        assert_eq!(gb.cpu.sp, initial_sp - 4); // Two return addresses on stack

        // First RET (from 0x3000)
        gb.memory.write_byte(0x3000, 0xC9); // RET
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.pc, 0x2003); // Returns to after second CALL
        assert_eq!(gb.cpu.sp, initial_sp - 2);

        // Second RET (from 0x2003)
        gb.memory.write_byte(0x2003, 0xC9); // RET
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.pc, 0x0103); // Returns to after first CALL
        assert_eq!(gb.cpu.sp, initial_sp); // Stack fully unwound
    }

    #[test]
    fn test_push_all_registers() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_bc(0x1111);
        gb.cpu.registers.set_de(0x2222);
        gb.cpu.registers.set_hl(0x3333);
        gb.cpu.registers.set_af(0x4440); // Low nibble masked for flags
        let initial_sp = gb.cpu.sp;

        // PUSH BC, DE, HL, AF
        gb.memory.write_byte(0x0100, 0xC5); // PUSH BC
        gb.cpu.execute(&mut gb.memory);
        gb.memory.write_byte(0x0101, 0xD5); // PUSH DE
        gb.cpu.execute(&mut gb.memory);
        gb.memory.write_byte(0x0102, 0xE5); // PUSH HL
        gb.cpu.execute(&mut gb.memory);
        gb.memory.write_byte(0x0103, 0xF5); // PUSH AF
        gb.cpu.execute(&mut gb.memory);

        assert_eq!(gb.cpu.sp, initial_sp - 8); // 4 pushes = 8 bytes

        // Modify all registers
        gb.cpu.registers.set_bc(0x0000);
        gb.cpu.registers.set_de(0x0000);
        gb.cpu.registers.set_hl(0x0000);
        gb.cpu.registers.set_af(0x0000);

        // POP AF, HL, DE, BC (reverse order)
        gb.memory.write_byte(0x0104, 0xF1); // POP AF
        gb.cpu.execute(&mut gb.memory);
        gb.memory.write_byte(0x0105, 0xE1); // POP HL
        gb.cpu.execute(&mut gb.memory);
        gb.memory.write_byte(0x0106, 0xD1); // POP DE
        gb.cpu.execute(&mut gb.memory);
        gb.memory.write_byte(0x0107, 0xC1); // POP BC
        gb.cpu.execute(&mut gb.memory);

        assert_eq!(gb.cpu.sp, initial_sp); // Stack fully restored
        assert_eq!(gb.cpu.registers.bc(), 0x1111);
        assert_eq!(gb.cpu.registers.de(), 0x2222);
        assert_eq!(gb.cpu.registers.hl(), 0x3333);
        assert_eq!(gb.cpu.registers.af(), 0x4440);
    }

    // Additional load instruction tests
    #[test]
    fn test_ld_a_bc() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_bc(0xC000);
        gb.memory.write_byte(0xC000, 0x42);
        gb.memory.write_byte(0x0100, 0x0A); // LD A,(BC)
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.a, 0x42);
        assert_eq!(gb.cpu.pc, 0x0101);
    }

    #[test]
    fn test_ld_a_de() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_de(0xD000);
        gb.memory.write_byte(0xD000, 0x99);
        gb.memory.write_byte(0x0100, 0x1A); // LD A,(DE)
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.a, 0x99);
    }

    #[test]
    fn test_ld_bc_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_bc(0xC100);
        gb.cpu.registers.a = 0x55;
        gb.memory.write_byte(0x0100, 0x02); // LD (BC),A
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.memory.read_byte(0xC100), 0x55);
    }

    #[test]
    fn test_ld_de_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_de(0xD100);
        gb.cpu.registers.a = 0xAA;
        gb.memory.write_byte(0x0100, 0x12); // LD (DE),A
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.memory.read_byte(0xD100), 0xAA);
    }

    #[test]
    fn test_ld_a_nn() {
        let mut gb = GameBoy::new();
        gb.memory.write_byte(0x8000, 0x77);
        gb.memory.write_byte(0x0100, 0xFA); // LD A,(nn)
        gb.memory.write_word(0x0101, 0x8000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16);
        assert_eq!(gb.cpu.registers.a, 0x77);
        assert_eq!(gb.cpu.pc, 0x0103);
    }

    #[test]
    fn test_ld_nn_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x88;
        gb.memory.write_byte(0x0100, 0xEA); // LD (nn),A
        gb.memory.write_word(0x0101, 0x9000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16);
        assert_eq!(gb.memory.read_byte(0x9000), 0x88);
        assert_eq!(gb.cpu.pc, 0x0103);
    }

    #[test]
    fn test_ldi_hl_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.cpu.registers.a = 0x11;
        gb.memory.write_byte(0x0100, 0x22); // LDI (HL),A
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.memory.read_byte(0xC000), 0x11);
        assert_eq!(gb.cpu.registers.hl(), 0xC001); // HL incremented
    }

    #[test]
    fn test_ldi_a_hl() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.memory.write_byte(0xC000, 0x22);
        gb.memory.write_byte(0x0100, 0x2A); // LDI A,(HL)
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.a, 0x22);
        assert_eq!(gb.cpu.registers.hl(), 0xC001); // HL incremented
    }

    #[test]
    fn test_ldd_hl_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.cpu.registers.a = 0x33;
        gb.memory.write_byte(0x0100, 0x32); // LDD (HL),A
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.memory.read_byte(0xC000), 0x33);
        assert_eq!(gb.cpu.registers.hl(), 0xBFFF); // HL decremented
    }

    #[test]
    fn test_ldd_a_hl() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.memory.write_byte(0xC000, 0x44);
        gb.memory.write_byte(0x0100, 0x3A); // LDD A,(HL)
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.a, 0x44);
        assert_eq!(gb.cpu.registers.hl(), 0xBFFF); // HL decremented
    }

    #[test]
    fn test_ldi_sequence() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.cpu.registers.a = 0x10;

        // Write multiple bytes using LDI
        for i in 0..3 {
            gb.cpu.registers.a = 0x10 + i;
            gb.memory.write_byte(0x0100, 0x22); // LDI (HL),A
            gb.cpu.pc = 0x0100;
            gb.cpu.execute(&mut gb.memory);
        }

        assert_eq!(gb.memory.read_byte(0xC000), 0x10);
        assert_eq!(gb.memory.read_byte(0xC001), 0x11);
        assert_eq!(gb.memory.read_byte(0xC002), 0x12);
        assert_eq!(gb.cpu.registers.hl(), 0xC003);
    }

    #[test]
    fn test_ld_nn_sp() {
        let mut gb = GameBoy::new();
        gb.cpu.sp = 0xFFFE;
        gb.memory.write_byte(0x0100, 0x08); // LD (nn),SP
        gb.memory.write_word(0x0101, 0xC000);
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 20);
        assert_eq!(gb.memory.read_word(0xC000), 0xFFFE);
        assert_eq!(gb.cpu.pc, 0x0103);
    }

    #[test]
    fn test_ld_sp_hl() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xDEAD);
        gb.memory.write_byte(0x0100, 0xF9); // LD SP,HL
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.sp, 0xDEAD);
    }

    #[test]
    fn test_ld_hl_sp_n_positive() {
        let mut gb = GameBoy::new();
        gb.cpu.sp = 0xFFF8;
        gb.memory.write_byte(0x0100, 0xF8); // LD HL,SP+n
        gb.memory.write_byte(0x0101, 0x02); // +2
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12);
        assert_eq!(gb.cpu.registers.hl(), 0xFFFA);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, false);
    }

    #[test]
    fn test_ld_hl_sp_n_negative() {
        let mut gb = GameBoy::new();
        gb.cpu.sp = 0xFFF8;
        gb.memory.write_byte(0x0100, 0xF8); // LD HL,SP+n
        gb.memory.write_byte(0x0101, 0xFE as u8); // -2
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12);
        assert_eq!(gb.cpu.registers.hl(), 0xFFF6);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, false);
    }

    #[test]
    fn test_ld_hl_sp_n_flags() {
        let mut gb = GameBoy::new();
        gb.cpu.sp = 0xFF0F;
        gb.memory.write_byte(0x0100, 0xF8); // LD HL,SP+n
        gb.memory.write_byte(0x0101, 0x01); // +1
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.hl(), 0xFF10);
        assert_eq!(gb.cpu.registers.f.h, true); // Half carry from low byte
        assert_eq!(gb.cpu.registers.f.c, false);
    }
}
