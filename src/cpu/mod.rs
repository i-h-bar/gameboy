use crate::cpu::registers::Registers;

mod instructions;
pub mod registers;

pub struct Cpu {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            pc: 0x0100, // Start after boot ROM
            sp: 0xFFFE,
            halted: false,
        }
    }

    // NOP - No operation
    fn nop(&mut self) -> u8 {
        4 // 4 cycles
    }

    // HALT - Halt CPU until interrupt
    fn halt(&mut self) -> u8 {
        self.halted = true;
        4
    }

    // LD r, r' - Load register to register (all take 4 cycles)
    fn ld_a_a(&mut self) -> u8 {
        4
    }
    fn ld_a_b(&mut self) -> u8 {
        self.registers.a = self.registers.b;
        4
    }
    fn ld_a_c(&mut self) -> u8 {
        self.registers.a = self.registers.c;
        4
    }
    fn ld_a_d(&mut self) -> u8 {
        self.registers.a = self.registers.d;
        4
    }
    fn ld_a_e(&mut self) -> u8 {
        self.registers.a = self.registers.e;
        4
    }
    fn ld_a_h(&mut self) -> u8 {
        self.registers.a = self.registers.h;
        4
    }
    fn ld_a_l(&mut self) -> u8 {
        self.registers.a = self.registers.l;
        4
    }

    fn ld_b_a(&mut self) -> u8 {
        self.registers.b = self.registers.a;
        4
    }
    fn ld_b_b(&mut self) -> u8 {
        4
    }
    fn ld_b_c(&mut self) -> u8 {
        self.registers.b = self.registers.c;
        4
    }
    fn ld_b_d(&mut self) -> u8 {
        self.registers.b = self.registers.d;
        4
    }
    fn ld_b_e(&mut self) -> u8 {
        self.registers.b = self.registers.e;
        4
    }
    fn ld_b_h(&mut self) -> u8 {
        self.registers.b = self.registers.h;
        4
    }
    fn ld_b_l(&mut self) -> u8 {
        self.registers.b = self.registers.l;
        4
    }

    fn ld_c_a(&mut self) -> u8 {
        self.registers.c = self.registers.a;
        4
    }
    fn ld_c_b(&mut self) -> u8 {
        self.registers.c = self.registers.b;
        4
    }
    fn ld_c_c(&mut self) -> u8 {
        4
    }
    fn ld_c_d(&mut self) -> u8 {
        self.registers.c = self.registers.d;
        4
    }
    fn ld_c_e(&mut self) -> u8 {
        self.registers.c = self.registers.e;
        4
    }
    fn ld_c_h(&mut self) -> u8 {
        self.registers.c = self.registers.h;
        4
    }
    fn ld_c_l(&mut self) -> u8 {
        self.registers.c = self.registers.l;
        4
    }

    // LD r, n - Load immediate 8-bit value (8 cycles each)
    fn ld_a_n(&mut self, memory: &crate::memory::Memory) -> u8 {
        self.registers.a = self.fetch_byte(memory);
        8
    }

    fn ld_b_n(&mut self, memory: &crate::memory::Memory) -> u8 {
        self.registers.b = self.fetch_byte(memory);
        8
    }

    fn ld_c_n(&mut self, memory: &crate::memory::Memory) -> u8 {
        self.registers.c = self.fetch_byte(memory);
        8
    }

    fn ld_d_n(&mut self, memory: &crate::memory::Memory) -> u8 {
        self.registers.d = self.fetch_byte(memory);
        8
    }

    fn ld_e_n(&mut self, memory: &crate::memory::Memory) -> u8 {
        self.registers.e = self.fetch_byte(memory);
        8
    }

    fn ld_h_n(&mut self, memory: &crate::memory::Memory) -> u8 {
        self.registers.h = self.fetch_byte(memory);
        8
    }

    fn ld_l_n(&mut self, memory: &crate::memory::Memory) -> u8 {
        self.registers.l = self.fetch_byte(memory);
        8
    }

    // LD r, (HL) - Load from memory at HL (8 cycles each)
    fn ld_a_hl(&mut self, memory: &crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.a = memory.read_byte(addr);
        8
    }

    fn ld_b_hl(&mut self, memory: &crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.b = memory.read_byte(addr);
        8
    }

    fn ld_c_hl(&mut self, memory: &crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.c = memory.read_byte(addr);
        8
    }

    fn ld_d_hl(&mut self, memory: &crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.d = memory.read_byte(addr);
        8
    }

    fn ld_e_hl(&mut self, memory: &crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.e = memory.read_byte(addr);
        8
    }

    fn ld_h_hl(&mut self, memory: &crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.h = memory.read_byte(addr);
        8
    }

    fn ld_l_hl(&mut self, memory: &crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.l = memory.read_byte(addr);
        8
    }

    // LD (HL), r - Store to memory at HL (8 cycles each)
    fn ld_hl_a(&mut self, memory: &mut crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.a);
        8
    }

    fn ld_hl_b(&mut self, memory: &mut crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.b);
        8
    }

    fn ld_hl_c(&mut self, memory: &mut crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.c);
        8
    }

    fn ld_hl_d(&mut self, memory: &mut crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.d);
        8
    }

    fn ld_hl_e(&mut self, memory: &mut crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.e);
        8
    }

    fn ld_hl_h(&mut self, memory: &mut crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.h);
        8
    }

    fn ld_hl_l(&mut self, memory: &mut crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.l);
        8
    }

    fn ld_hl_n(&mut self, memory: &mut crate::memory::Memory) -> u8 {
        let value = self.fetch_byte(memory);
        let addr = self.registers.hl();
        memory.write_byte(addr, value);
        12
    }

    // 16-bit loads (12 cycles each)
    fn ld_bc_nn(&mut self, memory: &crate::memory::Memory) -> u8 {
        let value = self.fetch_word(memory);
        self.registers.set_bc(value);
        12
    }

    fn ld_de_nn(&mut self, memory: &crate::memory::Memory) -> u8 {
        let value = self.fetch_word(memory);
        self.registers.set_de(value);
        12
    }

    fn ld_hl_nn(&mut self, memory: &crate::memory::Memory) -> u8 {
        let value = self.fetch_word(memory);
        self.registers.set_hl(value);
        12
    }

    fn ld_sp_nn(&mut self, memory: &crate::memory::Memory) -> u8 {
        self.sp = self.fetch_word(memory);
        12
    }

    // XOR operations - all XOR with A register and store result in A
    // Flags: Z if result is 0, N=0, H=0, C=0
    fn xor_a(&mut self) -> u8 {
        self.registers.a = 0;
        self.registers.f.z = true;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        4
    }

    fn xor_b(&mut self) -> u8 {
        self.registers.a ^= self.registers.b;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        4
    }

    fn xor_c(&mut self) -> u8 {
        self.registers.a ^= self.registers.c;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        4
    }

    fn xor_d(&mut self) -> u8 {
        self.registers.a ^= self.registers.d;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        4
    }

    fn xor_e(&mut self) -> u8 {
        self.registers.a ^= self.registers.e;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        4
    }

    fn xor_h(&mut self) -> u8 {
        self.registers.a ^= self.registers.h;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        4
    }

    fn xor_l(&mut self) -> u8 {
        self.registers.a ^= self.registers.l;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        4
    }

    fn xor_hl(&mut self, memory: &crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        let value = memory.read_byte(addr);
        self.registers.a ^= value;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        8
    }

    fn xor_n(&mut self, memory: &crate::memory::Memory) -> u8 {
        let value = self.fetch_byte(memory);
        self.registers.a ^= value;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        8
    }

    // INC 8-bit - Increment register
    // Flags: Z if result is 0, N=0, H if carry from bit 3, C not affected
    fn inc_a(&mut self) -> u8 {
        self.registers.f.h = (self.registers.a & 0x0F) == 0x0F;
        self.registers.a = self.registers.a.wrapping_add(1);
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        4
    }

    fn inc_b(&mut self) -> u8 {
        self.registers.f.h = (self.registers.b & 0x0F) == 0x0F;
        self.registers.b = self.registers.b.wrapping_add(1);
        self.registers.f.z = self.registers.b == 0;
        self.registers.f.n = false;
        4
    }

    fn inc_c(&mut self) -> u8 {
        self.registers.f.h = (self.registers.c & 0x0F) == 0x0F;
        self.registers.c = self.registers.c.wrapping_add(1);
        self.registers.f.z = self.registers.c == 0;
        self.registers.f.n = false;
        4
    }

    fn inc_d(&mut self) -> u8 {
        self.registers.f.h = (self.registers.d & 0x0F) == 0x0F;
        self.registers.d = self.registers.d.wrapping_add(1);
        self.registers.f.z = self.registers.d == 0;
        self.registers.f.n = false;
        4
    }

    fn inc_e(&mut self) -> u8 {
        self.registers.f.h = (self.registers.e & 0x0F) == 0x0F;
        self.registers.e = self.registers.e.wrapping_add(1);
        self.registers.f.z = self.registers.e == 0;
        self.registers.f.n = false;
        4
    }

    fn inc_h(&mut self) -> u8 {
        self.registers.f.h = (self.registers.h & 0x0F) == 0x0F;
        self.registers.h = self.registers.h.wrapping_add(1);
        self.registers.f.z = self.registers.h == 0;
        self.registers.f.n = false;
        4
    }

    fn inc_l(&mut self) -> u8 {
        self.registers.f.h = (self.registers.l & 0x0F) == 0x0F;
        self.registers.l = self.registers.l.wrapping_add(1);
        self.registers.f.z = self.registers.l == 0;
        self.registers.f.n = false;
        4
    }

    fn inc_hl(&mut self, memory: &mut crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        let value = memory.read_byte(addr);
        self.registers.f.h = (value & 0x0F) == 0x0F;
        let result = value.wrapping_add(1);
        memory.write_byte(addr, result);
        self.registers.f.z = result == 0;
        self.registers.f.n = false;
        12
    }

    // DEC 8-bit - Decrement register
    // Flags: Z if result is 0, N=1, H if borrow from bit 4, C not affected
    fn dec_a(&mut self) -> u8 {
        self.registers.f.h = (self.registers.a & 0x0F) == 0x00;
        self.registers.a = self.registers.a.wrapping_sub(1);
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = true;
        4
    }

    fn dec_b(&mut self) -> u8 {
        self.registers.f.h = (self.registers.b & 0x0F) == 0x00;
        self.registers.b = self.registers.b.wrapping_sub(1);
        self.registers.f.z = self.registers.b == 0;
        self.registers.f.n = true;
        4
    }

    fn dec_c(&mut self) -> u8 {
        self.registers.f.h = (self.registers.c & 0x0F) == 0x00;
        self.registers.c = self.registers.c.wrapping_sub(1);
        self.registers.f.z = self.registers.c == 0;
        self.registers.f.n = true;
        4
    }

    fn dec_d(&mut self) -> u8 {
        self.registers.f.h = (self.registers.d & 0x0F) == 0x00;
        self.registers.d = self.registers.d.wrapping_sub(1);
        self.registers.f.z = self.registers.d == 0;
        self.registers.f.n = true;
        4
    }

    fn dec_e(&mut self) -> u8 {
        self.registers.f.h = (self.registers.e & 0x0F) == 0x00;
        self.registers.e = self.registers.e.wrapping_sub(1);
        self.registers.f.z = self.registers.e == 0;
        self.registers.f.n = true;
        4
    }

    fn dec_h(&mut self) -> u8 {
        self.registers.f.h = (self.registers.h & 0x0F) == 0x00;
        self.registers.h = self.registers.h.wrapping_sub(1);
        self.registers.f.z = self.registers.h == 0;
        self.registers.f.n = true;
        4
    }

    fn dec_l(&mut self) -> u8 {
        self.registers.f.h = (self.registers.l & 0x0F) == 0x00;
        self.registers.l = self.registers.l.wrapping_sub(1);
        self.registers.f.z = self.registers.l == 0;
        self.registers.f.n = true;
        4
    }

    fn dec_hl(&mut self, memory: &mut crate::memory::Memory) -> u8 {
        let addr = self.registers.hl();
        let value = memory.read_byte(addr);
        self.registers.f.h = (value & 0x0F) == 0x00;
        let result = value.wrapping_sub(1);
        memory.write_byte(addr, result);
        self.registers.f.z = result == 0;
        self.registers.f.n = true;
        12
    }

    // INC 16-bit - Increment 16-bit register (no flags affected)
    fn inc_bc(&mut self) -> u8 {
        let value = self.registers.bc();
        self.registers.set_bc(value.wrapping_add(1));
        8
    }

    fn inc_de(&mut self) -> u8 {
        let value = self.registers.de();
        self.registers.set_de(value.wrapping_add(1));
        8
    }

    fn inc_hl_16(&mut self) -> u8 {
        let value = self.registers.hl();
        self.registers.set_hl(value.wrapping_add(1));
        8
    }

    fn inc_sp(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        8
    }

    // DEC 16-bit - Decrement 16-bit register (no flags affected)
    fn dec_bc(&mut self) -> u8 {
        let value = self.registers.bc();
        self.registers.set_bc(value.wrapping_sub(1));
        8
    }

    fn dec_de(&mut self) -> u8 {
        let value = self.registers.de();
        self.registers.set_de(value.wrapping_sub(1));
        8
    }

    fn dec_hl_16(&mut self) -> u8 {
        let value = self.registers.hl();
        self.registers.set_hl(value.wrapping_sub(1));
        8
    }

    fn dec_sp(&mut self) -> u8 {
        self.sp = self.sp.wrapping_sub(1);
        8
    }

    // JP nn - Absolute jump to 16-bit address
    fn jp_nn(&mut self, memory: &crate::memory::Memory) -> u8 {
        self.pc = self.fetch_word(memory);
        16
    }

    // JR n - Relative jump by signed 8-bit offset

    fn jr_n(&mut self, memory: &crate::memory::Memory) -> u8 {
        let offset_16 = i16::from(
            // Game Boy JR instruction stores signed offset as a byte in memory.
            // Values 0x80-0xFF represent negative offsets in two's complement.
            // The "wrap" is intentional - we're reinterpreting the bit pattern.
            #[allow(clippy::cast_possible_wrap)]
            {
                self.fetch_byte(memory) as i8
            }
        );

        // Cast to u16 preserves the two's complement bit pattern.
        // wrapping_add correctly handles both positive and negative offsets.
        // Example: -2i16 (0xFFFE) as u16 = 0xFFFE, which wraps correctly when added.
        #[allow(clippy::cast_sign_loss)]
        {
            self.pc = self.pc.wrapping_add(offset_16 as u16);
        }
        12
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
