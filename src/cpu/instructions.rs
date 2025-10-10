use super::Cpu;
use crate::memory::Memory;

impl Cpu {
    /// Fetch the next byte and increment PC
    pub fn fetch_byte(&mut self, memory: &Memory) -> u8 {
        let byte = memory.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    /// Fetch the next word (16-bit) and increment PC by 2
    pub fn fetch_word(&mut self, memory: &Memory) -> u16 {
        let word = memory.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        word
    }

    /// Execute one instruction and return cycles taken
    pub fn execute(&mut self, memory: &mut Memory) -> u8 {
        let opcode = self.fetch_byte(memory);
        self.execute_opcode(opcode, memory)
    }

    /// Execute a single opcode
    fn execute_opcode(&mut self, opcode: u8, memory: &mut Memory) -> u8 {
        match opcode {
            // NOP
            0x00 => self.nop(),

            // LD r, r' (8-bit register to register loads)
            0x7F => self.ld_a_a(),
            0x78 => self.ld_a_b(),
            0x79 => self.ld_a_c(),
            0x7A => self.ld_a_d(),
            0x7B => self.ld_a_e(),
            0x7C => self.ld_a_h(),
            0x7D => self.ld_a_l(),

            0x47 => self.ld_b_a(),
            0x40 => self.ld_b_b(),
            0x41 => self.ld_b_c(),
            0x42 => self.ld_b_d(),
            0x43 => self.ld_b_e(),
            0x44 => self.ld_b_h(),
            0x45 => self.ld_b_l(),

            0x4F => self.ld_c_a(),
            0x48 => self.ld_c_b(),
            0x49 => self.ld_c_c(),
            0x4A => self.ld_c_d(),
            0x4B => self.ld_c_e(),
            0x4C => self.ld_c_h(),
            0x4D => self.ld_c_l(),

            // LD r, n (8-bit immediate to register)
            0x3E => self.ld_a_n(memory),
            0x06 => self.ld_b_n(memory),
            0x0E => self.ld_c_n(memory),
            0x16 => self.ld_d_n(memory),
            0x1E => self.ld_e_n(memory),
            0x26 => self.ld_h_n(memory),
            0x2E => self.ld_l_n(memory),

            // LD r, (HL) (load from memory at HL)
            0x7E => self.ld_a_hl(memory),
            0x46 => self.ld_b_hl(memory),
            0x4E => self.ld_c_hl(memory),
            0x56 => self.ld_d_hl(memory),
            0x5E => self.ld_e_hl(memory),
            0x66 => self.ld_h_hl(memory),
            0x6E => self.ld_l_hl(memory),

            // LD (HL), r (store to memory at HL)
            0x77 => self.ld_hl_a(memory),
            0x70 => self.ld_hl_b(memory),
            0x71 => self.ld_hl_c(memory),
            0x72 => self.ld_hl_d(memory),
            0x73 => self.ld_hl_e(memory),
            0x74 => self.ld_hl_h(memory),
            0x75 => self.ld_hl_l(memory),

            // LD (HL), n
            0x36 => self.ld_hl_n(memory),

            // 16-bit loads
            0x01 => self.ld_bc_nn(memory),
            0x11 => self.ld_de_nn(memory),
            0x21 => self.ld_hl_nn(memory),
            0x31 => self.ld_sp_nn(memory),

            // HALT
            0x76 => self.halt(),

            // XOR operations
            0xAF => self.xor_a(),
            0xA8 => self.xor_b(),
            0xA9 => self.xor_c(),
            0xAA => self.xor_d(),
            0xAB => self.xor_e(),
            0xAC => self.xor_h(),
            0xAD => self.xor_l(),
            0xAE => self.xor_hl(memory),
            0xEE => self.xor_n(memory),

            // INC 8-bit
            0x3C => self.inc_a(),
            0x04 => self.inc_b(),
            0x0C => self.inc_c(),
            0x14 => self.inc_d(),
            0x1C => self.inc_e(),
            0x24 => self.inc_h(),
            0x2C => self.inc_l(),
            0x34 => self.inc_hl(memory),

            // DEC 8-bit
            0x3D => self.dec_a(),
            0x05 => self.dec_b(),
            0x0D => self.dec_c(),
            0x15 => self.dec_d(),
            0x1D => self.dec_e(),
            0x25 => self.dec_h(),
            0x2D => self.dec_l(),
            0x35 => self.dec_hl(memory),

            // INC 16-bit
            0x03 => self.inc_bc(),
            0x13 => self.inc_de(),
            0x23 => self.inc_hl_16(),
            0x33 => self.inc_sp(),

            // DEC 16-bit
            0x0B => self.dec_bc(),
            0x1B => self.dec_de(),
            0x2B => self.dec_hl_16(),
            0x3B => self.dec_sp(),

            // Jump instructions
            0xC3 => self.jp_nn(memory),
            0x18 => self.jr_n(memory),

            _ => panic!(
                "Unimplemented opcode: 0x{:02X} at PC: 0x{:04X}",
                opcode,
                self.pc.wrapping_sub(1)
            ),
        }
    }
}
