// a quasi object shim that delegates to C implementation singleton
pub struct Sysvia {
  via: *mut Cvia,
  state: *mut State, // Pointer owns state, must drop in Rust land!
}

pub type Keypress = dyn FnMut() -> (u8, bool);

impl Sysvia {
  pub fn new(callback: Box<Keypress>) -> Self {
    let state = Box::new(State::new(callback));
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
  interrupt: u32,
  keyboard: Keyboard
}

#[repr(C)]
struct Keyboard {
  keyrow: u32,
  keycol: u32,

  keypress: Box<Keypress>,
  kbdips: u8,
  bbcmatrix: [[bool; 8]; 10],
}

impl Keyboard {
  const MAXCOL: u32 = 10;
  fn scan_all(&self) -> bool {
    for col in 0..Self::MAXCOL {
      for row in 1..8 {
        if self.bbcmatrix[col as usize][row as usize] {
          return true;
        }
      }
    }
    false
  }

  fn scan_col(&self) -> bool {
    if self.keycol < Self::MAXCOL {
      for row in 1..8 {
        if self.bbcmatrix[self.keycol as usize][row as usize] {
          return true;
        }
      }
    }
    false
  }

  fn scan_key(&self) -> bool {
    assert!(self.keyrow < 8 && self.keycol < Self::MAXCOL);
    self.bbcmatrix[self.keycol as usize][self.keyrow as usize]
  }

  fn scan_dip(&self) -> bool {
    assert!(2 <= self.keycol && self.keycol <= 9);
    self.kbdips & (1 << (9 - self.keycol)) != 0
  }

  fn update_keys(&mut self) {
    let (key_code, pressed) = (self.keypress)();
    let row = (key_code & 0b0111_0000) >> 4;
    let col = (key_code & 0b0000_1111) >> 0;
    assert!(row < 8 && col < Self::MAXCOL as u8);
    self.bbcmatrix[col as usize][row as usize] = pressed;
  }
}

impl State {
  fn new(keypress: Box<Keypress>) -> Self {
    let bbcmatrix = [[false; 8]; 10];
    let keyboard = Keyboard {
      keyrow: 0, keycol: 0,
      keypress, kbdips: 0b0000_0000, bbcmatrix,
    };
    State { ic32: 0, sdbval: 0, sysvia_sdb_out: 0, scrsize: 0,
            via: std::ptr::null_mut(), interrupt: 0, keyboard }
  }
}

#[repr(C)]
struct Cvia([u8; 0]); // opaque

extern {
  fn sysvia_new(state: *mut State) -> *mut Cvia;
  fn sysvia_delete(via: *mut Cvia);
  fn sysvia_read(via: *mut Cvia, address: u16) -> u8;
  fn sysvia_write(via: *mut Cvia, address: u16, value: u8);
  fn sysvia_set_ca2(via: *mut Cvia, level: u32);
  fn sysvia_poll(via: *mut Cvia, cycles: u32);
}

#[no_mangle]
pub extern fn key_scan(state: *mut State, row: u32, col: u32) {
  let cvia = unsafe { (*state).via };
  let ic32 = unsafe { (*state).ic32 };
  let keyboard = unsafe { &mut (*state).keyboard };

  keyboard.keyrow = row;
  keyboard.keycol = col;

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
pub extern fn key_is_down(state: *const State) -> bool {
  let keyboard = unsafe { &(*state).keyboard };
  if keyboard.keyrow == 0 && keyboard.keycol >= 2 && keyboard.keycol <= 9 {
    keyboard.scan_dip()
  } else {
    keyboard.scan_key()
  }
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
  let keyboard = unsafe { &mut (*state).keyboard };
  keyboard.update_keys();
}

#[allow(unused)]
unsafe fn characterization_test() {
  let s = State::new(Box::new(|| (0x69, false)));
  let s = Box::into_raw(Box::new(s));
  let via = sysvia_new(s);
  let v = sysvia_read(via, 0);
  assert_eq!(v, 0xFF); 
  sysvia_write(via, 0, 1);
  sysvia_poll(via, 1);

  assert_eq!((*s).interrupt, 0);
  sysvia_delete(via);
  drop(Box::from_raw(s));
}

#[test]
fn test() {
  // do stuff
  unsafe { characterization_test() };

  let sysvia = Sysvia::new(Box::new(|| (0x42, true)));
  let v = sysvia.read(0);
  assert_eq!(v, 0xFF); // via_read_null()
  sysvia.write(0, 1);
  sysvia.step(100);

//assert_eq!(unsafe { interrupt }, 0);
}

