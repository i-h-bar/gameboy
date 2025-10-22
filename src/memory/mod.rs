use crate::cartridge::Cartridge;
use crate::timer::Timer;

const MEMORY_SIZE: usize = 0x10000; // 64KB

pub struct Memory {
    pub data: Vec<u8>,
    pub cartridge: Option<Cartridge>,
    pub timer: Timer,
}

#[allow(clippy::match_same_arms)] // Temporary whilst developing
impl Memory {
    pub fn new() -> Self {
        Self {
            data: vec![0; MEMORY_SIZE],
            cartridge: None,
            timer: Timer::default(),
        }
    }

    /// Load a cartridge into memory
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge);
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // Cartridge ROM Bank 0 (0x0000-0x3FFF)
            0x0000..=0x3FFF => {
                if let Some(ref cart) = self.cartridge {
                    cart.read_byte(address)
                } else {
                    self.data[address as usize]
                }
            }

            // Cartridge ROM Bank 1-N (0x4000-0x7FFF)
            0x4000..=0x7FFF => {
                if let Some(ref cart) = self.cartridge {
                    cart.read_byte(address)
                } else {
                    self.data[address as usize]
                }
            }

            // Video RAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => self.data[address as usize],

            // External RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if let Some(ref cart) = self.cartridge {
                    cart.read_byte(address)
                } else {
                    self.data[address as usize]
                }
            }

            // Timer
            0xFF04..=0xFF07 => self.timer.read_register(address),

            // Work RAM, Echo RAM, OAM, I/O, HRAM (0xC000-0xFFFF)
            0xC000..=0xFFFF => self.data[address as usize],
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // Cartridge ROM area (0x0000-0x7FFF) - MBC control writes
            0x0000..=0x7FFF => {
                if let Some(ref mut cart) = self.cartridge {
                    cart.write_byte(address, value);
                } else {
                    // No cartridge - allow writes for testing
                    self.data[address as usize] = value;
                }
            }

            // Video RAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => {
                self.data[address as usize] = value;
            }

            // External RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if let Some(ref mut cart) = self.cartridge {
                    cart.write_byte(address, value);
                } else {
                    self.data[address as usize] = value;
                }
            }

            // Timer
            0xFF04..=0xFF07 => self.timer.write_register(address, value),

            // Work RAM, Echo RAM, OAM, I/O, HRAM (0xC000-0xFFFF)
            0xC000..=0xFFFF => {
                self.data[address as usize] = value;
            }
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let low = u16::from(self.read_byte(address));
        let high = u16::from(self.read_byte(address.wrapping_add(1)));
        (high << 8) | low
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address.wrapping_add(1), (value >> 8) as u8);
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod timer_registers {
        use super::*;

        #[test]
        fn read_div_register_at_0xff04() {
            let memory = Memory::new();

            // DIV starts at 0
            let value = memory.read_byte(0xFF04);
            assert_eq!(value, 0x00, "DIV register should be accessible at 0xFF04");
        }

        #[test]
        fn read_tima_register_at_0xff05() {
            let memory = Memory::new();

            let value = memory.read_byte(0xFF05);
            assert_eq!(value, 0x00, "TIMA register should be accessible at 0xFF05");
        }

        #[test]
        fn read_tma_register_at_0xff06() {
            let memory = Memory::new();

            let value = memory.read_byte(0xFF06);
            assert_eq!(value, 0x00, "TMA register should be accessible at 0xFF06");
        }

        #[test]
        fn read_tac_register_at_0xff07() {
            let memory = Memory::new();

            let value = memory.read_byte(0xFF07);
            assert_eq!(value, 0x00, "TAC register should be accessible at 0xFF07");
        }

        #[test]
        fn write_div_register_at_0xff04() {
            let mut memory = Memory::new();

            // Writing to DIV should reset it (timer behavior)
            memory.write_byte(0xFF04, 0xFF);
            let value = memory.read_byte(0xFF04);
            assert_eq!(value, 0x00, "Writing to 0xFF04 should reset DIV");
        }

        #[test]
        fn write_tima_register_at_0xff05() {
            let mut memory = Memory::new();

            memory.write_byte(0xFF05, 0x42);
            let value = memory.read_byte(0xFF05);
            assert_eq!(value, 0x42, "Writing to 0xFF05 should set TIMA");
        }

        #[test]
        fn write_tma_register_at_0xff06() {
            let mut memory = Memory::new();

            memory.write_byte(0xFF06, 0xAB);
            let value = memory.read_byte(0xFF06);
            assert_eq!(value, 0xAB, "Writing to 0xFF06 should set TMA");
        }

        #[test]
        fn write_tac_register_at_0xff07() {
            let mut memory = Memory::new();

            memory.write_byte(0xFF07, 0x05);
            let value = memory.read_byte(0xFF07);
            assert_eq!(value, 0x05, "Writing to 0xFF07 should set TAC");
        }

        #[test]
        fn all_timer_registers_accessible() {
            let mut memory = Memory::new();

            memory.write_byte(0xFF04, 0xFF); // DIV (resets to 0)
            memory.write_byte(0xFF05, 0x11);
            memory.write_byte(0xFF06, 0x22);
            memory.write_byte(0xFF07, 0x33);

            assert_eq!(memory.read_byte(0xFF04), 0x00);
            assert_eq!(memory.read_byte(0xFF05), 0x11);
            assert_eq!(memory.read_byte(0xFF06), 0x22);
            assert_eq!(memory.read_byte(0xFF07), 0x33);
        }

        #[test]
        fn timer_registers_dont_affect_adjacent_memory() {
            let mut memory = Memory::new();

            // Write to addresses around timer registers
            memory.write_byte(0xFF03, 0xAA);
            memory.write_byte(0xFF08, 0xBB);

            // Write to timer registers
            memory.write_byte(0xFF05, 0x50);
            memory.write_byte(0xFF06, 0x60);

            // Adjacent memory should be unchanged
            assert_eq!(
                memory.read_byte(0xFF03),
                0xAA,
                "0xFF03 should not be affected by timer"
            );
            assert_eq!(
                memory.read_byte(0xFF08),
                0xBB,
                "0xFF08 should not be affected by timer"
            );

            // Timer registers should work correctly
            assert_eq!(memory.read_byte(0xFF05), 0x50);
            assert_eq!(memory.read_byte(0xFF06), 0x60);
        }

        #[test]
        fn address_0xff03_is_not_timer() {
            let mut memory = Memory::new();

            memory.write_byte(0xFF03, 0x99);
            assert_eq!(
                memory.read_byte(0xFF03),
                0x99,
                "0xFF03 should use regular memory, not timer"
            );
        }

        #[test]
        fn address_0xff08_is_not_timer() {
            let mut memory = Memory::new();

            memory.write_byte(0xFF08, 0x88);
            assert_eq!(
                memory.read_byte(0xFF08),
                0x88,
                "0xFF08 should use regular memory, not timer"
            );
        }

        #[test]
        fn timer_range_boundaries() {
            let mut memory = Memory::new();

            // Test exact boundaries of timer range (0xFF04-0xFF07)

            // 0xFF04 should route to timer
            memory.write_byte(0xFF04, 0x01);
            assert_eq!(
                memory.read_byte(0xFF04),
                0x00,
                "0xFF04 is DIV (resets on write)"
            );

            // 0xFF07 should route to timer
            memory.write_byte(0xFF07, 0x07);
            assert_eq!(memory.read_byte(0xFF07), 0x07, "0xFF07 is TAC");

            // 0xFF03 should NOT route to timer
            memory.write_byte(0xFF03, 0x03);
            assert_eq!(memory.read_byte(0xFF03), 0x03, "0xFF03 is regular memory");

            // 0xFF08 should NOT route to timer
            memory.write_byte(0xFF08, 0x08);
            assert_eq!(memory.read_byte(0xFF08), 0x08, "0xFF08 is regular memory");
        }

        #[test]
        fn multiple_sequential_timer_operations() {
            let mut memory = Memory::new();

            // Simulate multiple timer register accesses
            for i in 0..10 {
                memory.write_byte(0xFF05, i);
                assert_eq!(memory.read_byte(0xFF05), i);
            }

            memory.write_byte(0xFF06, 0x50);
            memory.write_byte(0xFF07, 0x05);

            assert_eq!(memory.read_byte(0xFF05), 9);
            assert_eq!(memory.read_byte(0xFF06), 0x50);
            assert_eq!(memory.read_byte(0xFF07), 0x05);
        }
    }
}
