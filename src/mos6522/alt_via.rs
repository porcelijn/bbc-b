use b_em_sysvia::{Keypress, Sysvia};

use crate::devices::Clocked;
use crate::memory::{Address, MemoryBus};
use crate::mos6522::Device;

pub struct AltVIA {
  via: Sysvia,
  micros: u64
}

impl AltVIA {
  pub fn new() -> Self {
    let poll_keyboard = make_keypress(); // TODO
    AltVIA { via: Sysvia::new(poll_keyboard), micros: 0 }
  }
}

impl Device for AltVIA {
  fn name(&self) -> &'static str { "B-em System VIA" }
}

impl Clocked for AltVIA {
  fn step(&mut self, us: u64) {
    assert!(self.micros < us);
    let ticks = us - self.micros;
    assert!(ticks < u32::MAX.into());
    self.via.step(ticks as u32);
    self.micros = us;
  }
}

impl MemoryBus for AltVIA {
  fn read(&self, address: Address) -> u8 {
    self.via.read(address.to_u16())
  }

  fn write(&mut self, address: Address, value: u8) {
    self.via.write(address.to_u16(), value);
  }
}

fn make_keypress() -> Box<Keypress> {
  // somewhat random keypress and release
  let mut seed = 0u8;
  Box::new(move || -> (u8, bool) {
    let key_code = (seed / 3) & 0b0111_0111;
    let pressed = seed % 3 == 0;
    if seed > 200 {
      seed = 0;
    } else {
      seed += 1;
    }
//  println!("Keyboard: pressed={pressed}, key_code={key_code:x}");
    (key_code, pressed)
  })
}
