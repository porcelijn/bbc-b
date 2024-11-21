pub mod system_via;

use std::cell::Cell;

use crate::memory::{Address, MemoryBus};
use crate::devices::Device;

pub trait Port: std::fmt::Debug {
  // Control lines CA1-2 / CB1-2
  fn control(&self) -> (bool, bool);
  // I/O lines PA0-7 / PB0-7
  fn read(&self, ddr_mask: u8) -> u8;
  fn write(&mut self, value: u8, ddr_mask: u8);
}

#[derive(Debug)]
pub struct BogusPort<const ID: char>(u8);
impl<const ID: char> BogusPort<ID> {
  pub const fn new(value: u8) -> Self { Self(value) }
}
impl<const ID: char> Port for BogusPort<ID> {
  fn control(&self) -> (bool, bool) {
    // c1 true if contained value non-zero
    (self.0 != 0, false)
  }
  // A 0 in a bit of the DDR causes corresponding peripheral pin to act as input
  fn read(&self, ddr_mask: u8) -> u8 {
    let result = self.0 & !ddr_mask;
    log::trace!("reading from port {ID} -> {result}");
    result
  }
  // A 1 in DDR causes the corresponding pin to act as output
  fn write(&mut self, value: u8, ddr_mask: u8) {
    self.0 = value & ddr_mask;
    log::trace!("writing to port {ID} <- {value}");
  }
}

pub type UserPortA = BogusPort<'a'>;
pub type UserPortB = BogusPort<'b'>;

//  &60–&7F 6522 VIA USER VIA 24
pub type UserVIA = VIA<UserPortA, UserPortB>;
impl Device for UserVIA {
  fn name(&self) -> &'static str { "6522 User VIA" }
}

#[derive(Debug)]
pub struct VIA<PA: Port, PB: Port> {
  iora: u8,           // input / output
  iorb: u8,
  ddra: u8,           // data direction registers
  ddrb: u8,
  t1l: u16, t2l: u16, // latches
  t1c: u16, t2c: u16, // timers
  sr: u8,             // shift register
  acr: u8,            // auxiliary control register
  pcr: u8,            // peripheral control
  ifr: Cell<u8>,      // interrupt flags:  S/C|t1|t2|cb1|cb2|sr|ca1|ca2
  ier: u8,            // interrupt enable mask

  port_a: PA, port_b: PB,
}

const BIT7: u8  = 1u8 << 7;
const NBIT7: u8 = ! BIT7;

impl<PA: Port, PB: Port> VIA<PA, PB> {
  const IFR_CA2_BIT: u8 = 1 << 0; // set: active edge CA2, clear: r/w ORA*
  const IFR_CA1_BIT: u8 = 1 << 1; // set: active edge CA1, clear: r/w ORA
//const IFR_SR_BIT:  u8 = 1 << 2; // set: complete 8 shifts, clear: r/w SR
//const IFR_CB2_BIT: u8 = 1 << 3; // set: active edge CB2, clear: r/w ORB*
//const IFR_CB1_BIT: u8 = 1 << 4; // set: active edge CB1, clear: r/w ORB
  const IFR_T2_BIT:  u8 = 1 << 5; // set: timeout T2, clear: r T2 low / w T2 high
  const IFR_T1_BIT:  u8 = 1 << 6; // set: timeout T1, clear: r T1C low / w T1L high
//const IFR_IRQ_BIT: u8 = BIT7;   // set: any above, clear: clear all interrupts

  const ACR_PA_LATCH_BIT:  u8 = 1 << 0; // PA enable latching
  const ACR_PB_LATCH_BIT:  u8 = 1 << 1; // set: active edge CA1, clear: r/w ORA
//// ACR 2-4: SR control (8 shift register modes; all 0 = disabled)
//const ACR_T2_PB6_BIT:    u8 = 1 << 5; // T2 count down with each PB6 pulse
  const ACR_T1_REPEAT_BIT: u8 = 1 << 6; // T1 continuous interrupts (0=on load)
  const ACR_T1_PB7_BIT:    u8 = 1 << 7; // T1 PB7 one shot / square wave

  pub const fn new(port_a: PA, port_b: PB) -> Self {
    VIA::<PA, PB>{  iora: 0, iorb: 0,
                    ddra: 0, ddrb: 0,
                    t1l: 0, t2l: 0,
                    t1c: 0, t2c: 0,
                    sr: 0,
                    acr: 0, pcr: 0,
                    ifr: Cell::new(0), ier: 0,

                    port_a, port_b
    }
  }

  const fn mask_bits(register: u8, value: u8, mask: u8) -> u8 {
    (register & !mask) | (value & mask)
  }

  pub fn interrupt_requested(&self) -> bool {
    self.ifr.get() & BIT7 != 0
  }

