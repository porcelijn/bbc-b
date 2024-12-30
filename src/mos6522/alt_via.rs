use std::cell::RefCell;
use std::rc::Rc;

use b_em_sysvia::{Interrupt, Sysvia};

use crate::devices::Clocked;
use crate::devices::keyboard::Keyboard;
use crate::memory::{Address, MemoryBus};
use crate::mos6522::{Device, Signal};

pub struct AltVIA {
  pub irq: Rc<Signal>,// shared, hard-wired to other IRQ sources for logic "OR"
  pub crtc_vsync: Rc<Signal>, // 50Hz CRT flyback
  via: Sysvia,
  micros: u64
}

impl AltVIA {
  pub fn new(keyboard: Rc<RefCell<Keyboard>>) -> Self {
    let irq = Rc::new(Signal::new());
    let raise_interrupt = make_interrupt(irq.clone());
    let b_em = keyboard.borrow().b_em.clone();
    AltVIA {
      irq,
      crtc_vsync: Rc::new(Signal::new()),
      via: Sysvia::new(b_em, raise_interrupt),
      micros: 0
    }
  }
}

impl Device for AltVIA {
  fn name(&self) -> &'static str { "B-em System VIA" }
}

impl Clocked for AltVIA {
  fn step(&mut self, us: u64) {
    assert!(self.micros < us);
    let ticks = 2 * (us - self.micros); // B-em ticks are at 2MHz
    assert!(ticks < u32::MAX.into());
    self.via.set_ca1_level(self.crtc_vsync.sense());
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

fn make_interrupt(irq: Rc<Signal>) -> Box<Interrupt> {
  Box::new(move | value | {
    if value != 0 {
      log::trace!("AltVIA: interrupt {value}");
      irq.raise();
    }
  })
}
