/// CPU Flags register
#[derive(Debug, Clone, Copy)]
pub struct Flags {
    pub z: bool,  // Zero flag
    pub n: bool,  // Subtraction flag (BCD)
    pub h: bool,  // Half-carry flag (BCD)
    pub c: bool,  // Carry flag
}

impl Flags {
    pub fn new() -> Self {
        Self {
            z: false,
            n: false,
            h: false,
            c: false,
        }
    }

    /// Convert flags to u8 (lower 4 bits always 0)
    pub fn to_u8(&self) -> u8 {
        (if self.z { 0b1000_0000 } else { 0 }) |
            (if self.n { 0b0100_0000 } else { 0 }) |
            (if self.h { 0b0010_0000 } else { 0 }) |
            (if self.c { 0b0001_0000 } else { 0 })
    }

    /// Set flags from u8
    pub fn set_from_u8(&mut self, value: u8) {
        self.z = (value & 0b1000_0000) != 0;
        self.n = (value & 0b0100_0000) != 0;
        self.h = (value & 0b0010_0000) != 0;
        self.c = (value & 0b0001_0000) != 0;
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU Registers
#[derive(Debug)]
pub struct Registers {
    pub a: u8,
    pub f: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0x01,  // DMG initial values
            f: Flags::default(),
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
        }
    }

    // 16-bit register pair getters
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f.to_u8() as u16)
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    // 16-bit register pair setters
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f.set_from_u8((value & 0xFF) as u8);
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}