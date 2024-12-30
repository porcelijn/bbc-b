use std::cell::RefCell;
use std::rc::Rc;

use bbc_b::devices::{Clocked, Signal};
use bbc_b::devices::ic32::IC32;
use bbc_b::devices::keyboard::Keyboard;
use bbc_b::memory::{Address, MemoryBus};
use bbc_b::mos6522::alt_via::AltVIA;
use bbc_b::mos6522::system_via::{SystemPortA, SystemPortB, SystemVIA};

const IORB: Address = Address::from(0);
const IORA: Address = Address::from(1);
const DDRB: Address = Address::from(2);
const DDRA: Address = Address::from(3);
const T1C_H: Address = Address::from(5);
const T1L_L: Address = Address::from(6);
const ACR:   Address = Address::from(11);
const IFR:   Address = Address::from(13);
const IER:   Address = Address::from(14);

#[test]
fn timer1_100hz_mine() {
  let keyboard = Rc::new(RefCell::new(Keyboard::new()));
  let ic32 = Rc::new(IC32::new());
  let port_a = SystemPortA::new(ic32.clone(), keyboard.clone());
  let port_b = SystemPortB::new(ic32.clone());
  let via = SystemVIA::new(port_a, port_b);
  let irq = via.irq.clone();
  test_timer1_100hz(via, irq);
}

#[test]
fn timer1_100hz_b_em() {
  let keyboard = Rc::new(RefCell::new(Keyboard::new()));
  let via = AltVIA::new(keyboard.clone());
  let irq = via.irq.clone();
  test_timer1_100hz(via, irq);
}

fn test_timer1_100hz<VIA: MemoryBus + Clocked>(mut via: VIA, irq: Rc<Signal>) {
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
    assert!(!irq.sense());
    assert_eq!(via.read(IFR), 0);
  }

  via.step(10000);
  assert!(!irq.sense());

  via.step(10001);
  assert!(irq.sense()); // <-- BOOM!
  assert_eq!(via.read(IFR), 0b1100_0000); // T1 = bit 6, IRQ = bit 7

  via.step(10002);
  assert!(!irq.sense());
  assert_eq!(via.read(IFR), 0b1100_0000); // still set
  via.write(IFR, 0b0100_0000);            // clear it
  assert_eq!(via.read(IFR), 0);           // cleared

  for us in 10003..20000 {
    via.step(us);
    assert!(!irq.sense());
  }

  via.step(20000);
  assert!(!irq.sense());

  via.step(20001);
  assert!(irq.sense()); // <-- BOOM!

  // etc.
  for us in 20002..=30000 {
    via.step(us);
    assert!(!irq.sense());
  }

  via.step(30001);
  assert!(irq.sense());

  via.step(40001);
  assert!(irq.sense());
}

#[test]
fn timer1_b7_square_wave_mine() {
  let keyboard = Rc::new(RefCell::new(Keyboard::new()));
  let ic32 = Rc::new(IC32::new());
  let port_a = SystemPortA::new(ic32.clone(), keyboard.clone());
  let port_b = SystemPortB::new(ic32.clone());
  test_timer1_b7_square_wave(SystemVIA::new(port_a, port_b));
}

#[test]
fn timer1_b7_square_wave_b_em() {
  let keyboard = Rc::new(RefCell::new(Keyboard::new()));
  test_timer1_b7_square_wave(AltVIA::new(keyboard));
}

fn test_timer1_b7_square_wave<VIA: MemoryBus + Clocked>(mut via: VIA) {
  via.write(ACR, 0b1100_0000); // set T1 repeat + set B7 square wave bit
  const BIT7: u8 = 1 << 7;
  via.write(IER, BIT7 | 1 << 6); // set IER_T1_BIT
  const TICKS_10KHZ: u16 = 98; // 100 @ 1MHz
  via.write(T1L_L, (TICKS_10KHZ & 0xFF) as u8);
  via.write(T1C_H, (TICKS_10KHZ >> 8) as u8); // activate t1

  via.write(DDRB, 0b1111_1111);
  via.write(IORB, 0b0011_1100); // Some random pattern
  for us in 1..101 {
    via.step(us);
    assert_eq!(via.read(IORB), 0b0011_1100);
  }
  for us in 101..201 {
    via.step(us);
    assert_eq!(via.read(IORB), 0b1011_1100); // <-- B7 is set!
  }
  for us in 201..301 {
    via.step(us);
    assert_eq!(via.read(IORB), 0b0011_1100); // B7 is clear
  }
  for us in 301..401 {
    via.step(us);
    assert_eq!(via.read(IORB), 0b1011_1100); // B7 is set
  }
  via.step(401);
  assert_eq!(via.read(IORB), 0b0011_1100); // B7 is clear
}

#[test]
fn crtc_vsync_mine() {
  let keyboard = Rc::new(RefCell::new(Keyboard::new()));
  let ic32 = Rc::new(IC32::new());
  let port_a = SystemPortA::new(ic32.clone(), keyboard.clone());
  let port_b = SystemPortB::new(ic32.clone());
  let vsync = port_a.crtc_vsync.clone();
  test_crtc_vsync(SystemVIA::new(port_a, port_b), vsync);
}

#[test]
fn crtc_vsync_b_em() {
  let b_em = AltVIA::new(Rc::new(RefCell::new(Keyboard::new())));
  let vsync = b_em.crtc_vsync.clone();
  test_crtc_vsync(b_em, vsync);
}

fn test_crtc_vsync<VIA: MemoryBus + Clocked>(mut via: VIA, vsync: Rc<Signal>) {
  assert_eq!(via.read(IFR), 0);
  vsync.raise(); // i.e. positive edge
  via.step(1);
  assert_eq!(via.read(IFR), 0);
  via.step(2); // neagtive edge
  assert_eq!(via.read(IFR), 1 << 1); // CA1 is set

  // clear CA1 flag explicitly
  via.write(IFR, 1 << 1);
  assert_eq!(via.read(IFR), 0);
  vsync.raise(); // again
  via.step(3);
  assert_eq!(via.read(IFR), 0);
  via.step(4); // neagtive edge
  assert_eq!(via.read(IFR), 1 << 1); // CA1 is set

  // clear CA1 flag by reading from register 1 (IRA)
  via.read(IORA);
  assert_eq!(via.read(IFR), 0);
  vsync.raise(); // again
  via.step(5);
  assert_eq!(via.read(IFR), 0);
  via.step(6); // neagtive edge
  assert_eq!(via.read(IFR), 1 << 1); // CA1 is set

  // clear CA1 flag by writing to register 1 (ORA)
  via.write(DDRA, 0xFF);
  via.write(IORA, 0);
  assert_eq!(via.read(IFR), 0);
  vsync.raise(); // again
  via.step(7);
  assert_eq!(via.read(IFR), 0);
  via.step(8); // neagtive edge
  assert_eq!(via.read(IFR), 1 << 1); // CA1 is set
}
