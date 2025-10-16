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
    #[allow(clippy::too_many_lines)]
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

            // Load A from memory at register pairs
            0x0A => self.ld_a_bc(memory),
            0x1A => self.ld_a_de(memory),
            0xFA => self.ld_a_nn(memory),

            // Store A to memory at register pairs
            0x02 => self.ld_bc_a(memory),
            0x12 => self.ld_de_a(memory),
            0xEA => self.ld_nn_a(memory),

            // LDI/LDD - Load with increment/decrement
            0x22 => self.ldi_hl_a(memory),
            0x2A => self.ldi_a_hl(memory),
            0x32 => self.ldd_hl_a(memory),
            0x3A => self.ldd_a_hl(memory),

            // SP-related loads
            0x08 => self.ld_nn_sp(memory),
            0xF9 => self.ld_sp_hl(),
            0xF8 => self.ld_hl_sp_n(memory),

            // HALT
            0x76 => self.halt(),

            // Rotate/shift instructions
            0x07 => self.rlca(),
            0x0F => self.rrca(),
            0x17 => self.rla(),
            0x1F => self.rra(),

            // Miscellaneous
            0x27 => self.daa(),
            0x2F => self.cpl(),
            0x37 => self.scf(),
            0x3F => self.ccf(),

            // Interrupt control
            0xF3 => self.di(),
            0xFB => self.ei(),

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

            // Conditional relative jumps
            0x28 => self.jr_z(memory),
            0x20 => self.jr_nz(memory),
            0x38 => self.jr_c(memory),
            0x30 => self.jr_nc(memory),

            // Conditional absolute jumps
            0xCA => self.jp_z(memory),
            0xC2 => self.jp_nz(memory),
            0xDA => self.jp_c(memory),
            0xD2 => self.jp_nc(memory),

            // ADD A, r
            0x87 => self.add_a_a(),
            0x80 => self.add_a_b(),
            0x81 => self.add_a_c(),
            0x82 => self.add_a_d(),
            0x83 => self.add_a_e(),
            0x84 => self.add_a_h(),
            0x85 => self.add_a_l(),
            0x86 => self.add_a_hl(memory),
            0xC6 => self.add_a_n(memory),

            // SUB A, r
            0x97 => self.sub_a_a(),
            0x90 => self.sub_a_b(),
            0x91 => self.sub_a_c(),
            0x92 => self.sub_a_d(),
            0x93 => self.sub_a_e(),
            0x94 => self.sub_a_h(),
            0x95 => self.sub_a_l(),
            0x96 => self.sub_a_hl(memory),
            0xD6 => self.sub_a_n(memory),

            // AND A, r
            0xA7 => self.and_a_a(),
            0xA0 => self.and_a_b(),
            0xA1 => self.and_a_c(),
            0xA2 => self.and_a_d(),
            0xA3 => self.and_a_e(),
            0xA4 => self.and_a_h(),
            0xA5 => self.and_a_l(),
            0xA6 => self.and_a_hl(memory),
            0xE6 => self.and_a_n(memory),

            // OR A, r
            0xB7 => self.or_a_a(),
            0xB0 => self.or_a_b(),
            0xB1 => self.or_a_c(),
            0xB2 => self.or_a_d(),
            0xB3 => self.or_a_e(),
            0xB4 => self.or_a_h(),
            0xB5 => self.or_a_l(),
            0xB6 => self.or_a_hl(memory),
            0xF6 => self.or_a_n(memory),

            // CP A, r
            0xBF => self.cp_a_a(),
            0xB8 => self.cp_a_b(),
            0xB9 => self.cp_a_c(),
            0xBA => self.cp_a_d(),
            0xBB => self.cp_a_e(),
            0xBC => self.cp_a_h(),
            0xBD => self.cp_a_l(),
            0xBE => self.cp_a_hl(memory),
            0xFE => self.cp_a_n(memory),

            // ADC A, r
            0x8F => self.adc_a_a(),
            0x88 => self.adc_a_b(),
            0x89 => self.adc_a_c(),
            0x8A => self.adc_a_d(),
            0x8B => self.adc_a_e(),
            0x8C => self.adc_a_h(),
            0x8D => self.adc_a_l(),
            0x8E => self.adc_a_hl(memory),
            0xCE => self.adc_a_n(memory),

            // SBC A, r
            0x9F => self.sbc_a_a(),
            0x98 => self.sbc_a_b(),
            0x99 => self.sbc_a_c(),
            0x9A => self.sbc_a_d(),
            0x9B => self.sbc_a_e(),
            0x9C => self.sbc_a_h(),
            0x9D => self.sbc_a_l(),
            0x9E => self.sbc_a_hl(memory),
            0xDE => self.sbc_a_n(memory),

            // RST - Restart (call to fixed address)
            0xC7 => self.rst_00(memory),
            0xCF => self.rst_08(memory),
            0xD7 => self.rst_10(memory),
            0xDF => self.rst_18(memory),
            0xE7 => self.rst_20(memory),
            0xEF => self.rst_28(memory),
            0xF7 => self.rst_30(memory),
            0xFF => self.rst_38(memory),

            // Stack operations - PUSH
            0xC5 => self.push_bc(memory),
            0xD5 => self.push_de(memory),
            0xE5 => self.push_hl(memory),
            0xF5 => self.push_af(memory),

            // Stack operations - POP
            0xC1 => self.pop_bc(memory),
            0xD1 => self.pop_de(memory),
            0xE1 => self.pop_hl(memory),
            0xF1 => self.pop_af(memory),

            // CALL instructions
            0xCD => self.call_nn(memory),
            0xCC => self.call_z(memory),
            0xC4 => self.call_nz(memory),
            0xDC => self.call_c(memory),
            0xD4 => self.call_nc(memory),

            // RET instructions
            0xC9 => self.ret(memory),
            0xC8 => self.ret_z(memory),
            0xC0 => self.ret_nz(memory),
            0xD8 => self.ret_c(memory),
            0xD0 => self.ret_nc(memory),
            0xD9 => self.reti(memory),

            // CB-prefixed instructions
            0xCB => {
                let cb_opcode = self.fetch_byte(memory);
                self.execute_cb_opcode(cb_opcode, memory)
            }

            _ => panic!(
                "Unimplemented opcode: 0x{:02X} at PC: 0x{:04X}",
                opcode,
                self.pc.wrapping_sub(1)
            ),
        }
    }

    /// Execute a CB-prefixed opcode
    #[allow(clippy::too_many_lines)]
    fn execute_cb_opcode(&mut self, opcode: u8, memory: &mut Memory) -> u8 {
        match opcode {
            // RLC r - Rotate left with carry
            0x00 => self.rlc_b(),
            0x01 => self.rlc_c(),
            0x02 => self.rlc_d(),
            0x03 => self.rlc_e(),
            0x04 => self.rlc_h(),
            0x05 => self.rlc_l(),
            0x06 => self.rlc_hl(memory),
            0x07 => self.rlc_a(),

            // RRC r - Rotate right with carry
            0x08 => self.rrc_b(),
            0x09 => self.rrc_c(),
            0x0A => self.rrc_d(),
            0x0B => self.rrc_e(),
            0x0C => self.rrc_h(),
            0x0D => self.rrc_l(),
            0x0E => self.rrc_hl(memory),
            0x0F => self.rrc_a(),

            // RL r - Rotate left through carry
            0x10 => self.rl_b(),
            0x11 => self.rl_c(),
            0x12 => self.rl_d(),
            0x13 => self.rl_e(),
            0x14 => self.rl_h(),
            0x15 => self.rl_l(),
            0x16 => self.rl_hl(memory),
            0x17 => self.rl_a(),

            // RR r - Rotate right through carry
            0x18 => self.rr_b(),
            0x19 => self.rr_c(),
            0x1A => self.rr_d(),
            0x1B => self.rr_e(),
            0x1C => self.rr_h(),
            0x1D => self.rr_l(),
            0x1E => self.rr_hl(memory),
            0x1F => self.rr_a(),

            // SLA r - Shift left arithmetic
            0x20 => self.sla_b(),
            0x21 => self.sla_c(),
            0x22 => self.sla_d(),
            0x23 => self.sla_e(),
            0x24 => self.sla_h(),
            0x25 => self.sla_l(),
            0x26 => self.sla_hl(memory),
            0x27 => self.sla_a(),

            // SRA r - Shift right arithmetic (preserve sign bit)
            0x28 => self.sra_b(),
            0x29 => self.sra_c(),
            0x2A => self.sra_d(),
            0x2B => self.sra_e(),
            0x2C => self.sra_h(),
            0x2D => self.sra_l(),
            0x2E => self.sra_hl(memory),
            0x2F => self.sra_a(),

            // SWAP r - Swap nibbles
            0x30 => self.swap_b(),
            0x31 => self.swap_c(),
            0x32 => self.swap_d(),
            0x33 => self.swap_e(),
            0x34 => self.swap_h(),
            0x35 => self.swap_l(),
            0x36 => self.swap_hl(memory),
            0x37 => self.swap_a(),

            // SRL r - Shift right logical
            0x38 => self.srl_b(),
            0x39 => self.srl_c(),
            0x3A => self.srl_d(),
            0x3B => self.srl_e(),
            0x3C => self.srl_h(),
            0x3D => self.srl_l(),
            0x3E => self.srl_hl(memory),
            0x3F => self.srl_a(),

            // BIT b,r - Test bit b in register r
            0x40..=0x7F => self.bit(opcode, memory),

            // RES b,r - Reset bit b in register r
            0x80..=0xBF => self.res(opcode, memory),

            // SET b,r - Set bit b in register r
            0xC0..=0xFF => self.set(opcode, memory),
        }
    }
}
