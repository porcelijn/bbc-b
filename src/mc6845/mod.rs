// Motorola 6845 video controller
// Just provide 50 Hz VSync signal to System VIA

use std::rc::Rc;

use crate::devices::{Clocked, Device, Signal};
use crate::memory::{Address, MemoryBus};

//  &00–&07 6845 CRTC Video controller 18
#[derive(Debug)]
pub struct CRTC {
  pub vsync: Rc<Signal>,
  clock_us: u64,
}

impl CRTC {
  const FIFTY_HERZ: u64 = 20_000;
  pub fn new() -> Self {
    let vsync = Rc::new(Signal::new());
    let clock_us = 0;
    CRTC { vsync, clock_us }
  }
}

impl Device for CRTC {
  fn name(&self) -> &'static str { "6845 CRTC video controller" }
}

impl MemoryBus for CRTC {
  fn read(&self, _address: Address) -> u8 {
    0x00 // TODO
  }
  fn write(&mut self, _address: Address, _value: u8) {
    // TODO
  }
}

impl Clocked for CRTC {
  fn step(&mut self, us: u64) {
    assert!(self.clock_us < us); // can't go back in time
    if self.clock_us / Self::FIFTY_HERZ != us / Self::FIFTY_HERZ {
      self.vsync.raise();
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
