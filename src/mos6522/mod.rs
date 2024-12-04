pub mod alt_via; // delegate to B-em backend
pub mod system_via;

use std::cell::Cell;
use std::rc::Rc;

use crate::devices::{Clocked, Device, Signal};
use crate::memory::{Address, MemoryBus};

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
    // c1 true if contained value 1
    // c2 true if contained value 0
    (self.0 == 1, self.0 == 2)
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
  pub irq: Rc<Signal>,// shared, hard-wired to other IRQ sources for logic "OR"

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
  clock_ms: Cell<u64>,
  t1_active: Cell<bool>, t2_active: Cell<bool>,
}

const BIT7: u8  = 1u8 << 7;
const NBIT7: u8 = ! BIT7;

impl<PA: Port, PB: Port> VIA<PA, PB> {
  const IFR_CA2_BIT: u8 = 1 << 0; // set: active edge CA2, clear: r/w ORA*
  const IFR_CA1_BIT: u8 = 1 << 1; // set: active edge CA1, clear: r/w ORA
//const IFR_SR_BIT:  u8 = 1 << 2; // set: complete 8 shifts, clear: r/w SR
  const IFR_CB2_BIT: u8 = 1 << 3; // set: active edge CB2, clear: r/w ORB*
  const IFR_CB1_BIT: u8 = 1 << 4; // set: active edge CB1, clear: r/w ORB
  const IFR_T2_BIT:  u8 = 1 << 5; // set: timeout T2, clear: r T2 low / w T2 high
  const IFR_T1_BIT:  u8 = 1 << 6; // set: timeout T1, clear: r T1C low / w T1L high
//const IFR_IRQ_BIT: u8 = BIT7;   // set: any above, clear: clear all interrupts

  const ACR_PA_LATCH_BIT:  u8 = 1 << 0; // PA enable latching
  const ACR_PB_LATCH_BIT:  u8 = 1 << 1; // set: active edge CA1, clear: r/w ORA
  // ACR 2-4: SR control (8 shift register modes; all 0 = disabled)
  const ACR_T2_PB6_BIT:    u8 = 1 << 5; // T2 count down with each PB6 pulse
  const ACR_T1_REPEAT_BIT: u8 = 1 << 6; // T1 continuous interrupts (0=on load)
  const ACR_T1_PB7_BIT:    u8 = 1 << 7; // T1 PB7 one shot / square wave

  pub fn new(port_a: PA, port_b: PB) -> Self {
    VIA::<PA, PB>{
      irq: Rc::new(Signal::new()),

      iora: 0, iorb: 0,
      ddra: 0, ddrb: 0,
      t1l: 0, t2l: 0,
      t1c: 0, t2c: 0,
      sr: 0,
      acr: 0, pcr: 0,
      ifr: Cell::new(0), ier: 0,

      port_a, port_b,
      clock_ms: Cell::new(0),
      t1_active: Cell::new(false), 
      t2_active: Cell::new(false), 
    }
  }

  const fn mask_bits(register: u8, value: u8, mask: u8) -> u8 {
    (register & !mask) | (value & mask)
  }

