use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;

use super::{Port, VIA};
use crate::devices::{Device, Signal};
use crate::devices::ic32::IC32;
use crate::devices::keyboard::Keyboard;

//  &40–&5F 6522 VIA SYSTEM VIA
pub type SystemVIA = VIA<SystemPortA, SystemPortB>;
impl Device for SystemVIA {
  fn name(&self) -> &'static str { "6522 System VIA" }
}

#[derive(Debug)]
pub struct SystemPortA {
  pub crtc_vsync: Rc<Signal>, // 6845 video controller for 50Hz vsync signal
  pa: u8,                     // latched PA0-7 pin value written or read
  ic32: Rc<IC32>,             // addressable latch
  keyboard: Rc<RefCell<Keyboard>>,
}

impl SystemPortA {
  pub fn new(ic32: Rc<IC32>, keyboard: Rc<RefCell<Keyboard>>) -> Self {
    let crtc_vsync = Rc::new(Signal::new());
    SystemPortA { pa: 0, crtc_vsync, ic32, keyboard }
  }
}

// PA0-7 is a slow databus connecting
// - Keyboard
// - SN76489 sound generator
// - Speech synthesizer
impl Port for SystemPortA {
  fn control(&self) -> (bool, bool) {
    // CA1 input — This is the vertical sync input from the 6845. CA1 is set up
    // to interrupt the 6502 every 20 ms (50 Hz) as a vertical sync from the
    // video circuitry is detected.
    let ca1 = self.crtc_vsync.sense();

    // CA2 input from keyboard circuit when ic32 latch set to auto scan
    let auto_scan = self.ic32.has::<{IC32::KEYBOARD}>();
    let ca2 = if auto_scan {
      self.keyboard.borrow().scan_interrupt()
    } else {
      let key_code = self.pa;
      let (_row, col) = Keyboard::decode(key_code);
      self.keyboard.borrow().scan_column(col)
    };

    (ca1, ca2)
  }

  fn read(&self, _ddr_mask: u8) -> u8 {
    let mut value = self.pa; // retrieve last value

    let auto_scan = self.ic32.has::<{IC32::KEYBOARD}>();
    if !auto_scan {
      log::trace!("sysvia A: reading {value:x} from keyboard");
      let key_pressed = self.keyboard.borrow().is_key_pressed(value);
      // result in bit 7
      if key_pressed {
        value |= 0b1000_0000;
      } else {
        value &= 0b0111_1111;
      }
    }

    let speech_disabled = self.ic32.has::<{IC32::SPEECH_R}>();
    if !speech_disabled {
      // receive value from TMS5220 speech chip over slow data bus
    }

//  let value = value & !ddr_mask; // return only bits marked input/read
    value
  }

  fn write(&mut self, value: u8, ddr_mask: u8) {
    self.pa &= !ddr_mask;
    self.pa |= value & ddr_mask;

    let sound_disabled = self.ic32.has::<{IC32::SOUND}>();
    if !sound_disabled {
      // send self.pa value over slow data bus to sound chip
    }

    let speech_disabled = self.ic32.has::<{IC32::SPEECH_W}>();
    if !speech_disabled {
      // send self.pa value over slow data bus to speech chip
    }
  }
}

#[derive(Debug)]
pub struct SystemPortB {
  // PB0..PB3: output to addressible latch
  // PB4, PB5: input from fire buttons
  // PB6, PB7: input from speech processor
  pb: u8, // Latched value written to / read from PB0-7
  ic32: Rc<IC32>,
  joybuttons: (bool, bool), // TODO
}

impl SystemPortB {
  pub fn new(ic32: Rc<IC32>) -> Self {
    SystemPortB { pb: 0, ic32, joybuttons: (false, false) }
  }

  const fn decode(value: u8) -> (u8, bool) {
    let address = value & 0b0000_0111;      // PB0-PB2
    let value =   value & 0b0000_1000 != 0; // PB3
    (address, value)
  }
}

impl Port for SystemPortB {
  fn control(&self) -> (bool, bool) {
    // TODO The CB1 input is the end of conversion (EOC) signal from the 7002
    // analogue to digital converter
    let cb1 = false;
    // CB2 input is the light pen strobe signal sent by 6845 video processor
    let cb2 = false;

    (cb1, cb2)
  }

  fn read(&self, ddr_mask: u8) -> u8 {
    let mut result = self.pb & !ddr_mask;
    // PB4 and PB5: joystick buttons
    if self.joybuttons.0 { result &= !(1 << 4); } // PB4
    if self.joybuttons.1 { result &= !(1 << 5); } // PB5

    // PB6 and PB7: inputs from speech processor (interrupt & ready, resp)
    result
  }

  fn write(&mut self, value: u8, ddr_mask: u8) {
    self.pb &= !ddr_mask;
    self.pb |= value & ddr_mask;
    let (address, value) = Self::decode(value);
    let msg = IC32::get_message(address, value);
    log::trace!("System VIA port B: {address}={value}: {msg}");
    self.ic32.borrow_mut().write(address, value);
  }
}

