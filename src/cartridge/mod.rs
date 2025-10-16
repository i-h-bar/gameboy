use std::fs;
use std::io;
use std::path::Path;

/// Cartridge types based on header byte 0x0147
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    // Add more as needed
    Unknown(u8),
}

impl From<u8> for CartridgeType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::Mbc1,
            0x02 => CartridgeType::Mbc1Ram,
            0x03 => CartridgeType::Mbc1RamBattery,
            _ => CartridgeType::Unknown(value),
        }
    }
}

/// Cartridge header information
#[derive(Debug)]
pub struct CartridgeHeader {
    pub title: String,
    pub cartridge_type: CartridgeType,
    pub rom_size: usize,
    pub ram_size: usize,
}

impl CartridgeHeader {
    /// Parse header from ROM data
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() < 0x0150 {
            return Err("ROM too small to contain valid header".to_string());
        }

        // Title at 0x0134-0x0143 (16 bytes)
        let title_bytes = &rom[0x0134..=0x0143];
        let title = String::from_utf8_lossy(title_bytes)
            .trim_end_matches('\0')
            .to_string();

        // Cartridge type at 0x0147
        let cartridge_type = CartridgeType::from(rom[0x0147]);

        // ROM size at 0x0148 (32KB << value)
        let rom_banks = match rom[0x0148] {
            0x00 => 2,   // 32KB (no banking)
            0x01 => 4,   // 64KB
            0x02 => 8,   // 128KB
            0x03 => 16,  // 256KB
            0x04 => 32,  // 512KB
            0x05 => 64,  // 1MB
            0x06 => 128, // 2MB
            0x07 => 256, // 4MB
            0x08 => 512, // 8MB
            _ => return Err(format!("Invalid ROM size: 0x{:02X}", rom[0x0148])),
        };
        let rom_size = rom_banks * 16384; // 16KB per bank

        // RAM size at 0x0149
        let ram_size = match rom[0x0149] {
            0x00 => 0,      // No RAM
            0x01 => 2048,   // 2KB (unused)
            0x02 => 8192,   // 8KB (1 bank)
            0x03 => 32768,  // 32KB (4 banks)
            0x04 => 131072, // 128KB (16 banks)
            0x05 => 65536,  // 64KB (8 banks)
            _ => return Err(format!("Invalid RAM size: 0x{:02X}", rom[0x0149])),
        };

        Ok(CartridgeHeader {
            title,
            cartridge_type,
            rom_size,
            ram_size,
        })
    }
}

/// Main cartridge structure
pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    header: CartridgeHeader,
    rom_bank: usize, // Current ROM bank (for MBC)
    ram_bank: usize, // Current RAM bank (for MBC)
    ram_enabled: bool,
    banking_mode: u8, // 0 = ROM banking, 1 = RAM banking (MBC1)
}

impl Cartridge {
    /// Load a cartridge from a file
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let rom = fs::read(path)?;

        let header = CartridgeHeader::from_rom(&rom)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        println!("Loaded ROM: {}", header.title);
        println!("Type: {:?}", header.cartridge_type);
        println!("ROM size: {} bytes ({} banks)", header.rom_size, header.rom_size / 16384);
        println!("RAM size: {} bytes", header.ram_size);

        let ram = vec![0; header.ram_size];

        Ok(Cartridge {
            rom,
            ram,
            header,
            rom_bank: 1, // Bank 1 is the default switchable bank
            ram_bank: 0,
            ram_enabled: false,
            banking_mode: 0,
        })
    }

    /// Read a byte from the cartridge
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // ROM Bank 0 (0x0000-0x3FFF)
            0x0000..=0x3FFF => self.rom[addr as usize],

            // ROM Bank 1-N (0x4000-0x7FFF) - switchable
            0x4000..=0x7FFF => {
                let offset = (self.rom_bank * 0x4000) + (addr as usize - 0x4000);
                if offset < self.rom.len() {
                    self.rom[offset]
                } else {
                    0xFF // Out of bounds
                }
            }

            // External RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if self.ram_enabled && !self.ram.is_empty() {
                    let offset = (self.ram_bank * 0x2000) + (addr as usize - 0xA000);
                    if offset < self.ram.len() {
                        self.ram[offset]
                    } else {
                        0xFF
                    }
                } else {
                    0xFF
                }
            }

            _ => 0xFF,
        }
    }

    /// Write a byte to the cartridge (for MBC control)
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match self.header.cartridge_type {
            CartridgeType::RomOnly => {} // No writes for ROM-only

            CartridgeType::Mbc1 | CartridgeType::Mbc1Ram | CartridgeType::Mbc1RamBattery => {
                self.write_mbc1(addr, value);
            }

            _ => {} // Other MBC types not implemented yet
        }
    }

    /// Handle MBC1 writes
    fn write_mbc1(&mut self, addr: u16, value: u8) {
        match addr {
            // RAM Enable (0x0000-0x1FFF)
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }

            // ROM Bank Number (0x2000-0x3FFF)
            0x2000..=0x3FFF => {
                let bank = (value & 0x1F) as usize;
                // Bank 0 is not allowed, use bank 1 instead
                self.rom_bank = if bank == 0 { 1 } else { bank };
            }

            // RAM Bank Number or Upper Bits of ROM Bank (0x4000-0x5FFF)
            0x4000..=0x5FFF => {
                let bank = (value & 0x03) as usize;
                if self.banking_mode == 0 {
                    // ROM banking mode - upper bits of ROM bank
                    self.rom_bank = (self.rom_bank & 0x1F) | (bank << 5);
                } else {
                    // RAM banking mode
                    self.ram_bank = bank;
                }
            }

            // Banking Mode Select (0x6000-0x7FFF)
            0x6000..=0x7FFF => {
                self.banking_mode = value & 0x01;
            }

            // External RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if self.ram_enabled && !self.ram.is_empty() {
                    let offset = (self.ram_bank * 0x2000) + (addr as usize - 0xA000);
                    if offset < self.ram.len() {
                        self.ram[offset] = value;
                    }
                }
            }

            _ => {}
        }
    }

    /// Get cartridge header
    pub fn header(&self) -> &CartridgeHeader {
        &self.header
    }
}
