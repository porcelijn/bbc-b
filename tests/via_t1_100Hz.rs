use std::cell::RefCell;
use std::rc::Rc;

use bbc_b::devices::Clocked;
use bbc_b::devices::keyboard::Keyboard;
use bbc_b::memory::{Address, MemoryBus};
use bbc_b::mos6522::alt_via::AltVIA;

const T1C_H: Address = Address::from(5);
const T1L_L: Address = Address::from(6);
const ACR:   Address = Address::from(11);
const IFR:   Address = Address::from(13);
const IER:   Address = Address::from(14);

#[test]
fn timer1_100hz() {
  let keyboard = Rc::new(RefCell::new(Keyboard::new()));
  let mut via = AltVIA::new(keyboard.clone());

  via.write(ACR, 1 << 6); // set ACR_T1_REPEAT_BIT
  const BIT7: u8 = 1 << 7;
  via.write(IER, BIT7 | 1 << 6); // set IER_T1_BIT

  // See MOS1.20
  //   Initialise[s] the 100Hz timer to $270E = 9998 uS (two less than expected
  //   because the latch reload costs 2uS)
  const TICKS_100HZ: u16 = 9998;
  via.write(T1L_L, (TICKS_100HZ & 0xFF) as u8);
  via.write(T1C_H, (TICKS_100HZ >> 8) as u8); // activate t1

  for us in 1..10000 {
    via.step(us);
    assert!(!via.irq.sense());
  }

  via.step(10000);
  assert!(!via.irq.sense());

  via.step(10001);
  assert!(via.irq.sense()); // <-- BOOM!

  via.step(10002);
  assert!(!via.irq.sense());

  for us in 10003..20000 {
    via.step(us);
    assert!(!via.irq.sense());
  }

  via.step(20000);
  assert!(!via.irq.sense());

  via.step(20001);
  assert!(via.irq.sense()); // <-- BOOM!

  // etc.
  for us in 20002..=30000 {
    via.step(us);
    assert!(!via.irq.sense());
  }

  via.step(30001);
  assert!(via.irq.sense());

  via.step(40001);
  assert!(via.irq.sense());
}

