use crate::cartridge::Cartridge;

const MEMORY_SIZE: usize = 0x10000; // 64KB

pub struct Memory {
    data: Vec<u8>,
    cartridge: Option<Cartridge>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: vec![0; MEMORY_SIZE],
            cartridge: None,
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
