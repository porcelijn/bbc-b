use std::rc::Rc;

// a quasi object shim that delegates to C implementation singleton
pub struct Sysvia {
  via: *mut Cvia,
  state: *mut State, // Pointer owns state, must drop in Rust land!
}

pub type Interrupt = dyn FnMut(u32);

impl Sysvia {
  pub fn new(keyboard: Keyboard, interrupt: Box<Interrupt>) -> Self {
    let state = Box::new(State::new(keyboard, interrupt));
    let state = Box::into_raw(state);
    let via = unsafe { sysvia_new(state) };
    Sysvia{ via, state }
  }

  pub fn read(&self, address: u16) -> u8 {
    let value = unsafe { sysvia_read(self.via, address) };
    println!("B-em read: {address:x} -> {value:#04x}");
    value
  }

  pub fn write(&self, address: u16, value: u8) {
    println!("B-em write: {address:x} <- {value:#04x}");
    unsafe { sysvia_write(self.via, address, value) };
  }

  pub fn step(&self, ticks: u32) {
    unsafe { sysvia_poll(self.via, ticks) };
  }

  // vsync signal
  pub fn set_ca1_level(&mut self, level: bool) {
    unsafe { sysvia_set_ca1(self.via, level as u32); }
  }
}

impl Drop for Sysvia {
  fn drop(&mut self) {
    // Call C cleanup
    unsafe { sysvia_delete(self.via) };

    // Sysvia owns state, must drop explicitly
    let state = unsafe { Box::from_raw(self.state) }; // take back ownership
    drop(state);
  }
}

pub trait Matrix {
  fn read(&self, row: u8, col: u8) -> bool;
}

#[repr(C)]
pub struct Keyboard {
  pub keyrow: u32,
  pub keycol: u32,
  matrix: Rc<dyn Matrix>,
}

impl std::fmt::Debug for Keyboard {
  fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Ok(()) // TODO
  }
}

impl Keyboard {
  const MAXCOL: u32 = 10;
  pub fn new(matrix: Rc<dyn Matrix>) -> Self {
    Keyboard {
      keyrow: 0, keycol: 0,
      matrix,
    }
  }

  pub fn scan_all(&self) -> bool {
    for col in 0..Self::MAXCOL {
      for row in 1..8 {
        if self.matrix.read(row, col as u8) {
          return true;
        }
      }
    }
    false
  }

  pub fn scan_col(&self) -> bool {
    if self.keycol < Self::MAXCOL {
      for row in 1..8 {
        if self.matrix.read(row, self.keycol as u8) {
          return true;
        }
      }
    }
    false
  }

  pub fn scan_key(&self) -> bool {
    if self.keycol == 15 {
      assert_eq!(self.keyrow, 0);
      // this is the exceptional case where MOS1.20 strobes invalid col before
      // probing keys
      return false;
    }
    assert!(self.keyrow < 8 && self.keycol < Self::MAXCOL);
    self.matrix.read(self.keyrow as u8, self.keycol as u8)
  }
}

#[repr(C)]
pub struct State {
  /*Current state of IC32 output*/
  ic32: u8,
  /*Current effective state of the slow data bus*/
  sdbval: u8,
  /*What the System VIA is actually outputting to the slow data bus
    For use when contending with whatever else is outputting to the bus*/
  sysvia_sdb_out: u8,

  scrsize: u32,

  via: *mut Cvia,
  interrupt: Box<Interrupt>,
  keyboard: Keyboard,
}

impl State {
  fn new(keyboard: Keyboard, interrupt: Box<Interrupt>) -> Self {
    State { ic32: 0, sdbval: 0, sysvia_sdb_out: 0, scrsize: 0,
            via: std::ptr::null_mut(), interrupt,
            keyboard }
  }
}

#[repr(C)]
struct Cvia([u8; 0]); // opaque

extern {
  fn sysvia_new(state: *mut State) -> *mut Cvia;
  fn sysvia_delete(via: *mut Cvia);
  fn sysvia_read(via: *mut Cvia, address: u16) -> u8;
  fn sysvia_write(via: *mut Cvia, address: u16, value: u8);
  fn sysvia_set_ca1(via: *mut Cvia, level: u32);
  fn sysvia_set_ca2(via: *mut Cvia, level: u32);
  fn sysvia_poll(via: *mut Cvia, cycles: u32);
}

#[no_mangle]
pub extern fn raise_interrupt(state: *mut State, value: u32) {
  let interrupt = unsafe { &mut (*state).interrupt };
  interrupt(value);
}

fn key_update(state: *mut State) {
  let cvia = unsafe { (*state).via };
  let ic32 = unsafe { (*state).ic32 };
  let keyboard = unsafe { &mut (*state).keyboard };

  let scan = if ic32 & 8 != 0 {
    /* autoscan mode */
    keyboard.scan_all()
  } else {
    /* scan specific key mode */
    keyboard.scan_col()
  };

  unsafe { sysvia_set_ca2(cvia, scan as u32); }
}

#[no_mangle]
pub extern fn key_scan(state: *mut State, row: u32, col: u32) {
  let keyboard = unsafe { &mut (*state).keyboard };

  keyboard.keyrow = row;
  keyboard.keycol = col;

  key_update(state);
}

#[no_mangle]
pub extern fn key_is_down(state: *const State) -> bool {
  let keyboard = unsafe { &(*state).keyboard };
  keyboard.scan_key()
}