  pub fn step(&mut self, ticks: u16) {
    let (ca1, ca2) = self.port_a.control();
    if ca1 {
      self.set_ifr_bits(Self::IFR_CA1_BIT);
    }

    if ca2 {
      self.set_ifr_bits(Self::IFR_CA2_BIT);
    }

    if self.t1c <= ticks {
      let difference = ticks - self.t1c;
      self.set_ifr_bits(Self::IFR_T1_BIT);
      if self.acr & Self::ACR_T1_REPEAT_BIT != 0 {
        // free run
        assert!(difference <= self.t1l);
        self.t1c = self.t1l - difference;
      } else {
        // one shot
        self.t1c = 0xFFFF - difference;
      }
      self.update_port_b7();
    } else {
      self.t1c -= ticks;
    }

    if self.t2c <= ticks {
      self.set_ifr_bits(Self::IFR_T2_BIT);
    }

    // one-shot just continues counting (from 0xffff)
    self.t2c = self.t2c.wrapping_sub(ticks);
  }

  fn clear_ifr_bits(&self, bits: u8) {
    assert!(bits & BIT7 == 0);
    let mut ifr = self.ifr.get();
    ifr &= !bits;
    self.ifr.set(ifr);
    self.update_ifr_irq();
  }

  fn set_ifr_bits(&self, bits: u8) {
    assert!(bits & BIT7 == 0);
    let mut ifr = self.ifr.get();
    ifr |= bits;
    self.ifr.set(ifr);
    self.update_ifr_irq();
  }

  fn update_ifr_irq(&self) {
    let ier_mask = self.ier & NBIT7;
    let mut ifr = self.ifr.get();
    let ifr_mask = ifr & NBIT7;
    if ier_mask & ifr_mask != 0 {
      ifr |= BIT7;
    } else {
      ifr &= NBIT7;
    }
    self.ifr.set(ifr);
  }

  fn update_port_b7(&mut self) {
    if self.acr & Self::ACR_T1_PB7_BIT != 0 {
      let value = self.port_b.read(!BIT7);
      let value = value ^ BIT7; // toggle PB7
      self.port_b.write(value, BIT7);
    }
  }
}

impl<PA: Port, PB: Port> MemoryBus for VIA<PA, PB> {
  fn read(&self, address: Address) -> u8 {
    match address.to_u16() & 0x000F {
      0b0000 => {
        // system via uses top four bits for reading, bottom four for writing
        // TODO: clear interrupt
        let mut result = self.iorb & self.ddrb; // read output bits
        if self.acr & Self::ACR_PB_LATCH_BIT != 0 {
          // read latch
          result |= self.iorb & !self.ddrb;
        } else {
          // read current port values
          result |= self.port_b.read(self.ddrb);
//        if self.acr & Self::ACR_T1_PB7_BIT != 0 {
//          result = (result & 0b0111_1111) | self.t1pb7;
//        }
        }
        log::trace!("read {address:?} IORB -> {result}");
        return result;
      },
      0b0001 => {
        // self.iora = self.port_a.read(self.ddra);
        let result = self.iora;
        log::trace!("read {address:?} IORA -> {result}");
        return result;
      },
      0b0010 => log::trace!("read {address:?} DDRB"),
      0b0011 => log::trace!("read {address:?} DDRA"),
      0b0100 => {
        // read T1C-L, clear T1 interrupt flag
        self.clear_ifr_bits(Self::IFR_T1_BIT);
        let result = ((self.t1c & 0x00FF) >> 0) as u8;
        log::trace!("read {address:?} T1C-L -> {result}");
        return result;
      },
      0b0101 => log::trace!("read {address:?} T1C-H"),
      0b0110 => log::trace!("read {address:?} T1L-L"),
      0b0111 => log::trace!("read {address:?} T1L-H"),
      0b1000 => {
        // read T2C-l, clear T2 interrupt flag
        self.clear_ifr_bits(Self::IFR_T2_BIT);
        // one shot timer, stop
//      self.t2_active = false;
        log::trace!("read {address:?} T2C-L");
        let result =((self.t2c & 0x00FF) >> 0) as u8;
        return result;
      },
      0b1001 => log::trace!("read {address:?} T2C-H"),
      0b1010 => {
        let sr = self.sr;
        log::trace!("read {address:?} SR -> {sr:#04x}");
      },
      0b1011 => log::trace!("read {address:?} ACR"),
      0b1100 => log::trace!("read {address:?} PCR"),
      0b1101 => {
        let ifr = self.ifr.get();
        log::trace!("read {address:?} IFR -> {ifr}");
      },
      0b1110 => {
        let result = self.ier | BIT7; // When read, bit 7 is *always* a logic 1
        log::trace!("read {address:?} IER -> {result}");
        return result;
      },
      0b1111 => {
        let result = self.port_a.read(self.ddra);
        log::trace!("read {address:?} IORAnh -> {result}");
        return result;
      },
      _      => unreachable!(),
    };

    0xFF // bogus
  }

