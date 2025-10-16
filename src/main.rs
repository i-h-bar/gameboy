pub mod cartridge;
pub mod cpu;
pub mod memory;
mod gameboy;

use crate::gameboy::GameBoy;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <rom_file> [log_file]", args[0]);
        eprintln!("  rom_file: Path to Game Boy ROM file (.gb)");
        eprintln!("  log_file: Optional path to CPU log file for gameboy-doctor");
        std::process::exit(1);
    }

    let rom_path = &args[1];
    let log_path = args.get(2);

    let mut game = GameBoy::new();

    // Load ROM
    if let Err(e) = game.load_rom(rom_path) {
        eprintln!("Error loading ROM: {}", e);
        std::process::exit(1);
    }

    // Enable logging if requested
    if let Some(log_path) = log_path {
        if let Err(e) = game.enable_logging(log_path) {
            eprintln!("Error creating log file: {}", e);
            std::process::exit(1);
        }
        println!("Logging enabled to: {}", log_path);
    }

    game.power_on();

    // Run for a large number of instructions (or until HALT)
    // For testing with gameboy-doctor, you typically want to run until
    // a specific point or until HALT
    println!("Running emulator...");
    game.run(1_000_000); // Run for 1 million instructions or until HALT

    println!("Emulator stopped. CPU halted: {}", game.cpu.halted);
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

    // ADC tests
    #[test]
    fn test_adc_a_b_no_carry() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x10;
        gb.cpu.registers.b = 0x05;
        gb.cpu.registers.f.c = false;
        gb.memory.write_byte(0x0100, 0x88); // ADC A,B
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0x15);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, false);
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_adc_a_b_with_carry() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x10;
        gb.cpu.registers.b = 0x05;
        gb.cpu.registers.f.c = true; // Carry set
        gb.memory.write_byte(0x0100, 0x88); // ADC A,B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x16); // 0x10 + 0x05 + 1
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_adc_a_overflow() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0xFF;
        gb.cpu.registers.b = 0x01;
        gb.cpu.registers.f.c = true;
        gb.memory.write_byte(0x0100, 0x88); // ADC A,B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x01); // Overflow
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.c, true); // Carry out
        assert_eq!(gb.cpu.registers.f.h, true); // Half carry
    }

    // SBC tests
    #[test]
    fn test_sbc_a_b_no_carry() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x20;
        gb.cpu.registers.b = 0x10;
        gb.cpu.registers.f.c = false;
        gb.memory.write_byte(0x0100, 0x98); // SBC A,B
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0x10);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.n, true);
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_sbc_a_b_with_carry() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x20;
        gb.cpu.registers.b = 0x10;
        gb.cpu.registers.f.c = true; // Borrow set
        gb.memory.write_byte(0x0100, 0x98); // SBC A,B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x0F); // 0x20 - 0x10 - 1
    }

    #[test]
    fn test_sbc_a_underflow() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x00;
        gb.cpu.registers.b = 0x01;
        gb.cpu.registers.f.c = false;
        gb.memory.write_byte(0x0100, 0x98); // SBC A,B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0xFF); // Underflow
        assert_eq!(gb.cpu.registers.f.c, true); // Borrow
    }

    // Rotate tests
    #[test]
    fn test_rlca() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b10000001;
        gb.memory.write_byte(0x0100, 0x07); // RLCA
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0b00000011); // Bit 7 rotates to bit 0
        assert_eq!(gb.cpu.registers.f.c, true); // Old bit 7 to carry
        assert_eq!(gb.cpu.registers.f.z, false);
    }

    #[test]
    fn test_rrca() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b10000001;
        gb.memory.write_byte(0x0100, 0x0F); // RRCA
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0b11000000); // Bit 0 rotates to bit 7
        assert_eq!(gb.cpu.registers.f.c, true); // Old bit 0 to carry
    }

    #[test]
    fn test_rla() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b10000000;
        gb.cpu.registers.f.c = true; // Carry set
        gb.memory.write_byte(0x0100, 0x17); // RLA
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0b00000001); // Carry rotates into bit 0
        assert_eq!(gb.cpu.registers.f.c, true); // Old bit 7 to carry
    }

    #[test]
    fn test_rra() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b00000001;
        gb.cpu.registers.f.c = true; // Carry set
        gb.memory.write_byte(0x0100, 0x1F); // RRA
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0b10000000); // Carry rotates into bit 7
        assert_eq!(gb.cpu.registers.f.c, true); // Old bit 0 to carry
    }

    // Miscellaneous instruction tests
    #[test]
    fn test_cpl() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b10101010;
        gb.memory.write_byte(0x0100, 0x2F); // CPL
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.registers.a, 0b01010101); // All bits flipped
        assert_eq!(gb.cpu.registers.f.n, true);
        assert_eq!(gb.cpu.registers.f.h, true);
    }

    #[test]
    fn test_scf() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.c = false;
        gb.memory.write_byte(0x0100, 0x37); // SCF
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.f.c, true);
        assert_eq!(gb.cpu.registers.f.n, false);
        assert_eq!(gb.cpu.registers.f.h, false);
    }

    #[test]
    fn test_ccf() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.f.c = true;
        gb.memory.write_byte(0x0100, 0x3F); // CCF
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.f.c, false); // Carry complemented
        assert_eq!(gb.cpu.registers.f.n, false);
        assert_eq!(gb.cpu.registers.f.h, false);
    }

    #[test]
    fn test_daa_after_add() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x09;
        gb.cpu.registers.b = 0x08;
        // ADD A,B (BCD: 9 + 8 = 17)
        gb.memory.write_byte(0x0100, 0x80);
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x11); // Binary result

        // DAA should adjust to BCD
        gb.memory.write_byte(0x0101, 0x27); // DAA
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x17); // BCD result
    }

    // Interrupt control tests
    #[test]
    fn test_di() {
        let mut gb = GameBoy::new();
        gb.cpu.interrupts_enabled = true;
        gb.memory.write_byte(0x0100, 0xF3); // DI
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.interrupts_enabled, false);
    }

    #[test]
    fn test_ei() {
        let mut gb = GameBoy::new();
        gb.cpu.interrupts_enabled = false;
        gb.memory.write_byte(0x0100, 0xFB); // EI
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 4);
        assert_eq!(gb.cpu.interrupts_enabled, true);
    }

    // RST tests
    #[test]
    fn test_rst_00() {
        let mut gb = GameBoy::new();
        let initial_sp = gb.cpu.sp;
        gb.memory.write_byte(0x0100, 0xC7); // RST 00h
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16);
        assert_eq!(gb.cpu.pc, 0x0000); // Jump to 0x0000
        assert_eq!(gb.cpu.sp, initial_sp - 2); // Return address pushed
        assert_eq!(gb.memory.read_word(gb.cpu.sp), 0x0101); // Return address
    }

    #[test]
    fn test_rst_38() {
        let mut gb = GameBoy::new();
        let initial_sp = gb.cpu.sp;
        gb.memory.write_byte(0x0100, 0xFF); // RST 38h
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.pc, 0x0038); // Jump to 0x0038
        assert_eq!(gb.cpu.sp, initial_sp - 2);
        assert_eq!(gb.memory.read_word(gb.cpu.sp), 0x0101);
    }

    #[test]
    fn test_rst_and_ret() {
        let mut gb = GameBoy::new();
        // RST 10h
        gb.memory.write_byte(0x0100, 0xD7); // RST 10h
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.pc, 0x0010);

        // RET
        gb.memory.write_byte(0x0010, 0xC9); // RET
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.pc, 0x0101); // Back to after RST
    }

    // CB-prefixed instruction tests
    #[test]
    fn test_rlc_b() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.b = 0b10000001;
        gb.memory.write_byte(0x0100, 0xCB); // CB prefix
        gb.memory.write_byte(0x0101, 0x00); // RLC B
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.b, 0b00000011);
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.c, true); // Bit 7 to carry
        assert_eq!(gb.cpu.pc, 0x0102);
    }

    #[test]
    fn test_rlc_zero() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0x00;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x07); // RLC A
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0x00);
        assert_eq!(gb.cpu.registers.f.z, true); // Zero flag set
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_rrc_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b11000001;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x0F); // RRC A
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.a, 0b11100000);
        assert_eq!(gb.cpu.registers.f.c, true); // Bit 0 to carry
        assert_eq!(gb.cpu.registers.f.z, false);
    }

    #[test]
    fn test_rl_c() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.c = 0b10000000;
        gb.cpu.registers.f.c = true; // Carry in
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x11); // RL C
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.c, 0b00000001); // Carry shifts into bit 0
        assert_eq!(gb.cpu.registers.f.c, true); // Bit 7 to carry
        assert_eq!(gb.cpu.registers.f.z, false);
    }

    #[test]
    fn test_rr_d() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.d = 0b00000001;
        gb.cpu.registers.f.c = true; // Carry in
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x1A); // RR D
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.d, 0b10000000); // Carry shifts into bit 7
        assert_eq!(gb.cpu.registers.f.c, true); // Bit 0 to carry
    }

    #[test]
    fn test_sla_e() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.e = 0b11000001;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x23); // SLA E
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.e, 0b10000010); // Shift left, 0 into bit 0
        assert_eq!(gb.cpu.registers.f.c, true); // Bit 7 to carry
        assert_eq!(gb.cpu.registers.f.z, false);
    }

    #[test]
    fn test_sra_h() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.h = 0b10000001;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x2C); // SRA H
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.h, 0b11000000); // Bit 7 preserved (sign)
        assert_eq!(gb.cpu.registers.f.c, true); // Bit 0 to carry
    }

    #[test]
    fn test_srl_l() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.l = 0b10000001;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x3D); // SRL L
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.l, 0b01000000); // Shift right logical, 0 into bit 7
        assert_eq!(gb.cpu.registers.f.c, true); // Bit 0 to carry
    }

    #[test]
    fn test_swap_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0xF0;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x37); // SWAP A
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.a, 0x0F); // Nibbles swapped
        assert_eq!(gb.cpu.registers.f.z, false);
        assert_eq!(gb.cpu.registers.f.c, false);
    }

    #[test]
    fn test_swap_zero() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.b = 0x00;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x30); // SWAP B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.b, 0x00);
        assert_eq!(gb.cpu.registers.f.z, true); // Zero flag
    }

    #[test]
    fn test_swap_mixed() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.c = 0x12;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x31); // SWAP C
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.c, 0x21); // 0x12 -> 0x21
    }

    #[test]
    fn test_bit_0_a_set() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b00000001;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x47); // BIT 0,A
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.a, 0b00000001); // A unchanged
        assert_eq!(gb.cpu.registers.f.z, false); // Bit is set
        assert_eq!(gb.cpu.registers.f.n, false);
        assert_eq!(gb.cpu.registers.f.h, true);
    }

    #[test]
    fn test_bit_7_b_clear() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.b = 0b01111111;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x78); // BIT 7,B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.b, 0b01111111); // B unchanged
        assert_eq!(gb.cpu.registers.f.z, true); // Bit is clear
        assert_eq!(gb.cpu.registers.f.h, true);
    }

    #[test]
    fn test_bit_3_c_set() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.c = 0b00001000;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x59); // BIT 3,C
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.f.z, false); // Bit 3 is set
        assert_eq!(gb.cpu.registers.f.h, true);
    }

    #[test]
    fn test_set_0_d() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.d = 0b00000000;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0xC2); // SET 0,D
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.d, 0b00000001); // Bit 0 set
    }

    #[test]
    fn test_set_7_e() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.e = 0b00000000;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0xFB); // SET 7,E
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.e, 0b10000000); // Bit 7 set
    }

    #[test]
    fn test_set_already_set() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.h = 0b11111111;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0xDC); // SET 3,H
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.h, 0b11111111); // Unchanged (bit already set)
    }

    #[test]
    fn test_res_0_l() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.l = 0b11111111;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x85); // RES 0,L
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 8);
        assert_eq!(gb.cpu.registers.l, 0b11111110); // Bit 0 cleared
    }

    #[test]
    fn test_res_7_a() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.a = 0b11111111;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0xBF); // RES 7,A
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.a, 0b01111111); // Bit 7 cleared
    }

    #[test]
    fn test_res_already_clear() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.b = 0b00000000;
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x90); // RES 2,B
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.cpu.registers.b, 0b00000000); // Unchanged (bit already clear)
    }

    #[test]
    fn test_cb_hl_operations() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.memory.write_byte(0xC000, 0b10101010);

        // RLC (HL)
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x06); // RLC (HL)
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16); // Memory operations take 16 cycles
        assert_eq!(gb.memory.read_byte(0xC000), 0b01010101);
        assert_eq!(gb.cpu.registers.f.c, true);
    }

    #[test]
    fn test_bit_hl() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.memory.write_byte(0xC000, 0b10000000);

        // BIT 7,(HL)
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x7E); // BIT 7,(HL)
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 12); // BIT (HL) takes 12 cycles
        assert_eq!(gb.cpu.registers.f.z, false); // Bit is set
        assert_eq!(gb.cpu.registers.f.h, true);
    }

    #[test]
    fn test_set_hl() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.memory.write_byte(0xC000, 0x00);

        // SET 4,(HL)
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0xE6); // SET 4,(HL)
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16); // SET (HL) takes 16 cycles
        assert_eq!(gb.memory.read_byte(0xC000), 0b00010000);
    }

    #[test]
    fn test_res_hl() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.memory.write_byte(0xC000, 0xFF);

        // RES 5,(HL)
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0xAE); // RES 5,(HL)
        let cycles = gb.cpu.execute(&mut gb.memory);
        assert_eq!(cycles, 16); // RES (HL) takes 16 cycles
        assert_eq!(gb.memory.read_byte(0xC000), 0b11011111);
    }

    #[test]
    fn test_sla_hl() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.memory.write_byte(0xC000, 0b11000000);

        // SLA (HL)
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x26); // SLA (HL)
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.memory.read_byte(0xC000), 0b10000000);
        assert_eq!(gb.cpu.registers.f.c, true); // Bit 7 to carry
    }

    #[test]
    fn test_swap_hl() {
        let mut gb = GameBoy::new();
        gb.cpu.registers.set_hl(0xC000);
        gb.memory.write_byte(0xC000, 0xAB);

        // SWAP (HL)
        gb.memory.write_byte(0x0100, 0xCB);
        gb.memory.write_byte(0x0101, 0x36); // SWAP (HL)
        gb.cpu.execute(&mut gb.memory);
        assert_eq!(gb.memory.read_byte(0xC000), 0xBA); // Nibbles swapped
        assert_eq!(gb.cpu.registers.f.z, false);
    }
}
