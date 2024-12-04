pub mod ic32;
pub mod keyboard;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::vec::Vec;

use ic32::IC32;
use keyboard::Keyboard;

use crate::memory::{Address, MemoryBus};
use crate::mc6845::CRTC;
use crate::mos6522::{UserVIA, UserPortA, UserPortB};
//use crate::mos6522::alt_via::AltVIA;
use crate::mos6522::system_via::{SystemVIA, SystemPortA, SystemPortB};

#[derive(Debug)]
pub struct Signal(Cell<bool>);
impl Signal {
  pub const fn new() -> Self {
    Signal(Cell::new(false))
  }

  pub fn raise(&self) {
    self.0.set(true);
  }

  pub fn sense(&self) -> bool {
    self.0.replace(false)
  }
}

pub trait Clocked {
  // us - absolute clock time in microseconds (MHz)
  fn step(&mut self, us: u64);
}

pub type ClockedDevices = Vec<Rc<RefCell<dyn Clocked>>>;

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
    0x00 // bogus value, seems to least confuse interrupt handling
  }
  fn write(&mut self, _address: Address, _value: u8) {
    // bogus
  }
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
  crtc: Rc<RefCell<CRTC>>,
  acia: RefCell<ACIA>,
//alt_sysvia: Rc<RefCell<AltVIA>>,
  system_via: Rc<RefCell<SystemVIA>>,
  user_via: RefCell<UserVIA>,
  device_todo: RefCell<UnimplementedDevice>,
  pub irq: Rc<Signal>,
  pub use_alt_system_via: bool,
}

impl SheilaPage {
  pub fn new(keyboard: Rc<RefCell<Keyboard>>) -> Self {
    let crtc = CRTC::new();
    let acia = RefCell::new(ACIA{});
    let ic32 = Rc::new(IC32::new());
    let mut system_port_a = SystemPortA::new(ic32.clone(), keyboard);
    system_port_a.crtc_vsync = crtc.vsync.clone(); // connect CA1 to 6845 vsync
    let crtc = Rc::new(RefCell::new(crtc));
//  let alt_sysvia = Rc::new(RefCell::new(AltVIA::new()));
    let system_port_b = SystemPortB::new(ic32);
    let system_via = SystemVIA::new(system_port_a, system_port_b);
    let irq = system_via.irq.clone();
    let system_via = Rc::new(RefCell::new(system_via));
    let mut user_via = UserVIA::new(UserPortA::new(0), UserPortB::new(0));
    user_via.irq = irq.clone(); // connect IRQB wires for logic "OR"
    let user_via = RefCell::new(user_via);
    let device_todo = RefCell::new(UnimplementedDevice{}); // catch all
    SheilaPage { crtc, acia,
                 /* alt_sysvia, */ system_via, user_via,
                 device_todo, irq, use_alt_system_via: false,
    }
  }

  pub fn get_clocked_devices(&self) -> ClockedDevices {
    let mut devices = ClockedDevices::new();
    devices.push(self.crtc.clone());
//  devices.push(self.alt_sysvia.clone());
    devices.push(self.system_via.clone());
    devices
  }

  fn get_device(&self, address: Address) -> &RefCell<dyn Device> {
    assert_eq!(address.hi_u8(), Self::page());
    match address.lo_u8() & 0b1111_0000 {
      0x00 => {
        if address.lo_u8() & 0b0000_1000 == 0 {
          &*self.crtc
        } else {
          &self.acia
        }
      },
      //...
      0x40 | 0x50 => if self.use_alt_system_via {
//      &*self.alt_sysvia
        &self.device_todo
      } else {
        &*self.system_via
      },
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
    log::trace!("{address:?} -> {value:02x} | Reading from SHEILA ({page}, {name})");
    value
  }

  fn write(&mut self, address: Address, value: u8) {
    let page = Self::page();
    let device = self.get_device(address);
    let name = device.borrow().name();
    log::trace!("{value:02x} -> {address:?} | Writing to SHEILA ({page}, {name})");
    device.borrow_mut().write(address, value);
  }
}

