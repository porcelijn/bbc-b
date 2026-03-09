// Motorola 6845 video controller

use std::rc::Rc;

use crate::devices::{Clocked, Device, Signal};
use crate::memory::{Address, MemoryBus};

//  &00–&07 6845 CRTC Video controller 18
#[derive(Debug)]
pub struct CRTC {
    pub vsync: Rc<Signal>,
    pub b_em_vsync: Rc<Signal>,
    clock_us: u64,
    reg_select: u8,
    regs: [u8; 18],
    h_counter: u16,
    v_counter: u16,
    r_counter: u8,
    display_start: u16,
}

impl CRTC {
    const FIFTY_HERZ: u64 = 20_000;

    pub fn new() -> Self {
        let vsync = Rc::new(Signal::new());
        let b_em_vsync = Rc::new(Signal::new());
        let clock_us = 0;
        let mut regs = [0u8; 18];
        regs[0] = 63; // R0: Horizontal total = 64 chars
        regs[1] = 40; // R1: Horizontal displayed = 40 chars
        regs[2] = 46; // R2: Horizontal sync position
        regs[3] = 0x8E; // R3: Sync widths (HS=8, VS=16)
        regs[4] = 38; // R4: Vertical total = 39 rows
        regs[5] = 0; // R5: Vertical total adjust
        regs[6] = 32; // R6: Vertical displayed = 32 rows
        regs[7] = 34; // R7: Vertical sync position
        regs[8] = 0; // R8: Interlace mode
        regs[9] = 7; // R9: Max raster address (8 rasters per char)
        CRTC {
            vsync,
            b_em_vsync,
            clock_us,
            reg_select: 0,
            regs,
            h_counter: 0,
            v_counter: 0,
            r_counter: 0,
            display_start: 0x3000 >> 1,
        }
    }

    pub fn horizontal_total(&self) -> u16 {
        (self.regs[0] as u16) + 1
    }
    pub fn horizontal_displayed(&self) -> u16 {
        self.regs[1] as u16
    }
    pub fn horizontal_sync_pos(&self) -> u16 {
        self.regs[2] as u16
    }
    pub fn vertical_total(&self) -> u16 {
        ((self.regs[4] as u16) & 0xFF) | (((self.regs[5] as u16) & 0x20) << 3)
    }
    pub fn vertical_displayed(&self) -> u16 {
        self.regs[6] as u16
    }
    pub fn vertical_sync_pos(&self) -> u16 {
        self.regs[7] as u16
    }
    pub fn max_raster(&self) -> u8 {
        self.regs[9] & 0x1F
    }
    pub fn interlace(&self) -> u8 {
        self.regs[8] & 0x03
    }

    pub fn display_start_address(&self) -> u16 {
        ((self.regs[12] as u16) << 8) | ((self.regs[13] as u16) & 0xFF)
    }

    pub fn set_display_start_address(&mut self, addr: u16) {
        self.regs[12] = (addr >> 8) as u8;
        self.regs[13] = (addr & 0xFF) as u8;
        self.display_start = addr >> 1;
    }

    pub fn is_display_enabled(&self) -> bool {
        let h = self.h_counter;
        let v = self.v_counter;
        h < self.horizontal_displayed() && v < self.vertical_displayed()
    }

    pub fn is_hsync(&self) -> bool {
        let h = self.h_counter;
        let sync_pos = self.horizontal_sync_pos();
        let sync_width = (self.regs[3] & 0x0F) as u16;
        h >= sync_pos && h < sync_pos + sync_width
    }

    pub fn is_vsync(&self) -> bool {
        let v = self.v_counter;
        let sync_pos = self.vertical_sync_pos();
        let sync_width = ((self.regs[3] >> 4) & 0x0F) as u16;
        v >= sync_pos && v < sync_pos + sync_width
    }

    pub fn memory_address(&self) -> u16 {
        let base = self.display_start;
        let char_addr = (self.v_counter * self.horizontal_displayed()) + self.h_counter;
        let raster_offset = self.r_counter as u16;
        (base + char_addr + raster_offset) & 0x3FFF
    }
}

impl Device for CRTC {
    fn name(&self) -> &'static str {
        "6845 CRTC video controller"
    }
}

impl MemoryBus for CRTC {
    fn read(&self, address: Address) -> u8 {
        let offset = address.lo_u8() & 0x01;
        if offset == 0 {
            self.reg_select
        } else {
            self.regs[self.reg_select as usize]
        }
    }

    fn write(&mut self, address: Address, value: u8) {
        let offset = address.lo_u8() & 0x01;
        if offset == 0 {
            self.reg_select = value & 0x1F;
        } else {
            let reg = self.reg_select as usize;
            if reg < 18 {
                self.regs[reg] = value;
                if reg == 12 || reg == 13 {
                    self.display_start = self.display_start_address() >> 1;
                }
            }
        }
    }
}

impl Clocked for CRTC {
    fn step(&mut self, us: u64) {
        assert!(self.clock_us < us); // can't go back in time

        if self.clock_us / Self::FIFTY_HERZ != us / Self::FIFTY_HERZ {
            self.vsync.raise();
            self.b_em_vsync.raise();
        }

        let chars_per_line = self.horizontal_total();
        let rasters_per_char = (self.max_raster() + 1) as u16;
        let _total_rasters = (self.vertical_total() + 1) * rasters_per_char;

        let delta = us - self.clock_us;
        let chars = delta as u16;

        let mut remaining = chars;
        while remaining > 0 {
            let h_before = self.h_counter;
            let advance = remaining.min(chars_per_line - h_before);
            self.h_counter += advance;
            remaining -= advance;

            if self.h_counter >= chars_per_line {
                self.h_counter = 0;
                self.r_counter += 1;

                if self.r_counter > self.max_raster() {
                    self.r_counter = 0;
                    self.v_counter += 1;

                    if self.v_counter >= self.vertical_total() + 1 {
                        self.v_counter = 0;
                    }
                }
            }
        }

        self.clock_us = us;
    }
}

#[test]
fn vsync_step1() {
    // run for a second
    let mut crtc = CRTC::new();
    let signal50hz = crtc.vsync.clone();
    let mut count = 0;
    for us in 1..1_000_000 {
        crtc.step(us);
        if signal50hz.sense() {
            count += 1;
        }
    }

    assert_eq!(count, 49);
}

#[test]
fn vsync_step3() {
    // run for a second
    let mut crtc = CRTC::new();
    let signal50hz = crtc.vsync.clone();
    let mut count = 0;
    for us in (1..1_000_000).step_by(3) {
        crtc.step(us);
        if signal50hz.sense() {
            count += 1;
        }
    }

    assert_eq!(count, 49);
}