  const fn is_independent(cx2_control: u8) -> bool {
    assert!(cx2_control < 8);
    // Input-positive active edge
    const INDEPENDENT_IRQ_NEG_EDGE: u8 = 1;
    // Input-negative active edge
    const INDEPENDENT_IRQ_POS_EDGE: u8 = 3;
    cx2_control == INDEPENDENT_IRQ_NEG_EDGE || cx2_control == INDEPENDENT_IRQ_POS_EDGE
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
      self.irq.raise();
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
        let cb2_control = (self.pcr & 0b1110_0000) >> 5; // pcr5-7
//      let cb1_control = (self.pcr & 0b0001_0000) >> 4; // pcr4
        let bits = match Self::is_independent(cb2_control) {
           true => Self::IFR_CB1_BIT,
          false => Self::IFR_CB1_BIT | Self::IFR_CB2_BIT
        };
        self.clear_ifr_bits(bits);

        let mut irb = self.iorb & self.ddrb; // read output bits
        if self.acr & Self::ACR_PB_LATCH_BIT != 0 {
          // read latch
          irb |= self.iorb & !self.ddrb;
        } else {
          // read current port values
          irb |= self.port_b.read(self.ddrb);
//        if self.acr & Self::ACR_T1_PB7_BIT != 0 {
//          irb = (irb & 0b0111_1111) | self.t1pb7;
//        }
        }
        log::trace!("read {address:?} IORB -> {irb:04x}");
        irb
      },
      0b0001 => {
        let ca2_control = (self.pcr & 0b000_01110) >> 1; // pcr1-3
        let bits = match Self::is_independent(ca2_control) {
           true => Self::IFR_CA1_BIT,
          false => Self::IFR_CA1_BIT | Self::IFR_CA2_BIT
        };
        self.clear_ifr_bits(bits);
       
        let ira = if self.acr & Self::ACR_PA_LATCH_BIT != 0 {
          self.iora
        } else {
          self.port_a.read(self.ddra)
        };
        log::trace!("read {address:?} IORA -> {ira:04x}");
        ira
      },
      0b0010 => {
        let ddrb = self.ddrb;
        log::trace!("read {address:?} DDRB -> {ddrb:#04x}");
        ddrb
      },
      0b0011 => {
        let ddra = self.ddra;
        log::trace!("read {address:?} DDRA -> {ddra:#04x}");
        ddra
      },
      0b0100 => {
        // read T1C-L, clear T1 interrupt flag
        self.clear_ifr_bits(Self::IFR_T1_BIT);
        let t1clock_lo = ((self.t1c & 0x00FF) >> 0) as u8;
        log::trace!("read {address:?} T1C-L -> {t1clock_lo}");
        t1clock_lo
      },
      0b0101 => {
        let t1clock_hi =((self.t1c & 0xFF00) >> 8) as u8;
        log::trace!("read {address:?} T1C-H -> {t1clock_hi:#04x}");
        t1clock_hi
      },
      0b0110 => {
        let t1latch_lo =((self.t1l & 0x00FF) >> 0) as u8;
        log::trace!("read {address:?} T1L-L -> {t1latch_lo:#04x}");
        t1latch_lo
      },
      0b0111 => {
        let t1latch_hi =((self.t1l & 0xFF00) >> 8) as u8;
        log::trace!("read {address:?} T1L-H -> {t1latch_hi:#04x}");
        t1latch_hi
      },
      0b1000 => {
        // read T2C-l, clear T2 interrupt flag
        self.clear_ifr_bits(Self::IFR_T2_BIT);
        // one shot timer, stop
        self.t2_active.set(false);
        let t2c_lo =((self.t2c & 0x00FF) >> 0) as u8;
        log::trace!("read {address:?} T2C-L -> {t2c_lo:#04x}");
        t2c_lo
      },
      0b1001 => {
        let t2c_hi  = ((self.t2c & 0xFF00) >> 8) as u8; // read T2C-H
        log::trace!("read {address:?} T2C-H -> {t2c_hi:#04x}");
        t2c_hi
      },
      0b1010 => {
        let shift_register = self.sr;
        log::trace!("read {address:?} SR -> {shift_register:#04x}");
        shift_register
      },
      0b1011 => {
        let acr = self.acr;
        log::trace!("read {address:?} ACR -> {acr:#04x}");
        acr
      },
      0b1100 => {
        let pcr = self.pcr;
        log::trace!("read {address:?} PCR -> {pcr:#04x}");
        pcr
      },
      0b1101 => {
        let ifr = self.ifr.get();
        log::trace!("read {address:?} IFR -> {ifr:#04x}");
        ifr
      },
      0b1110 => {
        let ier = self.ier | BIT7; // When read, bit 7 is *always* a logic 1
        log::trace!("read {address:?} IER -> {ier:#04x}");
        ier
      },
      0b1111 => {
        let ira_nh = self.port_a.read(self.ddra);
        log::trace!("read {address:?} IORAnh -> {ira_nh:#04x}");
        ira_nh
      },
      _      => unreachable!(),
    }
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

        // system via uses top four bits for reading, bottom four for writing
        let cb2_control = (self.pcr & 0b1110_0000) >> 5; // pcr5-7
