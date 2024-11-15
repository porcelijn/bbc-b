use std::cell::RefCell;

use crate::memory::{Address, MemoryBus};

//  SHEILA Integrated Description Section address circuit number (offset from
//  &FE00)
//
//
//  Note: Some Sheila addresses are not normally used. This is because the same
//  devices appear at several different Sheila addresses. For example, the
//  paged ROM select register is normally addressed at location &30, but it
//  could equally well be addressed at any one of the fifteen other locations
//  &31–&3F

trait Device : MemoryBus {
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

struct VIA {}

impl MemoryBus for VIA {
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
//struct SystemVIA {}
//impl VIA for SystemVIA {}
type SystemVIA = VIA;
impl Device for SystemVIA {
  fn name(&self) -> &'static str { "6522 System VIA" }
}

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
  device_todo: RefCell<UnimplementedDevice>,
}

impl SheilaPage {
  pub fn new() -> Self {
    let crtc = RefCell::new(CRTC{});
    let acia = RefCell::new(ACIA{});
    let system_via = RefCell::new(SystemVIA{});
    let device_todo = RefCell::new(UnimplementedDevice{}); // catch all
    SheilaPage { crtc, acia, system_via, device_todo }
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


