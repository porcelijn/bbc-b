use crate::devices::{Device, DevicePage};
use crate::memory::{Address, MemoryBus};

pub struct VideoULA {
    control: u8,
    palette: [u8; 16],
    palette_addr: u8,
}

impl VideoULA {
    pub const fn new() -> Self {
        VideoULA {
            control: 0,
            palette: [
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F,
            ],
            palette_addr: 0,
        }
    }

    pub fn mode(&self) -> u8 {
        self.control & 0x07
    }

    pub fn palette_entry(&self, index: usize) -> u8 {
        self.palette[index & 0x0F]
    }

    pub fn pixels_per_byte(&self) -> u8 {
        match self.mode() {
            0 | 3 | 4 | 6 => 8,
            1 | 5 => 4,
            2 => 2,
            _ => 8,
        }
    }

    pub fn colors(&self) -> u8 {
        match self.mode() {
            0 | 3 | 4 | 6 => 2,
            1 | 5 => 4,
            2 => 16,
            _ => 2,
        }
    }
}

impl Default for VideoULA {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for VideoULA {
    fn name(&self) -> &'static str {
        "Video ULA"
    }
}

impl MemoryBus for VideoULA {
    fn read(&self, address: Address) -> u8 {
        let offset = address.lo_u8();
        match offset {
            0x20 => self.control,
            0x21 => self.palette_addr,
            0x22..=0x2F => {
                let idx = ((offset - 0x22) as usize) & 0x0F;
                self.palette[idx]
            }
            _ => 0,
        }
    }

    fn write(&mut self, address: Address, value: u8) {
        let offset = address.lo_u8();
        match offset {
            0x20 => self.control = value,
            0x21 => self.palette_addr = value & 0x0F,
            0x22..=0x2F => {
                let idx = ((offset - 0x22) as usize) & 0x0F;
                self.palette[idx] = value & 0x0F;
            }
            _ => {}
        }
    }
}

impl DevicePage<0xFE> for VideoULA {}
