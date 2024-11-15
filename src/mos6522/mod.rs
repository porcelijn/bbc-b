use std::marker::PhantomData;

use crate::memory::{Address, devices::Device, MemoryBus};

pub struct SystemPorts;

//  &40–&5F 6522 VIA SYSTEM VIA 23
pub type SystemVIA = VIA<SystemPorts>;
impl SystemVIA {
  pub const fn new() -> Self {
    SystemVIA { _ports: PhantomData::<SystemPorts> }
  }
}
impl Device for SystemVIA {
  fn name(&self) -> &'static str { "6522 System VIA" }
}

pub struct UserPorts;

//  &60–&7F 6522 VIA USER VIA 24
pub type UserVIA = VIA<UserPorts>;
impl UserVIA {
  pub const fn new() -> Self {
    UserVIA { _ports: PhantomData::<UserPorts> }
  }
}
impl Device for UserVIA {
  fn name(&self) -> &'static str { "6522 User VIA" }
}

#[derive(Debug)]
pub struct VIA<P> {
  _ports: PhantomData<P>,
}

impl<P> MemoryBus for VIA<P> {
  fn read(&self, address: Address) -> u8 {
    match address.to_u16() & 0x000F {
      0b0000 => println!("read {address:?} IORB"),
      0b0001 => println!("read {address:?} IORa"),
      0b0010 => println!("read {address:?} DDRB"),
      0b0011 => println!("read {address:?} DDRA"),
      0b0100 => println!("read {address:?} T1C-L"),
      0b0101 => println!("read {address:?} T1C-H"),
      0b0110 => println!("read {address:?} T1L-L"),
      0b0111 => println!("read {address:?} T1L-H"),
      0b1000 => println!("read {address:?} T2C-L"),
      0b1001 => println!("read {address:?} T2C-H"),
      0b1010 => println!("read {address:?} SR"),
      0b1011 => println!("read {address:?} ACR"),
      0b1100 => println!("read {address:?} PCR"),
      0b1101 => println!("read {address:?} IFR"),
      0b1110 => println!("read {address:?} IER"),
      0b1111 => println!("read {address:?} IORAnh"),
      _      => unreachable!(),
    };

    0xFF // bogus
  }
  fn write(&mut self, address: Address, value: u8) {
    match address.to_u16() & 0x000F {
      0b0000 => println!("write {value:#04x} -> {address:?} IORB"),
      0b0001 => println!("write {value:#04x} -> {address:?} IORa"),
      0b0010 => println!("write {value:#04x} -> {address:?} DDRB"),
      0b0011 => println!("write {value:#04x} -> {address:?} DDRA"),
      0b0100 => println!("write {value:#04x} -> {address:?} T1C-L"),
      0b0101 => println!("write {value:#04x} -> {address:?} T1C-H"),
      0b0110 => println!("write {value:#04x} -> {address:?} T1L-L"),
      0b0111 => println!("write {value:#04x} -> {address:?} T1L-H"),
      0b1000 => println!("write {value:#04x} -> {address:?} T2C-L"),
      0b1001 => println!("write {value:#04x} -> {address:?} T2C-H"),
      0b1010 => println!("write {value:#04x} -> {address:?} SR"),
      0b1011 => println!("write {value:#04x} -> {address:?} ACR"),
      0b1100 => println!("write {value:#04x} -> {address:?} PCR"),
      0b1101 => println!("write {value:#04x} -> {address:?} IFR"),
      0b1110 => println!("write {value:#04x} -> {address:?} IER"),
      0b1111 => println!("write {value:#04x} -> {address:?} IORAnh"),
      _      => unreachable!(),
    };
  }
}


