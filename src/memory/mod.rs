pub mod ram;

//  SHEILA Integrated Description Section address circuit number (offset from
//  &FE00)
//
//  &00–&07 6845 CRTC Video controller 18
//  &08–&0F 6850 ACIA Serial controller 20.3
//  &10–&1F Serial ULA Serial system chip 20.9
//  &20–&2F Video ULA Video system chip 19
//  &30–&3F 74LS161 Paged ROM selector 21
//  &40–&5F 6522 VIA SYSTEM VIA 23
//  &60–&7F 6522 VIA USER VIA 24
//  &80–&9F 8271 FDC Floppy disc controller 25.1
//  &A0–&BF 68B54 ADLC ECONET controller 25.2
//  &C0–&DF uPD7002 Analogue to digital converter 26
//  &E0–&FF Tube ULA Tube system interface 27
//
//  Note: Some Sheila addresses are not normally used. This is because the same
//  devices appear at several different Sheila addresses. For example, the
//  paged ROM select register is normally addressed at location &30, but it
//  could equally well be addressed at any one of the fifteen other locations
//  &31–&3F

pub trait MemoryBus {
  fn read(&self, address: Address) -> u8;
  fn write(&mut self, address: Address, value: u8);
  fn try_slice(&self, _from: Address, _to: Address) -> Option<&[u8]> { None }
}

// Construct 16 bit Address from memory bytes in little endian order
pub fn read_address(memory: &dyn MemoryBus, address: Address) -> Address {
  let lo = memory.read(address);
  let hi = memory.read(address.next());
  Address::from_le_bytes(lo, hi)
}

pub fn slice(memory: &dyn MemoryBus, mut address: Address, mut size: usize) -> Vec<u8> {
  let mut vec = Vec::new();
  while size != 0 {
    vec.push(memory.read(address));
    address = address.next();
    size -= 1;
  }
  vec
}

#[derive(Clone, Copy)]
pub struct Address(u16);

impl Address {
  pub const fn from(address: u16) -> Self {
    Address(address)
  }

  pub const fn from_le_bytes(lo: u8, hi: u8) -> Address {
    let lo = lo as u16;
    let hi = hi as u16;
    Address(hi << 8 | lo)
  }

  pub const fn to_u16(&self) -> u16 {
    self.0
  }

  pub const fn hi_u8(&self) -> u8 {
    let page = (self.0 & 0xFF00) >> 8;
    page as u8
  }

  pub const fn lo_u8(&self) -> u8 {
    let offset = (self.0 & 0x00FF) >> 0;
    offset as u8
  }

  pub const fn next(&self) -> Address {
    Address(self.0.wrapping_add(1))
  }

  pub fn inc_by(&mut self, plus: u8) {
    self.0 = self.0.wrapping_add(plus.into());
  }

  pub fn dec_by(&mut self, plus: u8) {
    self.0 = self.0.wrapping_sub(plus.into());
  }
}

impl std::fmt::Debug for Address {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "&{:#06x}", self.0)
  }
}

pub struct PageDispatcher {
  mapping: [u8; 256],
  backends: Vec<Box<dyn MemoryBus>>,
}

impl PageDispatcher {
  pub fn new(backend: Box<dyn MemoryBus>) -> Self {
    let mut backends = Vec::new();
    backends.push(backend);
    PageDispatcher { mapping: [0u8; 256], backends }
  }

  pub fn add_backend(&mut self, page: u8, backend: Box<dyn MemoryBus>) {
    self.mapping[page as usize] = self.backends.len() as u8;
    self.backends.push(backend.into());
  }
}

impl MemoryBus for PageDispatcher {
  fn read(&self, address: Address) -> u8 {
    let page = address.hi_u8();
    let backend_index = self.mapping[page as usize];
    let backend = &self.backends[backend_index as usize];
    backend.read(address)
  }

  fn write(&mut self, address: Address, value: u8) {
    let page = address.hi_u8();
    let backend_index = self.mapping[page as usize];
    let backend = &mut self.backends[backend_index as usize];
    backend.write(address, value);
  }

  fn try_slice(&self, from: Address, to: Address) -> Option<&[u8]> {
    let page = from.hi_u8();
    let backend_index = self.mapping[page as usize];
    // TODO assert:assumption that pages are mapped contiguously in one backend
    let backend = &self.backends[backend_index as usize];
    backend.try_slice(from, to)
  }
}

impl crate::devices::Device for PageDispatcher {
  fn name(&self) -> &'static str {
    "Page dispatcher"
  }
}

#[test]
fn page_dispatcher() {
  use std::cell::RefCell;
  use std::rc::Rc;
  use crate::memory::ram::RAM;
  use crate::devices::{DevicePage, SheilaPage};
  use crate::devices::keyboard::Keyboard;
  let ram = RAM::new();
  let mut memory = PageDispatcher::new(Box::new(ram));
  let sheila = SheilaPage::new(Rc::new(RefCell::new(Keyboard::new())));
  memory.add_backend(SheilaPage::page(), Box::new(sheila));
  let addr = Address::from(0xFE00);
  memory.write(addr, 42);
//assert_eq!(memory.read(addr), 42); // No, we're not seeing what we just wrote!
  assert_eq!(memory.read(addr), 0);  // instead we get back bogus data
}
