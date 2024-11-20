// FIXME: "ic32", the addressable latch probably should not live under 6522 VIA
// 74LS259

use std::cell::Cell;

// IC32 is 8 bit addressable latch
// B0 – Write Enable to the sound generator IC
// B1 – READ select on the speech processor
// B2 – WRITE select on the speech processor
// B3 – Keyboard write enable (see Appendix J)
// B4,B5 – these two outputs define the number to be
//         added to the start of screen address in hardware to
//         control hardware scrolling
// B6 - CAPS lock
// B7 - SHIFT lock
#[derive(Debug)]
pub struct IC32(Cell<u8>);
impl IC32 {
  pub const SOUND:    u8 = 0;
  pub const SPEECH_R: u8 = 1;
  pub const SPEECH_W: u8 = 2;
  pub const KEYBOARD: u8 = 3;
//...
//pub const CAPS_LOCK:  u8 = 6;
//pub const SHIFT_LOCK: u8 = 6;
  const fn get_mask(address: u8) -> u8 {
    assert!(address < 8);
    let mask: u8 = 1 << address;
    mask
  }

  pub const fn get_message(address: u8, value: bool) -> &'static str {
    match (address, value) {
      (Self::SOUND,    false) => "Enable sound chip",
      (Self::SPEECH_R, false) => "Enable Read Speech",
      (Self::SPEECH_W, false) => "Enable Write Speech",
      (Self::KEYBOARD, false) => "Disable Keyboard auto scanning",
      (4,              false) => "Hardware scrolling - set C0=0",
      (5,              false) => "Hardware scrolling - set C1=0",
      (6,              false) => "Turn on CAPS LOCK LED",
      (7,              false) => "Turn on SHIFT LOCK LED",
      (Self::SOUND,    true)  => "Disable sound chip",
      (Self::SPEECH_R, true)  => "Disable Read Speech",
      (Self::SPEECH_W, true)  => "Disable Write Speech",
      (Self::KEYBOARD, true)  => "Enable Keyboard auto scanning",
      (4,              true)  => "Hardware scrolling - set C0=1",
      (5,              true)  => "Hardware scrolling - set C1=1",
      (6,              true)  => "Turn off CAPS LOCK LED",
      (7,              true)  => "Turn off SHIFT LOCK LED",
      _ => unreachable!()
    }
  }

  pub const fn new() -> Self {
    IC32(Cell::new(0u8))
  }

  pub fn has<const BIT: u8>(&self) -> bool {
    self.0.get() & Self::get_mask(BIT) != 0
  }
  
  pub fn write(&self, address: u8, value: bool) {
    let mut latch = self.0.get();
    if value {
      latch |=   Self::get_mask(address);
    } else {
      latch &= ! Self::get_mask(address);
    }
    self.0.set(latch);
  }
}

