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
//int kbdips;
  interrupt: u32,
  keyboard: Keyboard
}

#[repr(C)]
struct Keyboard {
  keyrow: u32,
  keycol: u32,

  keypress: Box<Keypress>,
  bbcmatrix: [[bool; 8]; 10],
}

impl State {
  fn new(keypress: Box<Keypress>) -> Self {
    let bbcmatrix = [[false; 8]; 10];
    let keyboard = Keyboard { keyrow: 0, keycol: 0, keypress, bbcmatrix };
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

//static mut SYSVIA: *mut Cvia = std::ptr::null_mut();
//static mut KEYROW: u32 = 0;
//static mut KEYCOL: u32 = 0;
//static mut IC32: u32 = 0;
//static mut BBCMATRIX: [[bool; 8]; 10] = [[false; 8]; 10];

fn key_update(state: *mut State) {
  let cvia = unsafe { (*state).via };
  let ic32 = unsafe { (*state).ic32 };
  let keyboard = unsafe { &mut (*state).keyboard };
  let maxcol = 10;
  if ic32 & 8 != 0 {
    /* autoscan mode */
    for col in 0..maxcol {
      for row in 1..8 {
        if keyboard.bbcmatrix[col as usize][row as usize] {
          unsafe { sysvia_set_ca2(cvia, 1) };
          return;
        }
      }
    }
  }
  else {
    /* scan specific key mode */
    if keyboard.keycol < maxcol {
      for row in 1..8 {
        if keyboard.bbcmatrix[keyboard.keycol as usize][row as usize] {
          unsafe { sysvia_set_ca2(cvia, 1) };
          return;
        }
      }
    }
  }
  unsafe { sysvia_set_ca2(cvia, 0); }
}

#[no_mangle]
pub extern fn key_scan(state: *mut State, row: u32, col: u32) {
  let keyboard = unsafe { &mut (*state).keyboard };
  keyboard.keyrow = row;
  keyboard.keycol = col;
  key_update(state);
}

#[no_mangle]
pub extern fn key_is_down(state: *mut State) -> bool {
  let keyboard = unsafe { &(*state).keyboard };
  let keyrow = keyboard.keyrow;
  let keycol = keyboard.keycol;
  assert!(keyrow < 8);
  assert!(keycol < 10);
  if keyrow == 0 && keycol >= 2 && keycol <= 9 {
    let kbdips = 0b0000_0000; // TODO, stub
    return kbdips & (1 << (9 - keycol)) != 0;
  } else {
    return keyboard.bbcmatrix[keycol as usize][keyrow as usize];
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
  let callback: &mut Box<Keypress> = &mut keyboard.keypress;
  let (key_code, pressed) = callback();
  let row = (key_code & 0b0111_0000) >> 4;
  let col = (key_code & 0b0000_1111) >> 0;
  keyboard.bbcmatrix[col as usize][row as usize] = pressed;
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

