use crate::cpu::registers::Registers;
use crate::memory::Memory;

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
    fn ld_a_n(&mut self, memory: &Memory) -> u8 {
        self.registers.a = self.fetch_byte(memory);
        8
    }

    fn ld_b_n(&mut self, memory: &Memory) -> u8 {
        self.registers.b = self.fetch_byte(memory);
        8
    }

    fn ld_c_n(&mut self, memory: &Memory) -> u8 {
        self.registers.c = self.fetch_byte(memory);
        8
    }

    fn ld_d_n(&mut self, memory: &Memory) -> u8 {
        self.registers.d = self.fetch_byte(memory);
        8
    }

    fn ld_e_n(&mut self, memory: &Memory) -> u8 {
        self.registers.e = self.fetch_byte(memory);
        8
    }

    fn ld_h_n(&mut self, memory: &Memory) -> u8 {
        self.registers.h = self.fetch_byte(memory);
        8
    }

    fn ld_l_n(&mut self, memory: &Memory) -> u8 {
        self.registers.l = self.fetch_byte(memory);
        8
    }

    // LD r, (HL) - Load from memory at HL (8 cycles each)
    fn ld_a_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.a = memory.read_byte(addr);
        8
    }

    fn ld_b_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.b = memory.read_byte(addr);
        8
    }

    fn ld_c_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.c = memory.read_byte(addr);
        8
    }

    fn ld_d_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.d = memory.read_byte(addr);
        8
    }

    fn ld_e_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.e = memory.read_byte(addr);
        8
    }

    fn ld_h_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.h = memory.read_byte(addr);
        8
    }

    fn ld_l_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.l = memory.read_byte(addr);
        8
    }

    // LD (HL), r - Store to memory at HL (8 cycles each)
    fn ld_hl_a(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.a);
        8
    }

    fn ld_hl_b(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.b);
        8
    }

    fn ld_hl_c(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.c);
        8
    }

    fn ld_hl_d(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.d);
        8
    }

    fn ld_hl_e(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.e);
        8
    }

    fn ld_hl_h(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.h);
        8
    }

    fn ld_hl_l(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.l);
        8
    }

    fn ld_hl_n(&mut self, memory: &mut Memory) -> u8 {
        let value = self.fetch_byte(memory);
        let addr = self.registers.hl();
        memory.write_byte(addr, value);
        12
    }

    // 16-bit loads (12 cycles each)
    fn ld_bc_nn(&mut self, memory: &Memory) -> u8 {
        let value = self.fetch_word(memory);
        self.registers.set_bc(value);
        12
    }

    fn ld_de_nn(&mut self, memory: &Memory) -> u8 {
        let value = self.fetch_word(memory);
        self.registers.set_de(value);
        12
    }

    fn ld_hl_nn(&mut self, memory: &Memory) -> u8 {
        let value = self.fetch_word(memory);
        self.registers.set_hl(value);
        12
    }

    fn ld_sp_nn(&mut self, memory: &Memory) -> u8 {
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

    fn xor_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        let value = memory.read_byte(addr);
        self.registers.a ^= value;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        8
    }

    fn xor_n(&mut self, memory: &Memory) -> u8 {
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

    fn inc_hl(&mut self, memory: &mut Memory) -> u8 {
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

    fn dec_hl(&mut self, memory: &mut Memory) -> u8 {
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
    fn jp_nn(&mut self, memory: &Memory) -> u8 {
        self.pc = self.fetch_word(memory);
        16
    }

    // JR n - Relative jump by signed 8-bit offset
    fn jr_n(&mut self, memory: &Memory) -> u8 {
        let offset_16 = i16::from(
            // Game Boy JR instruction stores signed offset as a byte in memory.
            // Values 0x80-0xFF represent negative offsets in two's complement.
            // The "wrap" is intentional - we're reinterpreting the bit pattern.
            #[allow(clippy::cast_possible_wrap)]
            {
                self.fetch_byte(memory) as i8
            },
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

    // JR Z, n - Relative jump if Zero flag is set
    fn jr_z(&mut self, memory: &Memory) -> u8 {
        let offset_16 = i16::from(
            #[allow(clippy::cast_possible_wrap)]
            {
                self.fetch_byte(memory) as i8
            },
        );

        if self.registers.f.z {
            #[allow(clippy::cast_sign_loss)]
            {
                self.pc = self.pc.wrapping_add(offset_16 as u16);
            }
            12 // Taken
        } else {
            8 // Not taken
        }
    }

    // JR NZ, n - Relative jump if Zero flag is not set
    fn jr_nz(&mut self, memory: &Memory) -> u8 {
        let offset_16 = i16::from(
            #[allow(clippy::cast_possible_wrap)]
            {
                self.fetch_byte(memory) as i8
            },
        );

        if self.registers.f.z {
            8 // Not taken
        } else {
            #[allow(clippy::cast_sign_loss)]
            {
                self.pc = self.pc.wrapping_add(offset_16 as u16);
            }
            12 // Taken
        }
    }

    // JR C, n - Relative jump if Carry flag is set
    fn jr_c(&mut self, memory: &Memory) -> u8 {
        let offset_16 = i16::from(
            #[allow(clippy::cast_possible_wrap)]
            {
                self.fetch_byte(memory) as i8
            },
        );

        if self.registers.f.c {
            #[allow(clippy::cast_sign_loss)]
            {
                self.pc = self.pc.wrapping_add(offset_16 as u16);
            }
            12 // Taken
        } else {
            8 // Not taken
        }
    }

    // JR NC, n - Relative jump if Carry flag is not set
    fn jr_nc(&mut self, memory: &Memory) -> u8 {
        let offset_16 = i16::from(
            #[allow(clippy::cast_possible_wrap)]
            {
                self.fetch_byte(memory) as i8
            },
        );

        if self.registers.f.c {
            8 // Not taken
        } else {
            #[allow(clippy::cast_sign_loss)]
            {
                self.pc = self.pc.wrapping_add(offset_16 as u16);
            }
            12 // Taken
        }
    }

    // JP Z, nn - Absolute jump if Zero flag is set
    fn jp_z(&mut self, memory: &Memory) -> u8 {
        let addr = self.fetch_word(memory);
        if self.registers.f.z {
            self.pc = addr;
            16 // Taken
        } else {
            12 // Not taken
        }
    }

    // JP NZ, nn - Absolute jump if Zero flag is not set
    fn jp_nz(&mut self, memory: &Memory) -> u8 {
        let addr = self.fetch_word(memory);
        if self.registers.f.z {
            12 // Not taken
        } else {
            self.pc = addr;
            16 // Taken
        }
    }

    // JP C, nn - Absolute jump if Carry flag is set
    fn jp_c(&mut self, memory: &Memory) -> u8 {
        let addr = self.fetch_word(memory);
        if self.registers.f.c {
            self.pc = addr;
            16 // Taken
        } else {
            12 // Not taken
        }
    }

    // JP NC, nn - Absolute jump if Carry flag is not set
    fn jp_nc(&mut self, memory: &Memory) -> u8 {
        let addr = self.fetch_word(memory);
        if self.registers.f.c {
            12 // Not taken
        } else {
            self.pc = addr;
            16 // Taken
        }
    }

    // ADD A, r - Add register to A
    // Flags: Z if result is 0, N=0, H if carry from bit 3, C if carry from bit 7
    fn add_a(&mut self, value: u8) -> u8 {
        let a = self.registers.a;
        let result = a.wrapping_add(value);

        self.registers.f.z = result == 0;
        self.registers.f.n = false;
        self.registers.f.h = (a & 0x0F) + (value & 0x0F) > 0x0F;
        self.registers.f.c = u16::from(a) + u16::from(value) > 0xFF;

        self.registers.a = result;
        4
    }

    fn add_a_a(&mut self) -> u8 {
        let v = self.registers.a;
        self.add_a(v)
    }
    fn add_a_b(&mut self) -> u8 {
        let v = self.registers.b;
        self.add_a(v)
    }
    fn add_a_c(&mut self) -> u8 {
        let v = self.registers.c;
        self.add_a(v)
    }
    fn add_a_d(&mut self) -> u8 {
        let v = self.registers.d;
        self.add_a(v)
    }
    fn add_a_e(&mut self) -> u8 {
        let v = self.registers.e;
        self.add_a(v)
    }
    fn add_a_h(&mut self) -> u8 {
        let v = self.registers.h;
        self.add_a(v)
    }
    fn add_a_l(&mut self) -> u8 {
        let v = self.registers.l;
        self.add_a(v)
    }

    fn add_a_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        let value = memory.read_byte(addr);
        self.add_a(value);
        8
    }

    fn add_a_n(&mut self, memory: &Memory) -> u8 {
        let value = self.fetch_byte(memory);
        self.add_a(value);
        8
    }

    // SUB A, r - Subtract register from A
    // Flags: Z if result is 0, N=1, H if borrow from bit 4, C if borrow
    fn sub_a(&mut self, value: u8) -> u8 {
        let a = self.registers.a;
        let result = a.wrapping_sub(value);

        self.registers.f.z = result == 0;
        self.registers.f.n = true;
        self.registers.f.h = (a & 0x0F) < (value & 0x0F);
        self.registers.f.c = a < value;

        self.registers.a = result;
        4
    }

    fn sub_a_a(&mut self) -> u8 {
        let v = self.registers.a;
        self.sub_a(v)
    }
    fn sub_a_b(&mut self) -> u8 {
        let v = self.registers.b;
        self.sub_a(v)
    }
    fn sub_a_c(&mut self) -> u8 {
        let v = self.registers.c;
        self.sub_a(v)
    }
    fn sub_a_d(&mut self) -> u8 {
        let v = self.registers.d;
        self.sub_a(v)
    }
    fn sub_a_e(&mut self) -> u8 {
        let v = self.registers.e;
        self.sub_a(v)
    }
    fn sub_a_h(&mut self) -> u8 {
        let v = self.registers.h;
        self.sub_a(v)
    }
    fn sub_a_l(&mut self) -> u8 {
        let v = self.registers.l;
        self.sub_a(v)
    }

    fn sub_a_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        let value = memory.read_byte(addr);
        self.sub_a(value);
        8
    }

    fn sub_a_n(&mut self, memory: &Memory) -> u8 {
        let value = self.fetch_byte(memory);
        self.sub_a(value);
        8
    }

    // AND A, r - Bitwise AND with A
    // Flags: Z if result is 0, N=0, H=1, C=0
    fn and_a(&mut self, value: u8) -> u8 {
        self.registers.a &= value;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = true;
        self.registers.f.c = false;
        4
    }

    fn and_a_a(&mut self) -> u8 {
        let v = self.registers.a;
        self.and_a(v)
    }
    fn and_a_b(&mut self) -> u8 {
        let v = self.registers.b;
        self.and_a(v)
    }
    fn and_a_c(&mut self) -> u8 {
        let v = self.registers.c;
        self.and_a(v)
    }
    fn and_a_d(&mut self) -> u8 {
        let v = self.registers.d;
        self.and_a(v)
    }
    fn and_a_e(&mut self) -> u8 {
        let v = self.registers.e;
        self.and_a(v)
    }
    fn and_a_h(&mut self) -> u8 {
        let v = self.registers.h;
        self.and_a(v)
    }
    fn and_a_l(&mut self) -> u8 {
        let v = self.registers.l;
        self.and_a(v)
    }

    fn and_a_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        let value = memory.read_byte(addr);
        self.and_a(value);
        8
    }

    fn and_a_n(&mut self, memory: &Memory) -> u8 {
        let value = self.fetch_byte(memory);
        self.and_a(value);
        8
    }

    // OR A, r - Bitwise OR with A
    // Flags: Z if result is 0, N=0, H=0, C=0
    fn or_a(&mut self, value: u8) -> u8 {
        self.registers.a |= value;
        self.registers.f.z = self.registers.a == 0;
        self.registers.f.n = false;
        self.registers.f.h = false;
        self.registers.f.c = false;
        4
    }

    fn or_a_a(&mut self) -> u8 {
        let v = self.registers.a;
        self.or_a(v)
    }
    fn or_a_b(&mut self) -> u8 {
        let v = self.registers.b;
        self.or_a(v)
    }
    fn or_a_c(&mut self) -> u8 {
        let v = self.registers.c;
        self.or_a(v)
    }
    fn or_a_d(&mut self) -> u8 {
        let v = self.registers.d;
        self.or_a(v)
    }
    fn or_a_e(&mut self) -> u8 {
        let v = self.registers.e;
        self.or_a(v)
    }
    fn or_a_h(&mut self) -> u8 {
        let v = self.registers.h;
        self.or_a(v)
    }
    fn or_a_l(&mut self) -> u8 {
        let v = self.registers.l;
        self.or_a(v)
    }

    fn or_a_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        let value = memory.read_byte(addr);
        self.or_a(value);
        8
    }

    fn or_a_n(&mut self, memory: &Memory) -> u8 {
        let value = self.fetch_byte(memory);
        self.or_a(value);
        8
    }

    // CP A, r - Compare A with register (SUB without storing result)
    // Flags: Z if equal, N=1, H if borrow from bit 4, C if A < value
    fn cp_a(&mut self, value: u8) -> u8 {
        let a = self.registers.a;
        let result = a.wrapping_sub(value);

        self.registers.f.z = result == 0;
        self.registers.f.n = true;
        self.registers.f.h = (a & 0x0F) < (value & 0x0F);
        self.registers.f.c = a < value;
        4
    }

    fn cp_a_a(&mut self) -> u8 {
        let v = self.registers.a;
        self.cp_a(v)
    }
    fn cp_a_b(&mut self) -> u8 {
        let v = self.registers.b;
        self.cp_a(v)
    }
    fn cp_a_c(&mut self) -> u8 {
        let v = self.registers.c;
        self.cp_a(v)
    }
    fn cp_a_d(&mut self) -> u8 {
        let v = self.registers.d;
        self.cp_a(v)
    }
    fn cp_a_e(&mut self) -> u8 {
        let v = self.registers.e;
        self.cp_a(v)
    }
    fn cp_a_h(&mut self) -> u8 {
        let v = self.registers.h;
        self.cp_a(v)
    }
    fn cp_a_l(&mut self) -> u8 {
        let v = self.registers.l;
        self.cp_a(v)
    }

    fn cp_a_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        let value = memory.read_byte(addr);
        self.cp_a(value);
        8
    }

    fn cp_a_n(&mut self, memory: &Memory) -> u8 {
        let value = self.fetch_byte(memory);
        self.cp_a(value);
        8
    }

    // PUSH rr - Push 16-bit register pair onto stack (all take 16 cycles)
    // Stack grows downward: SP decrements before each write
    fn push_bc(&mut self, memory: &mut Memory) -> u8 {
        let value = self.registers.bc();
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (value >> 8) as u8); // High byte
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (value & 0xFF) as u8); // Low byte
        16
    }

    fn push_de(&mut self, memory: &mut Memory) -> u8 {
        let value = self.registers.de();
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (value & 0xFF) as u8);
        16
    }

    fn push_hl(&mut self, memory: &mut Memory) -> u8 {
        let value = self.registers.hl();
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (value & 0xFF) as u8);
        16
    }

    fn push_af(&mut self, memory: &mut Memory) -> u8 {
        let value = self.registers.af();
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (value & 0xFF) as u8);
        16
    }

    // POP rr - Pop 16-bit register pair from stack (all take 12 cycles)
    // Stack grows downward: SP increments after each read
    fn pop_bc(&mut self, memory: &Memory) -> u8 {
        let low = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.registers.set_bc((u16::from(high) << 8) | u16::from(low));
        12
    }

    fn pop_de(&mut self, memory: &Memory) -> u8 {
        let low = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.registers.set_de((u16::from(high) << 8) | u16::from(low));
        12
    }

    fn pop_hl(&mut self, memory: &Memory) -> u8 {
        let low = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.registers.set_hl((u16::from(high) << 8) | u16::from(low));
        12
    }

    fn pop_af(&mut self, memory: &Memory) -> u8 {
        let low = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.registers.set_af((u16::from(high) << 8) | u16::from(low));
        12
    }

    // CALL nn - Call subroutine (push PC, then jump to nn)
    fn call_nn(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.fetch_word(memory);
        // Push current PC onto stack
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (self.pc >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (self.pc & 0xFF) as u8);
        // Jump to address
        self.pc = addr;
        24
    }

    // RET - Return from subroutine (pop PC from stack)
    fn ret(&mut self, memory: &Memory) -> u8 {
        let low = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.pc = (u16::from(high) << 8) | u16::from(low);
        16
    }

    // CALL Z, nn - Call if Zero flag is set
    fn call_z(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.fetch_word(memory);
        if self.registers.f.z {
            self.sp = self.sp.wrapping_sub(1);
            memory.write_byte(self.sp, (self.pc >> 8) as u8);
            self.sp = self.sp.wrapping_sub(1);
            memory.write_byte(self.sp, (self.pc & 0xFF) as u8);
            self.pc = addr;
            24 // Taken
        } else {
            12 // Not taken
        }
    }

    // CALL NZ, nn - Call if Zero flag is not set
    fn call_nz(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.fetch_word(memory);
        if self.registers.f.z {
            12 // Not taken
        } else {
            self.sp = self.sp.wrapping_sub(1);
            memory.write_byte(self.sp, (self.pc >> 8) as u8);
            self.sp = self.sp.wrapping_sub(1);
            memory.write_byte(self.sp, (self.pc & 0xFF) as u8);
            self.pc = addr;
            24 // Taken
        }
    }

    // CALL C, nn - Call if Carry flag is set
    fn call_c(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.fetch_word(memory);
        if self.registers.f.c {
            self.sp = self.sp.wrapping_sub(1);
            memory.write_byte(self.sp, (self.pc >> 8) as u8);
            self.sp = self.sp.wrapping_sub(1);
            memory.write_byte(self.sp, (self.pc & 0xFF) as u8);
            self.pc = addr;
            24 // Taken
        } else {
            12 // Not taken
        }
    }

    // CALL NC, nn - Call if Carry flag is not set
    fn call_nc(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.fetch_word(memory);
        if self.registers.f.c {
            12 // Not taken
        } else {
            self.sp = self.sp.wrapping_sub(1);
            memory.write_byte(self.sp, (self.pc >> 8) as u8);
            self.sp = self.sp.wrapping_sub(1);
            memory.write_byte(self.sp, (self.pc & 0xFF) as u8);
            self.pc = addr;
            24 // Taken
        }
    }

    // RET Z - Return if Zero flag is set
    fn ret_z(&mut self, memory: &Memory) -> u8 {
        if self.registers.f.z {
            let low = memory.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            let high = memory.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            self.pc = (u16::from(high) << 8) | u16::from(low);
            20 // Taken
        } else {
            8 // Not taken
        }
    }

    // RET NZ - Return if Zero flag is not set
    fn ret_nz(&mut self, memory: &Memory) -> u8 {
        if self.registers.f.z {
            8 // Not taken
        } else {
            let low = memory.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            let high = memory.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            self.pc = (u16::from(high) << 8) | u16::from(low);
            20 // Taken
        }
    }

    // RET C - Return if Carry flag is set
    fn ret_c(&mut self, memory: &Memory) -> u8 {
        if self.registers.f.c {
            let low = memory.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            let high = memory.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            self.pc = (u16::from(high) << 8) | u16::from(low);
            20 // Taken
        } else {
            8 // Not taken
        }
    }

    // RET NC - Return if Carry flag is not set
    fn ret_nc(&mut self, memory: &Memory) -> u8 {
        if self.registers.f.c {
            8 // Not taken
        } else {
            let low = memory.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            let high = memory.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            self.pc = (u16::from(high) << 8) | u16::from(low);
            20 // Taken
        }
    }

    // RETI - Return from interrupt (same as RET but enables interrupts)
    // Note: Interrupt handling not yet implemented, so this is just like RET for now
    fn reti(&mut self, memory: &Memory) -> u8 {
        let low = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = memory.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.pc = (u16::from(high) << 8) | u16::from(low);
        // TODO: Enable interrupts when interrupt system is implemented
        16
    }

    // LD A,(BC) - Load A from memory at address BC
    fn ld_a_bc(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.bc();
        self.registers.a = memory.read_byte(addr);
        8
    }

    // LD A,(DE) - Load A from memory at address DE
    fn ld_a_de(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.de();
        self.registers.a = memory.read_byte(addr);
        8
    }

    // LD (BC),A - Store A to memory at address BC
    fn ld_bc_a(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.bc();
        memory.write_byte(addr, self.registers.a);
        8
    }

    // LD (DE),A - Store A to memory at address DE
    fn ld_de_a(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.de();
        memory.write_byte(addr, self.registers.a);
        8
    }

    // LD A,(nn) - Load A from memory at 16-bit address
    fn ld_a_nn(&mut self, memory: &Memory) -> u8 {
        let addr = self.fetch_word(memory);
        self.registers.a = memory.read_byte(addr);
        16
    }

    // LD (nn),A - Store A to memory at 16-bit address
    fn ld_nn_a(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.fetch_word(memory);
        memory.write_byte(addr, self.registers.a);
        16
    }

    // LDI (HL),A or LD (HL+),A - Store A to memory at HL, then increment HL
    fn ldi_hl_a(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.a);
        self.registers.set_hl(addr.wrapping_add(1));
        8
    }

    // LDI A,(HL) or LD A,(HL+) - Load A from memory at HL, then increment HL
    fn ldi_a_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.a = memory.read_byte(addr);
        self.registers.set_hl(addr.wrapping_add(1));
        8
    }

    // LDD (HL),A or LD (HL-),A - Store A to memory at HL, then decrement HL
    fn ldd_hl_a(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.registers.hl();
        memory.write_byte(addr, self.registers.a);
        self.registers.set_hl(addr.wrapping_sub(1));
        8
    }

    // LDD A,(HL) or LD A,(HL-) - Load A from memory at HL, then decrement HL
    fn ldd_a_hl(&mut self, memory: &Memory) -> u8 {
        let addr = self.registers.hl();
        self.registers.a = memory.read_byte(addr);
        self.registers.set_hl(addr.wrapping_sub(1));
        8
    }

    // LD (nn),SP - Store SP to memory at 16-bit address
    fn ld_nn_sp(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.fetch_word(memory);
        memory.write_word(addr, self.sp);
        20
    }

    // LD SP,HL - Load SP from HL
    fn ld_sp_hl(&mut self) -> u8 {
        self.sp = self.registers.hl();
        8
    }

    // LD HL,SP+n or LDHL SP,n - Load HL with SP + signed 8-bit offset
    // Flags: Z=0, N=0, H=carry from bit 3, C=carry from bit 7
    fn ld_hl_sp_n(&mut self, memory: &Memory) -> u8 {
        let offset = i16::from(
            #[allow(clippy::cast_possible_wrap)]
            {
                self.fetch_byte(memory) as i8
            },
        );

        let sp = self.sp;
        #[allow(clippy::cast_sign_loss)]
        let result = sp.wrapping_add(offset as u16);

        // For LD HL,SP+n, half-carry and carry are calculated on the lower byte
        let sp_low = (sp & 0xFF) as u8;
        #[allow(clippy::cast_sign_loss)]
        let offset_low = (offset & 0xFF) as u8;

        self.registers.f.z = false;
        self.registers.f.n = false;
        self.registers.f.h = (sp_low & 0x0F) + (offset_low & 0x0F) > 0x0F;
        self.registers.f.c = u16::from(sp_low) + u16::from(offset_low) > 0xFF;

        self.registers.set_hl(result);
        12
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