/*
#[repr(C)]
struct VIA {
  ora: u8, orb: u8, ira: u8, irb: u8,
  ddra: u8, ddrb: u8,
  sr: u8,
  t1pb7: u8,
  t1l: u32, t2l:  u32,
  t1c: u32, t2c:  u32,
  acr: u8, pcr: u8, ifr: u8, ier: u8,
  t1hit: u32, t2hit:  u32,
  ca1: u32, ca2: u32, cb1: u32, cb2: u32,
  intnum: u32,
  sr_count: u32,
  uint8_t  (*read_portA)(void);
  uint8_t  (*read_portB)(void);
  void     (*write_portA)(uint8_t val);
  void     (*write_portB)(uint8_t val);

  void     (*set_ca1)(int level);
  void     (*set_ca2)(int level);
  void     (*set_cb1)(int level);
  void     (*set_cb2)(int level);
  void     (*timer_expire1)(void);
}
*/

#[no_mangle]
pub extern fn crtc_latchpen() {
  unimplemented!("6845 lightpen");
}

#[no_mangle]
pub extern fn sn_write(data: u8) {
  println!("sn76489 data: {data}");
}

#[no_mangle]
pub extern fn led_update(led_name: u32 /* led_name_t */, b: bool, ticks: u32) {
 println!("LED update: led_name={led_name}, b={b}, ticks={ticks}");
}

#[no_mangle]
pub extern fn key_paste_poll(state: *mut State) {
  // stick as close to the b-em/src/keyboard.c implementation as possible, but
  // wire to keyboard through callback

  // FIXME: removed keyboard polling callback
  key_update(state);  
}

#[allow(unused)]
struct Dummy;
impl Matrix for Dummy {
  fn read(&self, r: u8, c: u8) -> bool {
    (r+c) % 2 == 0
  }
}

#[allow(unused)]
unsafe fn characterization_test() {
  let s = State::new(Keyboard::new(Rc::new(Dummy)),
                     Box::new(|interrupt| {
                       assert_eq!(interrupt, 0);
                     }));
  let s = Box::into_raw(Box::new(s));
  let via = sysvia_new(s);
  let v = sysvia_read(via, 0);
  assert_eq!(v, 0xFF);
  sysvia_write(via, 0, 1);
  sysvia_poll(via, 1);

  sysvia_delete(via);
  drop(Box::from_raw(s));
}

#[test]
fn test() {
  // do stuff
  unsafe { characterization_test() };

  let keyboard = Keyboard::new(Rc::new(Dummy));
  let has_irq = std::rc::Rc::new(std::cell::Cell::new(false));
  let has_irq_alias = has_irq.clone();
  let interrupt = move |value| {
    has_irq_alias.set(value != 0);
  };

  let sysvia = Sysvia::new(keyboard, Box::new(interrupt));
  let v = sysvia.read(0);
  assert_eq!(v, 0xFF); // via_read_null()
  sysvia.write(0, 1);
  sysvia.step(100);

  assert_eq!(has_irq.get(), false);
  sysvia.write(14, 0xFF); // ier, enable all interrupts
  sysvia.write(13, 0x7F); // ifr, clear all interrupts

  assert_eq!(has_irq.get(), false);
  unsafe { // negative edge
    sysvia_set_ca2(sysvia.via, 1);
    sysvia_set_ca2(sysvia.via, 0);
  }
  assert_eq!(has_irq.get(), true);

  sysvia.write(13, 0x7F); // ifr, clear all interrupts
  assert_eq!(has_irq.get(), false);
}

#[test]
fn alt_via_timer1() {
  let has_irq = std::rc::Rc::new(std::cell::Cell::new(false));
  let has_irq_alias = has_irq.clone();
  let interrupt = move |value| {
    has_irq_alias.set(value != 0);
  };

  let via = Sysvia::new(Keyboard::new(Rc::new(Dummy)), Box::new(interrupt));
  via.write(6, 0);
  via.write(5, 0); // activate t1
  via.step(200);
  assert!(!has_irq.get());
  assert_eq!(via.read(13), 1 << 6); // check IFR_T1_BIT
  via.write(14, 0x80 | 1 << 6); // set IER_T1_BIT
  assert!(has_irq.get());
  via.write(13, 1 << 6); // clear IFR_T1_BIT

  assert!(!has_irq.get());
  via.write(6, 42); // T1L-low
  // timer won't start till we write to T1 high latch
  assert!(!has_irq.get());
//assert_eq!(via.port_b.read(0), 0);
  via.write(5, 0); // T1C-high, also T1L-low -> T1C-low
  via.step(200); // + 200/2 = 100 ticks
  assert!(has_irq.get());
//assert_eq!(via.port_b.read(0), 0); // Auxiliary control bit 7 not set

  // read clears interrupt
  via.read(4); // T1C-low

  assert!(!has_irq.get());

  // ACR square wave on Port B bit 7
  via.write(11, 1 << 7); // ACR_T1_PB7_BIT
  via.write(5, 1); // T1C-high; restart timer: 256 + 42
  via.step(1); // half cycle undocumented in via.c
  via.step(100);
  assert_eq!(via.read(4), 248); // = 256 + 42 + 1 - 100
  assert_eq!(via.read(5), 0);
  assert!(!has_irq.get());
  via.step(600);
  assert!(has_irq.get());
//assert_eq!(via.port_b.read(0), 0x80);
}

