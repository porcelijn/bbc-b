use std::cell::RefCell;

use crate::memory::{Address, MemoryBus};
use crate::mos6522::{SystemVIA, UserVIA};

//  SHEILA Integrated Description Section address circuit number (offset from
//  &FE00)
//
//
//  Note: Some Sheila addresses are not normally used. This is because the same
//  devices appear at several different Sheila addresses. For example, the
//  paged ROM select register is normally addressed at location &30, but it
//  could equally well be addressed at any one of the fifteen other locations
//  &31–&3F

pub trait Device : MemoryBus {
  fn name(&self) -> &'static str;
}

trait BogusDevice : Device {}

impl<D: BogusDevice> MemoryBus for D {
  fn read(&self, _address: Address) -> u8 {
    0xFF // bogus
  }
  fn write(&mut self, _address: Address, _value: u8) {
    // bogus
  }
}

//  &00–&07 6845 CRTC Video controller 18
struct CRTC {}
impl BogusDevice for CRTC {}
impl Device for CRTC {
  fn name(&self) -> &'static str { "6845 CRTC video controller" }
}

//  &08–&0F 6850 ACIA Serial controller 20.3
struct ACIA {}
impl BogusDevice for ACIA {}
impl Device for ACIA {
  fn name(&self) -> &'static str { "6850 ACIA Serial controller" }
}

//  &10–&1F Serial ULA Serial system chip 20.9
//  &20–&2F Video ULA Video system chip 19
//  &30–&3F 74LS161 Paged ROM selector 21
//  &40–&5F 6522 VIA SYSTEM VIA 23
//  &60–&7F 6522 VIA USER VIA 24
//  &80–&9F 8271 FDC Floppy disc controller 25.1
//  &A0–&BF 68B54 ADLC ECONET controller 25.2
//  &C0–&DF uPD7002 Analogue to digital converter 26
//  &E0–&FF Tube ULA Tube system interface 27

struct UnimplementedDevice {}
impl BogusDevice for UnimplementedDevice {}
impl Device for UnimplementedDevice {
  fn name(&self) -> &'static str { "Bogus device" }
}

pub trait DevicePage<const PAGE: u8> : MemoryBus {
  fn page() -> u8 { PAGE }
}

pub struct SheilaPage {
  crtc: RefCell<CRTC>,
  acia: RefCell<ACIA>,
  system_via: RefCell<SystemVIA>,
  user_via: RefCell<UserVIA>,
  device_todo: RefCell<UnimplementedDevice>,
}

impl SheilaPage {
  pub fn new() -> Self {
    let crtc = RefCell::new(CRTC{});
    let acia = RefCell::new(ACIA{});
    let system_via = RefCell::new(SystemVIA::new());
    let user_via = RefCell::new(UserVIA::new());
    let device_todo = RefCell::new(UnimplementedDevice{}); // catch all
    SheilaPage { crtc, acia, system_via, user_via, device_todo }
  }

  fn get_device(&self, address: Address) -> &RefCell<dyn Device> {
    assert_eq!(address.hi_u8(), Self::page());
    match address.lo_u8() & 0b1111_0000 {
      0x00 => {
        if address.lo_u8() & 0b0000_1000 == 0 {
          &self.crtc
        } else {
          &self.acia
        }
      },
      //...
      0x40 | 0x50 => &self.system_via,
      0x60 | 0x70 => &self.user_via,
      _ => &self.device_todo, // to be removed
    }
  }
}

impl DevicePage<0xFE> for SheilaPage {}
impl MemoryBus for SheilaPage {
  fn read(&self, address: Address) -> u8 {
    let page = Self::page();
    let device = self.get_device(address);
    let name = device.borrow().name();
    let value = device.borrow().read(address);
    println!("{address:?} -> {value:02x} | Reading from SHEILA ({page}, {name})");
    value
  }

  fn write(&mut self, address: Address, value: u8) {
    let page = Self::page();
    let device = self.get_device(address);
    let name = device.borrow().name();
    println!("{value:02x} -> {address:?} | Writing to SHEILA ({page}, {name})");
    device.borrow_mut().write(address, value);
  }
}