  fn write(&mut self, address: Address, value: u8) {
    match address.to_u16() & 0x000F {
      0b0000 => {
        log::trace!("write {value:#04x} -> {address:?} IORB");
        self.iorb = Self::mask_bits(self.iorb, value, self.ddrb);
        if self.acr & Self::ACR_PB_LATCH_BIT == 0 {
          // latching disabled, write straight to port B
          self.port_b.write(value, self.ddrb);
        }
        self.update_ifr_irq();
      },
      0b0001 => {
        log::trace!("write {value:#04x} -> {address:?} IORA");
        self.iora = Self::mask_bits(self.iora, value, self.ddra);
        if self.acr & Self::ACR_PA_LATCH_BIT == 0 {
          // latching disabled, write straight to port A
          self.port_a.write(value, self.ddra);
        }
        self.update_ifr_irq();
      },
      0b0010 => {
        log::trace!("write {value:#04x} -> {address:?} DDRB");
        self.ddrb = value;
      },
      0b0011 => {
        log::trace!("write {value:#04x} -> {address:?} DDRA");
        self.ddra = value;
      },
      0b0100 => log::trace!("write {value:#04x} -> {address:?} T1C-L"),
      0b0101 => {
        log::trace!("write {value:#04x} -> {address:?} T1C-H");
        // Write into high order latch and copy latch to counter
        self.t1l &= 0x00FF;
        self.t1l |= (value as u16) << 8;
        self.t1c = self.t1l;
        if self.acr & Self::ACR_T1_PB7_BIT != 0 {
          // one shot, pull PB7 low
          self.port_b.write(0, BIT7);
        }
        self.clear_ifr_bits(Self::IFR_T1_BIT);
        //self.t1_active = true;
      },
      0b0110 => log::trace!("write {value:#04x} -> {address:?} T1L-L"),
      0b0111 => log::trace!("write {value:#04x} -> {address:?} T1L-H"),
      0b1000 => log::trace!("write {value:#04x} -> {address:?} T2C-L"),
      0b1001 => {
        log::trace!("write {value:#04x} -> {address:?} T2C-H");
        // Write into high order latch and copy latch to counter
        self.t2l &= 0x00FF;
        self.t2l |= (value as u16) << 8;
        self.t2c = self.t2l;
        self.clear_ifr_bits(Self::IFR_T2_BIT);
        //self.t2_active = true;
      },
      0b1010 => log::trace!("write {value:#04x} -> {address:?} SR"),
      0b1011 => {
        log::trace!("write {value:#04x} -> {address:?} ACR");
        self.acr = value;
      },
      0b1100 => {
        log::trace!("write {value:#04x} -> {address:?} PCR");
        self.pcr = value;
      },
      0b1101 => {
        log::trace!("write {value:#04x} -> {address:?} IFR");
        self.ifr.set(value);
      },
      0b1110 => {
        log::trace!("write {value:#04x} -> {address:?} IER");
        if value & BIT7 != 0 {
          self.ier |= value & NBIT7;
        } else {
          self.ier &= ! (value & NBIT7);
        }
        self.update_ifr_irq();
      },
      0b1111 => {
        log::trace!("write {value:#04x} -> {address:?} IORAnh");
        self.port_a.write(value, self.ddra);
        self.update_ifr_irq();
      },
      _      => unreachable!(),
    };
  }
}

#[test]
fn via_ca1() {
  let pa = BogusPort::<'A'>::new(1); // CA1 is high
  let pb = BogusPort::<'B'>::new(0);
  let mut via = VIA::new(pa, pb);
  assert!(!via.interrupt_requested());
  via.write(Address::from(14), BIT7 | 1 << 1); // set IER_CA1_BIT
  assert!(!via.interrupt_requested()); // enabled, but CA1 not scanned yet
  via.step(1);
  assert!(via.interrupt_requested());
  via.write(Address::from(14), 1 << 1); // clear IER_CA1_BIT, mask interrupt
  assert!(!via.interrupt_requested());
}

#[test]
fn via_timer1() {
  let pa = BogusPort::<'A'>::new(0);
  let pb = BogusPort::<'B'>::new(0);
  let mut via = VIA::new(pa, pb);
  via.step(100);
  assert!(!via.interrupt_requested());
  via.write(Address::from(14), BIT7 | 1 << 6); // set IER_T1_BIT
  assert!(via.interrupt_requested());
  via.write(Address::from(13), 1 << 6); // clear IFR_T1_BIT
  assert!(!via.interrupt_requested());
  via.write(Address::from(6), 1); // T1L-low
  // timer won't start till we write to T1 high latch
  assert!(!via.interrupt_requested());
  assert_eq!(via.port_b.read(0), 0);
  via.write(Address::from(5), 0); // T1C-high
  via.step(100);
  assert!(via.interrupt_requested());
  assert_eq!(via.port_b.read(0), 0); // Auxiliary control bit 7 not set

  // read clears interrupt
  let v = via.read(Address::from(4)); // T1C-low
  assert!(!via.interrupt_requested());
  assert_eq!(v, 155);
  // ACR square wave on Port B bit 7
  via.write(Address::from(11), 1 << 7); // ACR_T1_PB7_BIT
  via.write(Address::from(5), 128); // T1C-high; restart timer
  via.step(100);
//assert!(via.interrupt_requested());
//assert_eq!(via.port_b.read(0), 0x80);
}

