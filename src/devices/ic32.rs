//
// "ic32", is an 8 bit addressable latch (74LS259) controlling
// - the slow memory bus
// - CRTC mode adjus wrapping offset
// - keyboard leds
// 3 address lines are wired to bits PB0-PB2 of system VIA
// 1 data line is connected to PB3 of system VIA
//

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
  pub const C0_B4:    u8 = 4;
  pub const C1_B5:    u8 = 5;
  pub const CAPS_L:   u8 = 6;
  pub const SHIFT_L:  u8 = 7;
  const fn get_mask(address: u8) -> u8 {
    assert!(address < 8);
    let mask: u8 = 1 << address;
    mask
  }

  pub const fn get_message(address: u8, value: bool) -> &'static str {
    match (address, value) {
      (Self::SOUND,   false) => "Enable sound chip",
      (Self::SPEECH_R,false) => "Enable Read Speech",
      (Self::SPEECH_W,false) => "Enable Write Speech",
      (Self::KEYBOARD,false) => "Disable Keyboard auto scanning",
      (Self::C0_B4,   false) => "Hardware scrolling - set C0=0",
      (Self::C1_B5,   false) => "Hardware scrolling - set C1=0",
      (Self::CAPS_L,  false) => "Turn on CAPS LOCK LED",
      (Self::SHIFT_L, false) => "Turn on SHIFT LOCK LED",
      (Self::SOUND,    true) => "Disable sound chip",
      (Self::SPEECH_R, true) => "Disable Read Speech",
      (Self::SPEECH_W, true) => "Disable Write Speech",
      (Self::KEYBOARD, true) => "Enable Keyboard auto scanning",
      (Self::C0_B4,    true) => "Hardware scrolling - set C0=1",
      (Self::C1_B5,    true) => "Hardware scrolling - set C1=1",
      (Self::CAPS_L,   true) => "Turn off CAPS LOCK LED",
      (Self::SHIFT_L,  true) => "Turn off SHIFT LOCK LED",
      _ => unreachable!()
    }
  }

  pub const fn new() -> Self {
    IC32(Cell::new(0u8))
  }

  pub fn has<const BIT: u8>(&self) -> bool {
    self.0.get() & Self::get_mask(BIT) != 0
  }

  // B4,B5 – these two outputs define the number to be added to the start of
  // screen address in hardware to control hardware scrolling:
  //
  //   Mode | Size | Start of screen | Increase | B5 | B4
  //   0,1,2| 20kB |     &3000       |   12k    |  1 |  1
  //     3  | 16kB |     &4000       |   16k    |  0 |  0
  //    4,5 | 10kB |     &5800       |   22k    |  1 |  0
  //     6  |  8kB |     &6000       |   24k    |  0 |  1
  //
  // Also: https://beebwiki.mdfs.net/Address_translation#Calculation_of_the_adjusted_address
#[allow(unused)]
  fn lookup_mode_adjust(&self) -> u8 {
    // one's complement to be subtracted with borrow
    let b2k = match (self.has::<{Self::C1_B5}>(), self.has::<{Self::C0_B4}>()) {
      (false,false) => 0b0111, //  (7+1)*2=16
      (false, true) => 0b1011, // (11+1)*2=24
      (true, false) => 0b0101, //  (5+1)*2=12
      (true,  true) => 0b1010, // (10+1)*2=22
    };
    b2k
  }

  pub fn write(&self, address: u8, value: bool) {
    log::trace!("IC32[{address}] = {value}, {}", Self::get_message(address, value));
    let mut latch = self.0.get();
    if value {
      latch |=   Self::get_mask(address);
    } else {
      latch &= ! Self::get_mask(address);
    }
    self.0.set(latch);
  }
}

#[test]
fn mode_adjust() {
  let ic32 = IC32::new();
  // c1, c0 = b5, b4 = 0, 0
  let offset = ic32.lookup_mode_adjust() as u16;
  let offset = (offset + 1) * 2 * 1024;
  let address = 0x8000;
  let address = address - offset;
  assert_eq!(address, 0x4000); // MODE 3: wraps to &4000
  assert_eq!(offset/1024, 16); // MODE 3: 16k increase
}