//      let cb1_control = (self.pcr & 0b0001_0000) >> 4; // pcr4
        let bits = match Self::is_independent(cb2_control) {
           true => Self::IFR_CB1_BIT,
          false => Self::IFR_CB1_BIT | Self::IFR_CB2_BIT
        };
        self.clear_ifr_bits(bits);
      },
      0b0001 => {
        log::trace!("write {value:#04x} -> {address:?} IORA");
        self.iora = Self::mask_bits(self.iora, value, self.ddra);
        if self.acr & Self::ACR_PA_LATCH_BIT == 0 {
          // latching disabled, write straight to port A
          self.port_a.write(value, self.ddra);
        }

        let ca2_control = (self.pcr & 0b000_01110) >> 1; // pcr1-3
        let bits = match Self::is_independent(ca2_control) {
           true => Self::IFR_CA1_BIT,
          false => Self::IFR_CA1_BIT | Self::IFR_CA2_BIT
        };
        self.clear_ifr_bits(bits);
      },
      0b0010 => {
        log::trace!("write {value:#04x} -> {address:?} DDRB");
        self.ddrb = value;
      },
      0b0011 => {
        log::trace!("write {value:#04x} -> {address:?} DDRA");
        self.ddra = value;
      },
      0b0100 => {
        log::trace!("write {value:#04x} -> {address:?} T1C-L");
        panic!("this ever used?!");
      },
      0b0101 => {
        log::trace!("write {value:#04x} -> {address:?} T1C-H");
        // Write into high order latch and copy latch to counter
        // (re) start counter
        self.t1l &= 0x00FF;
        self.t1l |= (value as u16) << 8;
        self.t1c = self.t1l;
        if self.acr & Self::ACR_T1_PB7_BIT != 0 {
          // one shot, pull PB7 low
          self.port_b.write(0, BIT7);
        }
        self.clear_ifr_bits(Self::IFR_T1_BIT);
        self.t1_active.set(true);
      },
      0b0110 => {
        log::trace!("write {value:#04x} -> {address:?} T1L-L");
        // write into low order latch
        self.t1l &= 0xFF00;
        self.t1l |= value as u16;
      },
      0b0111 => {
        log::trace!("write {value:#04x} -> {address:?} T1L-H");
        // Write into high order latch, clear T1 interrupt
        self.t1l &= 0x00FF;
        self.t1l |= (value as u16) << 8;
        self.clear_ifr_bits(Self::IFR_T1_BIT);
        self.t1_active.set(false);
      },
      0b1000 => {
        log::trace!("write {value:#04x} -> {address:?} T2C-L");
        // write into low order T2 latch
        self.t2l &= 0xFF00;
        self.t2l |= value as u16;
      },
      0b1001 => {
        log::trace!("write {value:#04x} -> {address:?} T2C-H");
        // Write into high order latch and copy latch to counter
        self.t2l &= 0x00FF;
        self.t2l |= (value as u16) << 8;
        self.t2c = self.t2l;
        self.clear_ifr_bits(Self::IFR_T2_BIT);
        self.t2_active.set(true);
      },
      0b1010 => {
        log::trace!("write {value:#04x} -> {address:?} SR");
        self.sr = value;
      },
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
        // individual flag bits may be cleared by writing a Logic 1 into the
        // appropriate bit of the IFR
        self.clear_ifr_bits(value & NBIT7);
      },
      0b1110 => {
        // To set or clear a particular Interrupt Enable bit, the
        // microprocessor must write to the IER address. During this write
        // operation, if IER7 is Logic 0, each Logic 1 in IER6 thru IER0 will
        // clear the corresponding bit in the IER. For each Logic 0 in IER6
        // thru IER0, the corresponding bit in the IER will be unaffected.
        //
        // Setting selected bits in the IER is accomplished by writing to the
        // same address with IER7 set to a Logic 1. In this case, each Logic 1
        // in IER6 through IER0 will set the corresponding bit to a Logic 1.
        // For each Logic 0 the corresponding bit will be unaffected.
        log::trace!("write {value:#04x} -> {address:?} IER");
        let mask = value & NBIT7;
        if value & BIT7 != 0 {
          self.ier |= mask;
        } else {
          self.ier &= ! mask;
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

impl<PA: Port, PB: Port> Clocked for VIA<PA, PB> {
  fn step(&mut self, ms: u64) {
    let ticks: u16 = {
      let prev_ms = self.clock_ms.get();
      assert!(prev_ms < ms);
      let ticks = ms - prev_ms;
      self.clock_ms.set(ms);
      assert!(ticks < 0x10000);
      ticks as u16
    };

    let (ca1, ca2) = self.port_a.control();
    if ca1 {
      self.set_ifr_bits(Self::IFR_CA1_BIT);
    }

    if ca2 {
      self.set_ifr_bits(Self::IFR_CA2_BIT);
    }

    if self.t1_active.get() {
      if self.t1c <= ticks {
        let difference = ticks - self.t1c;
        self.set_ifr_bits(Self::IFR_T1_BIT);

        if self.acr & Self::ACR_T1_REPEAT_BIT != 0 {
          // free run
          assert!(difference <= self.t1l);
          self.t1c = self.t1l - difference;
        } else {
          // one shot
          self.t1_active.set(false);
          self.t1c = 0xFFFF - difference;
        }
        self.update_port_b7();
      } else {
        self.t1c -= ticks;
      }
    }

    if self.acr & Self::ACR_T2_PB6_BIT == 0 {
      // T2 triggered by phase 2 clock (timed interrupt)
      if self.t2c <= ticks && self.t2_active.get() {
        self.t2_active.set(false);
        self.set_ifr_bits(Self::IFR_T2_BIT);
      }

      // one-shot just continues counting (from 0xffff)
      self.t2c = self.t2c.wrapping_sub(ticks);
    }
  }
}

#[test]
fn via_ca1() {
  let pa = BogusPort::<'A'>::new(1); // CA1 is high
  let pb = BogusPort::<'B'>::new(0);
  let mut via = VIA::new(pa, pb);
  assert!(!via.irq.sense());
  via.write(Address::from(14), BIT7 | 1 << 1); // set IER_CA1_BIT
  assert!(!via.irq.sense()); // enabled, but CA1 not scanned yet
  via.step(1);
  assert!(via.irq.sense());
  via.write(Address::from(14), 1 << 1); // clear IER_CA1_BIT, mask interrupt
  via.update_ifr_irq(); // re-evaluate IRQ
  assert!(!via.irq.sense());
}

#[test]
fn via_ca2() {
  let pa = BogusPort::<'A'>::new(2); // CA2 is high
  let pb = BogusPort::<'B'>::new(0);
  let mut via = VIA::new(pa, pb);
  assert!(!via.irq.sense());
  via.write(Address::from(14), BIT7 | 1 << 0); // set IER_CA2_BIT
  assert!(!via.irq.sense()); // enabled, but CA1 not scanned yet
  via.step(1);
  assert!(via.irq.sense());
  via.update_ifr_irq(); // re-evaluate IRQ
  assert!(via.irq.sense());
  via.read(Address::from(1)); // clear by reading from IRA
  via.update_ifr_irq(); // re-evaluate IRQ
  assert!(!via.irq.sense());

  via.step(2); // ca1 raises interrupt again
  assert!(via.irq.sense());

  via.write(Address::from(1), 42); // clear by writing o ORA
  via.update_ifr_irq(); // re-evaluate IRQ
  assert!(!via.irq.sense());
}

#[test]
fn via_timer1() {
  let pa = BogusPort::<'A'>::new(0);
  let pb = BogusPort::<'B'>::new(0);
  let mut via = VIA::new(pa, pb);
  via.write(Address::from(5), 0); // activate t1
  via.step(100);
  assert!(!via.irq.sense());
  via.write(Address::from(14), BIT7 | 1 << 6); // set IER_T1_BIT
  assert!(via.irq.sense());
  via.write(Address::from(13), 1 << 6); // clear IFR_T1_BIT
  via.update_ifr_irq(); // re-evaluate IRQ
  assert!(!via.irq.sense());
  via.write(Address::from(6), 1); // T1L-low
  // timer won't start till we write to T1 high latch
  assert!(!via.irq.sense());
  assert_eq!(via.port_b.read(0), 0);
  via.write(Address::from(5), 0); // T1C-high, also T1L-low -> T1C-low
  via.step(200);
  assert!(via.irq.sense());
  assert_eq!(via.port_b.read(0), 0); // Auxiliary control bit 7 not set

  // read clears interrupt
  let v = via.read(Address::from(4)); // T1C-low
  via.update_ifr_irq(); // re-evaluate IRQ
  assert!(!via.irq.sense());
  assert_eq!(v, 156); // 255 + 1 (from t1l) - 100 ticks
  // ACR square wave on Port B bit 7
  via.write(Address::from(11), 1 << 7); // ACR_T1_PB7_BIT
  via.write(Address::from(5), 128); // T1C-high; restart timer
  via.step(300);
//assert!(via.interrupt_requested());
//assert_eq!(via.port_b.read(0), 0x80);
}

