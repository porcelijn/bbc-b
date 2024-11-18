use std::cell::Cell;

use crate::memory::{Address, devices::Device, MemoryBus};

pub trait Port {
  fn read(&self, ddr_mask: u8) -> u8;
  fn write(&mut self, value: u8, ddr_mask: u8);
}

pub struct BogusPort<const ID: char, const VALUE: u8>;
impl<const ID: char, const VALUE: u8> Port for BogusPort<ID, VALUE> {
  // A 0 in a bit of the DDR causes corresponding peripheral pin to act as input
  fn read(&self, ddr_mask: u8) -> u8 {
    let result = VALUE & !ddr_mask;
    log::trace!("reading from port {ID} -> {result}");
    result
  }
  // A 1 in DDR causes the corresponding pin to act as output
  fn write(&mut self, value: u8, ddr_mask: u8) {
    let result = value & ddr_mask;
    log::trace!("writing to port {ID} <- {result}");
  }
}

pub type SystemPortA = BogusPort<'A', 42>;

//  &40–&5F 6522 VIA SYSTEM VIA 23
pub type SystemVIA = VIA<SystemPortA>;
impl Device for SystemVIA {
  fn name(&self) -> &'static str { "6522 System VIA" }
}

pub type UserPortA = BogusPort<'a', 0xFF>;

//  &60–&7F 6522 VIA USER VIA 24
pub type UserVIA = VIA<UserPortA>;
impl Device for UserVIA {
  fn name(&self) -> &'static str { "6522 User VIA" }
}

#[derive(Debug)]
pub struct VIA<P: Port> {
  iora: u8,
  iorb: u8,
  ddra: u8,
  ddrb: u8,
  // todo ...
  acr: u8,
  ier: u8, ifr: Cell<u8>,
  port_a: P,
}

const BIT7: u8 = 1u8 << 7;
const NBIT7: u8 = ! BIT7;

impl<P: Port> VIA<P> {
  pub const fn new(port_a: P) -> Self {
    VIA::<P>{ iora: 0, iorb: 0,
              ddra: 0, ddrb: 0,
              acr: 0, ier: 0, ifr: Cell::new(0),
              port_a
    }
  }

  const fn mask_bits(register: u8, value: u8, mask: u8) -> u8 {
    (register & !mask) | (value & mask)
  }

  fn clear_ifr_bits(&self, bits: u8) {
    assert!(bits & BIT7 == 0);
    let mut ifr = self.ifr.get();
    ifr &= !bits;
    self.ifr.set(ifr);
    self.update_ifr_irq();
  }

  #[allow(unused)]
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
}

impl<P: Port> MemoryBus for VIA<P> {
  fn read(&self, address: Address) -> u8 {
    match address.to_u16() & 0x000F {
      0b0000 => {
        // system via uses top four bits for reading, bottom four for writing
        // TODO: clear interrupt
        let mut result = self.iorb & self.ddrb; // read output bits
        if self.acr & 0b0000_0010 != 0 {
          // read latch
          result |= self.iorb & !self.ddrb;
        } else {
          // read current port values
          //result |= self.port_b.read() & !self.ddrb;
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
        self.clear_ifr_bits(1 << 6);
//      let result = ((self.t1c & 0x00FF) >> 0) as u8;
//      log::trace!("read {address:?} T1C-L -> {result}");
//      return result;
      },
      0b0101 => log::trace!("read {address:?} T1C-H"),
      0b0110 => log::trace!("read {address:?} T1L-L"),
      0b0111 => log::trace!("read {address:?} T1L-H"),
      0b1000 => log::trace!("read {address:?} T2C-L"),
      0b1001 => log::trace!("read {address:?} T2C-H"),
      0b1010 => log::trace!("read {address:?} SR"),
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
//      if self.acr & 0b0000_0010 == 0 {
//        // latching disabled, write straight to port B
//        self.port_b.write(value, self.ddrb);
//      }
        self.update_ifr_irq();
        return;
      },
      0b0001 => {
        log::trace!("write {value:#04x} -> {address:?} IORA");
        self.iora = Self::mask_bits(self.iora, value, self.ddra);
        if self.acr & 0b0000_0001 == 0 {
          // latching disabled, write straight to port A
          self.port_a.write(value, self.ddra);
        }
        self.update_ifr_irq();
        return;
      },
      0b0010 => {
        log::trace!("write {value:#04x} -> {address:?} DDRB");
        self.ddrb = value;
        return;
      },
      0b0011 => {
        log::trace!("write {value:#04x} -> {address:?} DDRA");
        self.ddra = value;
        return;
      },
      0b0100 => log::trace!("write {value:#04x} -> {address:?} T1C-L"),
      0b0101 => log::trace!("write {value:#04x} -> {address:?} T1C-H"),
      0b0110 => log::trace!("write {value:#04x} -> {address:?} T1L-L"),
      0b0111 => log::trace!("write {value:#04x} -> {address:?} T1L-H"),
      0b1000 => log::trace!("write {value:#04x} -> {address:?} T2C-L"),
      0b1001 => log::trace!("write {value:#04x} -> {address:?} T2C-H"),
      0b1010 => log::trace!("write {value:#04x} -> {address:?} SR"),
      0b1011 => {
        log::trace!("write {value:#04x} -> {address:?} ACR");
        self.acr = value;
        return;
      },
      0b1100 => log::trace!("write {value:#04x} -> {address:?} PCR"),
      0b1101 => log::trace!("write {value:#04x} -> {address:?} IFR"),
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

